//! Tabelas: `booktabs`, `\multicolumn`, `\multirow`, `\cline`, `longtable` e as cores de linha.

use std::collections::HashMap;

use featherdocs_core::modelo::{Celula, MesclagemTabela};

use super::avisos::TipoDeAviso;
use super::{comando, dividir_celulas, grupo, inline_em, opcional, Ctx};

/// Tira `\nome[…]{…}` (com `args` argumentos `{…}`) de todo o texto; devolve se achou algum.
fn sem_comando(s: &mut String, nome: &str, args: usize) -> bool {
    let alvo = format!("\\{nome}");
    let mut achou = false;
    let mut i = 0;
    while let Some(p) = s[i..].find(&alvo) {
        let ini = i + p;
        let mut j = ini + alvo.len();
        if s[j..].starts_with(|c: char| c.is_ascii_alphabetic()) {
            i = j;
            continue;
        }
        // O `(lr)` do `\cmidrule(lr){2-3}`.
        if s[j..].trim_start().starts_with('(') {
            j += s[j..].find(')').map_or(0, |k| k + 1);
        }
        while let Some((_, k)) = opcional(&s[j..]) {
            j += k;
        }
        for _ in 0..args {
            j += grupo(&s[j..]).map_or(0, |g| g.1);
        }
        s.replace_range(ini..j, "");
        achou = true;
        i = ini;
    }
    achou
}

/// O texto de uma célula e quantas colunas ela ocupa: `\multicolumn{2}{c|}{texto}` ocupa 2;
/// `\multirow{2}{*}{texto}` fica com o texto na primeira linha.
/// shortcut: o FeatherDocs só mescla na horizontal; a célula do `\multirow` não desce.
fn celula(src: &str) -> (&str, usize) {
    let t = src.trim();
    if !t.starts_with('\\') {
        return (src, 1);
    }
    let (nome, n) = comando(t);
    let mut r = &t[n..];
    let pular = |r: &mut &str| {
        while let Some((_, k)) = opcional(r) {
            *r = &r[k..];
        }
    };
    match nome {
        "multicolumn" => {
            let Some((cols, a)) = grupo(r) else { return (src, 1) };
            r = &r[a..];
            let Some((_, b)) = grupo(r) else { return (src, 1) };
            let Some((texto, _)) = grupo(&r[b..]) else { return (src, 1) };
            let (dentro, _) = celula(texto);
            (dentro, cols.trim().parse().unwrap_or(1).max(1))
        }
        "multirow" => {
            pular(&mut r);
            let Some((_, a)) = grupo(r) else { return (src, 1) };
            r = &r[a..];
            pular(&mut r);
            let Some((_, b)) = grupo(r) else { return (src, 1) };
            r = &r[b..];
            pular(&mut r);
            grupo(r).map_or((src, 1), |(texto, _)| (texto, 1))
        }
        _ => (src, 1),
    }
}

/// Os fios de um pedaço do `tabular` (entre dois `\\`), na ordem: `T`, `M`, `B`, `H`, `C2-3`, `c2-3`
/// (o formato de `Bloco::latex`).
fn fios_de(pedaco: &str) -> Vec<String> {
    let mut fios = Vec::new();
    for (p, _) in pedaco.match_indices('\\') {
        let (nome, n) = comando(&pedaco[p..]);
        let mut r = &pedaco[p + n..];
        let curto = match nome {
            "toprule" => "T",
            "midrule" => "M",
            "bottomrule" => "B",
            "hline" => "H",
            "cline" => "C",
            "cmidrule" => "c",
            _ => continue,
        };
        if matches!(curto, "C" | "c") {
            // O `(lr)` e o `[…]` do `\cmidrule(lr)[1pt]{2-3}`.
            if r.trim_start().starts_with('(') {
                r = &r[r.find(')').map_or(0, |k| k + 1)..];
            }
            while let Some((_, k)) = opcional(r) {
                r = &r[k..];
            }
            let Some((faixa, _)) = grupo(r) else { continue };
            fios.push(format!("{curto}{}", faixa.trim()));
        } else {
            fios.push(curto.to_string());
        }
    }
    fios
}

/// Linhas (`\\`) e células (`&`) de um `tabular`, sem as réguas, as mesclagens do `\multicolumn` e os
/// fios de cada divisa entre linhas (uma a mais que as linhas).
pub(super) fn linhas_de_tabela(corpo: &str, ctx: &Ctx) -> (Vec<Vec<Celula>>, Vec<MesclagemTabela>, Vec<String>) {
    let mut fios: Vec<Vec<String>> = vec![Vec::new()];
    let mut limpo = corpo.to_string();
    for regua in ["\\hline", "\\toprule", "\\midrule", "\\bottomrule", "\\endfirsthead", "\\endhead", "\\endfoot", "\\endlastfoot", "\\hfill"] {
        limpo = limpo.replace(regua, "");
    }
    for (nome, args) in [("cline", 1), ("cmidrule", 1), ("addlinespace", 0), ("specialrule", 3)] {
        sem_comando(&mut limpo, nome, args);
    }
    // A cor de uma linha ou de uma célula só ainda não é desenhada.
    for nome in ["rowcolor", "cellcolor"] {
        if sem_comando(&mut limpo, nome, 1) {
            ctx.avisos.anotar(TipoDeAviso::SemEfeito, &format!("\\{nome}"));
        }
    }
    let mut mesclagens = Vec::new();
    let mut linhas = Vec::new();
    // Os fios vêm do código ainda com as réguas, pedaço a pedaço (os mesmos pedaços que o texto limpo).
    let crus: Vec<&str> = corpo.split("\\\\").collect();
    for (k, l) in limpo.split("\\\\").enumerate() {
        if let Some(f) = fios.last_mut() {
            f.extend(crus.get(k).map(|c| fios_de(c)).unwrap_or_default());
        }
        // `\\[2pt]`: a medida é do espaço entre as linhas, não da célula seguinte.
        let l = opcional(l).filter(|_| l.trim_start().starts_with('[')).map_or(l, |(_, k)| &l[k..]);
        let l = l.strip_prefix('*').unwrap_or(l);
        if l.trim().is_empty() {
            continue;
        }
        let mut celulas = Vec::new();
        for c in dividir_celulas(l) {
            let (texto, colunas) = celula(&c);
            if colunas > 1 {
                mesclagens.push(MesclagemTabela { linha: linhas.len(), coluna: celulas.len(), colunas });
            }
            celulas.push(inline_em(texto, ctx, &mut ctx.estado.clone()));
            celulas.extend(std::iter::repeat_with(Vec::new).take(colunas - 1));
        }
        linhas.push(celulas);
        fios.push(Vec::new());
    }
    (linhas, mesclagens, fios.iter().map(|f| f.join(",")).collect())
}

/// A `longtable` repete cabeçalho e pé em cada página (o FeatherDocs não repete): fica o primeiro
/// cabeçalho (`\endfirsthead`, senão `\endhead`) no começo e o último pé (`\endlastfoot`, senão
/// `\endfoot`) no fim, depois das linhas.
pub(super) fn sem_cabecalho_repetido(corpo: &str) -> std::borrow::Cow<'_, str> {
    const MARCAS: [&str; 4] = ["\\endfirsthead", "\\endhead", "\\endfoot", "\\endlastfoot"];
    let mut partes: HashMap<&str, &str> = HashMap::new();
    let mut ini = 0;
    // `\endfoot` não pode ser achado dentro de `\endfirsthead` nem o contrário: os nomes não se contêm.
    while let Some((p, m)) = MARCAS.iter().filter_map(|m| corpo[ini..].find(m).map(|p| (p, *m))).min() {
        partes.insert(m, &corpo[ini..ini + p]);
        ini += p + m.len();
    }
    if partes.is_empty() {
        return corpo.into();
    }
    let cabeca = partes.get(MARCAS[0]).or(partes.get(MARCAS[1])).copied().unwrap_or("");
    let pe = partes.get(MARCAS[3]).or(partes.get(MARCAS[2])).copied().unwrap_or("");
    format!("{cabeca}{}{pe}", &corpo[ini..]).into()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn celulas_mescladas() {
        assert_eq!(celula("\\multicolumn{2}{c|}{\\textbf{Geração}}"), ("\\textbf{Geração}", 2));
        assert_eq!(celula(" \\multirow{2}{*}{Cenário} "), ("Cenário", 1));
        let mut s = "a & b \\\\ \\cline{2-3} c".to_string();
        assert!(sem_comando(&mut s, "cline", 1));
        assert_eq!(s, "a & b \\\\  c");
        assert_eq!(fios_de(" \\midrule \\cmidrule(lr){2-3} \\hline\\cline{1-2} "), ["M", "c2-3", "H", "C1-2"]);
        assert_eq!(sem_cabecalho_repetido("A \\\\ \\endfirsthead A \\\\ \\endhead 1 \\\\"), "A \\\\  1 \\\\");
        assert_eq!(sem_cabecalho_repetido("\\toprule A \\\\ \\midrule \\endfirsthead A \\\\ \\endhead \\bottomrule \\endfoot 1 \\\\"), "\\toprule A \\\\ \\midrule  1 \\\\ \\bottomrule ");
    }
}
