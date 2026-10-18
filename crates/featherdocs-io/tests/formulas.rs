//! Fórmulas de ponta a ponta: o `.tex` de exemplo (33 fórmulas: 25 em destaque, 8 em linha) é lido, composto pelo
//! `feather-math` na sessão, numerado, gravado em `.docx` como equação nativa do Word (OMML) e lido
//! de volta com a estrutura (fração, raiz, ∑) preservada.

use featherdocs_core::modelo::{Conteudo, Documento, MATEMATICA};
use featherdocs_core::sessao::Sessao;

fn exemplo() -> Documento {
    let fonte = include_str!("dados/formulas.tex");
    featherdocs_io::latex::ler(fonte)
}

fn equacoes(d: &Documento) -> Vec<&str> {
    d.blocos
        .iter()
        .filter_map(|b| match &b.conteudo {
            Conteudo::Equacao { latex } => Some(latex.as_str()),
            _ => None,
        })
        .collect()
}

fn em_linha(d: &Documento) -> Vec<String> {
    d.blocos
        .iter()
        .filter_map(|b| match &b.conteudo {
            Conteudo::Paragrafo { texto } => Some(texto.iter().filter(|t| t.estilo & MATEMATICA != 0).map(|t| t.texto.clone()).collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn le_as_33_formulas_com_os_ambientes_inteiros() {
    let d = exemplo();
    let eq = equacoes(&d);
    let linha = em_linha(&d);
    assert_eq!((eq.len(), linha.len()), (25, 8), "em destaque: {eq:#?}\nem linha: {linha:#?}");
    assert!(eq.iter().any(|l| l.starts_with("\\begin{align}")), "o align fica inteiro (para numerar e alinhar)");
    assert!(eq.iter().any(|l| l.starts_with("\\begin{multline}")));
    // Números: equation(1) + equation(2) + align com \nonumber (2) + gather (2) + multline (1) + split (1); o \tag não conta.
    let total: usize = eq.iter().map(|l| feather_math::quantos_numeros(l)).sum();
    assert_eq!(total, 8);
}

#[test]
fn sessao_compoe_e_numera_sem_texto_cru() {
    let s = Sessao::com_documento(exemplo());
    assert!(s.total_de_paginas() >= 1);
    let mut todos = Vec::new();
    for p in 0..s.total_de_paginas() {
        todos.extend(s.quadro(p, u64::MAX).unwrap());
    }
    // O número da sexta e da oitava equação saem no quadro (na fonte matemática).
    let tem = |t: &str| todos.windows(t.len()).any(|w| w == t.as_bytes());
    assert!(tem("\\text{(1)}") && tem("\\text{(8)}") && tem("\\text{(Euler)}"));
    assert!(!tem("\\text{(9)}"), "são 8 números automáticos");
}

#[test]
fn docx_grava_omml_e_volta_com_a_estrutura() {
    let d = exemplo();
    let bytes = featherdocs_io::docx::gravar(&d).unwrap();
    let xml = documento_xml(&bytes);
    for elemento in ["<m:f>", "<m:rad>", "<m:nary>", "<m:eqArr>", "<m:d>", "<m:acc>", "<m:m>", "<m:sSup>"] {
        assert!(xml.contains(elemento), "o .docx tem {elemento}");
    }
    let lido = featherdocs_io::docx::ler(&bytes).unwrap();
    let eq = equacoes(&lido);
    assert!(eq.iter().any(|l| l.contains("\\frac{n!}")), "{eq:#?}");
    assert!(eq.iter().any(|l| l.contains("\\sqrt[3]")), "{eq:#?}");
    let linha = em_linha(&lido);
    assert!(linha.iter().any(|l| l == "E=mc^2"), "{linha:#?}");
    // O que volta compõe do mesmo tamanho (mesma estrutura, não texto corrido).
    let original = feather_math::compor("E = mc^2", feather_math::Modo::Linha, 10.0);
    let volta = feather_math::compor("E=mc^2", feather_math::Modo::Linha, 10.0);
    assert!((original.largura - volta.largura).abs() < 0.01);
}

#[test]
fn latex_exportado_mantem_os_ambientes() {
    let d = exemplo();
    let tex = featherdocs_io::exportar::latex(&d);
    assert!(tex.contains("\\begin{align}") && tex.contains("\\begin{equation}"));
    assert!(!tex.contains("\\[\n\\begin{align}"), "ambiente não vai dentro de \\[ \\]");
    let de_novo = featherdocs_io::latex::ler(&tex);
    assert_eq!(equacoes(&de_novo).len(), equacoes(&d).len());
}

fn documento_xml(docx: &[u8]) -> String {
    use std::io::Read;
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut s = String::new();
    z.by_name("word/document.xml").unwrap().read_to_string(&mut s).unwrap();
    s
}
