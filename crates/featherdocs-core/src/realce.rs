//! Realce de sintaxe das listagens: palavra-chave, comentário e texto entre aspas, como o `listings`
//! do LaTeX os separa. Só as linguagens comuns em apostila; as outras saem sem realce.

use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tipo {
    Texto,
    Chave,
    Comentario,
    Aspas,
}

struct Linguagem {
    /// Comentário de linha (`#`, `//`).
    linha: &'static [&'static str],
    /// Comentário de bloco (`/*`, `*/`).
    bloco: Option<(&'static str, &'static str)>,
    aspas: &'static [char],
    chaves: &'static [&'static str],
}

const BASH: Linguagem = Linguagem {
    linha: &["#"],
    bloco: None,
    aspas: &['"', '\''],
    chaves: &[
        "alias", "awk", "bg", "break", "case", "cat", "cd", "chmod", "chown", "continue", "cp", "cut", "declare", "do", "done", "echo", "elif", "else", "esac",
        "eval", "exec", "exit", "export", "expr", "false", "fg", "fi", "find", "for", "function", "grep", "head", "if", "in", "kill", "let", "local", "ls",
        "mkdir", "mv", "printf", "pwd", "read", "readonly", "return", "rm", "sed", "select", "set", "shift", "sleep", "sort", "source", "tail", "test",
        "then", "touch", "tr", "trap", "true", "type", "unset", "until", "wait", "wc", "while",
    ],
};

const PYTHON: Linguagem = Linguagem {
    linha: &["#"],
    bloco: None,
    aspas: &['"', '\''],
    chaves: &[
        "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif", "else", "except", "False", "finally", "for", "from",
        "global", "if", "import", "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise", "return", "self", "True", "try", "while", "with",
        "yield",
    ],
};

const C: Linguagem = Linguagem {
    linha: &["//"],
    bloco: Some(("/*", "*/")),
    aspas: &['"', '\''],
    chaves: &[
        "auto", "bool", "break", "case", "char", "const", "continue", "default", "do", "double", "else", "enum", "extern", "float", "for", "goto", "if",
        "int", "long", "register", "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned", "void",
        "volatile", "while",
    ],
};

const JAVA: Linguagem = Linguagem {
    linha: &["//"],
    bloco: Some(("/*", "*/")),
    aspas: &['"', '\''],
    chaves: &[
        "abstract", "boolean", "break", "byte", "case", "catch", "char", "class", "continue", "default", "do", "double", "else", "enum", "extends",
        "false", "final", "finally", "float", "for", "if", "implements", "import", "instanceof", "int", "interface", "long", "new", "null", "package",
        "private", "protected", "public", "record", "return", "sealed", "short", "static", "super", "switch", "this", "throw", "throws", "true", "try",
        "var", "void", "while",
    ],
};

const JAVASCRIPT: Linguagem = Linguagem {
    linha: &["//"],
    bloco: Some(("/*", "*/")),
    aspas: &['"', '\'', '`'],
    chaves: &[
        "async", "await", "break", "case", "catch", "class", "const", "continue", "default", "delete", "do", "else", "export", "extends", "false",
        "finally", "for", "function", "if", "import", "in", "instanceof", "let", "new", "null", "return", "super", "switch", "this", "throw", "true",
        "try", "typeof", "undefined", "var", "void", "while", "yield",
    ],
};

fn linguagem(nome: &str) -> Option<&'static Linguagem> {
    match nome.to_ascii_lowercase().as_str() {
        "bash" | "sh" | "shell" | "zsh" => Some(&BASH),
        "python" | "py" => Some(&PYTHON),
        "c" | "c++" | "cpp" | "[ansi]c" => Some(&C),
        "java" | "kotlin" => Some(&JAVA),
        "javascript" | "js" | "typescript" | "ts" => Some(&JAVASCRIPT),
        _ => None,
    }
}

/// Os trechos de cada linha do código (em caracteres), na ordem; o comentário de bloco atravessa linhas.
pub fn realcar(nome: &str, codigo: &str) -> Vec<Vec<(Range<usize>, Tipo)>> {
    let Some(l) = linguagem(nome) else {
        return codigo.split('\n').map(|s| vec![(0..s.chars().count(), Tipo::Texto)]).collect();
    };
    let mut em_bloco = false;
    codigo.split('\n').map(|s| linha(l, s, &mut em_bloco)).collect()
}

fn linha(l: &Linguagem, s: &str, em_bloco: &mut bool) -> Vec<(Range<usize>, Tipo)> {
    let cs: Vec<char> = s.chars().collect();
    let comeca = |i: usize, m: &str| m.chars().enumerate().all(|(k, c)| cs.get(i + k) == Some(&c));
    let mut saida: Vec<(Range<usize>, Tipo)> = Vec::new();
    let mut por = |r: Range<usize>, t: Tipo| match saida.last_mut() {
        Some((u, tu)) if *tu == t && u.end == r.start => u.end = r.end,
        _ => saida.push((r, t)),
    };
    let mut i = 0;
    while i < cs.len() {
        if *em_bloco {
            let fim = l.bloco.and_then(|(_, f)| (i..cs.len()).find(|&k| comeca(k, f)).map(|k| k + f.chars().count()));
            *em_bloco = fim.is_none();
            let fim = fim.unwrap_or(cs.len());
            por(i..fim, Tipo::Comentario);
            i = fim;
        } else if l.linha.iter().any(|m| comeca(i, m)) {
            por(i..cs.len(), Tipo::Comentario);
            i = cs.len();
        } else if l.bloco.is_some_and(|(a, _)| comeca(i, a)) {
            *em_bloco = true;
            por(i..i + 2, Tipo::Comentario);
            i += 2;
        } else if l.aspas.contains(&cs[i]) {
            let mut k = i + 1;
            while k < cs.len() && cs[k] != cs[i] {
                k += if cs[k] == '\\' { 2 } else { 1 };
            }
            let fim = (k + 1).min(cs.len());
            por(i..fim, Tipo::Aspas);
            i = fim;
        } else if cs[i].is_alphabetic() || cs[i] == '_' {
            let k = (i..cs.len()).find(|&k| !(cs[k].is_alphanumeric() || cs[k] == '_')).unwrap_or(cs.len());
            let palavra: String = cs[i..k].iter().collect();
            por(i..k, if l.chaves.contains(&palavra.as_str()) { Tipo::Chave } else { Tipo::Texto });
            i = k;
        } else {
            por(i..i + 1, Tipo::Texto);
            i += 1;
        }
    }
    saida
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn bash_separa_chave_aspas_e_comentario() {
        let r = realcar("bash", "echo \"a # b\" | bc # fim");
        let tipos: Vec<Tipo> = r[0].iter().map(|(_, t)| *t).collect();
        assert_eq!(tipos, [Tipo::Chave, Tipo::Texto, Tipo::Aspas, Tipo::Texto, Tipo::Comentario]);
        assert_eq!(r[0][4].0, 18..23);
    }

    #[test]
    fn comentario_de_bloco_atravessa_linhas() {
        let r = realcar("c", "int a; /* x\ny */ return");
        assert_eq!(r[1], [(0..4, Tipo::Comentario), (4..5, Tipo::Texto), (5..11, Tipo::Chave)]);
    }
}
