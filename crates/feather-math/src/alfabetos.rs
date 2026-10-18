//! Letra + alfabeto → o caractere dos "Símbolos Alfanuméricos Matemáticos" do Unicode
//! (U+1D400…), com as exceções que já existiam antes no bloco de letras (ℎ, ℝ, ℭ…).

use crate::arvore::Alfabeto;

/// Bases (A maiúsculo, a minúsculo, 0) de cada alfabeto latino; 0 = não tem.
struct Bases {
    maiuscula: u32,
    minuscula: u32,
    digito: u32,
    grega_mai: u32,
    grega_min: u32,
}

const NEGRITO: Bases = Bases { maiuscula: 0x1D400, minuscula: 0x1D41A, digito: 0x1D7CE, grega_mai: 0x1D6A8, grega_min: 0x1D6C2 };
const ITALICO: Bases = Bases { maiuscula: 0x1D434, minuscula: 0x1D44E, digito: 0, grega_mai: 0x1D6E2, grega_min: 0x1D6FC };
const NEGRITO_ITALICO: Bases = Bases { maiuscula: 0x1D468, minuscula: 0x1D482, digito: 0x1D7CE, grega_mai: 0x1D71C, grega_min: 0x1D736 };
const ROTEIRO: Bases = Bases { maiuscula: 0x1D49C, minuscula: 0x1D4B6, digito: 0, grega_mai: 0, grega_min: 0 };
const FRAKTUR: Bases = Bases { maiuscula: 0x1D504, minuscula: 0x1D51E, digito: 0, grega_mai: 0, grega_min: 0 };
const DUPLO: Bases = Bases { maiuscula: 0x1D538, minuscula: 0x1D552, digito: 0x1D7D8, grega_mai: 0, grega_min: 0 };
const SEM_SERIFA: Bases = Bases { maiuscula: 0x1D5A0, minuscula: 0x1D5BA, digito: 0x1D7E2, grega_mai: 0, grega_min: 0 };
const MONO: Bases = Bases { maiuscula: 0x1D670, minuscula: 0x1D68A, digito: 0x1D7F6, grega_mai: 0, grega_min: 0 };

/// As letras que o Unicode já tinha antes do bloco matemático (o bloco deixa o buraco).
fn excecao(c: u32) -> Option<char> {
    Some(match c {
        0x1D455 => 'ℎ',
        0x1D49D => 'ℬ',
        0x1D4A0 => 'ℰ',
        0x1D4A1 => 'ℱ',
        0x1D4A3 => 'ℋ',
        0x1D4A4 => 'ℐ',
        0x1D4A7 => 'ℒ',
        0x1D4A8 => 'ℳ',
        0x1D4AD => 'ℛ',
        0x1D4BA => 'ℯ',
        0x1D4BC => 'ℊ',
        0x1D4C4 => 'ℴ',
        0x1D506 => 'ℭ',
        0x1D50B => 'ℌ',
        0x1D50C => 'ℑ',
        0x1D515 => 'ℜ',
        0x1D51D => 'ℨ',
        0x1D53A => 'ℂ',
        0x1D53F => 'ℍ',
        0x1D545 => 'ℕ',
        0x1D547 => 'ℙ',
        0x1D548 => 'ℚ',
        0x1D549 => 'ℝ',
        0x1D551 => 'ℤ',
        _ => return None,
    })
}

/// A posição de uma grega no bloco matemático (0..=25 maiúsculas, 0..=31 minúsculas).
fn grega(c: char) -> Option<(bool, u32)> {
    let n = c as u32;
    Some(match c {
        'Α'..='Ω' if n != 0x3A2 => (true, n - 0x391),
        'ϴ' => (true, 17),
        '∇' => (true, 25),
        'α'..='ω' => (false, n - 0x3B1),
        '∂' => (false, 25),
        'ϵ' => (false, 26),
        'ϑ' => (false, 27),
        'ϰ' => (false, 28),
        'ϕ' => (false, 29),
        'ϱ' => (false, 30),
        'ϖ' => (false, 31),
        _ => return None,
    })
}

fn aplicar(c: char, b: &Bases) -> Option<char> {
    let n = c as u32;
    let alvo = match c {
        'A'..='Z' if b.maiuscula != 0 => b.maiuscula + n - 'A' as u32,
        'a'..='z' if b.minuscula != 0 => b.minuscula + n - 'a' as u32,
        '0'..='9' if b.digito != 0 => b.digito + n - '0' as u32,
        _ => match grega(c) {
            Some((true, k)) if b.grega_mai != 0 => b.grega_mai + k,
            Some((false, k)) if b.grega_min != 0 => b.grega_min + k,
            _ => return None,
        },
    };
    excecao(alvo).or_else(|| char::from_u32(alvo))
}

/// O caractere que a fonte desenha para a letra `c` no alfabeto `a`.
pub fn estilizar(c: char, a: Alfabeto) -> char {
    let minuscula_grega = matches!(grega(c), Some((false, _)));
    let r = match a {
        Alfabeto::Normal => {
            if c.is_ascii_alphabetic() || minuscula_grega {
                aplicar(c, &ITALICO)
            } else {
                None
            }
        }
        Alfabeto::Romano => None,
        Alfabeto::Negrito => {
            if minuscula_grega {
                aplicar(c, &ITALICO)
            } else {
                aplicar(c, &NEGRITO)
            }
        }
        Alfabeto::Italico => aplicar(c, &ITALICO),
        Alfabeto::NegritoItalico => {
            if c.is_ascii_digit() || matches!(grega(c), Some((true, _))) {
                aplicar(c, &NEGRITO)
            } else {
                aplicar(c, &NEGRITO_ITALICO)
            }
        }
        Alfabeto::Duplo => aplicar(c, &DUPLO),
        Alfabeto::Caligrafico => aplicar(c, &ROTEIRO),
        Alfabeto::Fraktur => aplicar(c, &FRAKTUR),
        Alfabeto::SemSerifa => aplicar(c, &SEM_SERIFA),
        Alfabeto::Mono => aplicar(c, &MONO),
    };
    r.unwrap_or(c)
}

/// O caminho de volta: o caractere matemático estilizado → (letra, alfabeto). `None` = não é um.
pub fn desestilizar(c: char) -> Option<(char, Alfabeto)> {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    static M: OnceLock<HashMap<char, (char, Alfabeto)>> = OnceLock::new();
    let m = M.get_or_init(|| {
        let mut m = HashMap::new();
        let letras = ('A'..='Z').chain('a'..='z').chain('0'..='9').chain('Α'..='Ω').chain('α'..='ω');
        let todas: Vec<char> = letras.collect();
        for a in [Alfabeto::Italico, Alfabeto::Negrito, Alfabeto::NegritoItalico, Alfabeto::Duplo, Alfabeto::Caligrafico, Alfabeto::Fraktur, Alfabeto::SemSerifa, Alfabeto::Mono] {
            for &l in &todas {
                let e = estilizar(l, a);
                if e != l {
                    m.entry(e).or_insert((l, a));
                }
            }
        }
        m
    });
    m.get(&c).copied()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn letras_dos_alfabetos() {
        assert_eq!(estilizar('x', Alfabeto::Normal), '𝑥');
        assert_eq!(estilizar('h', Alfabeto::Normal), 'ℎ');
        assert_eq!(estilizar('R', Alfabeto::Duplo), 'ℝ');
        assert_eq!(estilizar('N', Alfabeto::Duplo), 'ℕ');
        assert_eq!(estilizar('A', Alfabeto::Caligrafico), '𝒜');
        assert_eq!(estilizar('g', Alfabeto::Fraktur), '𝔤');
        assert_eq!(estilizar('α', Alfabeto::Normal), '𝛼');
        assert_eq!(estilizar('Γ', Alfabeto::Normal), 'Γ');
        assert_eq!(estilizar('3', Alfabeto::Negrito), '𝟑');
        assert_eq!(estilizar('v', Alfabeto::NegritoItalico), '𝒗');
        assert_eq!(estilizar('ϕ', Alfabeto::Normal), '𝜙');
    }
}
