//! O "texto que se edita" de um bloco: o texto de um parágrafo, título, item,
//! citação, o código, a equação ou **uma célula** de tabela. Os blocos que não têm
//! texto (régua, quebra de página, sumário, imagem) são *atômicos*: o cursor passa
//! por cima deles inteiros.

use crate::modelo::{Conteudo, Trecho};

/// Uma célula de tabela: `(linha, coluna)`; `None` fora de tabela.
pub type Cel = Option<(u32, u32)>;

/// O bloco não tem texto: o cursor o trata como uma coisa só.
pub fn eh_atomico(c: &Conteudo) -> bool {
    matches!(c, Conteudo::Regua | Conteudo::QuebraDePagina | Conteudo::Sumario { .. } | Conteudo::Imagem { .. } | Conteudo::Bibliografia { .. } | Conteudo::IndiceRemissivo)
}

/// Código e equação guardam texto cru (sem formatação) e aceitam quebra de linha.
pub fn eh_texto_cru(c: &Conteudo) -> bool {
    matches!(c, Conteudo::Codigo { .. } | Conteudo::Equacao { .. })
}

/// Quantas linhas e colunas tem a tabela (colunas = a linha mais larga).
pub fn dimensoes(c: &Conteudo) -> Option<(usize, usize)> {
    match c {
        Conteudo::Tabela { linhas, .. } => Some((linhas.len(), linhas.iter().map(Vec::len).max().unwrap_or(1).max(1))),
        _ => None,
    }
}

/// Quantos caracteres tem o texto do alvo (0 se não tem texto).
pub fn tamanho_alvo(c: &Conteudo, cel: Cel) -> usize {
    match (c, cel) {
        (Conteudo::Titulo { texto, .. } | Conteudo::Paragrafo { texto } | Conteudo::Item { texto, .. } | Conteudo::Citacao { texto }, _) => crate::texto::tamanho(texto),
        (Conteudo::Codigo { codigo, .. }, _) => codigo.chars().count(),
        (Conteudo::Equacao { latex }, _) => latex.chars().count(),
        (Conteudo::Tabela { linhas, .. }, Some((l, k))) => linhas.get(l as usize).and_then(|r| r.get(k as usize)).map_or(0, |c| crate::texto::tamanho(c)),
        _ => 0,
    }
}

/// Os caracteres do alvo (para achar palavra e medir).
pub fn caracteres_alvo(c: &Conteudo, cel: Cel) -> Vec<char> {
    texto_alvo(c, cel).map(|t| crate::texto::plano(&t).chars().collect()).unwrap_or_default()
}

/// Uma cópia do texto do alvo.
pub fn texto_alvo(c: &Conteudo, cel: Cel) -> Option<Vec<Trecho>> {
    match (c, cel) {
        (Conteudo::Titulo { texto, .. } | Conteudo::Paragrafo { texto } | Conteudo::Item { texto, .. } | Conteudo::Citacao { texto }, _) => Some(texto.clone()),
        (Conteudo::Codigo { codigo, .. }, _) => Some(vec![Trecho::simples(codigo)]),
        (Conteudo::Equacao { latex }, _) => Some(vec![Trecho::simples(latex)]),
        (Conteudo::Tabela { linhas, .. }, Some((l, k))) => Some(linhas.get(l as usize).and_then(|r| r.get(k as usize)).cloned().unwrap_or_default()),
        _ => None,
    }
}

/// Edita o texto do alvo no lugar. Código e equação (texto cru) passam por uma cópia com formatação e voltam
/// como texto puro. Uma célula que falta na linha é criada.
pub fn com_texto_ref<R>(c: &mut Conteudo, cel: Cel, f: impl FnOnce(&mut Vec<Trecho>) -> R) -> Option<R> {
    match (c, cel) {
        (Conteudo::Titulo { texto, .. } | Conteudo::Paragrafo { texto } | Conteudo::Item { texto, .. } | Conteudo::Citacao { texto }, _) => Some(f(texto)),
        (Conteudo::Codigo { codigo: cru, .. } | Conteudo::Equacao { latex: cru }, _) => {
            let mut t = vec![Trecho::simples(cru)];
            let r = f(&mut t);
            *cru = crate::texto::plano(&t);
            Some(r)
        }
        (Conteudo::Tabela { linhas, .. }, Some((l, k))) => {
            let linha = linhas.get_mut(l as usize)?;
            while linha.len() <= k as usize {
                linha.push(Vec::new());
            }
            Some(f(&mut linha[k as usize]))
        }
        _ => None,
    }
}

/// Troca o texto do alvo. `false` se o bloco não tem esse alvo.
pub fn definir_texto(c: &mut Conteudo, cel: Cel, t: Vec<Trecho>) -> bool {
    match (c, cel) {
        (Conteudo::Titulo { texto, .. } | Conteudo::Paragrafo { texto } | Conteudo::Item { texto, .. } | Conteudo::Citacao { texto }, _) => *texto = t,
        (Conteudo::Codigo { codigo, .. }, _) => *codigo = crate::texto::plano(&t),
        (Conteudo::Equacao { latex }, _) => *latex = crate::texto::plano(&t),
        (Conteudo::Tabela { linhas, .. }, Some((l, k))) => {
            let (l, k) = (l as usize, k as usize);
            let Some(linha) = linhas.get_mut(l) else { return false };
            while linha.len() <= k {
                linha.push(Vec::new());
            }
            linha[k] = t;
        }
        _ => return false,
    }
    true
}

/// O alvo de uma posição, no começo do bloco: o texto, ou a primeira célula da tabela, ou nada (atômico).
pub fn primeiro_alvo(c: &Conteudo) -> Option<Cel> {
    if eh_atomico(c) {
        None
    } else if matches!(c, Conteudo::Tabela { .. }) {
        Some(Some((0, 0)))
    } else {
        Some(None)
    }
}

/// O alvo do fim do bloco: o texto, ou a última célula da tabela.
pub fn ultimo_alvo(c: &Conteudo) -> Option<Cel> {
    if eh_atomico(c) {
        return None;
    }
    match dimensoes(c) {
        Some((l, k)) => Some(Some((l as u32 - 1, k as u32 - 1))),
        None => Some(None),
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::inline::analisar;

    #[test]
    fn texto_de_celula_e_de_codigo() {
        let mut t = Conteudo::Tabela { cabecalho: false, linhas: vec![vec![analisar("a"), analisar("bc")]] };
        assert_eq!(tamanho_alvo(&t, Some((0, 1))), 2);
        assert!(definir_texto(&mut t, Some((0, 3)), analisar("x")), "célula que faltava é criada");
        assert_eq!(dimensoes(&t), Some((1, 4)));
        let mut c = Conteudo::Codigo { linguagem: String::new(), codigo: "a\nb".into() };
        assert_eq!(tamanho_alvo(&c, None), 3);
        assert!(definir_texto(&mut c, None, analisar("zz")));
        assert_eq!(texto_alvo(&c, None).unwrap()[0].texto, "zz");
        assert!(!definir_texto(&mut Conteudo::Regua, None, vec![]));
    }
}
