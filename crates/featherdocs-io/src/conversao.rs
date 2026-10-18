//! As duas vertentes do documento e o que se perde ao passar de uma para outra.
//!
//! - **Convencional** (editado na página): salva no `.fdoz` (o nosso) ou no `.docx`
//!   (Word, nativo — não é exportação).
//! - **Código** (LaTeX ou Markdown, com o modo dividido): salva no próprio texto,
//!   `.tex` ou `.md`. Markdown vira LaTeX sem perda; LaTeX vira Markdown perdendo o
//!   que o Markdown não tem, e quem salva é avisado antes (`perdas`).

use featherdocs_core::modelo::{Alinhamento, ConfigPagina, Conteudo, Documento, Extras, Numeracao, Trecho, SERIFA, SUBLINHADO};
use serde::Serialize;

/// De onde o documento veio, que decide em que ele salva.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Vertente {
    Convencional,
    Latex,
    Markdown,
}

impl Vertente {
    /// As extensões em que o documento salva, a primeira é a padrão.
    pub fn formatos(self) -> &'static [&'static str] {
        match self {
            Self::Convencional => &["fdoz", "docx"],
            Self::Latex => &["tex", "md"],
            Self::Markdown => &["md", "tex"],
        }
    }

    /// A vertente de um formato de arquivo (`fdoz`, `docx`, `tex`, `md`).
    pub fn do_formato(formato: &str) -> Option<Self> {
        match formato {
            "fdoz" | "docx" => Some(Self::Convencional),
            "tex" | "latex" => Some(Self::Latex),
            "md" | "markdown" => Some(Self::Markdown),
            _ => None,
        }
    }
}

/// O que não chega ao `formato` quando o documento da vertente `de` é gravado nele, como
/// chaves curtas (a interface traduz: `perda.<chave>`). Vazio = nada a avisar.
/// Só a passagem LaTeX → Markdown avisa: é a que tira do documento o que ele tinha.
pub fn perdas(doc: &Documento, de: Vertente, formato: &str) -> Vec<&'static str> {
    if de != Vertente::Latex || Vertente::do_formato(formato) != Some(Vertente::Markdown) {
        return Vec::new();
    }
    let mut p = vec!["comandos_latex"];
    let trechos = || doc.blocos.iter().flat_map(|b| trechos_do(&b.conteudo));
    if trechos().any(|t| !t.fmt.eh_padrao() || t.estilo & (SUBLINHADO | SERIFA) != 0) {
        p.push("formatacao");
    }
    let a = |f: &dyn Fn(&featherdocs_core::modelo::Atrib) -> bool| doc.blocos.iter().any(|b| f(&b.atrib));
    if a(&|x| x.alinhamento != Alinhamento::Esquerda || x.recuo_esq != 0.0 || x.recuo_dir != 0.0 || x.primeira != 0.0 || x.entrelinha > 0.0 || x.antes.is_some() || x.depois.is_some()) {
        p.push("paragrafo");
    }
    if a(&|x| x.fundo != 0 || x.borda != 0) {
        p.push("fundo_e_bordas");
    }
    if doc.estilos.numeracao != Numeracao::Nenhuma {
        p.push("numeracao");
    }
    if a(&|x| x.legenda > 0) || doc.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Imagem { legenda, .. } if !legenda.is_empty())) {
        p.push("legendas");
    }
    if doc.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::Imagem { largura, .. } if largura > 0.0)) {
        p.push("tamanho_imagem");
    }
    if doc.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::Sumario { .. })) {
        p.push("sumario");
    }
    if doc.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::QuebraDePagina)) {
        p.push("quebra_de_pagina");
    }
    if trechos().any(|t| matches!(t.marca.as_deref(), Some(featherdocs_core::modelo::Marca::Indice { .. }))) {
        p.push("indice");
    }
    if doc.pagina != ConfigPagina::a4() || doc.extras != Extras::default() {
        p.push("pagina");
    }
    if !doc.meta.titulo.is_empty() || !doc.meta.autor.is_empty() {
        p.push("titulo_e_autor");
    }
    p
}

fn trechos_do(c: &Conteudo) -> Vec<&Trecho> {
    match c {
        Conteudo::Titulo { texto, .. } | Conteudo::Paragrafo { texto } | Conteudo::Item { texto, .. } | Conteudo::Citacao { texto } => texto.iter().collect(),
        Conteudo::Tabela { linhas, .. } => linhas.iter().flatten().flatten().collect(),
        Conteudo::Imagem { legenda, .. } => legenda.iter().collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn so_latex_para_markdown_avisa() {
        let d = crate::latex::ler("\\documentclass{article}\\begin{document}\\section{A}\\textcolor{red}{oi}\\newpage fim\\end{document}");
        let p = perdas(&d, Vertente::Latex, "md");
        assert!(p.contains(&"comandos_latex"));
        assert!(p.contains(&"quebra_de_pagina"), "{p:?}");
        assert!(perdas(&d, Vertente::Markdown, "tex").is_empty());
        assert!(perdas(&d, Vertente::Latex, "tex").is_empty());
        assert!(perdas(&d, Vertente::Convencional, "docx").is_empty());
    }

    #[test]
    fn cada_vertente_salva_nos_seus_formatos() {
        assert_eq!(Vertente::Convencional.formatos(), ["fdoz", "docx"]);
        assert_eq!(Vertente::Latex.formatos()[0], "tex");
        assert_eq!(Vertente::Markdown.formatos()[0], "md");
    }
}
