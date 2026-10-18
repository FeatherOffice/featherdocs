//! Separação silábica do português para a hifenização no fim da linha: as regras da escola (consoante
//! entre vogais vai com a de depois; grupos com `l`/`r` e os dígrafos `ch`, `lh`, `nh` não se separam;
//! duas consoantes seguidas se separam). Vogais seguidas nunca se separam (hiato e ditongo ficam
//! juntos, o que erra só para menos). Como o babel do português: 2 letras antes e 3 depois, no mínimo.

fn vogal(c: char) -> bool {
    "aeiouáéíóúâêôãõàü".contains(c)
}

/// Os grupos que vão juntos para a sílaba de depois (`pr`, `bl`, `ch`, `lh`, `nh`…).
fn inseparavel(a: char, b: char) -> bool {
    (b == 'h' && "cln".contains(a)) || ("lr".contains(b) && "bcdfgkptv".contains(a))
}

/// Onde a palavra pode ser partida (em caracteres, do começo). Só palavra feita de letras, com
/// pontuação no fim no máximo (`herança.`); o resto não se parte.
pub fn pontos(palavra: &str) -> Vec<usize> {
    let cs: Vec<char> = palavra.chars().flat_map(char::to_lowercase).collect();
    if cs.len() != palavra.chars().count() {
        return Vec::new();
    }
    let n = cs.iter().rposition(|c| c.is_alphabetic()).map_or(0, |i| i + 1);
    if n < 5 || !cs[..n].iter().all(|c| c.is_alphabetic()) || cs[n..].iter().any(|c| c.is_alphanumeric()) {
        return Vec::new();
    }
    let vogais: Vec<usize> = (0..n).filter(|&i| vogal(cs[i])).collect();
    let mut saida = Vec::new();
    for par in vogais.windows(2) {
        let (p, q) = (par[0], par[1]);
        let k = match q - p - 1 {
            0 => continue,
            1 => q - 1,
            _ if inseparavel(cs[q - 2], cs[q - 1]) => q - 2,
            _ => q - 1,
        };
        if k >= 2 && n - k >= 3 {
            saida.push(k);
        }
    }
    saida
}

#[cfg(test)]
mod testes {
    use super::*;

    fn separar(p: &str) -> String {
        let cs: Vec<char> = p.chars().collect();
        let mut s = String::new();
        let mut de = 0;
        for k in pontos(p) {
            s.extend(&cs[de..k]);
            s.push('-');
            de = k;
        }
        s.extend(&cs[de..]);
        s
    }

    #[test]
    fn separa_como_na_escola() {
        assert_eq!(separar("Introdução"), "In-tro-du-ção");
        assert_eq!(separar("supervisão"), "su-per-vi-são");
        assert_eq!(separar("desenvolvimento,"), "de-sen-vol-vi-mento,");
        assert_eq!(separar("trabalhar"), "tra-ba-lhar");
        // "a-bran-ja": 1 letra antes e 2 depois, abaixo dos mínimos.
        assert_eq!(separar("abranja"), "abranja");
        assert_eq!(separar("UML"), "UML");
        assert_eq!(separar("x2y"), "x2y");
    }
}
