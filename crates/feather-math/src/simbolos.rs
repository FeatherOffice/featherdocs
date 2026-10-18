//! Os nomes do LaTeX (núcleo, `amsmath`, `amssymb`, `latexsym`, `stmaryrd` comuns) → caractere
//! Unicode e classe do átomo. A fonte Latin Modern Math traz todos estes caracteres.

use crate::arvore::Classe::{self, Abre, Bin, Fecha, Interno, Op, Ord, Pont, Rel};
use std::collections::HashMap;
use std::sync::OnceLock;

#[rustfmt::skip]
static TABELA: &[(&str, char, Classe)] = &[
    // Gregas minúsculas (itálico pelo alfabeto normal) e maiúsculas (em pé).
    ("alpha", 'α', Ord), ("beta", 'β', Ord), ("gamma", 'γ', Ord), ("delta", 'δ', Ord),
    ("epsilon", 'ϵ', Ord), ("varepsilon", 'ε', Ord), ("zeta", 'ζ', Ord), ("eta", 'η', Ord),
    ("theta", 'θ', Ord), ("vartheta", 'ϑ', Ord), ("iota", 'ι', Ord), ("kappa", 'κ', Ord),
    ("varkappa", 'ϰ', Ord), ("lambda", 'λ', Ord), ("mu", 'μ', Ord), ("nu", 'ν', Ord),
    ("xi", 'ξ', Ord), ("omicron", 'ο', Ord), ("pi", 'π', Ord), ("varpi", 'ϖ', Ord),
    ("rho", 'ρ', Ord), ("varrho", 'ϱ', Ord), ("sigma", 'σ', Ord), ("varsigma", 'ς', Ord),
    ("tau", 'τ', Ord), ("upsilon", 'υ', Ord), ("phi", 'ϕ', Ord), ("varphi", 'φ', Ord),
    ("chi", 'χ', Ord), ("psi", 'ψ', Ord), ("omega", 'ω', Ord), ("digamma", 'ϝ', Ord),
    ("Gamma", 'Γ', Ord), ("Delta", 'Δ', Ord), ("Theta", 'Θ', Ord), ("Lambda", 'Λ', Ord),
    ("Xi", 'Ξ', Ord), ("Pi", 'Π', Ord), ("Sigma", 'Σ', Ord), ("Upsilon", 'Υ', Ord),
    ("Phi", 'Φ', Ord), ("Psi", 'Ψ', Ord), ("Omega", 'Ω', Ord),
    ("varGamma", '𝛤', Ord), ("varDelta", '𝛥', Ord), ("varTheta", '𝛩', Ord), ("varLambda", '𝛬', Ord),
    ("varXi", '𝛯', Ord), ("varPi", '𝛱', Ord), ("varSigma", '𝛴', Ord), ("varUpsilon", '𝛶', Ord),
    ("varPhi", '𝛷', Ord), ("varPsi", '𝛹', Ord), ("varOmega", '𝛺', Ord),
    ("aleph", 'ℵ', Ord), ("beth", 'ℶ', Ord), ("gimel", 'ℷ', Ord), ("daleth", 'ℸ', Ord),
    // Símbolos comuns (Ord).
    ("infty", '∞', Ord), ("partial", '∂', Ord), ("nabla", '∇', Ord), ("hbar", 'ℏ', Ord),
    ("hslash", 'ℏ', Ord), ("ell", 'ℓ', Ord), ("wp", '℘', Ord), ("Re", 'ℜ', Ord), ("Im", 'ℑ', Ord),
    ("forall", '∀', Ord), ("exists", '∃', Ord), ("nexists", '∄', Ord), ("emptyset", '∅', Ord),
    ("varnothing", '∅', Ord), ("neg", '¬', Ord), ("lnot", '¬', Ord), ("top", '⊤', Ord),
    ("bot", '⊥', Ord), ("angle", '∠', Ord), ("measuredangle", '∡', Ord), ("sphericalangle", '∢', Ord),
    ("triangle", '△', Ord), ("triangledown", '▽', Ord), ("vartriangle", '△', Ord),
    ("blacktriangle", '▲', Ord), ("blacktriangledown", '▼', Ord),
    ("Box", '□', Ord), ("square", '□', Ord), ("blacksquare", '■', Ord), ("Diamond", '♢', Ord),
    ("lozenge", '◊', Ord), ("blacklozenge", '♦', Ord), ("clubsuit", '♣', Ord),
    ("diamondsuit", '♢', Ord), ("heartsuit", '♡', Ord), ("spadesuit", '♠', Ord),
    ("flat", '♭', Ord), ("natural", '♮', Ord), ("sharp", '♯', Ord), ("prime", '′', Ord),
    ("backprime", '‵', Ord), ("vdots", '⋮', Ord), ("surd", '√', Ord),
    ("imath", '𝚤', Ord), ("jmath", '𝚥', Ord), ("mho", '℧', Ord), ("eth", 'ð', Ord),
    ("Finv", 'Ⅎ', Ord), ("Game", '⅁', Ord), ("complement", '∁', Ord), ("diagup", '∕', Ord),
    ("diagdown", '∖', Ord), ("checkmark", '✓', Ord), ("maltese", '✠', Ord),
    ("circledS", 'Ⓢ', Ord), ("degree", '°', Ord), ("backslash", '\\', Ord),
    ("bigstar", '⋆', Ord), ("Bbbk", '𝕜', Ord), ("varkappa", 'ϰ', Ord),
    ("|", '‖', Ord), ("Vert", '‖', Ord), ("vert", '|', Ord), ("#", '#', Ord), ("$", '$', Ord),
    ("%", '%', Ord), ("&", '&', Ord), ("_", '_', Ord), ("S", '§', Ord), ("P", '¶', Ord),
    ("pounds", '£', Ord), ("copyright", '©', Ord), ("dag", '†', Ord), ("ddag", '‡', Ord),
    ("textbackslash", '\\', Ord), ("AA", 'Å', Ord),
    ("yen", '¥', Ord), ("circledR", '®', Ord), ("arrowvert", '⏐', Ord), ("Arrowvert", '‖', Ord),
    ("vline", '|', Ord), ("mathellipsis", '…', Interno), ("bracevert", '⎪', Ord), ("smallsum", '∑', Ord), ("euro", '€', Ord), ("textdegree", '°', Ord),
    // Reticências.
    ("ldots", '…', Interno), ("dots", '…', Interno), ("dotso", '…', Interno), ("dotsc", '…', Interno),
    ("cdots", '⋯', Interno), ("dotsb", '⋯', Interno), ("dotsm", '⋯', Interno), ("dotsi", '⋯', Interno),
    ("ddots", '⋱', Interno), ("iddots", '⋰', Interno), ("adots", '⋰', Interno),
    ("cdotp", '·', Pont), ("ldotp", '.', Pont), ("colon", ':', Pont),
    // Operações binárias.
    ("pm", '±', Bin), ("mp", '∓', Bin), ("times", '×', Bin), ("div", '÷', Bin), ("cdot", '⋅', Bin),
    ("ast", '∗', Bin), ("star", '⋆', Bin), ("circ", '∘', Bin), ("bullet", '∙', Bin),
    ("oplus", '⊕', Bin), ("ominus", '⊖', Bin), ("otimes", '⊗', Bin), ("oslash", '⊘', Bin),
    ("odot", '⊙', Bin), ("bigcirc", '◯', Bin), ("cap", '∩', Bin), ("cup", '∪', Bin),
    ("uplus", '⊎', Bin), ("sqcap", '⊓', Bin), ("sqcup", '⊔', Bin), ("vee", '∨', Bin),
    ("lor", '∨', Bin), ("wedge", '∧', Bin), ("land", '∧', Bin), ("setminus", '∖', Bin),
    ("smallsetminus", '∖', Bin), ("wr", '≀', Bin), ("diamond", '⋄', Bin),
    ("bigtriangleup", '△', Bin), ("bigtriangledown", '▽', Bin), ("triangleleft", '◁', Bin),
    ("triangleright", '▷', Bin), ("lhd", '⊲', Bin), ("rhd", '⊳', Bin), ("unlhd", '⊴', Bin),
    ("unrhd", '⊵', Bin), ("amalg", '⨿', Bin), ("dagger", '†', Bin), ("ddagger", '‡', Bin),
    ("dotplus", '∔', Bin), ("ltimes", '⋉', Bin), ("rtimes", '⋊', Bin),
    ("leftthreetimes", '⋋', Bin), ("rightthreetimes", '⋌', Bin), ("curlywedge", '⋏', Bin),
    ("curlyvee", '⋎', Bin), ("barwedge", '⌅', Bin), ("veebar", '⊻', Bin),
    ("doublebarwedge", '⌅', Bin), ("boxplus", '⊞', Bin), ("boxminus", '⊟', Bin),
    ("boxtimes", '⊠', Bin), ("boxdot", '⊡', Bin), ("circledast", '⊛', Bin),
    ("circledcirc", '⊚', Bin), ("circleddash", '⊝', Bin), ("centerdot", '⋅', Bin),
    ("intercal", '⊺', Bin), ("Cap", '⋒', Bin), ("Cup", '⋓', Bin), ("divideontimes", '⋇', Bin),
    ("doublecap", '⋒', Bin), ("doublecup", '⋓', Bin),
    // Relações.
    ("leq", '≤', Rel), ("le", '≤', Rel), ("geq", '≥', Rel), ("ge", '≥', Rel), ("neq", '≠', Rel),
    ("ne", '≠', Rel), ("equiv", '≡', Rel), ("approx", '≈', Rel), ("cong", '≅', Rel), ("sim", '∼', Rel),
    ("simeq", '≃', Rel), ("asymp", '≍', Rel), ("propto", '∝', Rel), ("varpropto", '∝', Rel),
    ("ll", '≪', Rel), ("gg", '≫', Rel), ("lll", '⋘', Rel), ("ggg", '⋙', Rel),
    ("subset", '⊂', Rel), ("supset", '⊃', Rel), ("subseteq", '⊆', Rel), ("supseteq", '⊇', Rel),
    ("subsetneq", '⊊', Rel), ("supsetneq", '⊋', Rel), ("nsubseteq", '⊈', Rel), ("nsupseteq", '⊉', Rel),
    ("Subset", '⋐', Rel), ("Supset", '⋑', Rel), ("subseteqq", '⊆', Rel), ("supseteqq", '⊇', Rel),
    ("sqsubset", '⊏', Rel), ("sqsupset", '⊐', Rel), ("sqsubseteq", '⊑', Rel), ("sqsupseteq", '⊒', Rel),
    ("in", '∈', Rel), ("ni", '∋', Rel), ("owns", '∋', Rel), ("notin", '∉', Rel),
    ("vdash", '⊢', Rel), ("dashv", '⊣', Rel), ("models", '⊨', Rel), ("vDash", '⊨', Rel),
    ("Vdash", '⊩', Rel), ("Vvdash", '⊪', Rel), ("nvdash", '⊬', Rel), ("nvDash", '⊭', Rel),
    ("perp", '⟂', Rel), ("mid", '∣', Rel), ("parallel", '∥', Rel), ("nmid", '∤', Rel),
    ("nparallel", '∦', Rel), ("shortmid", '∣', Rel), ("shortparallel", '∥', Rel),
    ("bowtie", '⋈', Rel), ("Join", '⋈', Rel), ("smile", '⌣', Rel), ("frown", '⌢', Rel),
    ("smallsmile", '⌣', Rel), ("smallfrown", '⌢', Rel), ("doteq", '≐', Rel), ("doteqdot", '≑', Rel),
    ("Doteq", '≑', Rel), ("prec", '≺', Rel), ("succ", '≻', Rel), ("preceq", '⪯', Rel),
    ("succeq", '⪰', Rel), ("nprec", '⊀', Rel), ("nsucc", '⊁', Rel), ("precsim", '≾', Rel),
    ("succsim", '≿', Rel), ("preccurlyeq", '≼', Rel), ("succcurlyeq", '≽', Rel),
    ("curlyeqprec", '⋞', Rel), ("curlyeqsucc", '⋟', Rel), ("nless", '≮', Rel), ("ngtr", '≯', Rel),
    ("nleq", '≰', Rel), ("ngeq", '≱', Rel), ("lneq", '⪇', Rel), ("gneq", '⪈', Rel),
    ("lneqq", '≨', Rel), ("gneqq", '≩', Rel), ("leqq", '≦', Rel), ("geqq", '≧', Rel),
    ("leqslant", '⩽', Rel), ("geqslant", '⩾', Rel), ("nleqslant", '⪇', Rel), ("ngeqslant", '⪈', Rel),
    ("eqslantless", '⪕', Rel), ("eqslantgtr", '⪖', Rel), ("lesssim", '≲', Rel), ("gtrsim", '≳', Rel),
    ("lessapprox", '⪅', Rel), ("gtrapprox", '⪆', Rel), ("lessgtr", '≶', Rel), ("gtrless", '≷', Rel),
    ("lesseqgtr", '⋚', Rel), ("gtreqless", '⋛', Rel), ("lessdot", '⋖', Rel), ("gtrdot", '⋗', Rel),
    ("approxeq", '≊', Rel), ("thicksim", '∼', Rel), ("thickapprox", '≈', Rel), ("backsim", '∽', Rel),
    ("backsimeq", '⋍', Rel), ("eqsim", '≂', Rel), ("triangleq", '≜', Rel), ("coloneqq", '≔', Rel),
    ("coloneq", '≔', Rel), ("eqqcolon", '≕', Rel), ("eqcolon", '≕', Rel), ("risingdotseq", '≓', Rel),
    ("fallingdotseq", '≒', Rel), ("circeq", '≗', Rel), ("bumpeq", '≏', Rel), ("Bumpeq", '≎', Rel),
    ("therefore", '∴', Rel), ("because", '∵', Rel), ("ncong", '≇', Rel), ("nsim", '≁', Rel),
    ("between", '≬', Rel), ("pitchfork", '⋔', Rel), ("trianglelefteq", '⊴', Rel),
    ("trianglerighteq", '⊵', Rel), ("vartriangleleft", '⊲', Rel), ("vartriangleright", '⊳', Rel),
    ("ntriangleleft", '⋪', Rel), ("ntriangleright", '⋫', Rel), ("ntrianglelefteq", '⋬', Rel),
    ("ntrianglerighteq", '⋭', Rel), ("blacktriangleleft", '◀', Rel), ("blacktriangleright", '▶', Rel),
    ("varsubsetneq", '⊊', Rel), ("varsupsetneq", '⊋', Rel), ("multimap", '⊸', Rel),
    ("eqcirc", '≖', Rel), ("questeq", '≟', Rel), ("measeq", '≞', Rel), ("wedgeq", '≙', Rel),
    ("Colon", '∷', Rel), ("dblcolon", '∷', Rel), ("sqsubsetneq", '⋤', Rel), ("nsubset", '⊄', Rel),
    ("nsupset", '⊅', Rel), ("approxident", '≋', Rel), ("nequiv", '≢', Rel), ("napprox", '≉', Rel),
    ("nsimeq", '≄', Rel), ("nni", '∌', Rel), ("notni", '∌', Rel),
    ("vcentcolon", '∶', Rel), ("ratio", '∶', Rel), ("backepsilon", '∍', Rel),
    ("Eqqcolon", '≕', Rel), ("coloneqq", '≔', Rel),
    // Setas (relações).
    ("to", '→', Rel), ("rightarrow", '→', Rel), ("leftarrow", '←', Rel), ("gets", '←', Rel),
    ("leftrightarrow", '↔', Rel), ("Rightarrow", '⇒', Rel), ("Leftarrow", '⇐', Rel),
    ("Leftrightarrow", '⇔', Rel), ("longrightarrow", '⟶', Rel), ("longleftarrow", '⟵', Rel),
    ("longleftrightarrow", '⟷', Rel), ("Longrightarrow", '⟹', Rel), ("Longleftarrow", '⟸', Rel),
    ("Longleftrightarrow", '⟺', Rel), ("mapsto", '↦', Rel), ("longmapsto", '⟼', Rel),
    ("mapsfrom", '↤', Rel), ("hookrightarrow", '↪', Rel), ("hookleftarrow", '↩', Rel),
    ("uparrow", '↑', Rel), ("downarrow", '↓', Rel), ("updownarrow", '↕', Rel), ("Uparrow", '⇑', Rel),
    ("Downarrow", '⇓', Rel), ("Updownarrow", '⇕', Rel), ("nearrow", '↗', Rel), ("searrow", '↘', Rel),
    ("swarrow", '↙', Rel), ("nwarrow", '↖', Rel), ("rightharpoonup", '⇀', Rel),
    ("rightharpoondown", '⇁', Rel), ("leftharpoonup", '↼', Rel), ("leftharpoondown", '↽', Rel),
    ("rightleftharpoons", '⇌', Rel), ("leftrightharpoons", '⇋', Rel), ("leadsto", '⇝', Rel),
    ("rightsquigarrow", '⇝', Rel), ("leftrightsquigarrow", '↭', Rel), ("twoheadrightarrow", '↠', Rel),
    ("twoheadleftarrow", '↞', Rel), ("rightarrowtail", '↣', Rel), ("leftarrowtail", '↢', Rel),
    ("looparrowright", '↬', Rel), ("looparrowleft", '↫', Rel), ("curvearrowright", '↷', Rel),
    ("curvearrowleft", '↶', Rel), ("circlearrowright", '↻', Rel), ("circlearrowleft", '↺', Rel),
    ("Lsh", '↰', Rel), ("Rsh", '↱', Rel), ("upuparrows", '⇈', Rel), ("downdownarrows", '⇊', Rel),
    ("leftleftarrows", '⇇', Rel), ("rightrightarrows", '⇉', Rel), ("leftrightarrows", '⇆', Rel),
    ("rightleftarrows", '⇄', Rel), ("Lleftarrow", '⇚', Rel), ("Rrightarrow", '⇛', Rel),
    ("nrightarrow", '↛', Rel), ("nleftarrow", '↚', Rel), ("nRightarrow", '⇏', Rel),
    ("nLeftarrow", '⇍', Rel), ("nleftrightarrow", '↮', Rel), ("nLeftrightarrow", '⇎', Rel),
    ("dashrightarrow", '→', Rel), ("dashleftarrow", '←', Rel), ("restriction", '↾', Rel),
    ("upharpoonleft", '↿', Rel), ("upharpoonright", '↾', Rel), ("downharpoonleft", '⇃', Rel),
    ("downharpoonright", '⇂', Rel), ("impliedby", '⟸', Rel), ("implies", '⟹', Rel), ("iff", '⟺', Rel),
    ("longleftrightarrows", '⇆', Rel), ("leftrightarrowtriangle", '↔', Rel),
    // Operadores grandes.
    ("sum", '∑', Op), ("prod", '∏', Op), ("coprod", '∐', Op), ("int", '∫', Op), ("intop", '∫', Op),
    ("iint", '∬', Op), ("iiint", '∭', Op), ("iiiint", '⨌', Op), ("oint", '∮', Op), ("oiint", '∯', Op),
    ("oiiint", '∰', Op), ("smallint", '∫', Op), ("bigcup", '⋃', Op), ("bigcap", '⋂', Op),
    ("bigvee", '⋁', Op), ("bigwedge", '⋀', Op), ("bigoplus", '⨁', Op), ("bigotimes", '⨂', Op),
    ("bigodot", '⨀', Op), ("biguplus", '⨄', Op), ("bigsqcup", '⨆', Op), ("bigsqcap", '⨅', Op),
    ("varointclockwise", '∲', Op), ("ointctrclockwise", '∳', Op), ("fint", '⨏', Op),
    ("varoint", '∲', Op), ("sqint", '⨖', Op), ("landupint", '∱', Op),
    // Delimitadores.
    ("{", '{', Abre), ("}", '}', Fecha), ("lbrace", '{', Abre), ("rbrace", '}', Fecha),
    ("lbrack", '[', Abre), ("rbrack", ']', Fecha), ("langle", '⟨', Abre), ("rangle", '⟩', Fecha),
    ("lvert", '|', Abre), ("rvert", '|', Fecha), ("lVert", '‖', Abre), ("rVert", '‖', Fecha),
    ("lfloor", '⌊', Abre), ("rfloor", '⌋', Fecha), ("lceil", '⌈', Abre), ("rceil", '⌉', Fecha),
    ("ulcorner", '⌜', Abre), ("urcorner", '⌝', Fecha), ("llcorner", '⌞', Abre), ("lrcorner", '⌟', Fecha),
    ("lgroup", '⟮', Abre), ("rgroup", '⟯', Fecha), ("lmoustache", '⎰', Abre), ("rmoustache", '⎱', Fecha),
    ("llbracket", '⟦', Abre), ("rrbracket", '⟧', Fecha), ("lBrack", '⟦', Abre), ("rBrack", '⟧', Fecha),
    ("lparen", '(', Abre), ("rparen", ')', Fecha),
];

/// O caractere e a classe de um nome (sem a barra).
pub fn simbolo(nome: &str) -> Option<(char, Classe)> {
    static MAPA: OnceLock<HashMap<&'static str, (char, Classe)>> = OnceLock::new();
    MAPA.get_or_init(|| TABELA.iter().map(|&(n, c, k)| (n, (c, k))).collect()).get(nome).copied()
}

/// O nome LaTeX de um caractere (o primeiro da tabela), para traduzir de volta (OMML → LaTeX).
pub fn nome_do_caractere(c: char) -> Option<&'static str> {
    static MAPA: OnceLock<HashMap<char, &'static str>> = OnceLock::new();
    MAPA.get_or_init(|| {
        let mut m = HashMap::new();
        for &(n, c, _) in TABELA {
            if !c.is_ascii() && n.chars().all(|x| x.is_ascii_alphabetic()) {
                // O primeiro da tabela, sem as variantes `\small…` (`\sum`, não `\smallsum`).
                if !n.starts_with("small") {
                    m.entry(c).or_insert(n);
                }
            }
        }
        m
    })
    .get(&c)
    .copied()
}

/// O nome do acento pelo caractere combinante (o primeiro que não cresce).
pub fn nome_do_acento(c: char) -> Option<&'static str> {
    const NOMES: &[&str] = &["hat", "check", "tilde", "acute", "grave", "dot", "ddot", "dddot", "breve", "bar", "vec", "mathring", "overrightarrow", "overleftarrow", "overleftrightarrow"];
    NOMES.iter().find(|n| acento(n).is_some_and(|(x, _)| x == c)).copied()
}

/// Os operadores grandes que, por padrão, não levam limites em cima/embaixo (as integrais).
pub fn integral(c: char) -> bool {
    matches!(c, '∫' | '∬' | '∭' | '⨌' | '∮' | '∯' | '∰' | '∲' | '∳' | '⨏' | '⨖' | '∱')
}

/// Nomes de função em pé: (nome, limites no display).
#[rustfmt::skip]
static FUNCOES: &[(&str, bool)] = &[
    ("arccos", false), ("arcsin", false), ("arctan", false), ("arg", false), ("cos", false),
    ("cosh", false), ("cot", false), ("coth", false), ("csc", false), ("deg", false), ("dim", false),
    ("exp", false), ("hom", false), ("ker", false), ("lg", false), ("ln", false), ("log", false),
    ("sec", false), ("sin", false), ("sinh", false), ("tan", false), ("tanh", false), ("sgn", false),
    ("arccot", false), ("arcsec", false), ("arccsc", false), ("cosec", false), ("cotg", false),
    ("tg", false), ("sen", false), ("senh", false), ("arcsen", false), ("tr", false), ("Tr", false),
    ("rank", false), ("span", false), ("erf", false),
    ("det", true), ("gcd", true), ("inf", true), ("lim", true), ("liminf", true), ("limsup", true),
    ("max", true), ("min", true), ("Pr", true), ("sup", true), ("argmax", true), ("argmin", true),
    ("injlim", true), ("projlim", true), ("mdc", true), ("mmc", true),
];

/// O texto e se leva limites no display, para `\sin`, `\lim`…
pub fn funcao(nome: &str) -> Option<(String, bool)> {
    let (n, lim) = FUNCOES.iter().find(|(n, _)| *n == nome)?;
    let texto = match *n {
        "liminf" => "lim inf",
        "limsup" => "lim sup",
        "argmax" => "arg max",
        "argmin" => "arg min",
        "injlim" => "inj lim",
        "projlim" => "proj lim",
        outro => outro,
    };
    Some((texto.to_string(), *lim))
}

/// Acentos: nome → (caractere combinante, cresce com a base).
pub fn acento(nome: &str) -> Option<(char, bool)> {
    Some(match nome {
        "hat" => ('\u{302}', false),
        "widehat" => ('\u{302}', true),
        "check" => ('\u{30C}', false),
        "widecheck" => ('\u{30C}', true),
        "tilde" => ('\u{303}', false),
        "widetilde" => ('\u{303}', true),
        "acute" => ('\u{301}', false),
        "grave" => ('\u{300}', false),
        "dot" => ('\u{307}', false),
        "ddot" => ('\u{308}', false),
        "dddot" => ('\u{20DB}', false),
        "ddddot" => ('\u{20DC}', false),
        "breve" => ('\u{306}', false),
        "bar" => ('\u{304}', false),
        "vec" => ('\u{20D7}', false),
        "mathring" => ('\u{30A}', false),
        "overrightarrow" => ('→', true),
        "overleftarrow" => ('←', true),
        "overleftrightarrow" => ('↔', true),
        "overrightharpoon" => ('⇀', true),
        "overleftharpoon" => ('↼', true),
        _ => return None,
    })
}

/// O caractere de um delimitador escrito depois de `\left`, `\big`… (`.` = nenhum).
pub fn delimitador(token: &str) -> Option<Option<char>> {
    Some(Some(match token {
        "." => return Some(None),
        "(" | ")" | "[" | "]" | "/" | "<" | ">" => match token {
            "<" => '⟨',
            ">" => '⟩',
            t => t.chars().next()?,
        },
        "|" => '|',
        "\\|" | "\\Vert" | "\\lVert" | "\\rVert" => '‖',
        "\\{" | "\\lbrace" => '{',
        "\\}" | "\\rbrace" => '}',
        "\\backslash" => '∖',
        "\\uparrow" => '↑',
        "\\downarrow" => '↓',
        "\\updownarrow" => '↕',
        "\\Uparrow" => '⇑',
        "\\Downarrow" => '⇓',
        "\\Updownarrow" => '⇕',
        t => {
            let (c, k) = simbolo(t.strip_prefix('\\')?)?;
            if !matches!(k, Abre | Fecha) && !matches!(c, '|' | '‖' | '/') {
                return None;
            }
            c
        }
    }))
}

#[cfg(test)]
mod cobertura {
    /// Os poucos símbolos que a Latin Modern Math não tem saem como caixinha vazia (e a paridade
    /// os conta como faltando); qualquer outro que suma é regressão.
    #[test]
    fn todos_os_simbolos_tem_glifo_salvo_os_conhecidos() {
        const SEM_GLIFO: &[&str] = &["digamma", "Finv", "Game", "circledS", "pitchfork", "fint", "sqint", "lmoustache", "rmoustache"];
        let faltam: Vec<&str> = super::TABELA.iter().filter(|(_, c, _)| crate::fonte::fonte().glifo(*c).is_none()).map(|(n, _, _)| *n).collect();
        assert!(faltam.iter().all(|n| SEM_GLIFO.contains(n)), "{faltam:?}");
    }
}
