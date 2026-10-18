//! O histórico de versões dentro do `.fdoz`: um repositório Git **embutido** (contrato em
//! `FORMATO.md`), em Rust puro com o `gix`.
//!
//! O `.fdoz` que o motor grava (`data/*.arrow`, `assets/`) não sabe de Git. Ao salvar no disco, o
//! aplicativo passa por [`gravar_com_historico`]: o `.git/` do arquivo anterior é desempacotado numa
//! pasta temporária, os arquivos do documento novo viram uma árvore Git e um commit novo (se algo
//! mudou), e tudo volta para dentro do ZIP em `.git/`. [`versoes`] é o `git log` do arquivo e
//! [`abrir_versao`] remonta o `.fdoz` de uma versão antiga.
//!
//! Nada sai da máquina: não há `remote`, nem rede.

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use gix::bstr::{BString, ByteSlice};
use gix::objs::tree::{Entry, EntryKind};
use gix::ObjectId;
use zip::write::SimpleFileOptions;

const PASTA_GIT: &str = ".git/";

/// Uma versão do documento (um commit).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Versao {
    /// O hash do commit, em hexadecimal.
    pub id: String,
    /// Segundos desde 1970 (UTC) e o fuso de quem salvou, em segundos.
    pub quando: i64,
    pub fuso: i32,
    pub autor: String,
    pub mensagem: String,
}

fn erro(e: impl std::fmt::Display) -> String {
    format!("Histórico de versões: {e}")
}

// ---------------------------------------------------------------------------
// ZIP e pasta temporária
// ---------------------------------------------------------------------------

/// As partes de um ZIP, na ordem: `(nome, bytes)`.
fn partes(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("Isto não é um .fdoz válido: {e}"))?;
    let mut saida = Vec::with_capacity(zip.len());
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(erro)?;
        if f.is_dir() {
            continue;
        }
        let mut v = Vec::new();
        f.read_to_end(&mut v).map_err(erro)?;
        saida.push((f.name().to_string(), v));
    }
    Ok(saida)
}

fn zipar(partes: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opcoes = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    // Os objetos do Git já vêm comprimidos (zlib): guardar de novo não ganha nada.
    let guardado = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (nome, bytes) in partes {
        let o = if nome.starts_with(".git/objects/") { guardado } else { opcoes };
        zip.start_file(nome.as_str(), o).map_err(erro)?;
        zip.write_all(bytes).map_err(erro)?;
    }
    Ok(zip.finish().map_err(erro)?.into_inner())
}

/// Uma pasta temporária que se apaga sozinha.
struct Temporaria(PathBuf);

impl Temporaria {
    fn nova() -> Result<Self, String> {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
        let p = std::env::temp_dir().join(format!("featherdocs-historico-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&p).map_err(erro)?;
        Ok(Self(p))
    }
}

impl Drop for Temporaria {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// O repositório Git do `.fdoz` numa pasta temporária: o `.git/` de dentro do ZIP ou, sem ele, um novo.
fn repositorio(fdoz: Option<&[(String, Vec<u8>)]>, pasta: &Path) -> Result<gix::Repository, String> {
    let mut tem = false;
    for (nome, bytes) in fdoz.into_iter().flatten() {
        let Some(rel) = nome.strip_prefix(PASTA_GIT) else { continue };
        if rel.split('/').any(|p| p == ".." || p.is_empty()) {
            return Err(erro(format!("caminho inválido dentro do .git: {nome}")));
        }
        let destino = pasta.join(rel);
        if let Some(pai) = destino.parent() {
            std::fs::create_dir_all(pai).map_err(erro)?;
        }
        std::fs::write(&destino, bytes).map_err(erro)?;
        tem = true;
    }
    if tem {
        gix::open(pasta).map_err(erro)
    } else {
        gix::init_bare(pasta).map_err(erro)
    }
}

/// Os arquivos de uma pasta (o `.git/`), com o caminho relativo, em ordem.
fn arquivos_da_pasta(raiz: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut saida = Vec::new();
    let mut pilha = vec![raiz.to_path_buf()];
    while let Some(p) = pilha.pop() {
        for e in std::fs::read_dir(&p).map_err(erro)? {
            let e = e.map_err(erro)?;
            let caminho = e.path();
            if caminho.is_dir() {
                pilha.push(caminho);
            } else {
                let rel = caminho.strip_prefix(raiz).map_err(erro)?.to_string_lossy().replace('\\', "/");
                saida.push((format!("{PASTA_GIT}{rel}"), std::fs::read(&caminho).map_err(erro)?));
            }
        }
    }
    saida.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(saida)
}

// ---------------------------------------------------------------------------
// Árvores
// ---------------------------------------------------------------------------

/// Grava a árvore Git de uma lista de arquivos (`data/x.arrow`, `assets/y.png`…) e devolve o id.
fn gravar_arvore(repo: &gix::Repository, arquivos: &[(&str, &[u8])]) -> Result<ObjectId, String> {
    let mut entradas: Vec<Entry> = Vec::new();
    let mut subpastas: std::collections::BTreeMap<&str, Vec<(&str, &[u8])>> = std::collections::BTreeMap::new();
    for &(nome, bytes) in arquivos {
        match nome.split_once('/') {
            Some((pasta, resto)) => subpastas.entry(pasta).or_default().push((resto, bytes)),
            None => {
                let oid = repo.write_blob(bytes).map_err(erro)?.detach();
                entradas.push(Entry { mode: EntryKind::Blob.into(), filename: nome.into(), oid });
            }
        }
    }
    for (pasta, dentro) in subpastas {
        let oid = gravar_arvore(repo, &dentro)?;
        entradas.push(Entry { mode: EntryKind::Tree.into(), filename: pasta.into(), oid });
    }
    // A ordem do Git: pelo nome, com a pasta comparada como "nome/".
    let chave = |e: &Entry| {
        let mut k: BString = e.filename.clone();
        if e.mode.is_tree() {
            k.push(b'/');
        }
        k
    };
    entradas.sort_by_key(chave);
    Ok(repo.write_object(gix::objs::Tree { entries: entradas }).map_err(erro)?.detach())
}

/// Os arquivos de uma árvore Git, com o caminho completo.
fn ler_arvore(repo: &gix::Repository, id: ObjectId, prefixo: &str, saida: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
    let arvore = repo.find_tree(id).map_err(erro)?;
    let decodificada = arvore.decode().map_err(erro)?;
    for e in &decodificada.entries {
        let nome = format!("{prefixo}{}", e.filename.to_str_lossy());
        if e.mode.is_tree() {
            ler_arvore(repo, e.oid.to_owned(), &format!("{nome}/"), saida)?;
        } else {
            saida.push((nome, repo.find_blob(e.oid.to_owned()).map_err(erro)?.data.clone()));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// O que o aplicativo chama
// ---------------------------------------------------------------------------

/// O `.fdoz` novo (sem Git, como o motor grava) com o histórico do arquivo `anterior` e um commit
/// a mais. Se o documento não mudou desde a última versão, não cria commit vazio.
pub fn gravar_com_historico(novo: &[u8], anterior: Option<&[u8]>, autor: &str, mensagem: &str) -> Result<Vec<u8>, String> {
    let documento: Vec<(String, Vec<u8>)> = partes(novo)?.into_iter().filter(|(n, _)| !n.starts_with(PASTA_GIT)).collect();
    let velho = anterior.map(partes).transpose()?;
    let tmp = Temporaria::nova()?;
    let repo = repositorio(velho.as_deref(), &tmp.0)?;
    let arquivos: Vec<(&str, &[u8])> = documento.iter().map(|(n, b)| (n.as_str(), b.as_slice())).collect();
    let arvore = gravar_arvore(&repo, &arquivos)?;
    let pai = repo.head_id().ok().map(gix::Id::detach);
    let mesma = pai.and_then(|p| repo.find_commit(p).ok()).and_then(|c| c.tree_id().ok()).is_some_and(|t| t == arvore);
    if !mesma {
        let nome = if autor.trim().is_empty() { "FeatherDocs" } else { autor.trim() };
        let assinatura = gix::actor::Signature { name: nome.into(), email: BString::default(), time: gix::date::Time::now_local_or_utc() };
        let mut buf = gix::date::parse::TimeBuf::default();
        let r = assinatura.to_ref(&mut buf);
        repo.commit_as(r, r, "HEAD", mensagem, arvore, pai).map_err(erro)?;
    }
    let mut tudo = documento;
    tudo.extend(arquivos_da_pasta(&tmp.0)?);
    zipar(&tudo)
}

/// O `git log` do `.fdoz`: as versões, da mais nova para a mais antiga. Sem histórico, vazio.
pub fn versoes(fdoz: &[u8]) -> Result<Vec<Versao>, String> {
    let todas = partes(fdoz)?;
    if !todas.iter().any(|(n, _)| n.starts_with(PASTA_GIT)) {
        return Ok(Vec::new());
    }
    let tmp = Temporaria::nova()?;
    let repo = repositorio(Some(&todas), &tmp.0)?;
    let mut saida = Vec::new();
    let mut atual = repo.head_id().ok().map(gix::Id::detach);
    while let Some(id) = atual {
        let c = repo.find_commit(id).map_err(erro)?;
        let autor = c.author().map_err(erro)?;
        let tempo = autor.time().map_err(erro)?;
        saida.push(Versao { id: id.to_string(), quando: tempo.seconds, fuso: tempo.offset, autor: autor.name.to_str_lossy().into_owned(), mensagem: c.message_raw_sloppy().to_str_lossy().trim().to_string() });
        atual = c.parent_ids().next().map(gix::Id::detach);
    }
    Ok(saida)
}

/// O `.fdoz` (sem o histórico) de uma versão antiga, pelo hash do commit.
pub fn abrir_versao(fdoz: &[u8], id: &str) -> Result<Vec<u8>, String> {
    let todas = partes(fdoz)?;
    let tmp = Temporaria::nova()?;
    let repo = repositorio(Some(&todas), &tmp.0)?;
    let oid = ObjectId::from_hex(id.as_bytes()).map_err(|_| erro(format!("versão inválida: {id}")))?;
    let commit = repo.find_commit(oid).map_err(|_| erro("essa versão não existe neste arquivo"))?;
    let mut arquivos = Vec::new();
    ler_arvore(&repo, commit.tree_id().map_err(erro)?.detach(), "", &mut arquivos)?;
    zipar(&arquivos)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn fdoz(partes: &[(&str, &[u8])]) -> Vec<u8> {
        zipar(&partes.iter().map(|(n, b)| (n.to_string(), b.to_vec())).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn cada_salvamento_vira_versao_e_cada_versao_volta() {
        let v1 = gravar_com_historico(&fdoz(&[("data/blocos.arrow", b"um"), ("assets/a.png", b"PNG")]), None, "Ana", "Primeira").unwrap();
        let v2 = gravar_com_historico(&fdoz(&[("data/blocos.arrow", b"dois")]), Some(&v1), "Ana", "Segunda").unwrap();
        let log = versoes(&v2).unwrap();
        assert_eq!(log.iter().map(|v| v.mensagem.as_str()).collect::<Vec<_>>(), ["Segunda", "Primeira"]);
        assert_eq!(log[0].autor, "Ana");
        let antiga = partes(&abrir_versao(&v2, &log[1].id).unwrap()).unwrap();
        assert_eq!(antiga, [("assets/a.png".to_string(), b"PNG".to_vec()), ("data/blocos.arrow".to_string(), b"um".to_vec())]);
        // O documento atual continua por fora do .git, para quem abre sem histórico.
        assert!(partes(&v2).unwrap().iter().any(|(n, b)| n == "data/blocos.arrow" && b == b"dois"));
    }

    #[test]
    fn salvar_sem_mudar_nada_nao_cria_versao_vazia() {
        let doc = fdoz(&[("data/blocos.arrow", b"igual")]);
        let v1 = gravar_com_historico(&doc, None, "", "Primeira").unwrap();
        let v2 = gravar_com_historico(&doc, Some(&v1), "", "De novo").unwrap();
        let log = versoes(&v2).unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].autor, "FeatherDocs");
    }

    #[test]
    fn arquivo_sem_historico_tem_lista_vazia_e_lixo_da_erro() {
        assert!(versoes(&fdoz(&[("data/blocos.arrow", b"x")])).unwrap().is_empty());
        assert!(versoes(b"lixo").is_err());
        let v1 = gravar_com_historico(&fdoz(&[("data/blocos.arrow", b"x")]), None, "", "a").unwrap();
        assert!(abrir_versao(&v1, "0000000000000000000000000000000000000000").is_err());
        assert!(abrir_versao(&v1, "não é hash").is_err());
    }
}
