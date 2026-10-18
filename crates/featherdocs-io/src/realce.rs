//! As cores do código no modo dividido (realce de sintaxe): o texto em trechos com um tipo
//! (comando, comentário, chave, fórmula…). A interface só pinta cada tipo com uma cor.
//!
//! As posições saem em unidades UTF-16 (as do JavaScript), em trios planos
//! `[início, fim, tipo, início, fim, tipo, …]`, para a ponte mandar pouco.

/// O que um trecho do código é.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tipo {
    Comando = 1,
    Comentario = 2,
    Chave = 3,
    Matematica = 4,
    Ambiente = 5,
    Titulo = 6,
    Enfase = 7,
    Codigo = 8,
    Link = 9,
    Marcador = 10,
    Citacao = 11,
}

/// Os trechos coloridos do código, como trios planos em UTF-16.
pub fn cores(fonte: crate::dividido::Fonte, texto: &str) -> Vec<u32> {
    let trechos = match fonte {
        crate::dividido::Fonte::Latex => latex(texto),
        crate::dividido::Fonte::Markdown => markdown(texto),
    };
    em_utf16(texto, &trechos)
}

/// `(início, fim)` em bytes para UTF-16.
fn em_utf16(texto: &str, trechos: &[(usize, usize, Tipo)]) -> Vec<u32> {
    let mut mapa = vec![0u32; texto.len() + 1];
    let mut u = 0u32;
    for (i, c) in texto.char_indices() {
        mapa[i] = u;
        u += c.len_utf16() as u32;
    }
    mapa[texto.len()] = u;
    trechos.iter().flat_map(|&(a, b, t)| [mapa[a], mapa[b], t as u32]).collect()
}

fn letras(s: &str) -> usize {
    s.bytes().take_while(u8::is_ascii_alphabetic).count()
}

/// LaTeX: comentário, comando (o nome do ambiente de `\begin`/`\end` à parte), chaves e fórmulas.
pub fn latex(texto: &str) -> Vec<(usize, usize, Tipo)> {
    let mut saida = Vec::new();
    let b = texto.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' => {
                let fim = texto[i..].find('\n').map_or(texto.len(), |n| i + n);
                saida.push((i, fim, Tipo::Comentario));
                i = fim;
            }
            b'\\' => i = comando(texto, i, &mut saida),
            b'{' | b'}' | b'[' | b']' => {
                saida.push((i, i + 1, Tipo::Chave));
                i += 1;
            }
            b'$' => {
                let abre = if texto[i..].starts_with("$$") { 2 } else { 1 };
                let fecha = &texto[i..i + abre];
                // A fórmula não atravessa parágrafo: sem fechamento antes de uma linha em branco, é só um `$`.
                let limite = texto[i + abre..].find("\n\n").map_or(texto.len(), |n| i + abre + n);
                match texto[i + abre..limite].find(fecha) {
                    Some(n) => {
                        let fim = i + abre + n + abre;
                        saida.push((i, fim, Tipo::Matematica));
                        i = fim;
                    }
                    None => i += 1,
                }
            }
            _ => i += texto[i..].chars().next().map_or(1, char::len_utf8),
        }
    }
    saida
}

/// Um comando em `i` (o `\`): `\nome`, `\nome*`, `\&`… `\begin{x}`/`\end{x}` pintam o nome do ambiente; `\[ … \]` é fórmula.
fn comando(texto: &str, i: usize, saida: &mut Vec<(usize, usize, Tipo)>) -> usize {
    let resto = &texto[i + 1..];
    if resto.starts_with('[') {
        if let Some(n) = resto.find("\\]") {
            let fim = i + 1 + n + 2;
            saida.push((i, fim, Tipo::Matematica));
            return fim;
        }
    }
    let n = letras(resto);
    let mut fim = if n == 0 { i + 1 + resto.chars().next().map_or(0, char::len_utf8) } else { i + 1 + n };
    if n > 0 && texto[fim..].starts_with('*') {
        fim += 1;
    }
    saida.push((i, fim, Tipo::Comando));
    let nome = &resto[..n];
    if (nome == "begin" || nome == "end") && texto[fim..].starts_with('{') {
        if let Some(f) = texto[fim..].find('}') {
            saida.push((fim, fim + 1, Tipo::Chave));
            saida.push((fim + 1, fim + f, Tipo::Ambiente));
            saida.push((fim + f, fim + f + 1, Tipo::Chave));
            return fim + f + 1;
        }
    }
    fim
}

/// Markdown, linha a linha: títulos, citação, marcadores, blocos de código, e em linha código,
/// ênfase, links, fórmulas e comentários HTML.
pub fn markdown(texto: &str) -> Vec<(usize, usize, Tipo)> {
    let mut saida = Vec::new();
    let mut cerca = false;
    let mut i = 0;
    for linha in texto.split_inclusive('\n') {
        let sem_fim = linha.trim_end_matches(['\n', '\r']);
        let fim = i + sem_fim.len();
        let aparada = sem_fim.trim_start();
        let recuo = sem_fim.len() - aparada.len();
        if aparada.starts_with("```") || aparada.starts_with("~~~") {
            cerca = !cerca;
            saida.push((i, fim, Tipo::Codigo));
        } else if cerca {
            saida.push((i, fim, Tipo::Codigo));
        } else if aparada.starts_with('#') {
            saida.push((i, fim, Tipo::Titulo));
        } else {
            let mut inicio_texto = i + recuo;
            if aparada.starts_with('>') {
                saida.push((i + recuo, i + recuo + 1, Tipo::Citacao));
                inicio_texto += 1;
            } else if let Some(m) = marcador_de_lista(aparada) {
                saida.push((i + recuo, i + recuo + m, Tipo::Marcador));
                inicio_texto += m;
            }
            em_linha(texto, inicio_texto, fim, &mut saida);
        }
        i += linha.len();
    }
    saida
}

/// `- `, `* `, `+ `, `1. `, `- [ ] `: o tamanho do marcador (sem o espaço), se a linha é item.
fn marcador_de_lista(s: &str) -> Option<usize> {
    let digitos = s.bytes().take_while(u8::is_ascii_digit).count();
    let n = if digitos > 0 && s[digitos..].starts_with(['.', ')']) {
        digitos + 1
    } else if s.starts_with(['-', '*', '+']) {
        1
    } else {
        return None;
    };
    if !s[n..].starts_with(' ') {
        return None;
    }
    let tarefa = ["[ ]", "[x]", "[X]"].iter().any(|t| s[n + 1..].starts_with(t));
    Some(if tarefa { n + 4 } else { n })
}

/// O que se marca dentro de uma linha de Markdown, de `de` a `ate`.
fn em_linha(texto: &str, de: usize, ate: usize, saida: &mut Vec<(usize, usize, Tipo)>) {
    let mut i = de;
    while i < ate {
        let r = &texto[i..ate];
        let par = |abre: &str, fecha: &str| r.starts_with(abre).then(|| r[abre.len()..].find(fecha).map(|n| abre.len() + n + fecha.len())).flatten();
        let achado = if r.starts_with("<!--") {
            Some((r.find("-->").map_or(r.len(), |n| n + 3), Tipo::Comentario))
        } else if r.starts_with('`') {
            par("`", "`").map(|n| (n, Tipo::Codigo))
        } else if r.starts_with("**") || r.starts_with("__") {
            par(&r[..2], &r[..2]).map(|n| (n, Tipo::Enfase))
        } else if r.starts_with(['*', '_']) && !r[1..].starts_with(' ') {
            par(&r[..1], &r[..1]).filter(|&n| n > 2).map(|n| (n, Tipo::Enfase))
        } else if r.starts_with('$') {
            par("$", "$").filter(|&n| n > 2).map(|n| (n, Tipo::Matematica))
        } else if r.starts_with("![") || r.starts_with('[') {
            link(r).map(|n| (n, Tipo::Link))
        } else {
            None
        };
        match achado {
            Some((n, t)) => {
                saida.push((i, i + n, t));
                i += n;
            }
            None => i += r.chars().next().map_or(1, char::len_utf8),
        }
    }
}

/// `[texto](url)` ou `![alt](url)`: o tamanho, se fecha.
fn link(r: &str) -> Option<usize> {
    let abre = r.find('[')?;
    let fecha = abre + r[abre..].find("](")?;
    let fim = fecha + r[fecha..].find(')')?;
    Some(fim + 1)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn tipos(v: &[(usize, usize, Tipo)], texto: &str) -> Vec<(String, Tipo)> {
        v.iter().map(|&(a, b, t)| (texto[a..b].to_string(), t)).collect()
    }

    #[test]
    fn latex_comandos_comentarios_ambientes_e_formulas() {
        let t = "\\section*{Olá} % nota\n\\begin{itemize}\\item $x^2$ e 50\\%\n\\[ a \\]";
        let v = tipos(&latex(t), t);
        assert!(v.contains(&("\\section*".into(), Tipo::Comando)));
        assert!(v.contains(&("% nota".into(), Tipo::Comentario)));
        assert!(v.contains(&("itemize".into(), Tipo::Ambiente)));
        assert!(v.contains(&("$x^2$".into(), Tipo::Matematica)));
        assert!(v.contains(&("\\%".into(), Tipo::Comando)), "o % escapado não abre comentário");
        assert!(v.contains(&("\\[ a \\]".into(), Tipo::Matematica)));
    }

    #[test]
    fn markdown_por_linha_e_em_linha() {
        let t = "# Título\n- item **forte** e `code`\n> cita\n```\nfn x\n```\n[link](http://x) e $y$";
        let v = tipos(&markdown(t), t);
        assert!(v.contains(&("# Título".into(), Tipo::Titulo)));
        assert!(v.contains(&("-".into(), Tipo::Marcador)));
        assert!(v.contains(&("**forte**".into(), Tipo::Enfase)));
        assert!(v.contains(&("`code`".into(), Tipo::Codigo)));
        assert!(v.contains(&("fn x".into(), Tipo::Codigo)));
        assert!(v.contains(&("[link](http://x)".into(), Tipo::Link)));
        assert!(v.contains(&("$y$".into(), Tipo::Matematica)));
    }

    #[test]
    fn posicoes_em_utf16() {
        // "ç" é um byte a mais em UTF-8 e uma unidade em UTF-16; "𝑥" são duas unidades.
        let t = "ç𝑥\\a";
        assert_eq!(cores(crate::dividido::Fonte::Latex, t), vec![3, 5, Tipo::Comando as u32]);
    }

    #[test]
    fn lixo_nao_trava() {
        for t in ["\\", "$", "$$", "\\begin{", "[", "**", "`", "\\[", "ç\\"] {
            let _ = cores(crate::dividido::Fonte::Latex, t);
            let _ = cores(crate::dividido::Fonte::Markdown, t);
        }
    }
}
