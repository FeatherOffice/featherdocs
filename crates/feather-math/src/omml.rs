//! A fórmula como OMML (Office Math Markup Language), a equação nativa do Word: o mesmo
//! analisador, com cada átomo traduzido para o elemento `m:` correspondente. O resultado é
//! o conteúdo de um `<m:oMath>` (sem a etiqueta de fora), para o gravador do .docx embrulhar.

use crate::analise::analisar;
use crate::arvore::{Alfabeto, Classe, Matriz, No, Nucleo, TipoMatriz};

pub use crate::omml_leitura::para_latex;

/// O conteúdo de um `<m:oMath>` para o `latex`. `exibida` = ambiente de várias linhas vira
/// `m:eqArr` com os pontos de alinhamento do `&`.
pub fn omml(latex: &str, exibida: bool) -> String {
    let nos = analisar(latex);
    let mut s = String::new();
    if exibida {
        if let [no] = nos.as_slice() {
            if let Nucleo::Matriz(m) = &no.nucleo {
                if matches!(m.tipo, Some(TipoMatriz::Alinhado | TipoMatriz::Reunido | TipoMatriz::Multlinha | TipoMatriz::Eqnarray)) && m.esq.is_none() {
                    if m.linhas.len() == 1 && m.linhas[0].len() == 1 {
                        lista(&m.linhas[0][0], &mut s);
                    } else {
                        eq_arr(m, &mut s);
                    }
                    return s;
                }
            }
        }
    }
    lista(&nos, &mut s);
    s
}

fn esc(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// As propriedades da letra: estilo (itálico/em pé/negrito) e alfabeto (duplo, caligráfico…).
fn rpr(alf: Alfabeto, em_pe: bool) -> String {
    let (scr, sty) = match alf {
        Alfabeto::Normal => ("", if em_pe { "p" } else { "" }),
        Alfabeto::Romano => ("", "p"),
        Alfabeto::Negrito => ("", "b"),
        Alfabeto::Italico => ("", "i"),
        Alfabeto::NegritoItalico => ("", "bi"),
        Alfabeto::Duplo => ("double-struck", "p"),
        Alfabeto::Caligrafico => ("script", "p"),
        Alfabeto::Fraktur => ("fraktur", "p"),
        Alfabeto::SemSerifa => ("sans-serif", "p"),
        Alfabeto::Mono => ("monospace", "p"),
    };
    let mut s = String::new();
    if !scr.is_empty() {
        s.push_str(&format!("<m:scr m:val=\"{scr}\"/>"));
    }
    if !sty.is_empty() {
        s.push_str(&format!("<m:sty m:val=\"{sty}\"/>"));
    }
    if s.is_empty() {
        s
    } else {
        format!("<m:rPr>{s}</m:rPr>")
    }
}

fn run(texto: &str, props: &str, s: &mut String) {
    s.push_str(&format!("<m:r>{props}<m:t xml:space=\"preserve\">{}</m:t></m:r>", esc(texto)));
}

fn elemento(tag: &str, conteudo: &[No], s: &mut String) {
    s.push_str(&format!("<m:{tag}>"));
    lista(conteudo, s);
    s.push_str(&format!("</m:{tag}>"));
}

/// Os átomos que um operador grande (∑, ∫) ou uma função levam como argumento: até a próxima
/// relação ou o próximo + / − no mesmo nível.
fn argumento_de(nos: &[No], de: usize) -> usize {
    let mut k = de;
    while k < nos.len() {
        let n = &nos[k];
        if matches!(n.classe, Classe::Rel | Classe::Pont) || (n.classe == Classe::Bin && k > de) || matches!(n.nucleo, Nucleo::SepColuna | Nucleo::SepLinha(_)) {
            break;
        }
        k += 1;
    }
    k
}

fn lista(nos: &[No], s: &mut String) {
    let mut k = 0;
    while k < nos.len() {
        let no = &nos[k];
        // Operador grande: m:nary com o argumento dentro.
        if let (Classe::Op, Nucleo::Car(c, _)) = (no.classe, &no.nucleo) {
            let fim = argumento_de(nos, k + 1);
            nary(*c, no, &nos[k + 1..fim], s);
            k = fim;
            continue;
        }
        if let (Classe::Op, Nucleo::Texto(t, _)) = (no.classe, &no.nucleo) {
            let fim = (k + 2).min(nos.len()).max(k + 1);
            funcao(t, no, &nos[k + 1..fim], s);
            k = fim;
            continue;
        }
        // Caracteres simples seguidos, sem scripts, viram uma corrida só.
        if let Nucleo::Car(_, alf) = no.nucleo {
            let mut texto = String::new();
            let mut j = k;
            while j < nos.len() {
                match (&nos[j].nucleo, nos[j].sup.is_none() && nos[j].sub.is_none() && nos[j].classe != Classe::Op) {
                    (Nucleo::Car(c, a), true) if *a == alf => texto.push(*c),
                    _ => break,
                }
                j += 1;
            }
            if j > k {
                run(&texto, &rpr(alf, !texto.chars().all(char::is_alphabetic)), s);
                k = j;
                continue;
            }
        }
        atomo(no, s);
        k += 1;
    }
}

fn com_scripts(no: &No, base: impl FnOnce(&mut String), s: &mut String) {
    match (&no.sup, &no.sub) {
        (None, None) => base(s),
        (Some(sup), None) => {
            s.push_str("<m:sSup><m:e>");
            base(s);
            s.push_str("</m:e>");
            elemento("sup", sup, s);
            s.push_str("</m:sSup>");
        }
        (None, Some(sub)) => {
            s.push_str("<m:sSub><m:e>");
            base(s);
            s.push_str("</m:e>");
            elemento("sub", sub, s);
            s.push_str("</m:sSub>");
        }
        (Some(sup), Some(sub)) => {
            s.push_str("<m:sSubSup><m:e>");
            base(s);
            s.push_str("</m:e>");
            elemento("sub", sub, s);
            elemento("sup", sup, s);
            s.push_str("</m:sSubSup>");
        }
    }
}

fn nary(c: char, no: &No, arg: &[No], s: &mut String) {
    let lim = if no.limites == Some(false) { "subSup" } else { "undOvr" };
    s.push_str(&format!("<m:nary><m:naryPr><m:chr m:val=\"{c}\"/><m:limLoc m:val=\"{lim}\"/>"));
    if no.sub.is_none() {
        s.push_str("<m:subHide m:val=\"1\"/>");
    }
    if no.sup.is_none() {
        s.push_str("<m:supHide m:val=\"1\"/>");
    }
    s.push_str("</m:naryPr>");
    elemento("sub", no.sub.as_deref().unwrap_or(&[]), s);
    elemento("sup", no.sup.as_deref().unwrap_or(&[]), s);
    elemento("e", arg, s);
    s.push_str("</m:nary>");
}

fn funcao(nome: &str, no: &No, arg: &[No], s: &mut String) {
    s.push_str("<m:func><m:fName>");
    let corrida = |s: &mut String| run(nome, "<m:rPr><m:sty m:val=\"p\"/></m:rPr>", s);
    if no.limites != Some(false) && (no.sub.is_some() || no.sup.is_some()) {
        if let Some(sub) = &no.sub {
            s.push_str("<m:limLow><m:e>");
            corrida(s);
            s.push_str("</m:e>");
            elemento("lim", sub, s);
            s.push_str("</m:limLow>");
        } else {
            corrida(s);
        }
    } else {
        com_scripts(no, corrida, s);
    }
    s.push_str("</m:fName>");
    elemento("e", arg, s);
    s.push_str("</m:func>");
}

fn delim(esq: Option<char>, dir: Option<char>, partes: &[&[No]], s: &mut String) {
    let c = |x: Option<char>| x.map_or(String::new(), |c| esc(&c.to_string()));
    s.push_str(&format!("<m:d><m:dPr><m:begChr m:val=\"{}\"/><m:endChr m:val=\"{}\"/></m:dPr>", c(esq), c(dir)));
    for p in partes {
        elemento("e", p, s);
    }
    s.push_str("</m:d>");
}

fn matriz(m: &Matriz, s: &mut String) {
    let colunas = m.linhas.iter().map(Vec::len).max().unwrap_or(1);
    let corpo = |s: &mut String| {
        s.push_str("<m:m>");
        for l in &m.linhas {
            s.push_str("<m:mr>");
            for j in 0..colunas {
                elemento("e", l.get(j).map_or(&[][..], Vec::as_slice), s);
            }
            s.push_str("</m:mr>");
        }
        s.push_str("</m:m>");
    };
    if m.esq.is_some() || m.dir.is_some() {
        let ce = |x: Option<char>| x.map_or(String::new(), |c| esc(&c.to_string()));
        s.push_str(&format!("<m:d><m:dPr><m:begChr m:val=\"{}\"/><m:endChr m:val=\"{}\"/></m:dPr><m:e>", ce(m.esq), ce(m.dir)));
        corpo(s);
        s.push_str("</m:e></m:d>");
    } else if matches!(m.tipo, Some(TipoMatriz::Alinhado | TipoMatriz::Reunido | TipoMatriz::Multlinha | TipoMatriz::Eqnarray | TipoMatriz::Pilha)) {
        eq_arr(m, s);
    } else {
        corpo(s);
    }
}

/// Várias linhas alinhadas: `m:eqArr`, com `m:aln` marcando o ponto de cada `&`.
fn eq_arr(m: &Matriz, s: &mut String) {
    s.push_str("<m:eqArr>");
    for l in &m.linhas {
        s.push_str("<m:e>");
        for (j, cel) in l.iter().enumerate() {
            if j > 0 {
                s.push_str("<m:r><m:rPr><m:aln/></m:rPr><m:t></m:t></m:r>");
            }
            lista(cel, s);
        }
        s.push_str("</m:e>");
    }
    s.push_str("</m:eqArr>");
}

fn atomo(no: &No, s: &mut String) {
    com_scripts(no, |s| nucleo(no, s), s);
}

fn nucleo(no: &No, s: &mut String) {
    match &no.nucleo {
        Nucleo::Car(c, alf) => run(&c.to_string(), &rpr(*alf, !c.is_alphabetic()), s),
        Nucleo::Texto(t, alf) => {
            let props = if *alf == Alfabeto::Romano { "<m:rPr><m:nor/></m:rPr>".to_string() } else { rpr(*alf, true) };
            run(t, &props, s);
        }
        Nucleo::Grupo(l) | Nucleo::Cor(_, l) | Nucleo::Achatado(l) => lista(l, s),
        Nucleo::Fracao { num, den, barra, esq, dir, .. } => {
            let f = |s: &mut String| {
                s.push_str(if *barra { "<m:f>" } else { "<m:f><m:fPr><m:type m:val=\"noBar\"/></m:fPr>" });
                elemento("num", num, s);
                elemento("den", den, s);
                s.push_str("</m:f>");
            };
            if esq.is_some() || dir.is_some() {
                let mut dentro = String::new();
                f(&mut dentro);
                let ce = |x: &Option<char>| x.map_or(String::new(), |c| esc(&c.to_string()));
                s.push_str(&format!("<m:d><m:dPr><m:begChr m:val=\"{}\"/><m:endChr m:val=\"{}\"/></m:dPr><m:e>{dentro}</m:e></m:d>", ce(esq), ce(dir)));
            } else {
                f(s);
            }
        }
        Nucleo::Raiz { indice, corpo } => {
            match indice {
                Some(i) => {
                    s.push_str("<m:rad>");
                    elemento("deg", i, s);
                }
                None => s.push_str("<m:rad><m:radPr><m:degHide m:val=\"1\"/></m:radPr><m:deg/>"),
            }
            elemento("e", corpo, s);
            s.push_str("</m:rad>");
        }
        Nucleo::Delimitado { esq, dir, partes, .. } => {
            let ps: Vec<&[No]> = partes.iter().map(Vec::as_slice).collect();
            delim(*esq, *dir, &ps, s);
        }
        Nucleo::Grande { c, .. } => run(&c.map_or(String::new(), |c| c.to_string()), "<m:rPr><m:sty m:val=\"p\"/></m:rPr>", s),
        Nucleo::Acento { c, corpo, baixo, .. } => {
            if *baixo {
                s.push_str(&format!("<m:groupChr><m:groupChrPr><m:chr m:val=\"{}\"/><m:pos m:val=\"bot\"/></m:groupChrPr>", esc(&c.to_string())));
                elemento("e", corpo, s);
                s.push_str("</m:groupChr>");
            } else {
                s.push_str(&format!("<m:acc><m:accPr><m:chr m:val=\"{}\"/></m:accPr>", esc(&c.to_string())));
                elemento("e", corpo, s);
                s.push_str("</m:acc>");
            }
        }
        Nucleo::Barra { corpo, cima } => {
            s.push_str(&format!("<m:bar><m:barPr><m:pos m:val=\"{}\"/></m:barPr>", if *cima { "top" } else { "bot" }));
            elemento("e", corpo, s);
            s.push_str("</m:bar>");
        }
        Nucleo::Chave { c, corpo, cima } => {
            let (pos, vert) = if *cima { ("top", "bot") } else { ("bot", "top") };
            s.push_str(&format!("<m:groupChr><m:groupChrPr><m:chr m:val=\"{c}\"/><m:pos m:val=\"{pos}\"/><m:vertJc m:val=\"{vert}\"/></m:groupChrPr>"));
            elemento("e", corpo, s);
            s.push_str("</m:groupChr>");
        }
        Nucleo::Matriz(m) => matriz(m, s),
        Nucleo::Caixa(l) => elemento_em("borderBox", l, s),
        Nucleo::Fantasma { corpo, .. } => elemento_em("phant", corpo, s),
        Nucleo::Negado(n) => {
            atomo(n, s);
            run("\u{338}", "", s);
        }
        Nucleo::Lap(l, _) => lista(l, s),
        Nucleo::Riscado { corpo, tipo, .. } => {
            let riscos = match tipo {
                2 => "<m:strikeTLBR m:val=\"1\"/>",
                3 => "<m:strikeBLTR m:val=\"1\"/><m:strikeTLBR m:val=\"1\"/>",
                _ => "<m:strikeBLTR m:val=\"1\"/>",
            };
            s.push_str(&format!("<m:borderBox><m:borderBoxPr><m:hideTop m:val=\"1\"/><m:hideBot m:val=\"1\"/><m:hideLeft m:val=\"1\"/><m:hideRight m:val=\"1\"/>{riscos}</m:borderBoxPr>"));
            elemento("e", corpo, s);
            s.push_str("</m:borderBox>");
        }
        Nucleo::Espaco(mu) if *mu > 0.0 => run(if *mu >= 5.0 { "\u{2005}" } else { "\u{2009}" }, "", s),
        Nucleo::EspacoEm(e) if *e > 0.0 => run(&"\u{2003}".repeat((e.round() as usize).max(1)), "", s),
        _ => {}
    }
}

fn elemento_em(tag: &str, conteudo: &[No], s: &mut String) {
    s.push_str(&format!("<m:{tag}>"));
    elemento("e", conteudo, s);
    s.push_str(&format!("</m:{tag}>"));
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn fracao_expoente_e_somatorio() {
        let x = omml("\\frac{a}{b}^2", false);
        assert!(x.contains("<m:sSup><m:e><m:f><m:num>"), "{x}");
        let x = omml("\\sum_{i=1}^n x_i = 1", false);
        assert!(x.starts_with("<m:nary><m:naryPr><m:chr m:val=\"∑\"/>"), "{x}");
        assert!(x.contains("<m:e><m:sSub>"), "o argumento vai dentro: {x}");
        let x = omml("\\sqrt[3]{x}", false);
        assert!(x.contains("<m:rad><m:deg>"), "{x}");
        let x = omml("\\begin{align} a &= b \\\\ c &= d \\end{align}", true);
        assert!(x.starts_with("<m:eqArr>") && x.matches("<m:aln/>").count() == 2, "{x}");
        let x = omml("\\begin{pmatrix} 1 & 2 \\\\ 3 & 4 \\end{pmatrix}", false);
        assert!(x.contains("<m:begChr m:val=\"(\"/>") && x.contains("<m:mr>"), "{x}");
        let x = omml("\\mathbb{R} < 2", false);
        assert!(x.contains("double-struck") && x.contains("&lt;"), "{x}");
    }
}
