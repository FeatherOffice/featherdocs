//! A quebra do parágrafo justificado como o TeX faz (Knuth e Plass): olha o parágrafo inteiro e escolhe
//! as quebras de menor custo, em vez de encher linha a linha. Os números são os do LaTeX:
//! `\tolerance` 200 (`\pretolerance` 100 na volta sem hifenizar), `\linepenalty` 10,
//! `\hyphenpenalty` 50, `\adjdemerits` e `\doublehyphendemerits` 10000, `\finalhyphendemerits` 5000.
//! O espaço estica metade e encolhe um terço (a métrica do TeX Gyre Heros e da Computer Modern); com o
//! `\nonfrenchspacing` (o do português no babel) o espaço depois do ponto é maior.

use super::{largura, Ficha};

const INFINITO: f32 = 1.0e6;

enum Item {
    Caixa { w: f32 },
    Cola { w: f32, estica: f32, encolhe: f32 },
    Pena { w: f32, custo: f32, marcada: bool },
}

#[derive(Clone, Copy)]
struct Nodo {
    /// O item da quebra (o primeiro nodo é o começo do parágrafo, `usize::MAX`).
    pos: usize,
    linha: usize,
    classe: u8,
    demeritos: f32,
    anterior: Option<usize>,
    marcada: bool,
    /// As somas até o começo da linha seguinte (o espaço depois da quebra some).
    w: f32,
    y: f32,
    z: f32,
}

/// O fator de espaço do TeX depois do texto `t` (`\nonfrenchspacing`): 3000 depois de `.?!`, 2000 de
/// `:`, 1500 de `;`, 1250 de `,`; a letra maiúscula antes do ponto o deixa em 1000 ("UML.").
fn fator_de_espaco(t: &str) -> f32 {
    // O fecha-parênteses e as aspas não mudam o fator (`\sfcode` 0): o que vale é o de antes deles.
    let fecha = |c: &char| matches!(c, ')' | ']' | '\'' | '"' | '”' | '’');
    let mut cs = t.chars().rev().skip_while(fecha).peekable();
    let fator = match cs.next() {
        Some('.' | '?' | '!') => 3000.0,
        Some(':') => 2000.0,
        Some(';') => 1500.0,
        Some(',') => 1250.0,
        _ => return 1000.0,
    };
    if fator > 1250.0 && cs.skip_while(fecha).next().is_some_and(char::is_uppercase) {
        1000.0
    } else {
        fator
    }
}

/// As linhas do trecho `fichas` (sem tabulação nem quebra forçada no meio), ou `None` se nenhuma
/// escolha cabe na tolerância (aí a quebra linha a linha resolve). `primeira` é a largura da primeira.
pub(super) fn quebrar(fichas: &mut [Ficha], primeira: f32, limite: f32) -> Option<Vec<Vec<Ficha>>> {
    // O espaço depois do ponto: mais largo, estica mais e encolhe menos.
    let mut fatores = vec![1000.0; fichas.len()];
    for k in 1..fichas.len() {
        if fichas[k].espaco && !fichas[k - 1].espaco {
            fatores[k] = fator_de_espaco(&fichas[k - 1].texto);
            if fatores[k] >= 2000.0 {
                fichas[k].largura *= 1.0 + 0.093 / 0.278;
            }
        }
    }
    [false, true].into_iter().find_map(|hifenizar| tentar(fichas, &fatores, primeira, limite, hifenizar))
}

/// Uma volta: sem hifenizar com a `\pretolerance`, hifenizando com a `\tolerance`.
fn tentar(fichas: &[Ficha], fatores: &[f32], primeira: f32, limite: f32, hifenizar: bool) -> Option<Vec<Vec<Ficha>>> {
    let tolerancia = if hifenizar { 200.0 } else { 100.0 };
    // Os itens e, de cada um, a ficha e o pedaço dela (em caracteres).
    let mut itens: Vec<(Item, usize, usize, usize)> = Vec::new();
    let mut comeco = true;
    for (k, f) in fichas.iter().enumerate() {
        let n = f.texto.chars().count();
        if f.espaco && !comeco {
            let (s, e) = (fatores[k] / 1000.0, f.largura);
            let base = if fatores[k] >= 2000.0 { e / (1.0 + 0.093 / 0.278) } else { e };
            itens.push((Item::Cola { w: e, estica: base * 0.5 * s, encolhe: base / 3.0 / s }, k, 0, n));
            continue;
        }
        comeco = false;
        let pontos = if hifenizar && !f.espaco && !f.atomo && !f.formula && f.marca.estilo & crate::modelo::CODIGO == 0 { crate::silabas::pontos(&f.texto) } else { Vec::new() };
        let cs: Vec<char> = f.texto.chars().collect();
        let hifen = largura("-", f.marca.estilo, f.marca.tamanho);
        let mut de = 0;
        for p in pontos.into_iter().chain(std::iter::once(n)) {
            let w = if p == n && de == 0 { f.largura } else { largura(&cs[de..p].iter().collect::<String>(), f.marca.estilo, f.marca.tamanho) };
            itens.push((Item::Caixa { w }, k, de, p));
            if p < n {
                itens.push((Item::Pena { w: hifen, custo: 50.0, marcada: true }, k, p, p));
            }
            de = p;
        }
    }
    // O fim do parágrafo: a cola que estica sem fim (`\parfillskip`) e a quebra obrigatória.
    let fim = fichas.len();
    itens.push((Item::Cola { w: 0.0, estica: INFINITO, encolhe: 0.0 }, fim, 0, 0));
    itens.push((Item::Pena { w: 0.0, custo: -INFINITO, marcada: false }, fim, 0, 0));

    // As somas antes de cada item.
    let (mut sw, mut sy, mut sz) = (vec![0.0f32; itens.len() + 1], vec![0.0f32; itens.len() + 1], vec![0.0f32; itens.len() + 1]);
    for (i, (it, ..)) in itens.iter().enumerate() {
        let (w, y, z) = match it {
            Item::Caixa { w } => (*w, 0.0, 0.0),
            Item::Cola { w, estica, encolhe } => (*w, *estica, *encolhe),
            Item::Pena { .. } => (0.0, 0.0, 0.0),
        };
        sw[i + 1] = sw[i] + w;
        sy[i + 1] = sy[i] + y;
        sz[i + 1] = sz[i] + z;
    }
    let depois_da_quebra = |b: usize| {
        let (mut w, mut y, mut z) = (sw[b], sy[b], sz[b]);
        for (i, (it, ..)) in itens.iter().enumerate().skip(b) {
            match it {
                Item::Caixa { .. } => break,
                Item::Cola { w: a, estica, encolhe } => {
                    w += a;
                    y += estica;
                    z += encolhe;
                }
                Item::Pena { custo, .. } if *custo <= -INFINITO && i > b => break,
                Item::Pena { .. } => {}
            }
        }
        (w, y, z)
    };

    let mut nodos = vec![Nodo { pos: usize::MAX, linha: 0, classe: 1, demeritos: 0.0, anterior: None, marcada: false, w: 0.0, y: 0.0, z: 0.0 }];
    let mut ativos: Vec<usize> = vec![0];
    for b in 0..itens.len() {
        let (pena, custo, marcada) = match &itens[b].0 {
            Item::Cola { .. } if b > 0 && matches!(itens[b - 1].0, Item::Caixa { .. }) => (0.0, 0.0, false),
            Item::Pena { w, custo, marcada } if *custo < INFINITO => (*w, *custo, *marcada),
            _ => continue,
        };
        let forcada = custo <= -INFINITO;
        // O melhor candidato de cada classe de ajuste (apertada, normal, folgada, muito folgada).
        let mut melhores: [Option<(f32, usize, f32)>; 4] = [None; 4];
        ativos.retain(|&a| {
            let n = nodos[a];
            let disponivel = if n.linha == 0 { primeira } else { limite };
            let l = sw[b] - n.w + pena;
            let r = if l < disponivel {
                let y = sy[b] - n.y;
                if y > 0.0 { (disponivel - l) / y } else { INFINITO }
            } else if l > disponivel {
                let z = sz[b] - n.z;
                if z > 0.0 { (disponivel - l) / z } else { -INFINITO }
            } else {
                0.0
            };
            if r >= -1.0 {
                let ruim = if r >= INFINITO { 10000.0 } else { (100.0 * r.abs().powi(3)).min(10000.0) };
                if ruim <= tolerancia {
                    let mut d = (10.0 + ruim).powi(2);
                    if custo >= 0.0 {
                        d += custo * custo;
                    } else if custo > -INFINITO {
                        d -= custo * custo;
                    }
                    if marcada && n.marcada {
                        d += 10000.0;
                    }
                    if forcada && n.marcada {
                        d += 5000.0;
                    }
                    let classe: u8 = if r < -0.5 { 0 } else if r <= 0.5 { 1 } else if r <= 1.0 { 2 } else { 3 };
                    if (i32::from(classe) - i32::from(n.classe)).abs() > 1 {
                        d += 10000.0;
                    }
                    let total = n.demeritos + d;
                    if melhores[usize::from(classe)].is_none_or(|m| total < m.0) {
                        melhores[usize::from(classe)] = Some((total, a, r));
                    }
                }
            }
            // A linha que já passou do encolhimento máximo não volta a caber; na quebra obrigatória tudo acaba.
            r >= -1.0 && !forcada
        });
        let (w, y, z) = depois_da_quebra(b);
        for (classe, m) in melhores.iter().enumerate() {
            if let Some((total, a, _)) = *m {
                nodos.push(Nodo { pos: b, linha: nodos[a].linha + 1, classe: classe as u8, demeritos: total, anterior: Some(a), marcada, w, y, z });
                ativos.push(nodos.len() - 1);
            }
        }
        if ativos.is_empty() {
            return None;
        }
    }
    let mut n = *ativos.iter().filter(|&&a| nodos[a].pos == itens.len() - 1).min_by(|&&a, &&b| nodos[a].demeritos.total_cmp(&nodos[b].demeritos))?;
    let mut quebras = Vec::new();
    while let Some(a) = nodos[n].anterior {
        quebras.push(nodos[n].pos);
        n = a;
    }
    quebras.reverse();
    Some(montar(fichas, &itens, &quebras))
}

/// As linhas de fichas pelas quebras escolhidas: o espaço da quebra fica no fim da linha (como na quebra
/// linha a linha) e a palavra partida vira dois pedaços, o primeiro com o hífen.
fn montar(fichas: &[Ficha], itens: &[(Item, usize, usize, usize)], quebras: &[usize]) -> Vec<Vec<Ficha>> {
    let mut linhas = Vec::new();
    let mut de = 0;
    for (q, &b) in quebras.iter().enumerate() {
        // A linha vai até a quebra; o espaço da quebra e os seguintes vão junto.
        let mut ate = b;
        if matches!(itens[b].0, Item::Cola { .. }) {
            while ate < itens.len() && matches!(itens[ate].0, Item::Cola { .. }) && itens[ate].1 < fichas.len() {
                ate += 1;
            }
        }
        let hifen = matches!(itens[b].0, Item::Pena { marcada: true, .. });
        // Os pedaços de cada ficha nesta linha (as caixas seguidas da mesma palavra se juntam).
        let mut partes: Vec<(usize, usize, usize)> = Vec::new();
        for &(ref it, k, a, z) in &itens[de..ate] {
            if k >= fichas.len() || matches!(it, Item::Pena { .. }) {
                continue;
            }
            match partes.last_mut() {
                Some(p) if p.0 == k && p.2 == a => p.2 = z,
                _ => partes.push((k, a, z)),
            }
        }
        let mut linha: Vec<Ficha> = partes
            .into_iter()
            .map(|(k, a, z)| {
                let f = &fichas[k];
                if a == 0 && z == f.texto.chars().count() {
                    f.clone()
                } else {
                    let cs: Vec<char> = f.texto.chars().collect();
                    super::ficha(&cs[a..z].iter().collect::<String>(), f.marca, false, f.de + a)
                }
            })
            .collect();
        if hifen {
            if let Some(f) = linha.last_mut() {
                f.hifen = true;
            }
        }
        if !linha.is_empty() || q + 1 < quebras.len() {
            linhas.push(linha);
        }
        de = ate;
    }
    linhas
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn espaco_depois_do_ponto_menos_depois_de_sigla() {
        assert_eq!(fator_de_espaco("herança."), 3000.0);
        assert_eq!(fator_de_espaco("(UML)."), 1000.0);
        assert_eq!(fator_de_espaco("classes,"), 1250.0);
        assert_eq!(fator_de_espaco("objeto"), 1000.0);
    }
}
