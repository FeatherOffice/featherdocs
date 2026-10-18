//! Abrir, salvar e exportar: caixas de diálogo e disco. Ler e gravar os formatos
//! é do `featherdocs-io`, dentro da ponte; aqui só se entrega os bytes. Todo comando
//! devolve o que a interface precisa (o estado novo, o nome) ou `None` se o usuário
//! cancelou a caixa.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::executar;

/// O arquivo `.fdoz` do documento aberto (`None` = documento novo ou importado, que ainda não tem).
#[derive(Default)]
pub struct Caminho(Mutex<Option<PathBuf>>);

impl Caminho {
    fn definir(&self, c: Option<PathBuf>) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = c;
    }
    fn atual(&self) -> Option<PathBuf> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

/// Um documento aberto no disco: o estado do motor e o nome para o título da janela.
#[derive(Serialize)]
pub struct Aberto {
    estado: Value,
    nome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texto_bruto: Option<String>,
}

/// A extensão (minúscula) de um caminho.
fn extensao(c: &Path) -> String {
    c.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase()
}

/// O comando da ponte que lê o arquivo, pela extensão.
fn comando_de_abrir(ext: &str) -> Option<&'static str> {
    match ext {
        "fdoz" => Some("documento_abrir_fdoz"),
        "md" | "markdown" | "mdown" => Some("documento_abrir_md"),
        "tex" | "latex" => Some("documento_abrir_tex"),
        "docx" => Some("documento_abrir_docx"),
        _ => None,
    }
}

fn nome_do(c: &Path) -> String {
    c.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string()
}

/// A pasta padrão: `Documentos/FeatherDocs`, criada se faltar (se não der, as caixas abrem onde o sistema quiser).
fn pasta_padrao(app: &AppHandle) -> Option<PathBuf> {
    let pasta = app.path().document_dir().ok()?.join("FeatherDocs");
    std::fs::create_dir_all(&pasta).ok()?;
    Some(pasta)
}

/// Grava de uma vez só: escreve num temporário ao lado e troca, para um corte no meio não deixar o arquivo pela metade.
fn gravar_atomico(destino: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporario = destino.with_extension("tmp-featherdocs");
    std::fs::write(&temporario, bytes).map_err(|e| format!("Não consegui gravar \"{}\": {e}", destino.display()))?;
    std::fs::rename(&temporario, destino).map_err(|e| format!("Não consegui gravar \"{}\": {e}", destino.display()))
}

fn estado_de(bytes: &[u8]) -> Result<Value, String> {
    serde_json::from_slice(bytes).map_err(|e| format!("estado ilegível: {e}"))
}

/// Abre o arquivo no motor. O arquivo fica ligado ao documento (o Salvar grava nele de novo):
/// `.fdoz` e `.docx` na vertente convencional, `.tex` e `.md` na de código.
fn abrir_caminho(app: &AppHandle, caminho: &Path) -> Result<Aberto, String> {
    let ext = extensao(caminho);
    let comando = comando_de_abrir(&ext).ok_or_else(|| format!("\"{}\" não é um .fdoz, .md, .tex ou .docx.", nome_do(caminho)))?;
    let bytes = std::fs::read(caminho).map_err(|e| format!("Não consegui ler \"{}\": {e}", caminho.display()))?;
    let estado = estado_de(&executar(app, comando, &bytes)?)?;
    app.state::<Caminho>().definir(Some(caminho.to_path_buf()));
    let texto_bruto = if matches!(ext.as_str(), "tex" | "latex" | "md" | "markdown") { String::from_utf8(bytes).ok() } else { None };
    Ok(Aberto { estado, nome: nome_do(caminho), texto_bruto })
}

/// Abre um documento: o do `caminho` ou, sem ele, o que o usuário escolher na caixa.
#[tauri::command]
pub async fn documento_abrir(app: AppHandle, caminho: Option<String>) -> Result<Option<Aberto>, String> {
    let caminho = match caminho {
        Some(c) => PathBuf::from(c),
        None => {
            let mut caixa = app.dialog().file();
            if let Some(p) = pasta_padrao(&app) {
                caixa = caixa.set_directory(p);
            }
            let Some(f) = caixa
                .add_filter("Documentos (*.fdoz, *.md, *.tex, *.docx)", &["fdoz", "md", "markdown", "tex", "docx"])
                .add_filter("FeatherDocs (.fdoz)", &["fdoz"])
                .add_filter("Markdown (.md)", &["md", "markdown"])
                .add_filter("LaTeX (.tex)", &["tex"])
                .add_filter("Word (.docx)", &["docx"])
                .blocking_pick_file()
            else {
                return Ok(None);
            };
            f.into_path().map_err(|e| format!("Caminho inválido: {e}"))?
        }
    };
    abrir_caminho(&app, &caminho).map(Some)
}

/// O arquivo com que o sistema abriu o aplicativo (dois cliques, "Abrir com"): entrega uma vez só.
#[tauri::command]
pub async fn documento_inicial(app: AppHandle) -> Result<Option<Aberto>, String> {
    static ENTREGUE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if ENTREGUE.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Ok(None);
    }
    let arquivo = std::env::args().skip(1).filter(|a| !a.starts_with('-')).map(PathBuf::from).find(|p| p.is_file() && comando_de_abrir(&extensao(p)).is_some());
    arquivo.map(|c| abrir_caminho(&app, &c)).transpose()
}

/// O resultado de salvar. Com `perdas`, nada foi gravado ainda: a conversão (LaTeX → Markdown) tira
/// coisas do documento, a interface mostra a lista e, se o usuário aceitar, chama `documento_salvar_confirmado`.
#[derive(Serialize)]
pub struct Salvo {
    nome: String,
    /// A extensão gravada (ou a que vai ser): `fdoz`, `docx`, `tex`, `md`.
    formato: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    perdas: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    caminho: Option<String>,
}

/// O rótulo da caixa de salvar para cada formato.
fn rotulo_do_formato(f: &str) -> &'static str {
    match f {
        "docx" => "Word (.docx)",
        "tex" => "LaTeX (.tex)",
        "md" => "Markdown (.md)",
        _ => "FeatherDocs (.fdoz)",
    }
}

/// O texto do modo dividido vai ao motor antes de gravar (só os blocos que mudaram são trocados).
fn aplicar_codigo(app: &AppHandle, texto: &Option<String>, formato: &Option<String>) -> Result<(), String> {
    if let (Some(t), Some(f)) = (texto, formato.as_deref().filter(|f| matches!(*f, "tex" | "md"))) {
        let corpo = serde_json::json!({ "formato": f, "texto": t });
        executar(app, "codigo_aplicar", corpo.to_string().as_bytes())?;
    }
    Ok(())
}

/// As extensões em que o documento salva, pela vertente dele (a primeira é a padrão).
fn formatos_da_vertente(app: &AppHandle) -> Result<Vec<String>, String> {
    let v: Value = serde_json::from_slice(&executar(app, "documento_vertente", b"")?).map_err(|e| format!("vertente ilegível: {e}"))?;
    Ok(v["formatos"].as_array().map(|a| a.iter().filter_map(|f| f.as_str().map(String::from)).collect()).unwrap_or_else(|| vec!["fdoz".into()]))
}

/// A caixa de salvar com os formatos da vertente; a extensão escrita decide (sem uma válida, vale a padrão).
fn perguntar_destino(app: &AppHandle, formatos: &[String]) -> Option<PathBuf> {
    let padrao = formatos.first().map_or("fdoz", String::as_str);
    let nome = app.state::<Caminho>().atual().and_then(|c| c.file_stem().map(|s| s.to_string_lossy().into_owned())).unwrap_or_else(|| "Documento".into());
    let mut caixa = app.dialog().file().set_file_name(format!("{nome}.{padrao}"));
    for f in formatos {
        caixa = caixa.add_filter(rotulo_do_formato(f), &[f.as_str()]);
    }
    if let Some(p) = pasta_padrao(app) {
        caixa = caixa.set_directory(p);
    }
    let mut c = caixa.blocking_save_file()?.into_path().ok()?;
    if !formatos.iter().any(|f| *f == extensao(&c)) {
        c.set_extension(padrao);
    }
    Some(c)
}

/// O `.fdoz` com o histórico de versões embutido: o `.git/` vem do arquivo que já está no `destino`
/// (ou, no "Salvar como", do arquivo de onde o documento veio) e cada Salvar vira uma versão.
fn com_historico(app: &AppHandle, destino: &Path, novo: Vec<u8>) -> Result<Vec<u8>, String> {
    let origem = app.state::<Caminho>().atual().filter(|c| extensao(c) == "fdoz");
    let anterior = [Some(destino.to_path_buf()), origem].into_iter().flatten().find_map(|c| std::fs::read(c).ok());
    let autor = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_default();
    featherdocs_historico::gravar_com_historico(&novo, anterior.as_deref(), &autor, "Versão salva")
}

/// Uma versão do documento, para a lista do histórico.
#[derive(Serialize)]
pub struct VersaoDoArquivo {
    id: String,
    /// Segundos desde 1970 (UTC).
    quando: i64,
    autor: String,
    mensagem: String,
}

/// O histórico de versões do `.fdoz` aberto (o `git log` de dentro dele), da mais nova para a mais antiga.
#[tauri::command]
pub async fn documento_versoes(app: AppHandle) -> Result<Vec<VersaoDoArquivo>, String> {
    let Some(c) = app.state::<Caminho>().atual().filter(|c| extensao(c) == "fdoz") else { return Ok(Vec::new()) };
    let bytes = std::fs::read(&c).map_err(|e| format!("Não consegui ler \"{}\": {e}", c.display()))?;
    Ok(featherdocs_historico::versoes(&bytes)?.into_iter().map(|v| VersaoDoArquivo { id: v.id, quando: v.quando, autor: v.autor, mensagem: v.mensagem }).collect())
}

/// Abre uma versão antiga do `.fdoz` aberto. O arquivo continua o mesmo: salvar grava uma versão nova
/// por cima (o histórico guarda todas), ou seja, é "restaurar".
#[tauri::command]
pub async fn documento_abrir_versao(app: AppHandle, id: String) -> Result<Aberto, String> {
    let c = app.state::<Caminho>().atual().filter(|c| extensao(c) == "fdoz").ok_or("O documento não está num .fdoz.")?;
    let bytes = std::fs::read(&c).map_err(|e| format!("Não consegui ler \"{}\": {e}", c.display()))?;
    let antigo = featherdocs_historico::abrir_versao(&bytes, &id)?;
    let estado = estado_de(&executar(&app, "documento_abrir_fdoz", &antigo)?)?;
    Ok(Aberto { estado, nome: nome_do(&c), texto_bruto: None })
}

/// Revisão → Comparar com uma versão antiga do `.fdoz` aberto: o motor marca no documento de agora o
/// que mudou desde ela. Devolve a mudança (para a tela repintar).
#[tauri::command]
pub async fn documento_comparar_versao(app: AppHandle, id: String) -> Result<Value, String> {
    let c = app.state::<Caminho>().atual().filter(|c| extensao(c) == "fdoz").ok_or("O documento não está num .fdoz.")?;
    let bytes = std::fs::read(&c).map_err(|e| format!("Não consegui ler \"{}\": {e}", c.display()))?;
    let antigo = featherdocs_historico::abrir_versao(&bytes, &id)?;
    estado_de(&executar(&app, "documento_comparar_fdoz", &antigo)?)
}

/// Grava o documento no `destino`, no formato da extensão dele.
fn gravar_em(app: &AppHandle, destino: &Path, texto: Option<String>, formato_texto: Option<String>) -> Result<Salvo, String> {
    let ext = extensao(destino);
    let bytes = match ext.as_str() {
        "docx" => executar(app, "documento_gravar_docx", b"")?,
        "tex" | "md" => match texto {
            // O texto do modo dividido, do jeito que o usuário escreveu.
            Some(t) if formato_texto.as_deref() == Some(ext.as_str()) => t.into_bytes(),
            _ => executar(app, "documento_converter", serde_json::json!({ "formato": ext }).to_string().as_bytes())?,
        },
        _ => com_historico(app, destino, executar(app, "documento_gravar_fdoz", b"")?)?,
    };
    gravar_atomico(destino, &bytes)?;
    executar(app, "documento_marcar_gravado", b"")?;
    app.state::<Caminho>().definir(Some(destino.to_path_buf()));
    Ok(Salvo { nome: nome_do(destino), formato: ext, perdas: None, caminho: None })
}

/// Salva: no arquivo do documento ou, com `como` (ou sem arquivo, ou num formato fora da vertente),
/// pela caixa. `texto`/`formato` são o código do modo dividido, se estiver aberto.
#[tauri::command]
pub async fn documento_salvar(app: AppHandle, como: bool, texto: Option<String>, formato: Option<String>) -> Result<Option<Salvo>, String> {
    aplicar_codigo(&app, &texto, &formato)?;
    let formatos = formatos_da_vertente(&app)?;
    let atual = app.state::<Caminho>().atual().filter(|c| !como && formatos.contains(&extensao(c)));
    let Some(destino) = atual.or_else(|| perguntar_destino(&app, &formatos)) else { return Ok(None) };
    let ext = extensao(&destino);
    let corpo = serde_json::json!({ "formato": ext }).to_string();
    let perdas: Vec<String> = serde_json::from_slice(&executar(&app, "documento_perdas", corpo.as_bytes())?).unwrap_or_default();
    if !perdas.is_empty() {
        return Ok(Some(Salvo { nome: nome_do(&destino), formato: ext, perdas: Some(perdas), caminho: Some(destino.to_string_lossy().into_owned()) }));
    }
    gravar_em(&app, &destino, texto, formato).map(Some)
}

/// Grava depois que o usuário aceitou o aviso de conversão de `documento_salvar`.
#[tauri::command]
pub async fn documento_salvar_confirmado(app: AppHandle, caminho: String, texto: Option<String>, formato: Option<String>) -> Result<Salvo, String> {
    aplicar_codigo(&app, &texto, &formato)?;
    gravar_em(&app, Path::new(&caminho), texto, formato)
}

/// Exporta (Markdown, LaTeX ou Word) pela caixa de salvar, sem mudar o arquivo nem a vertente do documento.
#[tauri::command]
pub async fn documento_exportar(app: AppHandle, formato: String, texto: Option<String>) -> Result<Option<String>, String> {
    let comando = match formato.as_str() {
        "md" => "documento_exportar_md",
        "tex" => "documento_exportar_tex",
        "docx" => "documento_gravar_docx",
        outro => return Err(format!("Não sei exportar \"{outro}\".")),
    };
    let nome_sugerido = app.state::<Caminho>().atual().and_then(|c| c.file_stem().map(|s| s.to_string_lossy().into_owned())).unwrap_or_else(|| "Documento".into());
    let mut caixa = app.dialog().file().add_filter(rotulo_do_formato(&formato), &[formato.as_str()]).set_file_name(format!("{nome_sugerido}.{formato}"));
    if let Some(p) = pasta_padrao(&app) {
        caixa = caixa.set_directory(p);
    }
    let Some(f) = caixa.blocking_save_file() else { return Ok(None) };
    let destino = f.into_path().map_err(|e| format!("Caminho inválido: {e}"))?;
    let bytes = match texto {
        Some(t) => {
            aplicar_codigo(&app, &Some(t.clone()), &Some(formato.clone()))?;
            t.into_bytes()
        }
        None => executar(&app, comando, b"")?,
    };
    gravar_atomico(&destino, &bytes)?;
    Ok(Some(nome_do(&destino)))
}

/// Abre um arquivo de texto qualquer (um `.bib`, por exemplo) pela caixa do sistema e devolve o
/// conteúdo; `None` se o usuário cancelou. O documento aberto não muda.
#[tauri::command]
pub async fn texto_abrir(app: AppHandle, filtro: String, extensoes: Vec<String>) -> Result<Option<String>, String> {
    let ext: Vec<&str> = extensoes.iter().map(String::as_str).collect();
    let mut caixa = app.dialog().file().add_filter(filtro, &ext);
    if let Some(p) = pasta_padrao(&app) {
        caixa = caixa.set_directory(p);
    }
    let Some(f) = caixa.blocking_pick_file() else { return Ok(None) };
    let caminho = f.into_path().map_err(|e| format!("Caminho inválido: {e}"))?;
    let bytes = std::fs::read(&caminho).map_err(|e| format!("Não consegui ler \"{}\": {e}", caminho.display()))?;
    String::from_utf8(bytes).map(Some).map_err(|_| "O arquivo não está em UTF-8.".to_string())
}

/// Grava um texto (um `.bib`, por exemplo) onde o usuário escolher; `None` se cancelou.
#[tauri::command]
pub async fn texto_salvar(app: AppHandle, nome: String, filtro: String, extensoes: Vec<String>, texto: String) -> Result<Option<String>, String> {
    let ext: Vec<&str> = extensoes.iter().map(String::as_str).collect();
    let mut caixa = app.dialog().file().add_filter(filtro, &ext).set_file_name(nome);
    if let Some(p) = pasta_padrao(&app) {
        caixa = caixa.set_directory(p);
    }
    let Some(f) = caixa.blocking_save_file() else { return Ok(None) };
    let destino = f.into_path().map_err(|e| format!("Caminho inválido: {e}"))?;
    gravar_atomico(&destino, texto.as_bytes())?;
    Ok(Some(nome_do(&destino)))
}

/// Documento novo: esquece o arquivo do anterior.
#[tauri::command]
pub fn documento_esquecer_caminho(app: AppHandle) {
    app.state::<Caminho>().definir(None);
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn extensao_decide_o_leitor() {
        assert_eq!(comando_de_abrir(&extensao(Path::new("a/B.DOCX"))), Some("documento_abrir_docx"));
        assert_eq!(comando_de_abrir(&extensao(Path::new("x.md"))), Some("documento_abrir_md"));
        assert_eq!(comando_de_abrir(&extensao(Path::new("x.pdf"))), None);
        assert_eq!(comando_de_abrir(&extensao(Path::new("sem_extensao"))), None);
    }

    #[test]
    fn gravacao_atomica_nao_deixa_temporario() {
        let pasta = std::env::temp_dir().join(format!("featherdocs-teste-{}", std::process::id()));
        std::fs::create_dir_all(&pasta).unwrap();
        let alvo = pasta.join("a.fdoz");
        gravar_atomico(&alvo, b"um").unwrap();
        gravar_atomico(&alvo, b"dois").unwrap();
        assert_eq!(std::fs::read(&alvo).unwrap(), b"dois");
        assert_eq!(std::fs::read_dir(&pasta).unwrap().count(), 1);
        std::fs::remove_dir_all(&pasta).unwrap();
    }
}
