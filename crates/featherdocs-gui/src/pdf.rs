//! Exportar PDF: as mesmas operações de página que a tela desenha, com as fontes padrão do PDF
//! (Helvetica, Times, Courier). O motor quebra linha com as métricas AFM dessas fontes, então o
//! texto cai no mesmo lugar sem embutir fonte.
use crate::quadro::{Operacao, Quadro};
use feather_draw::pdf::{DocumentoPdf, FontePdf};
use feather_draw::Cor;
use featherdocs_core::medida;
use featherdocs_core::modelo::{CODIGO, ITALICO, MATEMATICA, NEGRITO, SERIFA, SUBLINHADO, TACHADO};

fn cor(c: u32) -> Cor {
    Cor::rgb((c >> 24) as u8, (c >> 16) as u8, (c >> 8) as u8)
}

fn fonte(estilo: u8) -> FontePdf {
    use FontePdf::*;
    let (n, i) = (estilo & NEGRITO != 0, estilo & ITALICO != 0);
    let familia = if estilo & CODIGO != 0 {
        [Mono, MonoNegrito, MonoItalico, MonoNegritoItalico]
    } else if estilo & SERIFA != 0 {
        [Serifa, SerifaNegrito, SerifaItalico, SerifaNegritoItalico]
    } else {
        [Helvetica, Negrito, Italico, NegritoItalico]
    };
    familia[usize::from(n) + 2 * usize::from(i)]
}

pub fn gerar(quadros: &[Quadro]) -> Result<Vec<u8>, String> {
    let mut doc = DocumentoPdf::novo();
    // Caractere fora do WinAnsi (grego, símbolos) vira '?' em vez de falhar o arquivo todo.
    doc.substituir_nao_suportados(true);
    for q in quadros {
        let p = doc.adicionar_pagina(q.largura, q.altura);
        for o in &q.operacoes {
            match o {
                Operacao::Retangulo { rect: [x, y, w, h], cor: c } => {
                    if c & 0xff != 0 && *w > 0.0 && *h > 0.0 {
                        p.retangulo_preenchido(*x, *y, *w, *h, cor(*c));
                    }
                }
                Operacao::Traco { pontos: [x, y, a, b, largura], cor: c } => {
                    if c & 0xff != 0 && *largura > 0.0 {
                        p.linha(*x, *y, *a, *b, cor(*c), *largura, false);
                    }
                }
                Operacao::Imagem { rect: [x, y, w, h], origem } => {
                    if let Some((rgb, pw, ph)) = rgb_da_imagem(origem) {
                        p.imagem_rgb(*x, *y, *w, *h, rgb, pw, ph);
                    }
                }
                // shortcut: fórmulas usam os glifos da Latin Modern Math, que o PDF base não tem; `aviso` as conta.
                Operacao::Texto { estilo, .. } if estilo & MATEMATICA != 0 => {}
                Operacao::Texto { x, y, tamanho, estilo, cor: c, texto, angulo: Some(a) } => {
                    // O mesmo ponto de partida da tela: centrado em (x, y), girado `a` graus.
                    let (ox, oy) = (-medida::largura(texto, *estilo, *tamanho) / 2., tamanho * 0.3);
                    let (sen, cos) = (-a).to_radians().sin_cos();
                    p.texto_girado(x + ox * cos - oy * sen, y + ox * sen + oy * cos, *a, texto, fonte(*estilo), *tamanho, cor(*c));
                }
                Operacao::Texto { x, y, tamanho, estilo, cor: c, texto, angulo: None } => {
                    p.texto(*x, *y, texto, fonte(*estilo), *tamanho, cor(*c));
                    let w = medida::largura(texto, *estilo, *tamanho);
                    for (bit, dy) in [(SUBLINHADO, 0.12), (TACHADO, -0.3)] {
                        if estilo & bit != 0 {
                            let yy = y + tamanho * dy;
                            p.linha(*x, yy, x + w, yy, cor(*c), (tamanho / 16.).max(0.5), false);
                        }
                    }
                }
            }
        }
    }
    doc.finalizar().map_err(|e| format!("Não consegui gerar o PDF: {e:?}"))
}

/// Os pixels de uma imagem embutida em RGB sobre fundo branco (o PDF base não tem transparência).
fn rgb_da_imagem(origem: &str) -> Option<(Vec<u8>, u32, u32)> {
    if origem.starts_with("data:image/svg+xml;base64,") {
        let p = crate::pagina::pixmap_svg(origem)?;
        // Pixmap vem com alfa pré-multiplicado: sobre branco é cor + (255 − alfa).
        let rgb = p.data().chunks_exact(4).flat_map(|c| [0, 1, 2].map(|i| c[i].saturating_add(255 - c[3]))).collect();
        return Some((rgb, p.width(), p.height()));
    }
    let (w, h, rgba) = crate::imagem_compartilhada::preparar_previa(origem)?;
    let sobre_branco = |c: u8, a: u8| ((u16::from(c) * u16::from(a) + 255 * u16::from(255 - a)) / 255) as u8;
    let rgb = rgba.chunks_exact(4).flat_map(|c| [0, 1, 2].map(|i| sobre_branco(c[i], c[3]))).collect();
    Some((rgb, w, h))
}

/// O que ainda não sai no PDF (fórmulas, imagens que não abriram), para avisar a pessoa.
pub fn aviso(quadros: &[Quadro]) -> Option<String> {
    let (mut imagens, mut formulas) = (0, 0);
    for o in quadros.iter().flat_map(|q| &q.operacoes) {
        match o {
            Operacao::Imagem { origem, .. } if rgb_da_imagem(origem).is_none() => imagens += 1,
            Operacao::Texto { estilo, .. } if estilo & MATEMATICA != 0 => formulas += 1,
            _ => {}
        }
    }
    let faltas: Vec<String> = [(imagens, "imagem(ns) que não abriram"), (formulas, "trecho(s) de fórmula")]
        .iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, o)| format!("{n} {o}"))
        .collect();
    (!faltas.is_empty()).then(|| format!("PDF exportado sem: {}", faltas.join(", ")))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn texto_sai_com_a_fonte_da_familia_e_imagem_vira_aviso() {
        let q = Quadro {
            largura: 200.0,
            altura: 100.0,
            operacoes: vec![
                Operacao::Texto { x: 10.0, y: 20.0, tamanho: 12.0, estilo: SERIFA | NEGRITO, cor: 0x000000ff, texto: "Ação".into(), angulo: None },
                Operacao::Texto { x: 10.0, y: 40.0, tamanho: 10.0, estilo: CODIGO | SUBLINHADO, cor: 0x000000ff, texto: "x".into(), angulo: None },
                Operacao::Imagem { rect: [0.0, 0.0, 10.0, 10.0], origem: String::new() },
                Operacao::Texto { x: 100.0, y: 50.0, tamanho: 40.0, estilo: 0, cor: 0x888888ff, texto: "RASCUNHO".into(), angulo: Some(45.0) },
            ],
        };
        let q = [q];
        let (pdf, aviso) = (gerar(&q).unwrap(), aviso(&q));
        let s = String::from_utf8_lossy(&pdf);
        assert!(s.starts_with("%PDF"));
        assert!(s.contains("/BaseFont /Times-Bold") && s.contains("/BaseFont /Courier"));
        assert!(aviso.unwrap().contains("1 imagem"));
        assert!(s.contains("0.7071 0.7071 -0.7071 0.7071"), "marca d'água girada 45°");
    }
}
