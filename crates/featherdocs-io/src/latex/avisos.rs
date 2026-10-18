//! O aviso de compatibilidade: o que o leitor de LaTeX **não** entendeu (classe, pacote, comando,
//! ambiente, arquivo do `\input`), com a linha do fonte onde aparece pela primeira vez e quantas
//! vezes aparece. O leitor nunca falha por causa disso: o texto continua saindo, só que sem o
//! efeito daquele comando; o aviso diz à pessoa o que conferir.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// O tipo do que não foi entendido.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TipoDeAviso {
    Classe,
    Pacote,
    Ambiente,
    Comando,
    /// Arquivo do `\input`/`\include` que não foi achado ao lado do `.tex`.
    Arquivo,
    /// Comando conhecido que o leitor descarta: lido, mas sem efeito no documento (`\hspace`).
    SemEfeito,
    /// O que o pdfLaTeX também daria como erro. O nome é "mensagem: trecho", e o trecho acha a linha.
    Erro,
    /// O que o LaTeX avisa no log (sem parar). Mesmo formato de nome do `Erro`.
    AvisoDoLatex,
}

impl TipoDeAviso {
    /// O nome em português, para a lista da interface.
    pub fn rotulo(self) -> &'static str {
        match self {
            Self::Classe => "Classe",
            Self::Pacote => "Pacote",
            Self::Ambiente => "Ambiente",
            Self::Comando => "Comando",
            Self::Arquivo => "Arquivo",
            Self::SemEfeito => "Sem efeito",
            Self::Erro => "Erro do LaTeX",
            Self::AvisoDoLatex => "Aviso do LaTeX",
        }
    }
}

/// Um item que o leitor não entendeu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Aviso {
    pub tipo: TipoDeAviso,
    /// O nome como se escreve: `\foo`, `tikzpicture`, `pgfplots`, `capitulo1.tex`.
    pub nome: String,
    /// A linha (a partir de 1) da primeira vez no fonte; 0 = não achada (veio de macro).
    pub linha: usize,
    /// Quantas vezes o leitor passou por ele.
    pub vezes: usize,
    /// Todas as linhas onde ele aparece (a primeira é `linha`), para marcar no código.
    pub linhas: Vec<usize>,
}

impl Aviso {
    /// Uma linha legível: "Comando \foo — linha 12 (3 vezes)".
    pub fn descricao(&self) -> String {
        let mut s = format!("{} {}", self.tipo.rotulo(), self.nome);
        if self.linha > 0 {
            s.push_str(&format!(" — linha {}", self.linha));
        }
        if self.vezes > 1 {
            s.push_str(&format!(" ({} vezes)", self.vezes));
        }
        s
    }
}

/// O documento lido e o que ficou de fora.
#[derive(Clone, Debug)]
pub struct Leitura {
    pub documento: featherdocs_core::modelo::Documento,
    pub avisos: Vec<Aviso>,
}

/// O que o leitor junta enquanto lê (compartilhado entre os analisadores filhos): os avisos, os
/// rótulos (`\label`) presos a blocos, os números que cada rótulo mostra e os contadores dos
/// ambientes numerados (teoremas, notas avulsas).
#[derive(Debug, Default)]
pub(super) struct Juntado {
    pub avisos: Vec<(TipoDeAviso, String)>,
    /// Os rótulos de cada bloco marcado (o bloco leva `id` = `MARCA_DE_ROTULO | índice` até o fim da leitura).
    pub rotulos: Vec<Vec<String>>,
}

/// O bit alto do `id` provisório de um bloco que tem rótulo.
pub(super) const MARCA_DE_ROTULO: u64 = 1 << 62;

#[derive(Clone, Debug, Default)]
pub(super) struct Coletor(pub(super) Rc<RefCell<Juntado>>);

impl Coletor {
    pub(super) fn anotar(&self, tipo: TipoDeAviso, nome: &str) {
        if nome.is_empty() {
            return;
        }
        self.0.borrow_mut().avisos.push((tipo, nome.to_string()));
    }

    /// Guarda os rótulos de um bloco e devolve o `id` provisório que o identifica.
    pub(super) fn rotular(&self, rotulos: Vec<String>) -> u64 {
        let mut j = self.0.borrow_mut();
        j.rotulos.push(rotulos);
        MARCA_DE_ROTULO | (j.rotulos.len() as u64 - 1)
    }

    pub(super) fn comando(&self, nome: &str) {
        // Só comandos com nome de letras (os de um símbolo, como `\@`, são miudezas do TeX).
        if nome.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) && !comando_conhecido(nome) {
            self.anotar(TipoDeAviso::Comando, &format!("\\{nome}"));
        }
    }

    pub(super) fn ambiente(&self, nome: &str) {
        self.anotar(TipoDeAviso::Ambiente, nome);
    }

    /// Os avisos juntados por nome, com a linha da primeira ocorrência no `fonte`.
    pub(super) fn consolidar(&self, fonte: &str) -> Vec<Aviso> {
        let mut contagem: HashMap<(TipoDeAviso, String), usize> = HashMap::new();
        let mut ordem = Vec::new();
        for (t, n) in self.0.borrow().avisos.iter() {
            let chave = (*t, n.clone());
            let c = contagem.entry(chave.clone()).or_insert(0);
            if *c == 0 {
                ordem.push(chave);
            }
            *c += 1;
        }
        let mut saida: Vec<Aviso> = ordem
            .into_iter()
            .map(|(tipo, nome)| {
                let vezes = contagem[&(tipo, nome.clone())];
                let linhas = linhas_de(fonte, tipo, &nome);
                Aviso { tipo, linha: linhas.first().copied().unwrap_or(0), nome, vezes, linhas }
            })
            .collect();
        saida.sort_by_key(|a| (a.linha == 0, a.linha, a.tipo));
        saida
    }
}

/// As linhas (1…) onde o item aparece no fonte; vazia se não aparece (veio de macro). Comando e
/// ambiente são lidos no corpo: se aparecem nele, as do preâmbulo (já tratadas lá) ficam de fora.
fn linhas_de(fonte: &str, tipo: TipoDeAviso, nome: &str) -> Vec<usize> {
    let posicoes: Vec<usize> = match tipo {
        TipoDeAviso::Erro | TipoDeAviso::AvisoDoLatex => {
            let trecho = nome.rsplit_once(": ").map_or(nome, |(_, t)| t);
            fonte.match_indices(trecho).map(|(i, _)| i).collect()
        }
        TipoDeAviso::Comando | TipoDeAviso::SemEfeito => {
            fonte.match_indices(nome).filter(|(i, _)| !fonte[i + nome.len()..].starts_with(|c: char| c.is_ascii_alphabetic())).map(|(i, _)| i).collect()
        }
        TipoDeAviso::Ambiente => fonte.match_indices(&format!("\\begin{{{nome}}}")).map(|(i, _)| i).collect(),
        TipoDeAviso::Pacote | TipoDeAviso::Classe | TipoDeAviso::Arquivo => fonte
            .match_indices(nome)
            .filter(|(i, _)| {
                let antes = fonte[..*i].chars().next_back();
                let depois = fonte[i + nome.len()..].chars().next();
                !antes.is_some_and(|c| c.is_alphanumeric() || c == '-') && !depois.is_some_and(|c| c.is_alphanumeric() || c == '-')
            })
            .map(|(i, _)| i)
            .collect(),
    };
    let corpo = fonte.find("\\begin{document}").unwrap_or(0);
    let no_corpo: Vec<usize> = posicoes.iter().copied().filter(|p| *p > corpo).collect();
    let escolhidas = if matches!(tipo, TipoDeAviso::Comando | TipoDeAviso::SemEfeito | TipoDeAviso::Ambiente) && !no_corpo.is_empty() { no_corpo } else { posicoes };
    let mut linhas: Vec<usize> = escolhidas.into_iter().map(|p| fonte[..p].matches('\n').count() + 1).collect();
    linhas.dedup();
    linhas
}

/// Comandos que o leitor reconhece de propósito sem efeito visível (o que o FeatherDocs faz do
/// jeito dele, ou que só ajusta o TeX): não são avisados.
fn comando_conhecido(nome: &str) -> bool {
    const SEM_EFEITO: &[&str] = &[
        "maketitle", "item", "end", "begin", "hyphenation", "frenchspacing", "sloppy", "fussy", "nopagebreak", "allowbreak", "ignorespaces", "unskip",
        "selectfont", "normalcolor", "makeatletter", "makeatother", "onecolumn", "twocolumn", "balance", "FloatBarrier", "addcontentsline", "addtocontents",
        "phantomsection", "pagenumbering", "thepage", "frontmatter", "mainmatter", "backmatter", "appendix", "hypersetup", "urlstyle", "graphicspath",
        "DeclareGraphicsExtensions", "captionsetup", "sisetup", "lstset", "setminted", "setcounter", "addtocounter", "stepcounter", "newcounter",
        "newlength", "addtolength", "settowidth", "linespread", "onehalfspacing", "doublespacing", "singlespacing", "titleformat", "titlespacing",
        "pagestyle", "thispagestyle", "fancyhf", "fancyhead", "fancyfoot", "lhead", "chead", "rhead", "lfoot", "cfoot", "rfoot", "headrulewidth",
        "footrulewidth", "renewcommand", "newcommand", "providecommand", "newenvironment", "renewenvironment", "def", "let", "newtheorem",
        "theoremstyle", "bibliographystyle", "addbibresource", "printbibliography", "nocite", "makeindex", "index", "glsaddall", "printglossaries",
        "listfiles", "special", "pdfbookmark", "enlargethispage", "kern", "hskip", "vskip", "penalty", "break", "ensuremath", "xspace", "@",
        "noindent", "indent", "par", "relax", "protect", "null", "strut", "mbox", "hbox", "vbox", "centering", "raggedright", "raggedleft",
        "RaggedRight", "RaggedLeft", "justifying", "small", "normalsize", "large", "Large", "LARGE", "huge", "Huge", "tiny", "scriptsize", "footnotesize",
    ];
    SEM_EFEITO.contains(&nome)
}

/// Pacotes cujo efeito o leitor reproduz, ou que não mudam o que se vê (codificação, fontes que o
/// FeatherDocs troca pelas dele, ajustes finos do TeX). Os outros entram no aviso.
pub(super) fn pacote_conhecido(nome: &str) -> bool {
    const CONHECIDOS: &[&str] = &[
        // Codificação, língua e fontes (o FeatherDocs usa as dele).
        "inputenc", "fontenc", "babel", "polyglossia", "lmodern", "times", "mathptmx", "helvet", "palatino", "mathpazo", "charter", "courier",
        "newtxtext", "newtxmath", "fontspec", "textcomp", "microtype", "lipsum", "blindtext", "csquotes", "ragged2e", "setspace", "indentfirst",
        "parskip", "xspace", "etoolbox", "ifthen", "calc", "xparse", "url", "cmap", "fix-cm", "fixltx2e", "latexsym", "upquote", "tgheros", "tgtermes",
        "sourcesanspro", "roboto", "fontawesome", "fontawesome5", "marvosym", "ae", "aecompl", "eurosym", "lastpage", "placeins", "afterpage",
        // Página, cores, links, figuras e tabelas.
        "geometry", "xcolor", "color", "hyperref", "bookmark", "graphicx", "graphics", "float", "caption", "subcaption", "subfig", "wrapfig",
        "tabularx", "booktabs", "multirow", "longtable", "colortbl", "array", "multicol", "fancyhdr", "titlesec", "enumitem", "listings",
        "minted", "verbatim", "fancyvrb", "ulem", "soul", "siunitx", "footmisc", "endnotes", "makeidx", "imakeidx",
        // Matemática (montada por outro motor) e referências.
        "amsmath", "amssymb", "amsfonts", "amsthm", "mathtools", "bm", "natbib", "biblatex", "cite", "cleveref", "varioref", "filecontents",
        "abntex2cite", "tcolorbox", "tikz",
        // Só valem pelos comandos e ambientes deles, que entram no aviso um a um quando o leitor não os faz.
        "iftex", "ifxetex", "ifluatex", "unicode-math", "newunicodechar", "pgf", "pdflscape", "lscape", "breakcites", "xargs", "xifthen",
        "supertabular", "fontawesome6", "ccicons", "pifont", "mdframed", "censor",
    ];
    CONHECIDOS.contains(&nome)
}

/// Classes que o leitor sabe ler (as outras saem como `article`, com aviso).
pub(super) fn classe_conhecida(nome: &str) -> bool {
    matches!(nome, "article" | "report" | "book" | "letter" | "scrartcl" | "scrreprt" | "scrbook" | "extarticle" | "extreport" | "memoir" | "abntex2" | "minimal" | "exam")
}

/// Os pacotes de `\usepackage[…]{a,b}` e `\RequirePackage` do preâmbulo.
pub(super) fn pacotes_do_preambulo(preambulo: &str) -> Vec<String> {
    let mut saida = Vec::new();
    for marca in ["\\usepackage", "\\RequirePackage"] {
        for (i, _) in preambulo.match_indices(marca) {
            let resto = &preambulo[i + marca.len()..];
            let resto = super::opcional(resto).map_or(resto, |(_, k)| &resto[k..]);
            if let Some((g, _)) = super::grupo(resto) {
                saida.extend(g.split(',').map(str::trim).filter(|p| !p.is_empty()).map(str::to_string));
            }
        }
    }
    saida
}

/// O nome da classe em `\documentclass[…]{nome}`.
pub(super) fn classe_do_preambulo(preambulo: &str) -> Option<String> {
    let i = preambulo.find("\\documentclass")?;
    let resto = &preambulo[i + 14..];
    let resto = super::opcional(resto).map_or(resto, |(_, k)| &resto[k..]);
    super::grupo(resto).map(|(g, _)| g.trim().to_string())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn junta_por_nome_e_acha_a_linha() {
        let c = Coletor::default();
        c.comando("foo");
        c.comando("foo");
        c.comando("textbf");
        c.ambiente("tikzpicture");
        let fonte = "a\n\\foobar\n\\foo{x}\n\\begin{tikzpicture}\n";
        let v = c.consolidar(fonte);
        assert_eq!(v.len(), 3, "{v:?}");
        assert_eq!((v[0].nome.as_str(), v[0].linha, v[0].vezes), ("\\foo", 3, 2));
        assert_eq!((v[1].tipo, v[1].linha), (TipoDeAviso::Ambiente, 4));
        assert_eq!((v[2].nome.as_str(), v[2].linha), ("\\textbf", 0), "o que não está no fonte (veio de macro) vai para o fim");
        assert_eq!(v[0].descricao(), "Comando \\foo — linha 3 (2 vezes)");
    }

    #[test]
    fn comandos_de_ajuste_nao_sao_avisados() {
        let c = Coletor::default();
        c.comando("selectfont");
        c.comando("@");
        assert!(c.consolidar("").is_empty());
    }
}
