//! Matrizes e alinhamentos: `matrix`/`pmatrix`/…, `array`, `cases`, `substack` e os ambientes
//! de equação do `amsmath` (`align`, `gather`, `multline`, `split`, `aligned`, `eqnarray`).
//!
//! Medidas do LaTeX (corpo de 10 pt): `\arraycolsep` = 5 pt (1 em entre colunas), `\baselineskip`
//! = 12 pt com o "strut" de 0,7/0,3 dele; nos ambientes do `amsmath` o `\openup\jot` soma 3 pt à
//! entrelinha (15 pt) e ao `\lineskip` (4 pt).

use crate::arvore::{Alinh, Classe, Matriz, Nivel, No, Nucleo, Numero, TipoMatriz};
use crate::caixa::Caixa;
use crate::composicao::{altura_do_delimitador, delimitador_no_eixo, hlista, Ctx};
use crate::fonte::fonte;

const ESPACO_ENTRE_PARES: f32 = 2.0;
const ESPACO_ENTRE_PARES_ALIGNED: f32 = 1.0;
const ESPESSURA_HLINE: f32 = 0.04;

/// Uma linha composta: a posição da linha de base (desde o topo da matriz) e o número dela.
#[derive(Clone, Debug)]
pub struct LinhaComposta {
    pub base: f32,
    pub numero: Numero,
    pub rotulo: Option<String>,
}

fn alinhamento(m: &Matriz, tipo: TipoMatriz, j: usize) -> Alinh {
    match tipo {
        TipoMatriz::Matriz => m.colunas.get(j).copied().unwrap_or(Alinh::Centro),
        TipoMatriz::Alinhado => {
            if j % 2 == 0 {
                Alinh::Dir
            } else {
                Alinh::Esq
            }
        }
        TipoMatriz::Eqnarray => [Alinh::Dir, Alinh::Centro, Alinh::Esq][j.min(2)],
        TipoMatriz::Pilha => m.colunas.get(j).copied().unwrap_or(Alinh::Centro),
        TipoMatriz::Reunido | TipoMatriz::Multlinha => Alinh::Centro,
    }
}

/// O espaço depois da coluna `j` (entre ela e a próxima), em pontos.
fn espaco_depois(m: &Matriz, tipo: TipoMatriz, j: usize, ctx: &Ctx, topo: bool) -> f32 {
    match tipo {
        TipoMatriz::Alinhado => {
            // `alignat`: o espaço entre os pares é o que o autor escreve.
            if j % 2 == 1 && m.sep > 0.0 {
                ctx.base * if topo { ESPACO_ENTRE_PARES } else { ESPACO_ENTRE_PARES_ALIGNED }
            } else {
                0.0
            }
        }
        TipoMatriz::Eqnarray => ctx.base,
        TipoMatriz::Pilha => 0.0,
        _ => m.sep * ctx.base,
    }
}

/// Compõe a matriz; devolve a caixa (centrada no eixo, ou com a linha de base na primeira linha
/// quando `topo`) e as linhas (base de cada uma a partir da linha de base da caixa).
pub fn matriz(m: &Matriz, ctx: &Ctx) -> (Caixa, Vec<LinhaComposta>) {
    compor(m, ctx, false)
}

pub fn compor(m: &Matriz, ctx: &Ctx, topo: bool) -> (Caixa, Vec<LinhaComposta>) {
    let tipo = m.tipo.unwrap_or(TipoMatriz::Matriz);
    let estilo = match tipo {
        TipoMatriz::Matriz => Ctx { nivel: if ctx.nivel <= Nivel::T { Nivel::T } else { ctx.nivel }, ..*ctx },
        TipoMatriz::Pilha => *ctx,
        _ => Ctx { nivel: Nivel::D, ..*ctx },
    };
    let colunas = m.linhas.iter().map(Vec::len).max().unwrap_or(0);
    let celulas: Vec<Vec<Caixa>> = m
        .linhas
        .iter()
        .map(|l| {
            l.iter()
                .enumerate()
                .map(|(j, cel)| {
                    if tipo == TipoMatriz::Alinhado && j % 2 == 1 {
                        // O `{}` que o amsmath põe no começo da coluna da esquerda: o `=` ganha espaço de relação.
                        let mut com_vazio = vec![No::ord(Nucleo::Vazio)];
                        com_vazio.extend(cel.iter().cloned());
                        hlista(&com_vazio, &estilo)
                    } else if tipo == TipoMatriz::Eqnarray && j == 1 {
                        // Coluna do meio do eqnarray: a relação sozinha, sem o espaço de relação.
                        let l: Vec<No> = cel.iter().cloned().map(|mut n| {
                            if n.classe == Classe::Rel {
                                n.classe = Classe::Ord;
                            }
                            n
                        }).collect();
                        hlista(&l, &estilo)
                    } else {
                        hlista(cel, &estilo)
                    }
                })
                .collect()
        })
        .collect();
    let mut larguras = vec![0.0f32; colunas];
    for l in &celulas {
        for (j, c) in l.iter().enumerate() {
            larguras[j] = larguras[j].max(c.w);
        }
    }
    // Alturas e profundidades das linhas.
    let matriz_de_texto = matches!(tipo, TipoMatriz::Matriz | TipoMatriz::Pilha);
    let bs = 1.2 * estilo.tam();
    let (strut_h, strut_d) = if matriz_de_texto { (0.7 * bs, 0.3 * bs) } else { (0.0, 0.0) };
    let alturas: Vec<(f32, f32)> = celulas
        .iter()
        .map(|l| {
            let h = l.iter().map(|c| c.h).fold(strut_h, f32::max);
            let d = l.iter().map(|c| c.d).fold(strut_d, f32::max);
            (h, d)
        })
        .collect();
    // Linhas de base, do topo.
    let mut bases = Vec::with_capacity(alturas.len());
    let mut y = 0.0f32;
    // `\intertext`: o texto entra numa linha própria, encostado à esquerda, com um `\jot` em volta.
    let textos: Vec<(usize, Caixa)> = m.intertextos.iter().map(|(k, l)| (*k, hlista(l, &Ctx { nivel: Nivel::T, ..*ctx }))).collect();
    let mut textos_postos: Vec<(f32, &Caixa)> = Vec::new();
    for (k, (h, d)) in alturas.iter().enumerate() {
        if k == 0 {
            y = *h;
        } else if let Some((_, t)) = textos.iter().find(|(i, _)| *i == k) {
            let jot = 0.3 * ctx.base;
            y += alturas[k - 1].1 + jot + t.h;
            textos_postos.push((y, t));
            y += t.d + jot + h;
        } else {
            let (_, d_ant) = alturas[k - 1];
            let extra = m.extra.get(k - 1).copied().unwrap_or(0.0) * ctx.base;
            let passo = if matriz_de_texto {
                d_ant + h
            } else {
                let (entrelinha, limite, salto) = (1.5 * ctx.base, 0.3 * ctx.base, 0.4 * ctx.base);
                if entrelinha - d_ant - h >= limite {
                    entrelinha
                } else {
                    d_ant + h + salto
                }
            };
            y += passo + extra;
        }
        bases.push(y);
        let _ = d;
    }
    let altura_total = bases.last().copied().unwrap_or(0.0) + alturas.last().map_or(0.0, |a| a.1);
    // Posições x das colunas.
    let mut xs = Vec::with_capacity(colunas);
    let margem = if m.margem { m.sep * ctx.base / 2.0 } else { 0.0 };
    let mut x = margem;
    for (j, w) in larguras.iter().enumerate() {
        xs.push(x);
        x += w;
        if j + 1 < colunas {
            x += espaco_depois(m, tipo, j, ctx, topo);
        }
    }
    let largura = if tipo == TipoMatriz::Multlinha && celulas.len() > 1 { x + 2.0 * ctx.base } else { x + margem };
    let mut cx = Caixa { w: largura, ..Caixa::vazia() };
    for (k, l) in celulas.iter().enumerate() {
        let largura_linha: f32 = if tipo == TipoMatriz::Multlinha { l.first().map_or(0.0, |c| c.w) } else { 0.0 };
        for (j, c) in l.iter().enumerate() {
            let sobra = larguras[j] - c.w;
            let dx = match alinhamento(m, tipo, j) {
                Alinh::Esq => 0.0,
                Alinh::Centro => sobra / 2.0,
                Alinh::Dir => sobra,
            };
            let dx = if tipo == TipoMatriz::Multlinha { multline_x(k, celulas.len(), largura, largura_linha, ctx) } else { dx };
            cx.anexar(c, xs[j] + dx, bases[k]);
        }
    }
    for (yt, t) in &textos_postos {
        cx.anexar(t, 0.0, *yt);
    }
    // Traços do array: `\hline` entre as linhas e `|` entre as colunas.
    let r = ESPESSURA_HLINE * ctx.base;
    for &k in &m.hlines {
        let y = if k == 0 {
            0.0
        } else if k >= bases.len() {
            altura_total
        } else {
            (bases[k - 1] + alturas[k - 1].1 + bases[k] - alturas[k].0) / 2.0
        };
        cx.retangulo(0.0, y - r / 2.0, largura, r, ctx.cor);
    }
    for &j in &m.tracos {
        let xt = if j == 0 {
            0.0
        } else if j >= colunas {
            largura
        } else {
            xs[j] - 0.5 * m.sep * ctx.base
        };
        cx.retangulo(xt - r / 2.0, 0.0, r, altura_total, ctx.cor);
    }
    // A caixa toda: com a linha de base na primeira linha (topo) ou centrada no eixo.
    let desce = if topo {
        -bases.first().copied().unwrap_or(0.0)
    } else {
        let eixo = ctx.pt(fonte().c.axis_height);
        -(altura_total / 2.0 + eixo)
    };
    // Os itens foram postos com y desde o topo: leva para a linha de base escolhida.
    for g in &mut cx.glifos {
        g.y += desce;
    }
    for rr in &mut cx.rets {
        rr.y += desce;
    }
    for t in &mut cx.tracos {
        t.y1 += desce;
        t.y2 += desce;
    }
    cx.h = -desce;
    cx.d = altura_total + desce;
    let linhas = bases
        .iter()
        .enumerate()
        .map(|(k, b)| LinhaComposta { base: b + desce, numero: m.numeros.get(k).cloned().unwrap_or_default(), rotulo: m.rotulos.get(k).cloned().flatten() })
        .collect();
    if m.esq.is_some() || m.dir.is_some() {
        let alvo = altura_do_delimitador(cx.h, cx.d, ctx);
        let mut com = Caixa::vazia();
        com.seguir(&delimitador_no_eixo(m.esq, alvo, ctx));
        if m.esq.is_some() && tipo == TipoMatriz::Matriz && m.dir.is_some() {
            com.seguir(&Caixa::espaco(0.0));
        }
        com.seguir(&cx);
        com.seguir(&delimitador_no_eixo(m.dir, alvo, ctx));
        return (com, linhas);
    }
    (cx, linhas)
}

/// `multline`: a primeira linha à esquerda, a última à direita, as do meio centradas
/// (o `\multlinegap` de 10 pt fica nas pontas).
fn multline_x(k: usize, n: usize, largura: f32, w: f32, ctx: &Ctx) -> f32 {
    let _ = ctx;
    if n == 1 {
        return (largura - w) / 2.0;
    }
    if k == 0 {
        0.0
    } else if k + 1 == n {
        largura - w
    } else {
        (largura - w) / 2.0
    }
}
