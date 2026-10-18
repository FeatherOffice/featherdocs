//! Ler e gravar documentos. Arquivo vindo do disco nunca é confiado: o que está
//! malformado dá erro em português, nunca `panic`.

pub mod base64;
pub mod bib;
pub mod conversao;
pub mod csv;
pub mod dividido;
pub mod docx;
mod docx_comentarios;
mod docx_fontes;
mod docx_gravar;
pub mod exportar;
pub mod fdoz;
pub mod grafico_docs;
pub mod latex;
pub mod markdown;
pub mod realce;

use featherdocs_core::modelo::Documento;

/// Os formatos que o FeatherDocs lê.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Formato {
    /// O contêiner próprio (ZIP + JSON).
    Fdoz,
    Markdown,
    Latex,
    /// Word e Google Docs (`.docx`).
    Docx,
}

impl Formato {
    /// O formato de uma extensão (sem o ponto, qualquer caixa).
    pub fn da_extensao(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "fdoz" => Some(Self::Fdoz),
            "md" | "markdown" | "mdown" => Some(Self::Markdown),
            "tex" | "latex" => Some(Self::Latex),
            "docx" => Some(Self::Docx),
            _ => None,
        }
    }
}

/// Lê um documento nos bytes de um arquivo.
pub fn abrir(formato: Formato, bytes: &[u8]) -> Result<Documento, String> {
    match formato {
        Formato::Fdoz => fdoz::ler(bytes),
        Formato::Markdown => Ok(markdown::ler(&texto_utf8(bytes)?)),
        Formato::Latex => Ok(latex::ler(&texto_utf8(bytes)?)),
        Formato::Docx => docx::ler(bytes),
    }
}

/// Texto UTF-8 (com ou sem BOM); outra codificação dá erro claro em vez de lixo.
fn texto_utf8(bytes: &[u8]) -> Result<String, String> {
    let sem_bom = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    String::from_utf8(sem_bom.to_vec()).map_err(|_| "O arquivo não está em UTF-8. Salve-o como UTF-8 e tente de novo.".to_string())
}
