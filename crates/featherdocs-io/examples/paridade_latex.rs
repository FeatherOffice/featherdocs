//! Mede a paridade do leitor LaTeX com o LaTeX e gera `featherdocs/docs/PARIDADE-LATEX.md`.
//!
//! Para cada item de `dados/paridade_latex.txt` monta um documento mínimo, lê pelo leitor
//! (`featherdocs_io::latex::ler`) e classifica olhando o modelo (não o desenho):
//!   `[ ]` o comando sumiu, virou texto cru (`\comando`, `begin{`) ou o argumento se perdeu;
//!   `[~]` reconhecido, mas o efeito esperado não aparece (ou o item não declara efeito);
//!   `[x]` o efeito esperado (estilo, bloco, cor, alinhamento…) está no modelo.
//! Rodar: `cargo run --example paridade_latex -- --saida ../../docs/PARIDADE-LATEX.md`
//! (a partir de `featherdocs/crates/featherdocs-io`; sem `--saida` imprime na tela;
//! `--comparar arquivo.md` lista o que mudou de marca em relação a um arquivo anterior;
//! `--detalhe` explica cada item). Formato do arquivo de dados: ver o cabeçalho dele.

#![allow(clippy::too_many_lines, clippy::cognitive_complexity)]

use featherdocs_core::modelo::{Alinhamento, Conteudo, Documento, Marca, Trecho, CODIGO, ITALICO, MATEMATICA, NEGRITO, SERIFA, SUBLINHADO, TACHADO};
use featherdocs_io::latex;
use std::collections::HashMap;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};

const MARCA: &str = "ZQX";

struct Item {
    nome: String,
    corpo: String,
    espera: String,
}

enum Linha {
    Literal(String),
    Secao(String),
    Sub(String),
    Item(Item),
}

const PREAMBULO: &[&str] = &[
    "usepackage", "RequirePackage", "newcommand", "renewcommand", "providecommand", "newenvironment", "renewenvironment", "DeclareMathOperator", "newtheorem", "theoremstyle", "definecolor", "setlength", "addtolength", "setcounter", "pagestyle", "geometry", "hypersetup", "title", "author", "date", "makeindex", "makeglossaries", "newglossaryentry", "newacronym", "DeclareAcronym", "crefname", "setmainfont", "setsansfont", "setmonofont", "newfontfamily", "DeclareFontFamily", "sethlcolor", "addbibresource", "bibliography", "urlstyle", "newcounter", "def", "let", "PassOptionsToPackage", "AtBeginDocument", "setlist", "newlist", "fancyhf", "fancyhead", "fancyfoot", "lhead", "chead", "rhead", "lfoot", "cfoot", "rfoot", "titleformat", "titlespacing", "captionsetup", "lstset", "tcbuselibrary", "newtcolorbox", "DeclareTCBox", "lstdefinestyle", "SetKeys", "sisetup", "NewDocumentCommand", "DeclareDocumentCommand", "newif", "DeclareRobustCommand", "newlength", "setstretch", "linespread", "numberwithin", "makenomenclature", "makeatletter", "makeatother", "newboolean", "setboolean", "DeclareOption", "ProcessOptions", "bibliographystyle", "usetikzlibrary", "pgfplotsset", "tikzset", "usetikzlibrary", "newfloat", "floatstyle", "restylefloat", "DeclareCaptionFormat", "DeclareCaptionLabelFormat", "newmdenv", "surroundwithmdframed", "newalgorithm", "SetAlgoLined", "glsaddall", "pagenumbering", "sloppy", "fussy", "frenchspacing", "setcounter", "hyphenation", "selectlanguage", "babelprovide", "addto", "renewcommand*", "newcommand*", "DeclareUnicodeCharacter", "DeclareTextCommand", "usetheme", "newacro", "acsetup", "setkeys", "glsdisablehyper", "loadglsentries", "DeclareBibliographyCategory", "DefineBibliographyStrings", "defbibheading", "DeclareFieldFormat", "AtEveryBibitem", "renewbibmacro", "newrobustcmd", "NewDocumentEnvironment", "ExplSyntaxOn", "ExplSyntaxOff", "DeclareSIUnit", "DeclareMathSymbol", "DeclareSymbolFont", "mathcode", "catcode", "newtheoremstyle", "declaretheorem", "colorlet", "setcitestyle", "bibpunct", "hypersetup", "pdfinfo", "includeonly", "extrafloats", "maxdeadcycles", "raggedbottom", "flushbottom", "onehalfspacing", "doublespacing", "singlespacing", "setcounter",
];

/// Os comandos de preâmbulo que abrem o trecho vão antes de `\begin{document}`.
fn separar_preambulo(s: &str) -> (String, String) {
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut fim_pre = 0;
    loop {
        while i < cs.len() && cs[i].is_whitespace() {
            i += 1;
        }
        if i >= cs.len() || cs[i] != '\\' {
            break;
        }
        let mut j = i + 1;
        while j < cs.len() && cs[j].is_ascii_alphabetic() {
            j += 1;
        }
        let nome: String = cs[i + 1..j].iter().collect();
        if !PREAMBULO.contains(&nome.as_str()) {
            break;
        }
        if j < cs.len() && cs[j] == '*' {
            j += 1;
        }
        loop {
            let mut k = j;
            while k < cs.len() && cs[k] == ' ' {
                k += 1;
            }
            if k < cs.len() && cs[k] == '\\' && k + 1 < cs.len() && cs[k + 1].is_ascii_alphabetic() && ["newcommand", "renewcommand", "providecommand", "def", "let", "newfontfamily", "newif", "newlength", "newcounter", "DeclareRobustCommand", "DeclareMathOperator"].contains(&nome.as_str()) {
                k += 1;
                while k < cs.len() && cs[k].is_ascii_alphabetic() {
                    k += 1;
                }
                j = k;
                continue;
            }
            if k < cs.len() && (cs[k] == '{' || cs[k] == '[') {
                let (a, b) = if cs[k] == '{' { ('{', '}') } else { ('[', ']') };
                let mut nivel = 0;
                let mut m = k;
                while m < cs.len() {
                    if cs[m] == a {
                        nivel += 1;
                    } else if cs[m] == b {
                        nivel -= 1;
                        if nivel == 0 {
                            break;
                        }
                    }
                    m += 1;
                }
                j = (m + 1).min(cs.len());
                continue;
            }
            break;
        }
        i = j;
        fim_pre = i;
    }
    (cs[..fim_pre].iter().collect(), cs[fim_pre..].iter().collect())
}

fn analisar_dados(texto: &str) -> Vec<Linha> {
    let (mut pkg, mut cls, mut pre) = (String::new(), String::from("article"), String::new());
    let mut saida = Vec::new();
    for bruta in texto.lines() {
        let l = bruta.trim_end();
        if l.is_empty() || l.starts_with("//") {
            continue;
        }
        if let Some(r) = l.strip_prefix('>') {
            let t = r.strip_prefix(' ').unwrap_or(r);
            saida.push(Linha::Literal(if t == "." { String::new() } else { t.to_string() }));
        } else if let Some(r) = l.strip_prefix("## ") {
            saida.push(Linha::Sub(r.to_string()));
        } else if let Some(r) = l.strip_prefix("# ") {
            saida.push(Linha::Secao(r.to_string()));
        } else if let Some(r) = l.strip_prefix("@pkg") {
            pkg = r.trim().trim_start_matches('-').to_string();
        } else if let Some(r) = l.strip_prefix("@cls") {
            cls = r.trim().to_string();
        } else if let Some(r) = l.strip_prefix("@pre") {
            pre = r.trim().replace("⏎", "\n");
        } else {
            let p: Vec<&str> = l.splitn(3, " @@ ").collect();
            let snippet = p.get(1).copied().unwrap_or("").replace("⏎", "\n");
            let (extra_pre, corpo) = match snippet.split_once(" §§ ") {
                Some((a, b)) => (a.to_string(), b.to_string()),
                None => separar_preambulo(&snippet),
            };
            let mut cls_local = cls.clone();
            let mut corpo = corpo;
            corpo = corpo.replace("\\begin{document}", "").replace("\\end{document}", "");
            if let Some(r) = corpo.strip_prefix("\\documentclass") {
                if let Some(f) = r.find('}') {
                    cls_local = r[..=f].to_string();
                    corpo = r[f + 1..].to_string();
                }
            }
            let mut doc = format!("\\documentclass{{{cls_local}}}\n");
            if cls_local.contains('{') {
                doc = format!("\\documentclass{cls_local}\n");
            }
            for pk in pkg.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                doc.push_str(&format!("\\usepackage{{{pk}}}\n"));
            }
            if !pre.is_empty() {
                doc.push_str(&pre);
                doc.push('\n');
            }
            doc.push_str(&extra_pre);
            doc.push_str("\n\\begin{document}\n");
            doc.push_str(&corpo);
            if !corpo.contains(MARCA) {
                doc.push_str(&format!("\n\nTexto {MARCA} final."));
            }
            doc.push_str("\n\\end{document}\n");
            saida.push(Linha::Item(Item { nome: p[0].to_string(), corpo: doc, espera: p.get(2).copied().unwrap_or("").trim().to_string() }));
        }
    }
    saida
}

/// Todos os trechos do documento (células, legendas e o texto das notas), com o bloco de origem.
fn trechos(d: &Documento) -> Vec<(usize, &Trecho)> {
    fn rec<'a>(i: usize, t: &'a Trecho, v: &mut Vec<(usize, &'a Trecho)>) {
        v.push((i, t));
        if let Some(Marca::Nota { texto, .. }) = t.marca.as_deref() {
            for n in texto {
                rec(i, n, v);
            }
        }
    }
    let mut v = Vec::new();
    for (i, b) in d.blocos.iter().enumerate() {
        for tx in b.conteudo.textos() {
            for t in tx {
                rec(i, t, &mut v);
            }
        }
    }
    v
}

fn texto_visivel(d: &Documento) -> String {
    let mut s = String::new();
    for (i, b) in d.blocos.iter().enumerate() {
        if matches!(b.conteudo, Conteudo::Codigo { .. } | Conteudo::Equacao { .. }) {
            continue;
        }
        let _ = i;
        for tx in b.conteudo.textos() {
            for t in tx {
                if t.estilo & MATEMATICA != 0 {
                    continue;
                }
                s.push_str(&t.texto);
                if let Some(Marca::Nota { texto, .. }) = t.marca.as_deref() {
                    for n in texto {
                        s.push_str(&n.texto);
                    }
                }
            }
            s.push('\n');
        }
    }
    s
}

fn vazou(visivel: &str, fonte: &str) -> Option<String> {
    for pad in ["\\begin", "\\end", "begin{", "end{"] {
        if visivel.contains(pad) {
            return Some(pad.to_string());
        }
    }
    let b = visivel.as_bytes();
    for i in 0..b.len().saturating_sub(1) {
        if b[i] == b'\\' && b[i + 1].is_ascii_alphabetic() {
            let fim = visivel[i..].find(|c: char| !(c == '\\' || c.is_ascii_alphabetic())).map_or(visivel.len(), |f| i + f);
            return Some(visivel[i..fim].to_string());
        }
    }
    // Nome de ambiente que aparece como `nome}` ou `{nome` no texto.
    let mut resto = fonte;
    while let Some(i) = resto.find("\\begin{") {
        let a = &resto[i + 7..];
        if let Some(f) = a.find('}') {
            let env = a[..f].trim_end_matches('*');
            if env.len() >= 4 && !["document", "table", "figure", "abstract", "quote", "center", "itemize", "enumerate", "description", "verbatim", "minipage", "tabular", "equation", "align", "gather", "multline", "matrix", "cases", "split", "array", "proof", "theorem", "tikzpicture", "frame", "block"].contains(&env) && (visivel.contains(&format!("{env}}}")) || visivel.contains(&format!("{{{env}"))) {
                return Some(format!("{{{env}}}"));
            }
        }
        resto = &resto[i + 7..];
    }
    None
}

fn ha(d: &Documento, f: impl Fn(&Trecho) -> bool) -> bool {
    trechos(d).iter().any(|(_, t)| t.texto.contains(MARCA) && f(t))
}
fn bloco_marca(d: &Documento) -> Option<&featherdocs_core::modelo::Bloco> {
    let alvo = trechos(d).into_iter().find(|(_, t)| t.texto.contains(MARCA)).map(|(i, _)| i)?;
    d.blocos.get(alvo)
}

/// O LaTeX das fórmulas (em destaque ou em linha) que trazem a marca.
fn formulas_com_marca(d: &Documento) -> Vec<String> {
    let mut v: Vec<String> = d.blocos.iter().filter_map(|b| match &b.conteudo {
        Conteudo::Equacao { latex } if latex.contains(MARCA) => Some(latex.clone()),
        _ => None,
    }).collect();
    v.extend(trechos(d).into_iter().filter(|(_, t)| t.estilo & MATEMATICA != 0 && t.texto.contains(MARCA)).map(|(_, t)| t.texto.clone()));
    v
}

fn criterio(d: &Documento, tok: &str, vis: &str) -> bool {
    let tok = tok.trim();
    if let Some(x) = tok.strip_prefix("txt:").or_else(|| tok.strip_prefix("exp:")) {
        return vis.contains(x) || d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Codigo { codigo, .. } if codigo.contains(x)));
    }
    if let Some(x) = tok.strip_prefix("sem:") {
        return !vis.contains(x);
    }
    if let Some(x) = tok.strip_prefix("h:") {
        let n: u8 = x.parse().unwrap_or(1);
        return d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Titulo { nivel, texto } if *nivel == n && texto.iter().any(|t| t.texto.contains(MARCA))));
    }
    if let Some(x) = tok.strip_prefix("linhas:") {
        let n: usize = x.parse().unwrap_or(0);
        return d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Tabela { linhas, .. } if linhas.len() == n));
    }
    if let Some(x) = tok.strip_prefix("colunas:") {
        let n: usize = x.parse().unwrap_or(0);
        return d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Tabela { linhas, .. } if linhas.iter().map(Vec::len).max() == Some(n)));
    }
    if let Some(x) = tok.strip_prefix("marca:") {
        return trechos(d).iter().any(|(_, t)| match (t.marca.as_deref(), x) {
            (Some(Marca::Nota { .. }), "nota") | (Some(Marca::Citacao { .. }), "cit") | (Some(Marca::Indice { .. }), "indice") | (Some(Marca::Campo { .. }), "campo") | (Some(Marca::Comentario { .. }), "coment") => true,
            _ => false,
        });
    }
    if let Some(x) = tok.strip_prefix("campo:") {
        return trechos(d).iter().any(|(_, t)| matches!(t.marca.as_deref(), Some(Marca::Campo { campo }) if campo == x));
    }
    if let Some(x) = tok.strip_prefix("estilocit:") {
        return format!("{:?}", d.bibliografia.estilo).to_lowercase() == x;
    }
    if let Some(x) = tok.strip_prefix("cab:") {
        let c = &d.extras.cabecalho;
        return [&c.esquerda, &c.centro, &c.direita].iter().any(|s| s.contains(x));
    }
    if let Some(x) = tok.strip_prefix("rod:") {
        let c = &d.extras.rodape;
        return [&c.esquerda, &c.centro, &c.direita].iter().any(|s| s.contains(x));
    }
    let bm = bloco_marca(d);
    match tok {
        "b" => ha(d, |t| t.estilo & NEGRITO != 0),
        "i" => ha(d, |t| t.estilo & ITALICO != 0),
        "u" => ha(d, |t| t.estilo & SUBLINHADO != 0),
        "s" => ha(d, |t| t.estilo & TACHADO != 0),
        "tt" => ha(d, |t| t.estilo & CODIGO != 0) || d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Codigo { codigo, .. } if codigo.contains(MARCA))),
        "serifa" | "sf" => ha(d, |t| t.estilo & SERIFA != 0),
        "m" => ha(d, |t| t.estilo & MATEMATICA != 0),
        "sup" => ha(d, |t| t.fmt.posicao == 1),
        "sub" => ha(d, |t| t.fmt.posicao == -1),
        "cor" => ha(d, |t| t.fmt.cor != 0),
        "realce" => ha(d, |t| t.fmt.realce != 0),
        "tam" => ha(d, |t| t.fmt.tamanho != 0.0),
        "link" => ha(d, |t| t.link.is_some()),
        "mark" => trechos(d).iter().any(|(_, t)| t.marca.is_some()),
        "cit" => d.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::Citacao { .. })),
        "cod" => d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Codigo { codigo, .. } if codigo.contains(MARCA))),
        "tab" => d.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::Tabela { .. })),
        "img" => d.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::Imagem { .. })),
        "imgleg" => d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Imagem { legenda, .. } if legenda.iter().any(|t| t.texto.contains(MARCA)))),
        "imgtam" => d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Imagem { largura, altura, .. } if *largura > 0.0 || *altura > 0.0)),
        "eq" => d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Equacao { latex } if latex.contains(MARCA))),
        "math" => d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Equacao { latex } if latex.contains(MARCA))) || ha(d, |t| t.estilo & MATEMATICA != 0),
        "reg" => d.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::Regua)),
        "qp" => d.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::QuebraDePagina)),
        "sum" => d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Sumario { lista: 0, .. })),
        "lof" => d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Sumario { lista: 1, .. })),
        "lot" => d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Sumario { lista: 2, .. })),
        "bib" => d.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::Bibliografia { .. })),
        "idx" => d.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::IndiceRemissivo)),
        "item" => d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Item { texto, .. } if texto.iter().any(|t| t.texto.contains(MARCA)))),
        "ord" => d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Item { ordenado: true, texto, .. } if texto.iter().any(|t| t.texto.contains(MARCA)))),
        "nivel2" => d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Item { nivel, texto, .. } if *nivel >= 1 && texto.iter().any(|t| t.texto.contains(MARCA)))),
        "centro" => bm.is_some_and(|b| b.atrib.alinhamento == Alinhamento::Centro),
        "dir" => bm.is_some_and(|b| b.atrib.alinhamento == Alinhamento::Direita),
        "esq" => bm.is_some_and(|b| b.atrib.alinhamento == Alinhamento::Esquerda),
        "just" => bm.is_some_and(|b| b.atrib.alinhamento == Alinhamento::Justificado),
        "fundo" => bm.is_some_and(|b| b.atrib.fundo != 0),
        "borda" => bm.is_some_and(|b| b.atrib.borda != 0),
        "entrelinha" => d.blocos.iter().any(|b| b.atrib.entrelinha != 0.0),
        "antes" => bm.is_some_and(|b| b.atrib.antes.is_some_and(|x| x > 0.0)),
        "recuo" => bm.is_some_and(|b| b.atrib.recuo_esq > 0.0 || b.atrib.recuo_dir > 0.0),
        "semprimeira" => d.blocos.iter().any(|b| b.atrib.primeira == 0.0),
        "primeira" => bm.is_some_and(|b| b.atrib.primeira != 0.0),
        "legenda" => d.blocos.iter().any(|b| b.atrib.legenda != 0),
        "mescla" => d.blocos.iter().any(|b| !b.mesclagens.is_empty()),
        "semnumero" => d.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::Titulo { .. }) && b.atrib.sem_numero),
        "lista:alfa" => d.blocos.iter().any(|b| b.atrib.lista == featherdocs_core::modelo::Lista::Alfabetica),
        "lista:romana" => d.blocos.iter().any(|b| b.atrib.lista == featherdocs_core::modelo::Lista::Romana),
        "cormarcador" => d.blocos.iter().any(|b| b.atrib.cor_marcador != 0),
        "paisagem" => d.pagina.largura > d.pagina.altura,
        "carta" => (d.pagina.largura - 612.0).abs() < 1.0,
        "a5" => (d.pagina.largura - 419.5).abs() < 3.0,
        "margem" => (d.pagina.margem_esq - 72.0).abs() > 0.5 || (d.pagina.margem_sup - 72.0).abs() > 0.5,
        "titulo" => !d.meta.titulo.is_empty(),
        "autor" => !d.meta.autor.is_empty(),
        "pagprimeira" => d.extras.primeira_diferente,
        "corpagina" => d.extras.cor_pagina != 0,
        "marcadagua" => !d.extras.marca.is_empty(),
        "bordapag" => d.extras.borda_cor != 0,
        "numlinhas" => d.extras.numerar_linhas != featherdocs_core::modelo::NumeracaoDeLinhas::Nenhuma,
        "fonte" => !d.bibliografia.fontes.is_empty(),
        "marcadores" => !d.marcadores.is_empty(),
        "estilos" => d.estilos.lista.len() > 0,
        "romano" => vis.contains("i.") || d.extras.rodape.centro.contains("romano"),
        "capnum" => d.estilos.numeracao == featherdocs_core::modelo::Numeracao::Capitulo,
        "decnum" => d.estilos.numeracao == featherdocs_core::modelo::Numeracao::Decimal,
        "nblocos2" => d.blocos.len() >= 2,
        "ign" | "ok" => true,
        _ => false,
    }
}

struct Res {
    marca: char,
    porque: String,
}

/// Item que lista vários comandos: confere se o trecho de teste exercita pelo menos metade deles.
fn grupo_incompleto(it: &Item) -> bool {
    let mut nomes: Vec<String> = Vec::new();
    let mut resto = it.nome.as_str();
    while let Some(i) = resto.find('\\') {
        let a = &resto[i + 1..];
        let fim = a.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(a.len());
        let n = a[..fim].to_string();
        if n.len() > 1 && !nomes.contains(&n) {
            nomes.push(n);
        }
        resto = a;
    }
    if nomes.len() < 4 {
        return false;
    }
    let corpo = it.corpo.split("\\begin{document}").nth(1).unwrap_or("");
    let achados = nomes.iter().filter(|n| corpo.contains(&format!("\\{n}")) || corpo.contains(&format!("{{{n}}}"))).count();
    achados * 2 < nomes.len()
}

fn avaliar(it: &Item) -> Res {
    let mut r = avaliar_um(it);
    if r.marca == 'x' && grupo_incompleto(it) {
        r.marca = '~';
        r.porque = format!("grupo: o teste cobre só parte dos comandos listados ({})", r.porque);
    }
    r
}

fn avaliar_um(it: &Item) -> Res {
    if let Some(m) = it.espera.strip_prefix('=') {
        let c = match m.trim() {
            "x" => 'x',
            "~" => '~',
            _ => ' ',
        };
        return Res { marca: c, porque: "marcado à mão (não medível pelo leitor)".into() };
    }
    let fonte = it.corpo.clone();
    let lido = catch_unwind(AssertUnwindSafe(|| latex::ler(&fonte)));
    let Ok(d) = lido else { return Res { marca: ' ', porque: "o leitor deu pânico".into() } };
    let vis = texto_visivel(&d);
    if std::env::var("VER").is_ok_and(|v| !v.is_empty() && it.nome.contains(&v)) {
        eprintln!("=== {}\n{}\n--- blocos:", it.nome, it.corpo);
        for b in &d.blocos {
            eprintln!("{:?} {:?}", b.conteudo, b.atrib);
        }
        eprintln!("extras={:?}\npagina={:?}", d.extras, d.pagina);
    }
    if let Some(v) = vazou(&vis, &it.corpo.split("\\begin{document}").nth(1).unwrap_or("")) {
        return Res { marca: ' ', porque: format!("vaza no texto: {v}") };
    }
    let esperado: Vec<&str> = it.espera.split('&').map(str::trim).filter(|s| !s.is_empty()).collect();
    let tem_marca = vis.contains(MARCA) || !trechos(&d).iter().all(|(_, t)| !t.texto.contains(MARCA)) || d.blocos.iter().any(|b| match &b.conteudo { Conteudo::Codigo { codigo, .. } => codigo.contains(MARCA), Conteudo::Equacao { latex } => latex.contains(MARCA), _ => false });
    let pede_sumir = esperado.iter().any(|e| e.starts_with("sem:") || *e == "ign");
    if !tem_marca && !esperado.iter().any(|e| e.starts_with("sem:") && e.contains(MARCA)) && !pede_sumir {
        return Res { marca: ' ', porque: "o texto do argumento sumiu".into() };
    }
    if esperado.is_empty() {
        if igual_ao_desconhecido(&it.corpo, &d) {
            if reconhecido_no_leitor(it) {
                return Res { marca: '~', porque: "reconhecido e ignorado (sem efeito no modelo)".into() };
            }
            return Res { marca: ' ', porque: "ignorado: o modelo é igual ao de um comando desconhecido".into() };
        }
        return Res { marca: '~', porque: "reconhecido; sem efeito declarado para conferir".into() };
    }
    let falhas: Vec<&&str> = esperado.iter().filter(|e| !criterio(&d, e, &vis)).collect();
    let lista = |v: &[&&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ");
    // Fórmulas: o modelo guarda o LaTeX cru e o desenho mostra o texto; só `[~]` até a montagem (`ok` libera).
    if esperado.iter().any(|e| *e == "math" || *e == "eq") {
        if !falhas.is_empty() {
            return Res { marca: ' ', porque: format!("fórmula não reconhecida: {}", lista(&falhas)) };
        }
        if esperado.contains(&"ok") {
            return Res { marca: 'x', porque: "fórmula montada (marcada à mão)".into() };
        }
        // Montagem medida: o feather-math compõe a fórmula com a marca sem nenhum comando desconhecido.
        let desconhecidos: Vec<String> = formulas_com_marca(&d).iter().flat_map(|f| feather_math::desconhecidos(f)).collect();
        if desconhecidos.is_empty() {
            return Res { marca: 'x', porque: "fórmula montada pelo feather-math (nenhum comando desconhecido)".into() };
        }
        return Res { marca: '~', porque: format!("fórmula montada, com comando desconhecido: {}", desconhecidos.join(" ")) };
    }
    // Critérios fracos (o texto sobrevive; nada some) não provam que o leitor entende o comando.
    let fraco = |e: &str| e == "ign" || e.starts_with("sem:") || e == "txt:ZQX";
    let so_fracos = esperado.iter().all(|e| fraco(e));
    let desconhecido = igual_ao_desconhecido(&it.corpo, &d);
    if std::env::var("VER").is_ok_and(|v| !v.is_empty() && it.nome.contains(&v)) {
        let m = mangular(&it.corpo);
        let d2 = latex::ler(&m);
        eprintln!("MANGLED:\n{m}\n--- blocos mangled:");
        for b in &d2.blocos {
            eprintln!("{:?}", b.conteudo);
        }
        eprintln!("desconhecido={desconhecido}");
    }
    let reconhecido = reconhecido_no_leitor(it);
    if so_fracos && desconhecido {
        let ign = esperado.contains(&"ign");
        return match (ign, reconhecido) {
            (true, true) => Res { marca: 'x', porque: "reconhecido; sem efeito visível no LaTeX também".into() },
            (true, false) => Res { marca: '~', porque: "não citado no leitor; passa sem quebrar e sem efeito".into() },
            (false, true) => Res { marca: '~', porque: "reconhecido e ignorado (sem efeito no modelo)".into() },
            (false, false) => Res { marca: ' ', porque: "ignorado: o modelo é igual ao de um comando desconhecido".into() },
        };
    }
    if !falhas.is_empty() {
        if desconhecido && !reconhecido {
            return Res { marca: ' ', porque: "ignorado: o modelo é igual ao de um comando desconhecido".into() };
        }
        return Res { marca: '~', porque: format!("falta: {}", lista(&falhas)) };
    }
    Res { marca: 'x', porque: "efeito conferido".into() }
}

const PROTEGIDOS: &[&str] = &["begin", "end", "documentclass", "usepackage", "item", "par", "newcommand", "renewcommand", "providecommand", "def", "document", "label", "ref", "textbf", "emph"];

/// Troca o nome de cada comando (e ambiente) por um inexistente, para saber se o leitor lhe dá algum efeito.
fn mangular(fonte: &str) -> String {
    let mut o = String::new();
    let cs: Vec<char> = fonte.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '\\' && i + 1 < cs.len() && cs[i + 1].is_ascii_alphabetic() {
            let mut j = i + 1;
            while j < cs.len() && cs[j].is_ascii_alphabetic() {
                j += 1;
            }
            let nome: String = cs[i + 1..j].iter().collect();
            o.push('\\');
            if !["begin", "end", "documentclass", "usepackage", "newcommand", "renewcommand", "item", "par", "def", "providecommand"].contains(&nome.as_str()) {
                o.push_str("zz");
            }
            o.push_str(&nome);
            i = j;
            continue;
        }
        o.push(cs[i]);
        i += 1;
    }
    // Ambientes e pacotes: o nome entre chaves depois de begin/end/usepackage.
    for pre in ["\\begin{", "\\end{", "\\usepackage{", "\\usepackage[", "\\documentclass["] {
        let _ = pre;
    }
    let mut r = String::new();
    let mut resto = o.as_str();
    while let Some(k) = resto.find("\\begin{").map(|a| (a, 7)).into_iter().chain(resto.find("\\end{").map(|a| (a, 5))).min() {
        let (a, n) = k;
        let (antes, depois) = resto.split_at(a + n);
        r.push_str(antes);
        if !depois.starts_with("document") {
            r.push_str("zz");
        }
        resto = depois;
    }
    r.push_str(resto);
    let _ = PROTEGIDOS;
    r
}

fn limpar_zz(s: &str) -> String {
    let mut o = String::new();
    let mut resto = s;
    while let Some(i) = resto.find("zz") {
        o.push_str(&resto[..i]);
        let fim = resto[i + 2..].find(|c: char| !c.is_ascii_alphabetic()).map_or(resto.len(), |f| i + 2 + f);
        resto = &resto[fim..];
    }
    o.push_str(resto);
    o.replace('\\', "")
}

fn igual_ao_desconhecido(fonte: &str, d: &Documento) -> bool {
    let m = mangular(fonte);
    if m == fonte {
        return false;
    }
    let Ok(d2) = catch_unwind(AssertUnwindSafe(|| latex::ler(&m))) else { return false };
    let a = limpar_zz(&format!("{:?}{:?}{:?}{:?}", d.blocos, d.extras, d.pagina, d.bibliografia));
    let b = limpar_zz(&format!("{:?}{:?}{:?}{:?}", d2.blocos, d2.extras, d2.pagina, d2.bibliografia));
    a == b
}

/// O código do leitor (latex.rs e a pasta latex/), lido na hora: serve só para saber se um nome é citado.
fn fonte_leitor() -> &'static str {
    static FONTE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    FONTE.get_or_init(|| {
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        let mut t = fs::read_to_string(format!("{base}/latex.rs")).unwrap_or_default();
        if let Ok(rd) = fs::read_dir(format!("{base}/latex")) {
            for e in rd.flatten() {
                if e.path().extension().is_some_and(|x| x == "rs") {
                    t.push_str(&fs::read_to_string(e.path()).unwrap_or_default());
                }
            }
        }
        t
    })
}

/// Algum comando ou ambiente do trecho aparece citado no leitor (reconhecido, ainda que só para ignorar)?
fn reconhecido_no_leitor(it: &Item) -> bool {
    // Os nomes em teste: comandos (`\\x`) e palavras soltas entre crases do nome do item.
    let mut nomes: Vec<String> = Vec::new();
    for (i, trecho) in it.nome.split('`').enumerate() {
        if i % 2 == 0 {
            continue;
        }
        let mut resto = trecho;
        while let Some(k) = resto.find('\\') {
            let a = &resto[k + 1..];
            let fim = a.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(a.len());
            nomes.push(a[..fim].to_string());
            resto = a;
        }
        for palavra in trecho.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
            if palavra.len() >= 3 && !palavra.starts_with('-') {
                nomes.push(palavra.to_string());
            }
        }
    }
    let _ = &it.corpo;
    nomes.iter().filter(|n| n.len() > 1 && !["begin", "end", "item", "par", "document", "usepackage", "documentclass", "ZQX"].contains(&n.as_str())).any(|n| fonte_leitor().contains(&format!("\"{n}\"")))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pega = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let dados_path = pega("--dados").unwrap_or_else(|| concat!(env!("CARGO_MANIFEST_DIR"), "/dados/paridade_latex.txt").to_string());
    let detalhe = args.iter().any(|a| a == "--detalhe");
    let dados = fs::read_to_string(&dados_path).expect("dados/paridade_latex.txt");
    let linhas = analisar_dados(&dados);

    // Primeiro passa: mede tudo, junta por seção.
    struct Sec {
        nome: String,
        corpo: Vec<String>,
        x: usize,
        total: usize,
        p: usize,
    }
    let mut secs: Vec<Sec> = Vec::new();
    let mut cabeca: Vec<String> = Vec::new();
    let mut cauda: Vec<String> = Vec::new();
    let mut medidas: Vec<(String, char)> = Vec::new();
    for l in &linhas {
        match l {
            Linha::Literal(t) => {
                if let Some(s) = secs.last_mut() {
                    if t.starts_with("@fim") {
                        cauda.push(t.trim_start_matches("@fim").trim().to_string());
                    } else if cauda.is_empty() {
                        s.corpo.push(t.clone());
                    } else {
                        cauda.push(t.clone());
                    }
                } else {
                    cabeca.push(t.clone());
                }
            }
            Linha::Secao(n) => secs.push(Sec { nome: n.clone(), corpo: Vec::new(), x: 0, total: 0, p: 0 }),
            Linha::Sub(n) => {
                if let Some(s) = secs.last_mut() {
                    s.corpo.push(format!("\n### {n}"));
                }
            }
            Linha::Item(it) => {
                let r = avaliar(it);
                let s = secs.last_mut().expect("item fora de seção");
                s.total += 1;
                match r.marca {
                    'x' => s.x += 1,
                    '~' => s.p += 1,
                    _ => {}
                }
                medidas.push((it.nome.clone(), r.marca));
                let nota = if detalhe { format!(" — _{}_", r.porque) } else { String::new() };
                s.corpo.push(format!("- [{}] {}{nota}", r.marca, it.nome));
            }
        }
    }
    let tx: usize = secs.iter().map(|s| s.x).sum();
    let tp: usize = secs.iter().map(|s| s.p).sum();
    let tt: usize = secs.iter().map(|s| s.total).sum();
    let mut tabela = String::from("| Seção | Pronto | Parcial | Falta | Total | Paridade |\n|---|---|---|---|---|---|\n");
    for (i, s) in secs.iter().enumerate() {
        tabela.push_str(&format!("| {}. {} | {} | {} | {} | {} | {}% |\n", i + 1, s.nome, s.x, s.p, s.total - s.x - s.p, s.total, if s.total == 0 { 0 } else { s.x * 100 / s.total }));
    }
    tabela.push_str(&format!("| **Total** | **{tx}** | **{tp}** | **{}** | **{tt}** | **{}%** |\n", tt - tx - tp, if tt == 0 { 0 } else { tx * 100 / tt }));
    let total = format!("**Total: {tx} / {tt} ({}%)** pronto · {tp} parcial · {} falta", if tt == 0 { 0 } else { tx * 100 / tt }, tt - tx - tp);
    let mut md = String::new();
    for l in &cabeca {
        md.push_str(&l.replace("{{TABELA}}", &tabela).replace("{{TOTAL}}", &total));
        md.push('\n');
    }
    for (i, s) in secs.iter().enumerate() {
        md.push_str(&format!("\n## {}. {} ({} / {})\n", i + 1, s.nome, s.x, s.total));
        for c in &s.corpo {
            md.push_str(c);
            md.push('\n');
        }
    }
    md.push('\n');
    for l in &cauda {
        md.push_str(&l.replace("{{TABELA}}", &tabela).replace("{{TOTAL}}", &total));
        md.push('\n');
    }

    if let Some(ant) = pega("--comparar") {
        let velho = fs::read_to_string(&ant).unwrap_or_default();
        let mut mapa: HashMap<String, char> = HashMap::new();
        for l in velho.lines() {
            if let Some(r) = l.strip_prefix("- [") {
                if let Some(c) = r.chars().next() {
                    mapa.insert(r[4..].split(" — _").next().unwrap_or("").trim().to_string(), c);
                }
            }
        }
        let rank = |c: char| match c { 'x' => 2, '~' => 1, _ => 0 };
        for (n, c) in &medidas {
            if let Some(v) = mapa.get(n.trim()) {
                if rank(*v) > rank(*c) {
                    eprintln!("ARQUIVO MAIS ALTO que a medição: [{v}] -> [{c}] {n}");
                } else if rank(*v) < rank(*c) {
                    eprintln!("medição melhorou: [{v}] -> [{c}] {n}");
                }
            }
        }
    }
    match pega("--saida") {
        Some(p) => {
            fs::write(&p, md).expect("gravar saída");
            eprintln!("gravado {p}: {tx} prontos, {tp} parciais, {} faltam, de {tt}", tt - tx - tp);
        }
        None => print!("{md}"),
    }
}
