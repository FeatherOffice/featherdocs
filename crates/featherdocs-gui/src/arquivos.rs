//! Leitura por extensão e gravação atômica sem expor arquivos parcialmente escritos.
use featherdocs_core::modelo::Documento;
use featherdocs_io::{abrir, Formato};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub fn ler(p: &Path) -> Result<Documento, String> {
    let formato = p
        .extension()
        .and_then(|e| e.to_str())
        .and_then(Formato::da_extensao)
        .ok_or("Formato não suportado. Use .fdoz, .docx, .md ou .tex.")?;
    // Recusa por metadados antes de alocar e limita a leitura real:
    // evita arquivo esparso ou stream que cresça enquanto é lido.
    const LIMITE: u64 = 256 * 1024 * 1024;
    let arquivo = std::fs::File::open(p)
        .map_err(|e| format!("Não consegui abrir {}: {e}", p.display()))?;
    if arquivo.metadata().map(|m| m.len() > LIMITE).unwrap_or(false) {
        return Err(format!(
            "O arquivo {} ultrapassa o limite seguro de 256 MiB.",
            p.display()
        ));
    }
    let mut bytes = Vec::new();
    arquivo.take(LIMITE + 1).read_to_end(&mut bytes)
        .map_err(|e| format!("Erro ao ler {}: {e}", p.display()))?;
    if bytes.len() as u64 > LIMITE {
        return Err(format!(
            "O arquivo {} ultrapassa o limite seguro de 256 MiB.",
            p.display()
        ));
    }
    abrir(formato, &bytes)
        .map_err(|e| format!("Documento {} inválido: {e}", p.display()))
}
pub fn bytes(doc: &Documento, ext: &str) -> Result<Vec<u8>, String> {
    match Formato::da_extensao(ext) {
        Some(Formato::Fdoz) => featherdocs_io::fdoz::gravar(doc),
        Some(Formato::Docx) => featherdocs_io::docx::gravar(doc),
        Some(Formato::Markdown) => Ok(featherdocs_io::exportar::markdown(doc).into_bytes()),
        Some(Formato::Latex) => Ok(featherdocs_io::exportar::latex(doc).into_bytes()),
        None => Err("Formato de saída não suportado.".into()),
    }
}
pub fn gravar(p: &Path, bytes: &[u8]) -> Result<(), String> {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let pasta = p
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp: PathBuf = pasta.join(format!(".featherdocs-{}-{seq}.tmp", std::process::id()));
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
        .map_err(|e| format!("Não consegui preparar o salvamento: {e}"))?;
    let resultado = (|| {
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, p)
    })();
    if resultado.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    resultado.map_err(|e| format!("Não consegui salvar {}: {e}", p.display()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn qa009_arquivo_corrompido_nao_vira_documento_valido() {
        let pasta = std::env::temp_dir();
        let p = pasta.join(format!("featherdocs-qa009-corrompido-{}.fdoz", std::process::id()));
        std::fs::write(&p, b"arquivo invalido e truncado").unwrap();
        let erro = ler(&p).unwrap_err();
        assert!(erro.contains("inválido"), "{erro}");
        std::fs::remove_file(p).unwrap();
    }

    #[test]
    fn qa009_arquivo_enorme_e_recusado_sem_leitura_completa() {
        let p = std::env::temp_dir().join(format!("featherdocs-qa009-limite-{}.docx", std::process::id()));
        let f = std::fs::File::create(&p).unwrap();
        f.set_len(256 * 1024 * 1024 + 1).unwrap();
        drop(f);
        let erro = ler(&p).unwrap_err();
        assert!(erro.contains("256 MiB"), "{erro}");
        std::fs::remove_file(p).unwrap();
    }

    #[test]
    fn qa009_arquivo_inexistente_retorna_erro_acionavel() {
        let p = std::env::temp_dir().join(format!(
            "featherdocs-qa009-inexistente-{}.fdoz", std::process::id()
        ));
        let erro = ler(&p).unwrap_err();
        assert!(erro.contains("Não consegui abrir"), "{erro}");
    }

    #[test]
    fn grava_abre_e_falha_preserva_original() {
        let p =
            std::env::temp_dir().join(format!("featherdocs-nativo-{}.fdoz", std::process::id()));
        let mut s = featherdocs_core::sessao::Sessao::default();
        s.inserir_texto("Texto com acentuação");
        let b = bytes(s.documento(), "fdoz").unwrap();
        gravar(&p, &b).unwrap();
        assert_eq!(ler(&p).unwrap(), *s.documento());
        assert!(gravar(&p.join("arquivo"), "não deve substituir".as_bytes()).is_err());
        assert_eq!(std::fs::read(&p).unwrap(), b);
        std::fs::remove_file(p).unwrap();
    }
}
