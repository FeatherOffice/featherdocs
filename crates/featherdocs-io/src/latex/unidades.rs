//! `siunitx`: `\SI{5,4}{\kilo\watt}`, `\si{…}`, `\num`, `\SIrange`, `\ang` (e os nomes novos
//! `\qty`, `\unit`, `\qtyrange`, `\numrange`) viram o número e a unidade em símbolos ("5,4 kW").

use super::{comando, grupo, opcional};

/// Prefixos: o nome do `siunitx` e o símbolo.
const PREFIXOS: &[(&str, &str)] = &[
    ("yocto", "y"), ("zepto", "z"), ("atto", "a"), ("femto", "f"), ("pico", "p"), ("nano", "n"), ("micro", "µ"),
    ("milli", "m"), ("centi", "c"), ("deci", "d"), ("deca", "da"), ("deka", "da"), ("hecto", "h"), ("kilo", "k"),
    ("mega", "M"), ("giga", "G"), ("tera", "T"), ("peta", "P"), ("exa", "E"), ("zetta", "Z"), ("yotta", "Y"),
];

/// Unidades: o nome do `siunitx` e o símbolo.
const UNIDADES: &[(&str, &str)] = &[
    ("ampere", "A"), ("candela", "cd"), ("kelvin", "K"), ("kilogram", "kg"), ("gram", "g"), ("metre", "m"), ("meter", "m"),
    ("mole", "mol"), ("second", "s"), ("becquerel", "Bq"), ("degreeCelsius", "°C"), ("celsius", "°C"), ("coulomb", "C"),
    ("farad", "F"), ("gray", "Gy"), ("hertz", "Hz"), ("henry", "H"), ("joule", "J"), ("lumen", "lm"), ("katal", "kat"),
    ("lux", "lx"), ("newton", "N"), ("ohm", "Ω"), ("pascal", "Pa"), ("radian", "rad"), ("siemens", "S"), ("sievert", "Sv"),
    ("steradian", "sr"), ("tesla", "T"), ("volt", "V"), ("watt", "W"), ("weber", "Wb"), ("day", "d"), ("hour", "h"),
    ("minute", "min"), ("hectare", "ha"), ("litre", "L"), ("liter", "L"), ("tonne", "t"), ("electronvolt", "eV"),
    ("decibel", "dB"), ("bel", "B"), ("bar", "bar"), ("percent", "%"), ("degree", "°"), ("arcminute", "′"), ("arcsecond", "″"),
    ("angstrom", "Å"), ("byte", "B"), ("bit", "bit"), ("astronomicalunit", "au"), ("dalton", "Da"), ("neper", "Np"),
    ("mmHg", "mmHg"), ("barn", "b"), ("knot", "kn"),
];

/// O número como o `siunitx` escreve: o ponto decimal vira vírgula, `e3` vira "× 10³", `+-` vira "±".
pub(super) fn numero(n: &str) -> String {
    let n = n.trim().replace("+-", " ± ").replace("\\pm", " ± ").replace('.', ",");
    let Some(p) = n.find(['e', 'E']).filter(|_| n.chars().any(|c| c.is_ascii_digit())) else { return n };
    let (base, exp) = (n[..p].trim(), &n[p + 1..]);
    let exp: String = exp.trim().chars().map(sobrescrito).collect();
    if base.is_empty() { format!("10{exp}") } else { format!("{base} × 10{exp}") }
}

fn sobrescrito(c: char) -> char {
    match c {
        '0' => '⁰', '1' => '¹', '2' => '²', '3' => '³', '4' => '⁴', '5' => '⁵', '6' => '⁶', '7' => '⁷', '8' => '⁸', '9' => '⁹',
        '-' => '⁻', '+' => '⁺',
        c => c,
    }
}

/// A unidade em símbolos: `\kilo\watt\hour` = "kWh", `\watt\per\square\metre` = "W/m²".
/// Texto direto (`kWh`, `m/s`) passa como está; `\si{…}` de dentro (de uma macro) é aberto.
pub(super) fn unidade(u: &str) -> String {
    let mut saida = String::new();
    let mut potencia = String::new();
    let mut i = 0;
    while i < u.len() {
        let r = &u[i..];
        if !r.starts_with('\\') {
            let c = r.chars().next().unwrap_or(' ');
            match c {
                '{' | '}' => {}
                '.' | '~' => saida.push('\u{2009}'),
                c if c.is_whitespace() => {}
                c => saida.push(c),
            }
            i += c.len_utf8();
            continue;
        }
        let (nome, n) = comando(r);
        i += n;
        match nome {
            "si" | "unit" => {
                if let Some((g, k)) = grupo(&u[i..]) {
                    saida.push_str(&unidade(g));
                    i += k;
                }
            }
            "per" => saida.push('/'),
            "square" => potencia = "²".into(),
            "cubic" => potencia = "³".into(),
            "squared" => saida.push('²'),
            "cubed" => saida.push('³'),
            "tothe" | "raiseto" => {
                if let Some((g, k)) = grupo(&u[i..]) {
                    let e: String = g.trim().chars().map(sobrescrito).collect();
                    if nome == "tothe" { saida.push_str(&e) } else { potencia = e }
                    i += k;
                }
            }
            "cdot" => saida.push('·'),
            "text" | "mathrm" => {
                if let Some((g, k)) = grupo(&u[i..]) {
                    saida.push_str(g);
                    i += k;
                }
            }
            _ => {
                if let Some((_, s)) = PREFIXOS.iter().chain(UNIDADES).find(|(n, _)| *n == nome) {
                    saida.push_str(s);
                    // A potência vale para a unidade (o `\square` antes do `\metre`), não para o prefixo.
                    if UNIDADES.iter().any(|(n, _)| *n == nome) && !potencia.is_empty() {
                        saida.push_str(&potencia);
                        potencia.clear();
                    }
                }
            }
        }
    }
    saida
}

/// O texto do comando do `siunitx` que começa em `depois` (logo após o nome) e quantos bytes ele leu.
/// `None` se não é um deles ou se faltam argumentos.
pub(super) fn ler(nome: &str, depois: &str) -> Option<(String, usize)> {
    let mut k = opcional(depois).map_or(0, |(_, n)| n);
    let mut arg = || {
        let (g, n) = grupo(&depois[k..])?;
        k += n;
        Some(g)
    };
    let texto = match nome {
        "SI" | "qty" => {
            let (v, u) = (arg()?, arg()?);
            juntar(&numero(v), &unidade(u))
        }
        "si" | "unit" => unidade(arg()?),
        "num" => numero(arg()?),
        "SIrange" | "qtyrange" => {
            let (a, b, u) = (arg()?, arg()?, arg()?);
            let u = unidade(u);
            format!("{} a {}", juntar(&numero(a), &u), juntar(&numero(b), &u))
        }
        "numrange" => {
            let (a, b) = (arg()?, arg()?);
            format!("{} a {}", numero(a), numero(b))
        }
        // `\ang{20}` = 20°; `\ang{12;30;0}` = 12°30′0″.
        "ang" => arg()?.split(';').zip(["°", "′", "″"]).filter(|(v, _)| !v.trim().is_empty()).map(|(v, s)| format!("{}{s}", numero(v))).collect(),
        _ => return None,
    };
    Some((texto, k))
}

/// Número e unidade: o "%" e o "°" encostam no número; as outras, com espaço que não quebra.
fn juntar(n: &str, u: &str) -> String {
    if u.is_empty() || u.starts_with(['%', '°', '′', '″']) { format!("{n}{u}") } else { format!("{n}\u{a0}{u}") }
}

/// Os comandos do `siunitx` dentro de uma fórmula viram `\text{…}` (o feather-math não os conhece).
pub(super) fn em_formula(f: &str) -> String {
    if !f.contains('\\') {
        return f.to_string();
    }
    let mut saida = String::with_capacity(f.len());
    let mut i = 0;
    while let Some(p) = f[i..].find('\\') {
        saida.push_str(&f[i..i + p]);
        i += p;
        let (nome, n) = comando(&f[i..]);
        match ler(nome, &f[i + n..]) {
            Some((t, k)) => {
                saida.push_str(&format!("\\text{{{}}}", t.replace('\u{a0}', "\\,").replace('%', "\\%")));
                i += n + k;
            }
            None => {
                saida.push_str(&f[i..i + n]);
                i += n;
            }
        }
    }
    saida.push_str(&f[i..]);
    saida
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn numeros_e_unidades() {
        assert_eq!(ler("SI", "{5,4}{\\kilo\\watt}").unwrap().0, "5,4\u{a0}kW");
        assert_eq!(ler("SI", "{98}{\\percent}").unwrap().0, "98%");
        assert_eq!(ler("si", "{\\watt\\per\\square\\meter}").unwrap().0, "W/m²");
        assert_eq!(ler("SI", "{520}{\\kilo\\watt\\hour}").unwrap().0, "520\u{a0}kWh");
        assert_eq!(ler("SIrange", "{8}{12}{\\percent}").unwrap().0, "8% a 12%");
        assert_eq!(ler("ang", "{20}").unwrap().0, "20°");
        assert_eq!(ler("num", "{1.5e3}").unwrap().0, "1,5 × 10³");
        // A macro `\kwp` (= `\si{\kilo\watt}p`) dentro da unidade.
        assert_eq!(ler("SI", "{4,71}{\\si{\\kilo\\watt}p}").unwrap().0, "4,71\u{a0}kWp");
        assert_eq!(em_formula("x = \\SI{8}{\\percent}"), "x = \\text{8\\%}");
    }
}
