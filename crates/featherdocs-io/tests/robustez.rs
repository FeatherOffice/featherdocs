//! Arquivo malformado dá erro ou documento, nunca `panic` nem laço infinito.

use featherdocs_io::{abrir, Formato};

/// Gerador pseudoaleatório fixo (reproduzível), sem dependência.
struct Sorteio(u64);

impl Sorteio {
    fn proximo(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n
    }
}

const PEDACOS_TEX: [&str; 24] = [
    "\\section{", "}", "{", "\\begin{itemize}", "\\end{itemize}", "\\item ", "\\textbf{", "$", "$$", "\\[", "\\]", "\\\\", "&", "%", "\\", "texto ", "\n\n", "\\begin{tabular}{ll}", "\\end{tabular}", "\\'", "\\href{", "~", "--", "é",
];
const PEDACOS_MD: [&str; 22] = ["# ", "- ", "1. ", "> ", "```", "**", "*", "`", "[", "](", ")", "|", "---", "$", "$$", "![", "\n", "\n\n", "texto ", "  ", "~~", "é"];

fn sorteado(pedacos: &[&str], semente: u64, n: usize) -> String {
    let mut s = Sorteio(semente);
    (0..n).map(|_| pedacos[s.proximo(pedacos.len())]).collect()
}

#[test]
fn latex_aleatorio_nao_quebra() {
    for semente in 0..300 {
        let fonte = sorteado(&PEDACOS_TEX, semente, 60);
        let _ = abrir(Formato::Latex, fonte.as_bytes());
    }
}

#[test]
fn markdown_aleatorio_nao_quebra() {
    for semente in 0..300 {
        let fonte = sorteado(&PEDACOS_MD, semente, 60);
        let _ = abrir(Formato::Markdown, fonte.as_bytes());
    }
}

#[test]
fn bytes_aleatorios_nao_quebram_nenhum_leitor() {
    for semente in 0..100 {
        let mut s = Sorteio(semente);
        let bytes: Vec<u8> = (0..200).map(|_| s.proximo(256) as u8).collect();
        for f in [Formato::Fdoz, Formato::Docx, Formato::Markdown, Formato::Latex] {
            let _ = abrir(f, &bytes);
        }
    }
}

#[test]
fn texto_que_nao_e_utf8_da_erro_claro() {
    let erro = abrir(Formato::Markdown, &[0xff, 0xfe, 0x41]).unwrap_err();
    assert!(erro.contains("UTF-8"));
}
