//! O caminho de volta: OMML (a equação do Word) → LaTeX. Lê o XML de um `m:oMath` (com ou sem
//! a etiqueta de fora) e devolve o LaTeX equivalente — o `.docx` gravado pelo FeatherDocs volta
//! com a mesma estrutura, e a equação feita no Word vira LaTeX editável.

use crate::alfabetos::desestilizar;
use crate::arvore::Alfabeto;
use crate::simbolos::{funcao, nome_do_acento, nome_do_caractere};

#[derive(Debug, Default)]
struct El {
    nome: String,
    val: Option<String>,
    filhos: Vec<No>,
}

#[derive(Debug)]
enum No {
    El(El),
    Texto(String),
}

fn entidades(t: &str) -> String {
    t.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

/// Um leitor de XML mínimo (etiquetas, `m:val`, texto, entidades): o OMML não precisa de mais.
fn ler_xml(xml: &str) -> El {
    let mut pilha: Vec<El> = vec![El { nome: "#raiz".into(), ..El::default() }];
    let mut resto = xml;
    while !resto.is_empty() {
        let Some(i) = resto.find('<') else {
            empurrar_texto(&mut pilha, resto);
            break;
        };
        if i > 0 {
            empurrar_texto(&mut pilha, &resto[..i]);
        }
        resto = &resto[i..];
        let Some(f) = resto.find('>') else { break };
        let etiqueta = &resto[1..f];
        resto = &resto[f + 1..];
        if etiqueta.starts_with('?') || etiqueta.starts_with('!') {
            continue;
        }
        if let Some(nome) = etiqueta.strip_prefix('/') {
            let nome = nome.trim();
            // Fecha até a etiqueta de mesmo nome (XML malformado não trava).
            if let Some(pos) = pilha.iter().rposition(|e| e.nome == nome) {
                while pilha.len() > pos.max(1) {
                    let e = pilha.pop().expect("tem");
                    if let Some(pai) = pilha.last_mut() {
                        pai.filhos.push(No::El(e));
                    }
                }
            }
            continue;
        }
        let vazio = etiqueta.ends_with('/');
        let corpo = etiqueta.trim_end_matches('/');
        let nome = corpo.split_whitespace().next().unwrap_or("").to_string();
        let val = atributo_val(corpo);
        let el = El { nome, val, filhos: Vec::new() };
        if vazio {
            if let Some(pai) = pilha.last_mut() {
                pai.filhos.push(No::El(el));
            }
        } else {
            pilha.push(el);
        }
    }
    while pilha.len() > 1 {
        let e = pilha.pop().expect("tem");
        if let Some(pai) = pilha.last_mut() {
            pai.filhos.push(No::El(e));
        }
    }
    pilha.pop().unwrap_or_default()
}

fn empurrar_texto(pilha: &mut [El], t: &str) {
    if let Some(e) = pilha.last_mut() {
        e.filhos.push(No::Texto(entidades(t)));
    }
}

fn atributo_val(corpo: &str) -> Option<String> {
    for chave in ["m:val=\"", "val=\""] {
        if let Some(i) = corpo.find(chave) {
            let r = &corpo[i + chave.len()..];
            let f = r.find('"')?;
            return Some(entidades(&r[..f]));
        }
    }
    None
}

impl El {
    fn filho(&self, nome: &str) -> Option<&El> {
        self.filhos.iter().find_map(|n| match n {
            No::El(e) if e.nome == nome => Some(e),
            _ => None,
        })
    }

    fn elementos(&self) -> impl Iterator<Item = &El> {
        self.filhos.iter().filter_map(|n| match n {
            No::El(e) => Some(e),
            No::Texto(_) => None,
        })
    }

    /// Uma propriedade do `xxPr` (`m:chr`, `m:pos`…): o `m:val` dela.
    fn prop(&self, pr: &str, nome: &str) -> Option<String> {
        self.filho(pr)?.filho(nome).map(|e| e.val.clone().unwrap_or_default())
    }

    fn texto(&self) -> String {
        let mut s = String::new();
        for n in &self.filhos {
            match n {
                No::Texto(t) => s.push_str(t),
                No::El(e) => s.push_str(&e.texto()),
            }
        }
        s
    }
}

/// O LaTeX do `m:oMath` (ou de vários, ou de um `m:oMathPara`) em `xml`.
pub fn para_latex(xml: &str) -> String {
    let raiz = ler_xml(xml);
    let mut s = String::new();
    filhos(&raiz, &mut s);
    s.trim().to_string()
}

fn filhos(e: &El, s: &mut String) {
    for f in e.elementos() {
        el(f, s);
    }
}

/// Um argumento entre chaves.
fn arg(e: Option<&El>, s: &mut String) {
    s.push('{');
    if let Some(e) = e {
        filhos(e, s);
    }
    s.push('}');
}

/// Como `arg`, sem as chaves quando o conteúdo é um caractere só (`x^2`, não `{x}^{2}`).
fn curto(e: Option<&El>, s: &mut String) {
    let mut dentro = String::new();
    if let Some(e) = e {
        filhos(e, &mut dentro);
    }
    let t = dentro.trim();
    if t.chars().count() == 1 && t != "{" && t != "}" {
        s.push_str(t);
    } else {
        s.push('{');
        s.push_str(&dentro);
        s.push('}');
    }
}

fn delim(c: Option<String>, padrao: &str) -> String {
    let c = c.unwrap_or_else(|| padrao.to_string());
    match c.as_str() {
        "" => ".".into(),
        "{" => "\\{".into(),
        "}" => "\\}".into(),
        "⟨" | "〈" => "\\langle".into(),
        "⟩" | "〉" => "\\rangle".into(),
        "‖" => "\\|".into(),
        "⌊" => "\\lfloor".into(),
        "⌋" => "\\rfloor".into(),
        "⌈" => "\\lceil".into(),
        "⌉" => "\\rceil".into(),
        outro => outro.into(),
    }
}

fn el(e: &El, s: &mut String) {
    let nome = e.nome.strip_prefix("m:").unwrap_or(&e.nome);
    match nome {
        "r" => corrida(e, s),
        "f" => {
            let tipo = e.prop("m:fPr", "m:type").unwrap_or_default();
            match tipo.as_str() {
                "noBar" => {
                    s.push('{');
                    if let Some(n) = e.filho("m:num") {
                        filhos(n, s);
                    }
                    s.push_str(" \\atop ");
                    if let Some(d) = e.filho("m:den") {
                        filhos(d, s);
                    }
                    s.push('}');
                    return;
                }
                "lin" | "skw" => {
                    arg(e.filho("m:num"), s);
                    s.push('/');
                    arg(e.filho("m:den"), s);
                    return;
                }
                _ => s.push_str("\\frac"),
            }
            arg(e.filho("m:num"), s);
            arg(e.filho("m:den"), s);
        }
        "sSup" | "sSub" | "sSubSup" => {
            curto(e.filho("m:e"), s);
            if let Some(sub) = e.filho("m:sub") {
                s.push('_');
                curto(Some(sub), s);
            }
            if let Some(sup) = e.filho("m:sup") {
                s.push('^');
                curto(Some(sup), s);
            }
        }
        "sPre" => {
            s.push_str("{}_");
            arg(e.filho("m:sub"), s);
            s.push('^');
            arg(e.filho("m:sup"), s);
            arg(e.filho("m:e"), s);
        }
        "rad" => {
            let sem_grau = e.prop("m:radPr", "m:degHide").is_some_and(|v| v != "0" && v != "off") || e.filho("m:deg").map_or(true, |d| d.texto().trim().is_empty() && d.elementos().next().is_none());
            s.push_str("\\sqrt");
            if !sem_grau {
                s.push('[');
                if let Some(d) = e.filho("m:deg") {
                    filhos(d, s);
                }
                s.push(']');
            }
            arg(e.filho("m:e"), s);
        }
        "nary" => nary(e, s),
        "d" => {
            let esq = delim(e.prop("m:dPr", "m:begChr"), "(");
            let dir = delim(e.prop("m:dPr", "m:endChr"), ")");
            let sep = delim(e.prop("m:dPr", "m:sepChr"), "|");
            s.push_str(&format!("\\left{esq} "));
            for (k, parte) in e.elementos().filter(|f| f.nome == "m:e").enumerate() {
                if k > 0 {
                    s.push_str(&format!(" \\middle{sep} "));
                }
                filhos(parte, s);
            }
            s.push_str(&format!(" \\right{dir}"));
        }
        "func" => {
            if let Some(f) = e.filho("m:fName") {
                filhos(f, s);
            }
            s.push(' ');
            arg(e.filho("m:e"), s);
        }
        "limLow" | "limUpp" => {
            let base = e.filho("m:e").map(|b| b.texto()).unwrap_or_default();
            let eh_funcao = funcao(base.trim()).is_some();
            if eh_funcao {
                s.push_str(&format!("\\{}", base.trim()));
                s.push(if nome == "limLow" { '_' } else { '^' });
                arg(e.filho("m:lim"), s);
            } else {
                s.push_str(if nome == "limLow" { "\\underset" } else { "\\overset" });
                arg(e.filho("m:lim"), s);
                arg(e.filho("m:e"), s);
            }
        }
        "acc" => {
            let c = e.prop("m:accPr", "m:chr").and_then(|v| v.chars().next()).unwrap_or('\u{302}');
            let n = nome_do_acento(c).unwrap_or("hat");
            s.push_str(&format!("\\{n}"));
            arg(e.filho("m:e"), s);
        }
        "bar" => {
            let topo = e.prop("m:barPr", "m:pos").as_deref() == Some("top");
            s.push_str(if topo { "\\overline" } else { "\\underline" });
            arg(e.filho("m:e"), s);
        }
        "groupChr" => {
            let c = e.prop("m:groupChrPr", "m:chr").and_then(|v| v.chars().next()).unwrap_or('⏟');
            let cmd = match c {
                '⏞' => "\\overbrace",
                '⏜' => "\\overparen",
                '⏝' => "\\underparen",
                '→' => "\\underrightarrow",
                '←' => "\\underleftarrow",
                _ => "\\underbrace",
            };
            s.push_str(cmd);
            arg(e.filho("m:e"), s);
        }
        "m" => {
            s.push_str("\\begin{matrix}");
            for (k, linha) in e.elementos().filter(|f| f.nome == "m:mr").enumerate() {
                if k > 0 {
                    s.push_str(" \\\\ ");
                }
                for (j, cel) in linha.elementos().filter(|f| f.nome == "m:e").enumerate() {
                    if j > 0 {
                        s.push_str(" & ");
                    }
                    filhos(cel, s);
                }
            }
            s.push_str("\\end{matrix}");
        }
        "eqArr" => {
            s.push_str("\\begin{aligned}");
            for (k, linha) in e.elementos().filter(|f| f.nome == "m:e").enumerate() {
                if k > 0 {
                    s.push_str(" \\\\ ");
                }
                filhos(linha, s);
            }
            s.push_str("\\end{aligned}");
        }
        "borderBox" if e.filho("m:borderBoxPr").is_some_and(|p| p.filho("m:strikeBLTR").is_some() || p.filho("m:strikeTLBR").is_some()) => {
            let pr = e.filho("m:borderBoxPr");
            let bltr = pr.is_some_and(|p| p.filho("m:strikeBLTR").is_some());
            let tlbr = pr.is_some_and(|p| p.filho("m:strikeTLBR").is_some());
            s.push_str(match (bltr, tlbr) {
                (true, true) => "\\xcancel",
                (false, true) => "\\bcancel",
                _ => "\\cancel",
            });
            arg(e.filho("m:e"), s);
        }
        "borderBox" => {
            s.push_str("\\boxed");
            arg(e.filho("m:e"), s);
        }
        "phant" => {
            s.push_str("\\phantom");
            arg(e.filho("m:e"), s);
        }
        // Propriedades e controles: não viram texto.
        n if n.ends_with("Pr") || n == "ctrlPr" => {}
        _ => filhos(e, s),
    }
}

fn nary(e: &El, s: &mut String) {
    let c = e.prop("m:naryPr", "m:chr").and_then(|v| v.chars().next()).unwrap_or('∫');
    match nome_do_caractere(c) {
        Some(n) => s.push_str(&format!("\\{n}")),
        None => s.push(c),
    }
    let esconde = |p: &str| e.prop("m:naryPr", p).is_some_and(|v| v != "0" && v != "off");
    if !esconde("m:subHide") {
        if let Some(sub) = e.filho("m:sub").filter(|x| x.elementos().next().is_some()) {
            s.push('_');
            arg(Some(sub), s);
        }
    }
    if !esconde("m:supHide") {
        if let Some(sup) = e.filho("m:sup").filter(|x| x.elementos().next().is_some()) {
            s.push('^');
            arg(Some(sup), s);
        }
    }
    s.push(' ');
    if let Some(corpo) = e.filho("m:e") {
        filhos(corpo, s);
    }
}

/// Uma corrida `m:r`: o texto com o estilo (`\mathrm`, `\mathbb`, `\text`…) e os símbolos pelo nome.
fn corrida(e: &El, s: &mut String) {
    let t: String = e.elementos().filter(|f| f.nome == "m:t").map(El::texto).collect();
    if t.is_empty() {
        if e.filho("m:rPr").and_then(|p| p.filho("m:aln")).is_some() {
            s.push_str(" & ");
        }
        return;
    }
    let pr = e.filho("m:rPr");
    if pr.and_then(|p| p.filho("m:aln")).is_some() {
        s.push_str(" & ");
    }
    if pr.and_then(|p| p.filho("m:nor")).is_some() {
        s.push_str(&format!("\\text{{{}}}", t.replace(['{', '}'], "")));
        return;
    }
    let scr = pr.and_then(|p| p.filho("m:scr")).and_then(|x| x.val.clone()).unwrap_or_default();
    let sty = pr.and_then(|p| p.filho("m:sty")).and_then(|x| x.val.clone()).unwrap_or_default();
    let envoltorio = match (scr.as_str(), sty.as_str()) {
        ("double-struck", _) => Some("mathbb"),
        ("script", _) => Some("mathcal"),
        ("fraktur", _) => Some("mathfrak"),
        ("sans-serif", _) => Some("mathsf"),
        ("monospace", _) => Some("mathtt"),
        (_, "b") => Some("mathbf"),
        (_, "bi") => Some("boldsymbol"),
        (_, "p") if t.chars().any(|c| c.is_ascii_alphabetic()) => {
            // Nome de função conhecido em pé: `\sin`; outro texto em pé: `\mathrm`.
            if let Some((_, _)) = funcao(t.trim()) {
                s.push_str(&format!("\\{} ", t.trim()));
                return;
            }
            Some("mathrm")
        }
        _ => None,
    };
    let corpo = caracteres(&t);
    match envoltorio {
        Some(w) => s.push_str(&format!("\\{w}{{{corpo}}}")),
        None => s.push_str(&corpo),
    }
}

fn caracteres(t: &str) -> String {
    let mut s = String::new();
    for c in t.chars() {
        let (c, alf) = desestilizar(c).unwrap_or((c, Alfabeto::Normal));
        let simples = match c {
            '−' => Some("-".to_string()),
            '{' => Some("\\{".into()),
            '}' => Some("\\}".into()),
            '%' | '#' | '&' | '$' | '_' => Some(format!("\\{c}")),
            '\u{2009}' | '\u{2005}' => Some("\\,".into()),
            '\u{2003}' => Some("\\quad ".into()),
            '\u{338}' => Some("\\not ".into()),
            _ => None,
        };
        let texto = simples.unwrap_or_else(|| match nome_do_caractere(c) {
            Some(n) => format!("\\{n} "),
            None => c.to_string(),
        });
        match alf {
            Alfabeto::Duplo => s.push_str(&format!("\\mathbb{{{texto}}}")),
            Alfabeto::Caligrafico => s.push_str(&format!("\\mathcal{{{texto}}}")),
            Alfabeto::Fraktur => s.push_str(&format!("\\mathfrak{{{texto}}}")),
            Alfabeto::Negrito => s.push_str(&format!("\\mathbf{{{texto}}}")),
            Alfabeto::NegritoItalico => s.push_str(&format!("\\boldsymbol{{{texto}}}")),
            Alfabeto::SemSerifa => s.push_str(&format!("\\mathsf{{{texto}}}")),
            Alfabeto::Mono => s.push_str(&format!("\\mathtt{{{texto}}}")),
            _ => s.push_str(&texto),
        }
    }
    s
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::omml::omml;

    fn ida_e_volta(latex: &str) -> String {
        para_latex(&format!("<m:oMath>{}</m:oMath>", omml(latex, false)))
    }

    #[test]
    fn estrutura_volta_igual() {
        assert_eq!(ida_e_volta("\\frac{a}{b}"), "\\frac{a}{b}");
        assert_eq!(ida_e_volta("x^{2}"), "x^2");
        assert_eq!(ida_e_volta("E=mc^2"), "E=mc^2");
        assert_eq!(ida_e_volta("a_{ij}^2"), "a_{ij}^2");
        assert_eq!(ida_e_volta("\\sqrt{x}"), "\\sqrt{x}");
        assert_eq!(ida_e_volta("\\sqrt[3]{x}"), "\\sqrt[3]{x}");
        assert!(ida_e_volta("\\sum_{i=1}^{n} x_i").starts_with("\\sum_{i=1}^{n}"), "{}", ida_e_volta("\\sum_{i=1}^{n} x_i"));
        assert_eq!(ida_e_volta("\\alpha \\leq \\beta"), "\\alpha \\leq \\beta");
        assert_eq!(ida_e_volta("\\mathbb{R}"), "\\mathbb{R}");
        assert!(ida_e_volta("\\left( a \\right)").contains("\\left( a \\right)"));
    }

    #[test]
    fn equacao_feita_no_word() {
        // O que o Word grava para a fórmula da área do círculo (A = πr²), com letras matemáticas.
        let xml = r#"<m:oMathPara><m:oMath><m:r><w:rPr><w:rFonts w:ascii="Cambria Math"/></w:rPr><m:t>A=π</m:t></m:r><m:sSup><m:sSupPr><m:ctrlPr/></m:sSupPr><m:e><m:r><m:t>r</m:t></m:r></m:e><m:sup><m:r><m:t>2</m:t></m:r></m:sup></m:sSup></m:oMath></m:oMathPara>"#;
        assert_eq!(para_latex(xml), "A=\\pi r^2");
        // XML quebrado não trava.
        let _ = para_latex("<m:oMath><m:f><m:num><m:r><m:t>1");
    }
}
