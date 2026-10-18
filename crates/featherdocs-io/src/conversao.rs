//! As duas vertentes do documento e o que se perde ao passar de uma para outra.
//!
//! - **Convencional** (editado na página): salva no `.fdoz` (o nosso) ou no `.docx`
//!   (Word, nativo — não é exportação).
//! - **Código** (LaTeX ou Markdown, com o modo dividido): salva no próprio texto,
//!   `.tex` ou `.md`. Markdown vira LaTeX sem perda; LaTeX vira Markdown perdendo o
//!   que o Markdown não tem, e quem salva é avisado antes (`perdas`).

use featherdocs_core::modelo::{Alinhamento, ConfigPagina, Conteudo, Documento, Extras, Fmt, Numeracao, Trecho, SERIFA, SUBLINHADO};

/// O corpo que o leitor de LaTeX põe no texto normal (o `11pt` da classe): não é formatação própria.
const CORPO_DO_LATEX: f32 = 11.0;
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
            "tex" | "latex" | "zip" => Some(Self::Latex),
            "md" | "markdown" => Some(Self::Markdown),
            _ => None,
        }
    }
}

/// O que não chega ao `formato` quando o documento da vertente `de` é gravado nele, como
/// chaves curtas (a interface diz cada uma com [`descricao`]). Vazio = nada a avisar.
/// Avisam a passagem para Markdown (que não tem quase nada além do texto) e a do documento
/// convencional para LaTeX (o que o código não escreve).
pub fn perdas(doc: &Documento, de: Vertente, formato: &str) -> Vec<&'static str> {
    match (de, Vertente::do_formato(formato)) {
        (Vertente::Latex | Vertente::Convencional, Some(Vertente::Markdown)) => perdas_no_markdown(doc, de == Vertente::Latex),
        (Vertente::Convencional, Some(Vertente::Latex)) => perdas_no_latex(doc, formato),
        _ => Vec::new(),
    }
}

/// O que o LaTeX que o FeatherDocs escreve não leva (o resto vai: página, cabeçalho, parágrafos…).
fn perdas_no_latex(doc: &Documento, formato: &str) -> Vec<&'static str> {
    let mut p = Vec::new();
    let tem = |f: &dyn Fn(&featherdocs_core::modelo::Bloco) -> bool| doc.blocos.iter().any(f);
    if tem(&|b| b.atrib.borda != 0) {
        p.push("bordas_de_paragrafo");
    }
    if tem(&|b| !b.mesclagens.is_empty()) {
        p.push("mesclagens");
    }
    if !doc.comentarios.is_empty() {
        p.push("comentarios");
    }
    if doc.blocos.iter().flat_map(|b| trechos_do(&b.conteudo)).any(|t| t.rev.is_some()) {
        p.push("sugestoes");
    }
    if !doc.estilos.lista.is_empty() {
        p.push("estilos_com_nome");
    }
    if tem(&|b| b.atrib.lista != featherdocs_core::modelo::Lista::Padrao || b.atrib.cor_marcador != 0) {
        p.push("marcadores_de_lista");
    }
    if !doc.marcadores.is_empty() {
        p.push("marcadores");
    }
    if tem(&|b| matches!(b.conteudo, Conteudo::QuebraDeColuna | Conteudo::QuebraDeSecao { inicio: featherdocs_core::modelo::InicioDeSecao::PaginaPar })) {
        p.push("quebra_de_coluna");
    }
    // O `.tex` sozinho só cita o nome da imagem; o `.zip` leva o arquivo.
    if formato != "zip" && tem(&|b| matches!(&b.conteudo, Conteudo::Imagem { origem, .. } if origem.starts_with("data:"))) {
        p.push("imagens_fora_do_tex");
    }
    p
}

/// A frase de cada perda, em português (a interface traduz).
pub fn descricao(chave: &str) -> &'static str {
    match chave {
        "comandos_latex" => "Os comandos de LaTeX que o Markdown não tem.",
        "formatacao" => "Cor, tamanho, marca-texto, sublinhado e fonte do texto.",
        "paragrafo" => "Alinhamento, recuos, entrelinha e espaço entre parágrafos.",
        "fundo_e_bordas" => "Fundo e bordas dos parágrafos.",
        "numeracao" => "A numeração automática dos títulos.",
        "legendas" => "As legendas de figuras e tabelas.",
        "tamanho_imagem" => "O tamanho das imagens.",
        "sumario" => "O sumário.",
        "quebra_de_pagina" => "As quebras de página.",
        "indice" => "As entradas do índice remissivo.",
        "pagina" => "Papel, margens, cabeçalho, rodapé e o resto da página.",
        "titulo_e_autor" => "O título e o autor do documento.",
        "bordas_de_paragrafo" => "As bordas dos parágrafos.",
        "mesclagens" => "As células mescladas das tabelas.",
        "comentarios" => "Os comentários.",
        "sugestoes" => "As sugestões do controle de alterações (as inserções ficam no texto, as remoções saem).",
        "estilos_com_nome" => "Os nomes dos estilos (a formatação deles fica).",
        "marcadores_de_lista" => "A forma e a cor dos marcadores de lista.",
        "marcadores" => "Os marcadores (Inserir → Marcador).",
        "quebra_de_coluna" => "As quebras de coluna e de seção em página par (viram quebra de página).",
        "imagens_fora_do_tex" => "As imagens: o .tex só cita o nome do arquivo. Exporte como Projeto LaTeX (.zip) para levá-las junto.",
        _ => "Outros elementos do documento.",
    }
}

/// O que o Markdown não tem; vindo do LaTeX, os comandos também se perdem.
fn perdas_no_markdown(doc: &Documento, do_latex: bool) -> Vec<&'static str> {
    let mut p = if do_latex { vec!["comandos_latex"] } else { Vec::new() };
    let trechos = || doc.blocos.iter().flat_map(|b| trechos_do(&b.conteudo));
    if trechos().any(|t| !(t.fmt.eh_padrao() || t.fmt == Fmt { tamanho: CORPO_DO_LATEX, ..Fmt::default() }) || t.estilo & (SUBLINHADO | SERIFA) != 0) {
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
    fn documento_convencional_para_latex_avisa_o_que_o_codigo_nao_leva() {
        let mut d = crate::markdown::ler("Um.\n\nDois.\n");
        assert!(perdas(&d, Vertente::Convencional, "tex").is_empty());
        d.blocos[0].atrib.borda = 15;
        d.blocos.push(featherdocs_core::modelo::Bloco::novo(9, Conteudo::Imagem { origem: "data:image/png;base64,AA==".into(), legenda: Vec::new(), largura: 0.0, altura: 0.0 }));
        assert_eq!(perdas(&d, Vertente::Convencional, "tex"), ["bordas_de_paragrafo", "imagens_fora_do_tex"]);
        assert_eq!(perdas(&d, Vertente::Convencional, "zip"), ["bordas_de_paragrafo"]);
    }

    #[test]
    fn cada_vertente_salva_nos_seus_formatos() {
        assert_eq!(Vertente::Convencional.formatos(), ["fdoz", "docx"]);
        assert_eq!(Vertente::Latex.formatos()[0], "tex");
        assert_eq!(Vertente::Markdown.formatos()[0], "md");
    }
}
