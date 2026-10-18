//! O analisador: LaTeX matemático → lista de átomos. Nunca falha: o que não conhece vira
//! texto vermelho com o próprio nome (como o KaTeX), para o usuário ver o que digitou.
//! Grupos e ambientes sem fechamento terminam no fim do texto.

use crate::arvore::{Alfabeto, Alinh, Classe, Matriz, Nivel, No, Nucleo, Numero, TipoMatriz};
use crate::simbolos;

mod pacotes;

/// A cor do comando desconhecido.
pub const COR_ERRO: u32 = 0xcc00_00ff;

/// Profundidade máxima de grupos: texto patológico não estoura a pilha.
const PROFUNDIDADE: usize = 64;

pub fn analisar(latex: &str) -> Vec<No> {
    let mut a = Analisador { s: latex.chars().collect(), i: 0, alf: Alfabeto::Normal, fundo: 0 };
    let mut saida = Vec::new();
    loop {
        saida.extend(a.lista(Fim::Grupo));
        if a.i >= a.s.len() {
            break;
        }
        a.i += 1; // um `}` sobrando: ignora
    }
    saida
}

#[derive(Clone, Copy, PartialEq)]
enum Fim {
    /// `}` ou o fim do texto.
    Grupo,
    /// `\right` (ou `\middle`).
    Direita,
    /// `]` do argumento opcional.
    Colchete,
}

/// O que um comando de fração faz.
struct Frac {
    barra: bool,
    nivel: Option<Nivel>,
    esq: Option<char>,
    dir: Option<char>,
}

struct Analisador {
    s: Vec<char>,
    i: usize,
    alf: Alfabeto,
    fundo: usize,
}

fn car(c: char, a: Alfabeto) -> No {
    No::ord(Nucleo::Car(c, a))
}

fn classe_do_caractere(c: char) -> (char, Classe) {
    match c {
        '+' => ('+', Classe::Bin),
        '-' | '−' => ('−', Classe::Bin),
        '*' => ('∗', Classe::Bin),
        '=' | '<' | '>' | ':' => (c, Classe::Rel),
        ',' | ';' => (c, Classe::Pont),
        '(' | '[' => (c, Classe::Abre),
        ')' | ']' | '!' | '?' => (c, Classe::Fecha),
        _ => (c, classe_unicode(c)),
    }
}

/// A classe de um caractere Unicode digitado direto (≤, →, ∑…): a da tabela de nomes.
fn classe_unicode(c: char) -> Classe {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    static M: OnceLock<HashMap<char, Classe>> = OnceLock::new();
    let m = M.get_or_init(|| {
        let mut m = HashMap::new();
        for nome in NOMES_PARA_CLASSE {
            if let Some((c, k)) = simbolos::simbolo(nome) {
                m.entry(c).or_insert(k);
            }
        }
        m
    });
    m.get(&c).copied().unwrap_or(Classe::Ord)
}

/// Os nomes cuja classe vale para o caractere digitado direto.
const NOMES_PARA_CLASSE: &[&str] = &[
    "pm", "mp", "times", "div", "cdot", "ast", "star", "circ", "bullet", "oplus", "ominus", "otimes", "odot", "cap", "cup", "vee", "wedge",
    "setminus", "leq", "geq", "neq", "equiv", "approx", "cong", "sim", "simeq", "propto", "ll", "gg", "subset", "supset", "subseteq", "supseteq",
    "in", "ni", "notin", "perp", "mid", "parallel", "to", "leftarrow", "leftrightarrow", "Rightarrow", "Leftarrow", "Leftrightarrow",
    "longrightarrow", "longleftarrow", "Longrightarrow", "mapsto", "uparrow", "downarrow", "sum", "prod", "coprod", "int", "iint", "iiint",
    "oint", "bigcup", "bigcap", "bigvee", "bigwedge", "bigoplus", "bigotimes", "langle", "rangle", "lfloor", "rfloor", "lceil", "rceil",
    "ldots", "cdots", "vdash", "models", "prec", "succ", "preceq", "succeq", "therefore", "because", "doteq", "asymp",
];

/// `\not` + relação: o caractere já negado, quando o Unicode tem.
fn negar(c: char) -> Option<char> {
    Some(match c {
        '=' => '≠',
        '<' => '≮',
        '>' => '≯',
        '≤' => '≰',
        '≥' => '≱',
        '∈' => '∉',
        '∋' => '∌',
        '⊂' => '⊄',
        '⊃' => '⊅',
        '⊆' => '⊈',
        '⊇' => '⊉',
        '≡' => '≢',
        '∼' => '≁',
        '≃' => '≄',
        '≅' => '≇',
        '≈' => '≉',
        '∣' => '∤',
        '∥' => '∦',
        '≺' => '⊀',
        '≻' => '⊁',
        '⊢' => '⊬',
        '⊨' => '⊭',
        '→' => '↛',
        '←' => '↚',
        '↔' => '↮',
        '⇒' => '⇏',
        '⇐' => '⇍',
        '⇔' => '⇎',
        '∃' => '∄',
        _ => return None,
    })
}

/// Cor com nome do `xcolor` (e `#RRGGBB`, `HTML`) → `0xRRGGBBAA`.
pub fn cor_com_nome(nome: &str) -> u32 {
    let n = nome.trim();
    if let Some(h) = n.strip_prefix('#').or_else(|| n.strip_prefix("HTML:")) {
        if let Ok(v) = u32::from_str_radix(h, 16) {
            return (v << 8) | 0xff;
        }
    }
    let rgb: u32 = match n {
        "red" => 0xff0000,
        "green" => 0x00a000,
        "blue" => 0x0000ff,
        "cyan" => 0x00b0b0,
        "magenta" => 0xff00ff,
        "yellow" => 0xc8b400,
        "orange" => 0xff8000,
        "purple" => 0xbf0040,
        "violet" => 0x800080,
        "brown" => 0xbf8040,
        "teal" => 0x008080,
        "olive" => 0x808000,
        "lime" => 0x80c000,
        "pink" => 0xff80c0,
        "gray" | "grey" => 0x808080,
        "darkgray" => 0x404040,
        "lightgray" => 0xbfbfbf,
        "white" => 0xffffff,
        _ => 0x000000,
    };
    (rgb << 8) | 0xff
}

/// Uma medida do TeX em em (para corpo de 10 pt: 1 em = 10 pt; 1 em = 18 mu).
pub fn medida_em(texto: &str) -> Option<f32> {
    let t = texto.trim();
    let fim = t.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == ',')).unwrap_or(t.len());
    let n: f32 = t[..fim].replace(',', ".").parse().ok()?;
    let unidade = t[fim..].trim();
    let fator = match unidade.get(..2).unwrap_or(unidade) {
        "em" => 1.0,
        "ex" => 0.43,
        "mu" => 1.0 / 18.0,
        "pt" => 0.1,
        "bp" => 0.1 * 72.27 / 72.0,
        "pc" => 1.2,
        "mm" => 0.1 * 72.27 / 25.4,
        "cm" => 0.1 * 72.27 / 2.54,
        "in" => 7.227,
        _ => return None,
    };
    Some(n * fator)
}

impl Analisador {
    fn espiar(&self) -> Option<char> {
        self.s.get(self.i).copied()
    }

    fn pular_brancos(&mut self) {
        while let Some(c) = self.espiar() {
            if c.is_whitespace() {
                self.i += 1;
            } else if c == '%' {
                while self.espiar().is_some_and(|c| c != '\n') {
                    self.i += 1;
                }
            } else {
                break;
            }
        }
    }

    /// Lê um nome de comando depois de `\` (já consumida).
    fn nome(&mut self) -> String {
        let inicio = self.i;
        while self.espiar().is_some_and(|c| c.is_ascii_alphabetic()) {
            self.i += 1;
        }
        if self.i == inicio {
            if let Some(c) = self.espiar() {
                self.i += 1;
                return c.to_string();
            }
            return String::new();
        }
        let mut n: String = self.s[inicio..self.i].iter().collect();
        if self.espiar() == Some('*') && matches!(n.as_str(), "operatorname" | "tag") {
            self.i += 1;
            n.push('*');
        }
        n
    }

    /// O próximo token como texto (`\nome`, ou um caractere), sem consumir brancos antes.
    fn token(&mut self) -> Option<String> {
        self.pular_brancos();
        let c = self.espiar()?;
        self.i += 1;
        if c == '\\' {
            Some(format!("\\{}", self.nome()))
        } else {
            Some(c.to_string())
        }
    }

    /// Argumento cru: `{…}` com chaves balanceadas, ou um token.
    fn cru(&mut self) -> String {
        self.pular_brancos();
        if self.espiar() != Some('{') {
            return self.token().unwrap_or_default();
        }
        self.i += 1;
        let inicio = self.i;
        let mut nivel = 1;
        while let Some(c) = self.espiar() {
            match c {
                '\\' => self.i += 1,
                '{' => nivel += 1,
                '}' => {
                    nivel -= 1;
                    if nivel == 0 {
                        break;
                    }
                }
                _ => {}
            }
            self.i += 1;
        }
        let fim = self.i.min(self.s.len());
        self.i = (self.i + 1).min(self.s.len());
        self.s[inicio..fim].iter().collect()
    }

    /// `[…]` opcional, cru.
    fn opcional(&mut self) -> Option<String> {
        let salvo = self.i;
        self.pular_brancos();
        if self.espiar() != Some('[') {
            self.i = salvo;
            return None;
        }
        self.i += 1;
        let inicio = self.i;
        let mut nivel = 0;
        while let Some(c) = self.espiar() {
            match c {
                '{' => nivel += 1,
                '}' => nivel -= 1,
                ']' if nivel == 0 => break,
                _ => {}
            }
            self.i += 1;
        }
        let fim = self.i.min(self.s.len());
        self.i = (self.i + 1).min(self.s.len());
        Some(self.s[inicio..fim].iter().collect())
    }

    fn sub(&self, texto: &str, alf: Alfabeto) -> Vec<No> {
        let mut a = Analisador { s: texto.chars().collect(), i: 0, alf, fundo: self.fundo + 1 };
        if a.fundo > PROFUNDIDADE {
            return Vec::new();
        }
        a.lista(Fim::Grupo)
    }

    /// Argumento matemático: `{…}` como lista, ou um único átomo.
    fn argumento(&mut self) -> Vec<No> {
        self.pular_brancos();
        match self.espiar() {
            Some('{') => {
                self.i += 1;
                self.grupo_interno()
            }
            Some(_) => self.atomo().into_iter().collect(),
            None => Vec::new(),
        }
    }

    /// O conteúdo de um grupo já aberto, até a `}` (consumida). O alfabeto declarado lá dentro não vaza.
    fn grupo_interno(&mut self) -> Vec<No> {
        let alf = self.alf;
        self.fundo += 1;
        let l = if self.fundo > PROFUNDIDADE { self.pular_grupo() } else { self.lista(Fim::Grupo) };
        self.fundo -= 1;
        self.alf = alf;
        if self.espiar() == Some('}') {
            self.i += 1;
        }
        l
    }

    fn pular_grupo(&mut self) -> Vec<No> {
        let mut nivel = 1;
        while let Some(c) = self.espiar() {
            if c == '{' {
                nivel += 1;
            } else if c == '}' {
                nivel -= 1;
                if nivel == 0 {
                    break;
                }
            }
            self.i += 1;
        }
        Vec::new()
    }

    fn argumento_com_alfabeto(&mut self, alf: Alfabeto) -> Vec<No> {
        let antes = self.alf;
        self.alf = alf;
        let l = self.argumento();
        self.alf = antes;
        l
    }

    /// Uma lista até o fim pedido. Trata `^`, `_`, `'`, `\limits` e as frações infixas (`\over`).
    fn lista(&mut self, fim: Fim) -> Vec<No> {
        let mut l: Vec<No> = Vec::new();
        let mut infixa: Option<(Vec<No>, Frac)> = None;
        loop {
            self.pular_brancos();
            let Some(c) = self.espiar() else { break };
            match c {
                '}' => break,
                ']' if fim == Fim::Colchete => break,
                '^' | '_' => {
                    self.i += 1;
                    let arg = self.argumento();
                    self.script(&mut l, c == '^', arg);
                    continue;
                }
                '\'' => {
                    // O primo da fonte OpenType já vem no alto e no tamanho de expoente (U+2032): entra
                    // como ordinário depois da base, como faz o unicode-math; `''` vira ″.
                    let mut n = 0;
                    while self.espiar() == Some('\'') {
                        self.i += 1;
                        n += 1;
                    }
                    let c = ['′', '″', '‴', '⁗'][(n - 1).min(3)];
                    l.push(car(c, Alfabeto::Romano));
                    continue;
                }
                '\\' => {
                    let salvo = self.i;
                    self.i += 1;
                    let n = self.nome();
                    match n.as_str() {
                        "right" | "middle" if fim == Fim::Direita => {
                            self.i = salvo;
                            break;
                        }
                        "end" => {
                            self.i = salvo;
                            break;
                        }
                        "limits" | "nolimits" | "displaylimits" => {
                            if let Some(u) = l.last_mut() {
                                u.limites = match n.as_str() {
                                    "limits" => Some(true),
                                    "nolimits" => Some(false),
                                    _ => None,
                                };
                            }
                            continue;
                        }
                        "over" | "choose" | "atop" | "brace" | "brack" | "above" => {
                            if n == "above" {
                                self.cru();
                            }
                            let f = match n.as_str() {
                                "over" | "above" => Frac { barra: true, nivel: None, esq: None, dir: None },
                                "choose" => Frac { barra: false, nivel: None, esq: Some('('), dir: Some(')') },
                                "brace" => Frac { barra: false, nivel: None, esq: Some('{'), dir: Some('}') },
                                "brack" => Frac { barra: false, nivel: None, esq: Some('['), dir: Some(']') },
                                _ => Frac { barra: false, nivel: None, esq: None, dir: None },
                            };
                            infixa = Some((std::mem::take(&mut l), f));
                            continue;
                        }
                        "color" => {
                            let cor = cor_com_nome(&self.cru());
                            let resto = self.lista(fim);
                            l.push(No::ord(Nucleo::Cor(cor, resto)));
                            break;
                        }
                        _ => {
                            self.i = salvo;
                        }
                    }
                }
                _ => {}
            }
            match self.atomo() {
                Some(no) => l.push(no),
                None => {
                    if self.i < self.s.len() && self.espiar() != Some('}') {
                        self.i += 1;
                    }
                }
            }
        }
        match infixa {
            Some((num, f)) => vec![No::ord(Nucleo::Fracao { num, den: l, barra: f.barra, nivel: f.nivel, esq: f.esq, dir: f.dir })],
            None => l,
        }
    }

    /// Prende um expoente (`sup`) ou índice ao último átomo (ou a um vazio).
    fn script(&mut self, l: &mut Vec<No>, sup: bool, arg: Vec<No>) {
        let precisa_novo = match l.last() {
            None => true,
            Some(u) => u.eh_marca() || (if sup { u.sup.is_some() && !primos(u.sup.as_deref()) } else { u.sub.is_some() }),
        };
        if precisa_novo {
            l.push(No::ord(Nucleo::Vazio));
        }
        let u = l.last_mut().expect("acabou de entrar");
        let alvo = if sup { &mut u.sup } else { &mut u.sub };
        match alvo {
            Some(v) => v.extend(arg),
            None => *alvo = Some(arg),
        }
    }

    /// Um átomo (sem os scripts). `None` = nada a acrescentar (fim, `}`, marca consumida).
    fn atomo(&mut self) -> Option<No> {
        self.pular_brancos();
        let c = self.espiar()?;
        self.i += 1;
        match c {
            '{' => Some(No::ord(Nucleo::Grupo(self.grupo_interno()))),
            '}' => {
                self.i -= 1;
                None
            }
            '\\' => {
                let n = self.nome();
                self.comando(&n)
            }
            '&' => Some(No::ord(Nucleo::SepColuna)),
            '~' => Some(No::ord(Nucleo::EspacoEm(0.333))),
            '#' => None,
            c if c.is_ascii_alphanumeric() => Some(car(c, self.alf)),
            '.' | '/' | '|' | '@' | '"' => Some(car(c, Alfabeto::Romano)),
            c => {
                let (c, classe) = classe_do_caractere(c);
                if c.is_alphabetic() {
                    return Some(No::novo(classe, Nucleo::Car(c, self.alf)));
                }
                let mut no = No::novo(classe, Nucleo::Car(c, Alfabeto::Romano));
                if classe == Classe::Op && simbolos::integral(c) {
                    no.limites = Some(false);
                }
                Some(no)
            }
        }
    }

    fn comando(&mut self, n: &str) -> Option<No> {
        if let Some(no) = self.comando_estrutura(n) {
            return Some(no);
        }
        if let Some(no) = self.comando_fonte(n) {
            return Some(no);
        }
        if let Some(no) = self.comando_espaco(n) {
            return Some(no);
        }
        if let Some(no) = self.comando_marca(n) {
            return Some(no);
        }
        if let Some(no) = self.comando_pacote(n) {
            return Some(no);
        }
        if let Some((c, classe)) = simbolos::simbolo(n) {
            let alf = if c.is_alphabetic() { self.alf } else { Alfabeto::Romano };
            let mut no = No::novo(classe, Nucleo::Car(c, alf));
            if classe == Classe::Op && simbolos::integral(c) {
                no.limites = Some(false);
            }
            if matches!(n, "implies" | "impliedby" | "iff") {
                return Some(No::ord(Nucleo::Grupo(vec![No::ord(Nucleo::Espaco(5.0)), no, No::ord(Nucleo::Espaco(5.0))])));
            }
            return Some(no);
        }
        if n == "Coloneqq" {
            // ⩴ não existe na Latin Modern Math: os três sinais juntos, como relação.
            return Some(No::novo(Classe::Rel, Nucleo::Texto("::=".into(), Alfabeto::Romano)));
        }
        if let Some((texto, lim)) = simbolos::funcao(n) {
            let mut no = No::novo(Classe::Op, Nucleo::Texto(texto, Alfabeto::Romano));
            no.limites = if lim { None } else { Some(false) };
            return Some(no);
        }
        if let Some((acento, largo)) = simbolos::acento(n) {
            let corpo = self.argumento();
            return Some(No::ord(Nucleo::Acento { c: acento, corpo, largo, baixo: false }));
        }
        Some(No::novo(Classe::Ord, Nucleo::Cor(COR_ERRO, vec![No::ord(Nucleo::Texto(format!("\\{n}"), Alfabeto::Romano))])))
    }

    fn fracao(&mut self, f: Frac) -> No {
        let num = self.argumento();
        let den = self.argumento();
        No::ord(Nucleo::Fracao { num, den, barra: f.barra, nivel: f.nivel, esq: f.esq, dir: f.dir })
    }

    fn comando_estrutura(&mut self, n: &str) -> Option<No> {
        let frac = |barra, nivel, esq, dir| Frac { barra, nivel, esq, dir };
        Some(match n {
            "frac" => self.fracao(frac(true, None, None, None)),
            "dfrac" => self.fracao(frac(true, Some(Nivel::D), None, None)),
            "tfrac" => self.fracao(frac(true, Some(Nivel::T), None, None)),
            "cfrac" => {
                self.opcional();
                self.fracao(frac(true, Some(Nivel::D), None, None))
            }
            "binom" => self.fracao(frac(false, None, Some('('), Some(')'))),
            "dbinom" => self.fracao(frac(false, Some(Nivel::D), Some('('), Some(')'))),
            "tbinom" => self.fracao(frac(false, Some(Nivel::T), Some('('), Some(')'))),
            "sqrt" => {
                let indice = self.opcional().map(|t| self.sub(&t, self.alf));
                let corpo = self.argumento();
                No::ord(Nucleo::Raiz { indice, corpo })
            }
            "left" => self.delimitado(),
            "big" | "Big" | "bigg" | "Bigg" | "bigl" | "Bigl" | "biggl" | "Biggl" | "bigr" | "Bigr" | "biggr" | "Biggr" | "bigm" | "Bigm" | "biggm" | "Biggm" => self.grande(n),
            "overline" => No::ord(Nucleo::Barra { corpo: self.argumento(), cima: true }),
            "underline" => No::ord(Nucleo::Barra { corpo: self.argumento(), cima: false }),
            "overbrace" | "underbrace" | "overparen" | "underparen" | "overbracket" | "underbracket" => {
                let c = match n {
                    "overbrace" => '⏞',
                    "underbrace" => '⏟',
                    "overparen" => '⏜',
                    "underparen" => '⏝',
                    "overbracket" => '⎴',
                    _ => '⎵',
                };
                let mut no = No::novo(Classe::Op, Nucleo::Chave { c, corpo: self.argumento(), cima: n.starts_with("over") });
                no.limites = Some(true);
                no
            }
            "underrightarrow" | "underleftarrow" | "underleftrightarrow" => {
                let c = match n {
                    "underrightarrow" => '→',
                    "underleftarrow" => '←',
                    _ => '↔',
                };
                No::ord(Nucleo::Acento { c, corpo: self.argumento(), largo: true, baixo: true })
            }
            "xrightarrow" | "xleftarrow" | "xRightarrow" | "xLeftarrow" | "xleftrightarrow" | "xLeftrightarrow" | "xmapsto" | "xhookrightarrow" | "xrightleftharpoons" => self.seta_x(n),
            "overset" | "underset" | "stackrel" => self.empilhar(n),
            "begin" => self.ambiente(),
            "substack" => {
                let corpo = self.argumento();
                No::ord(Nucleo::Matriz(Box::new(montar_matriz(corpo, TipoMatriz::Pilha, &[]))))
            }
            "boxed" => No::ord(Nucleo::Caixa(self.argumento())),
            "fbox" => No::ord(Nucleo::Caixa(self.texto(Alfabeto::Romano))),
            "phantom" => No::ord(Nucleo::Fantasma { corpo: self.argumento(), largura: true, altura: true }),
            "hphantom" => No::ord(Nucleo::Fantasma { corpo: self.argumento(), largura: true, altura: false }),
            "vphantom" => No::ord(Nucleo::Fantasma { corpo: self.argumento(), largura: false, altura: true }),
            "mathstrut" | "strut" => No::ord(Nucleo::Fantasma { corpo: vec![car('(', Alfabeto::Romano)], largura: false, altura: true }),
            "smash" => {
                self.opcional();
                No::ord(Nucleo::Achatado(self.argumento()))
            }
            "not" => {
                let mut prox = self.atomo()?;
                if let Nucleo::Car(c, a) = prox.nucleo {
                    if let Some(neg) = negar(c) {
                        prox.nucleo = Nucleo::Car(neg, a);
                        return Some(prox);
                    }
                }
                No::novo(prox.classe, Nucleo::Negado(Box::new(prox)))
            }
            "textcolor" => {
                let cor = cor_com_nome(&self.cru());
                No::ord(Nucleo::Cor(cor, self.argumento()))
            }
            "colorbox" => {
                self.cru();
                No::ord(Nucleo::Grupo(self.texto(Alfabeto::Romano)))
            }
            "mathop" | "mathbin" | "mathrel" | "mathopen" | "mathclose" | "mathpunct" | "mathinner" | "mathord" | "ensuremath" | "operatorname" | "operatorname*" | "DeclareMathOperator" => return self.classe_forcada(n),
            "pmod" | "pod" | "mod" | "bmod" => return Some(self.modulo(n)),
            "ref" | "eqref" | "autoref" | "cref" => {
                let r = self.cru();
                let t = if n == "eqref" { format!("({r})") } else { r };
                No::ord(Nucleo::Texto(t, Alfabeto::Romano))
            }
            "SI" | "qty" | "si" | "num" | "unit" | "ang" => return Some(self.siunitx(n)),
            _ => return None,
        })
    }

    fn classe_forcada(&mut self, n: &str) -> Option<No> {
        if n.starts_with("operatorname") || n == "DeclareMathOperator" {
            let t = self.cru();
            let texto: String = t.replace("\\!", "").replace("\\,", " ").replace(['{', '}'], "");
            let mut no = No::novo(Classe::Op, Nucleo::Texto(texto, Alfabeto::Romano));
            no.limites = if n.ends_with('*') { None } else { Some(false) };
            return Some(no);
        }
        let corpo = self.argumento();
        let classe = match n {
            "mathop" => Classe::Op,
            "mathbin" => Classe::Bin,
            "mathrel" => Classe::Rel,
            "mathopen" => Classe::Abre,
            "mathclose" => Classe::Fecha,
            "mathpunct" => Classe::Pont,
            "mathinner" => Classe::Interno,
            _ => Classe::Ord,
        };
        // Um caractere só, com classe trocada, continua sendo um caractere (o `\mathop{\sum}` cresce).
        if corpo.len() == 1 && corpo[0].sup.is_none() && corpo[0].sub.is_none() && classe != Classe::Ord {
            if let Nucleo::Car(c, a) = corpo[0].nucleo {
                return Some(No::novo(classe, Nucleo::Car(c, a)));
            }
        }
        Some(No::novo(classe, Nucleo::Grupo(corpo)))
    }

    fn modulo(&mut self, n: &str) -> No {
        let modtxt = || No::novo(Classe::Ord, Nucleo::Texto("mod".into(), Alfabeto::Romano));
        if n == "bmod" {
            return No::novo(Classe::Bin, Nucleo::Texto("mod".into(), Alfabeto::Romano));
        }
        let arg = self.argumento();
        let mut l = vec![No::ord(Nucleo::Espaco(18.0))];
        match n {
            "pmod" => {
                l.push(No::novo(Classe::Abre, Nucleo::Car('(', Alfabeto::Romano)));
                l.push(modtxt());
                l.push(No::ord(Nucleo::Espaco(6.0)));
                l.push(No::ord(Nucleo::Grupo(arg)));
                l.push(No::novo(Classe::Fecha, Nucleo::Car(')', Alfabeto::Romano)));
            }
            "pod" => {
                l.push(No::novo(Classe::Abre, Nucleo::Car('(', Alfabeto::Romano)));
                l.push(No::ord(Nucleo::Grupo(arg)));
                l.push(No::novo(Classe::Fecha, Nucleo::Car(')', Alfabeto::Romano)));
            }
            _ => {
                l.push(modtxt());
                l.push(No::ord(Nucleo::Espaco(6.0)));
                l.push(No::ord(Nucleo::Grupo(arg)));
            }
        }
        No::ord(Nucleo::Grupo(l))
    }

    fn delimitado(&mut self) -> No {
        let esq = self.delim_token();
        let mut partes = Vec::new();
        let mut meios = Vec::new();
        loop {
            partes.push(self.lista(Fim::Direita));
            self.pular_brancos();
            if self.espiar() != Some('\\') {
                break; // acabou sem \right
            }
            self.i += 1;
            let n = self.nome();
            if n == "middle" {
                meios.push(self.delim_token());
                continue;
            }
            if n == "right" {
                let dir = self.delim_token();
                return No::novo(Classe::Interno, Nucleo::Delimitado { esq, dir, partes, meios });
            }
            break;
        }
        No::novo(Classe::Interno, Nucleo::Delimitado { esq, dir: None, partes, meios })
    }

    fn delim_token(&mut self) -> Option<char> {
        let t = self.token()?;
        match simbolos::delimitador(&t) {
            Some(c) => c,
            None => t.chars().next().filter(|c| *c != '\\'),
        }
    }

    fn grande(&mut self, n: &str) -> No {
        let base = n.trim_end_matches(['l', 'r', 'm']);
        let altura = match base {
            "big" => 1.2,
            "Big" => 1.8,
            "bigg" => 2.4,
            _ => 3.0,
        };
        let c = self.delim_token();
        let classe = match n.as_bytes().last() {
            Some(b'l') => Classe::Abre,
            Some(b'r') => Classe::Fecha,
            Some(b'm') => Classe::Rel,
            _ => match c {
                Some('(' | '[' | '{' | '⟨' | '⌊' | '⌈') => Classe::Abre,
                Some(')' | ']' | '}' | '⟩' | '⌋' | '⌉') => Classe::Fecha,
                _ => Classe::Ord,
            },
        };
        No::novo(classe, Nucleo::Grande { c, altura })
    }

    fn seta_x(&mut self, n: &str) -> No {
        let embaixo = self.opcional().map(|t| self.sub(&t, self.alf));
        let em_cima = self.argumento();
        let c = match n {
            "xrightarrow" => '→',
            "xleftarrow" => '←',
            "xRightarrow" => '⇒',
            "xLeftarrow" => '⇐',
            "xleftrightarrow" => '↔',
            "xLeftrightarrow" => '⇔',
            "xmapsto" => '↦',
            "xhookrightarrow" => '↪',
            _ => '⇌',
        };
        let mut no = No::novo(Classe::Rel, Nucleo::Chave { c, corpo: Vec::new(), cima: true });
        no.limites = Some(true);
        no.sup = Some(em_cima);
        no.sub = embaixo;
        no
    }

    fn empilhar(&mut self, n: &str) -> No {
        let marca = self.argumento();
        let base = self.argumento();
        let classe = if n == "stackrel" {
            Classe::Rel
        } else if base.len() == 1 {
            base[0].classe
        } else {
            Classe::Ord
        };
        let mut no = No::novo(classe, Nucleo::Grupo(base));
        no.limites = Some(true);
        if n == "underset" {
            no.sub = Some(marca);
        } else {
            no.sup = Some(marca);
        }
        no
    }

    /// `\text{…}`: texto em pé; `$…$` dentro dele volta para a matemática.
    fn texto(&mut self, alf: Alfabeto) -> Vec<No> {
        let t = self.cru();
        let mut saida = Vec::new();
        for (k, parte) in t.split('$').enumerate() {
            if k % 2 == 1 {
                saida.push(No::ord(Nucleo::Grupo(self.sub(parte, Alfabeto::Normal))));
            } else if !parte.is_empty() {
                saida.push(No::ord(Nucleo::Texto(texto_de_comandos(parte), alf)));
            }
        }
        saida
    }

    fn comando_fonte(&mut self, n: &str) -> Option<No> {
        let alf = match n {
            "mathrm" | "mathup" | "rm" | "mathnormalup" => Alfabeto::Romano,
            "mathbf" | "bf" | "mathbfup" => Alfabeto::Negrito,
            "mathit" | "it" => Alfabeto::Italico,
            "mathsf" | "sf" | "mathsfup" => Alfabeto::SemSerifa,
            "mathtt" | "tt" => Alfabeto::Mono,
            "mathbb" | "Bbb" | "mathds" | "mathbbm" => Alfabeto::Duplo,
            "mathcal" | "mathscr" | "cal" | "mathpzc" => Alfabeto::Caligrafico,
            "mathfrak" | "frak" => Alfabeto::Fraktur,
            "boldsymbol" | "bm" | "pmb" | "mathbfit" => Alfabeto::NegritoItalico,
            "mathnormal" => Alfabeto::Normal,
            "text" | "textrm" | "textnormal" | "mbox" | "hbox" | "textup" | "textmd" | "emph" | "textsl" => {
                let a = if matches!(n, "emph" | "textsl") { Alfabeto::Italico } else { Alfabeto::Romano };
                return Some(No::ord(Nucleo::Grupo(self.texto(a))));
            }
            "textbf" => return Some(No::ord(Nucleo::Grupo(self.texto(Alfabeto::Negrito)))),
            "textit" => return Some(No::ord(Nucleo::Grupo(self.texto(Alfabeto::Italico)))),
            "textsf" => return Some(No::ord(Nucleo::Grupo(self.texto(Alfabeto::SemSerifa)))),
            "texttt" => return Some(No::ord(Nucleo::Grupo(self.texto(Alfabeto::Mono)))),
            "displaystyle" => return Some(No::ord(Nucleo::Estilo(Nivel::D))),
            "textstyle" => return Some(No::ord(Nucleo::Estilo(Nivel::T))),
            "scriptstyle" => return Some(No::ord(Nucleo::Estilo(Nivel::S))),
            "scriptscriptstyle" => return Some(No::ord(Nucleo::Estilo(Nivel::SS))),
            _ => return None,
        };
        // Forma antiga (`{\bf x}`): vale até o fim do grupo.
        if matches!(n, "rm" | "bf" | "it" | "sf" | "tt" | "cal" | "frak") {
            self.alf = alf;
            return Some(No::ord(Nucleo::EspacoEm(0.0)));
        }
        Some(No::ord(Nucleo::Grupo(self.argumento_com_alfabeto(alf))))
    }

    fn comando_espaco(&mut self, n: &str) -> Option<No> {
        let mu = |v: f32| Some(No::ord(Nucleo::Espaco(v)));
        let em = |v: f32| Some(No::ord(Nucleo::EspacoEm(v)));
        match n {
            "," | "thinspace" => mu(3.0),
            ":" | ">" | "medspace" => mu(4.0),
            ";" | "thickspace" => mu(5.0),
            "!" | "negthinspace" => mu(-3.0),
            "negmedspace" => mu(-4.0),
            "negthickspace" => mu(-5.0),
            " " | "space" | "\n" | "\t" => em(0.333),
            "quad" => em(1.0),
            "qquad" => em(2.0),
            "enspace" | "enskip" => em(0.5),
            "hspace" | "hspace*" | "hskip" | "kern" | "mkern" | "mskip" | "mspace" => {
                if self.espiar() == Some('*') {
                    self.i += 1;
                }
                let t = if matches!(n, "hskip" | "kern" | "mkern" | "mskip") { self.medida_solta() } else { self.cru() };
                em(medida_em(&t).unwrap_or(0.0))
            }
            "allowbreak" | "nobreak" | "displaybreak" | "nolinebreak" | "relax" | "protect" | "nonscript" | "nolimits@" => Some(No::ord(Nucleo::EspacoEm(0.0))),
            _ => None,
        }
    }

    /// Medida sem chaves (`\kern3pt`, `\mkern18mu`).
    fn medida_solta(&mut self) -> String {
        self.pular_brancos();
        let inicio = self.i;
        while self.espiar().is_some_and(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == ',') {
            self.i += 1;
        }
        let mut letras = 0;
        while letras < 2 && self.espiar().is_some_and(|c| c.is_ascii_alphabetic()) {
            self.i += 1;
            letras += 1;
        }
        self.s[inicio..self.i].iter().collect()
    }

    fn comando_marca(&mut self, n: &str) -> Option<No> {
        Some(No::ord(match n {
            "\\" | "cr" | "newline" => {
                if self.espiar() == Some('*') {
                    self.i += 1;
                }
                let extra = self.opcional().and_then(|t| medida_em(&t)).unwrap_or(0.0);
                Nucleo::SepLinha(extra)
            }
            "hline" | "midrule" | "toprule" | "bottomrule" => Nucleo::Hline,
            "cline" | "cmidrule" => {
                self.opcional();
                self.cru();
                Nucleo::EspacoEm(0.0)
            }
            "label" => Nucleo::Rotulo(self.cru()),
            "tag" | "tag*" => Nucleo::Tag(self.cru()),
            "nonumber" | "notag" => Nucleo::SemNumero,
            "intertext" | "shortintertext" => Nucleo::Intertexto(self.texto(Alfabeto::Romano)),
            _ => return None,
        }))
    }

    fn siunitx(&mut self, n: &str) -> No {
        let mut l = Vec::new();
        if matches!(n, "SI" | "qty" | "num" | "ang") {
            self.opcional();
            let cru = self.cru().replace(',', "{,}").replace('x', "\\times ");
            // 6.022e23 → 6,022 × 10²³ (o expoente inteiro entre chaves).
            let numero = match cru.split_once(['e', 'E']) {
                Some((m, e)) => format!("{m}\\times 10^{{{e}}}"),
                None => cru,
            };
            l.extend(self.sub(&numero, Alfabeto::Normal));
            if n == "ang" {
                l.push(car('°', Alfabeto::Romano));
                return No::ord(Nucleo::Grupo(l));
            }
        }
        if matches!(n, "SI" | "qty" | "si" | "unit") {
            if n == "si" || n == "unit" {
                self.opcional();
            }
            let u = self.cru();
            if !l.is_empty() {
                l.push(No::ord(Nucleo::Espaco(3.0)));
            }
            l.extend(unidade_si(&u));
        }
        No::ord(Nucleo::Grupo(l))
    }

    fn ambiente(&mut self) -> No {
        let nome = self.cru();
        let (base, estrela) = nome_do_ambiente(&nome);
        let mut colunas = Vec::new();
        let mut tracos = Vec::new();
        // Argumentos e opções antes do corpo.
        let mut alinh_matriz = None;
        match base.as_str() {
            "array" | "subarray" => {
                self.opcional();
                (colunas, tracos) = colunas_do_array(&self.cru());
            }
            "alignat" | "alignedat" | "xalignat" => {
                self.cru();
            }
            "aligned" | "gathered" | "lgathered" | "rgathered" | "multlined" => {
                self.opcional();
                self.opcional();
            }
            "empheq" => {
                // `\begin{empheq}[box=\fbox]{align}`: vale o ambiente de dentro (a moldura fica de fora).
                self.opcional();
                let dentro = self.cru();
                return self.ambiente_empheq(&dentro);
            }
            b if estrela && b.ends_with("matrix") => {
                alinh_matriz = self.opcional().map(|a| match a.trim() {
                    "l" => Alinh::Esq,
                    "r" => Alinh::Dir,
                    _ => Alinh::Centro,
                });
            }
            _ => {}
        }
        let corpo = self.lista(Fim::Grupo);
        self.fechar_ambiente();
        self.montar_ambiente(&base, estrela, corpo, (colunas, tracos), alinh_matriz)
    }

    /// Consome o `\end{…}` que fecha o ambiente (o corpo parou nele).
    fn fechar_ambiente(&mut self) {
        self.pular_brancos();
        if self.espiar() == Some('\\') {
            let salvo = self.i;
            self.i += 1;
            if self.nome() == "end" {
                self.cru();
            } else {
                self.i = salvo;
            }
        }
    }

    fn ambiente_empheq(&mut self, dentro: &str) -> No {
        let (base, estrela) = nome_do_ambiente(dentro);
        if matches!(base.as_str(), "alignat" | "alignedat") {
            self.cru();
        }
        let corpo = self.lista(Fim::Grupo);
        self.fechar_ambiente();
        self.montar_ambiente(&base, estrela, corpo, (Vec::new(), Vec::new()), None)
    }

    fn montar_ambiente(&mut self, base: &str, estrela: bool, corpo: Vec<No>, (mut colunas, tracos): (Vec<Alinh>, Vec<usize>), alinh_matriz: Option<Alinh>) -> No {
        let pequena = base.ends_with("smallmatrix");
        let (tipo, esq, dir) = tipo_do_ambiente(base);
        if matches!(base, "cases" | "dcases" | "rcases" | "drcases") {
            colunas = vec![Alinh::Esq, Alinh::Esq];
        }
        let mut m = montar_matriz(corpo, tipo, &colunas);
        if let Some(a) = alinh_matriz {
            let n = m.linhas.iter().map(Vec::len).max().unwrap_or(0);
            m.colunas = vec![a; n];
        }
        m.tracos = tracos;
        m.esq = esq;
        m.dir = dir;
        if pequena {
            m.sep = 0.556;
        }
        // O `array` tem o `\arraycolsep` também nas pontas; o `alignat` não põe espaço entre os pares.
        m.margem = base == "array";
        if matches!(base, "alignat" | "alignedat" | "xalignat") {
            m.sep = 0.0;
        }
        if matches!(base, "cases" | "rcases" | "dcases" | "drcases") {
            m.sep = 1.0;
        }
        numerar(&mut m, base, estrela);
        let mut no = No::ord(Nucleo::Matriz(Box::new(m)));
        if matches!(base, "dcases" | "drcases") {
            no = No::ord(Nucleo::Grupo(vec![No::ord(Nucleo::Estilo(Nivel::D)), no]));
        }
        if pequena {
            no = No::ord(Nucleo::Grupo(vec![No::ord(Nucleo::Estilo(Nivel::S)), no]));
        }
        no
    }
}

/// O nome-base do ambiente (sem a estrela; `pNiceMatrix` vale `pmatrix`) e se tinha estrela.
fn nome_do_ambiente(nome: &str) -> (String, bool) {
    let estrela = nome.ends_with('*');
    let base = nome.trim().trim_end_matches('*');
    let base = match base {
        "NiceMatrix" | "NiceArray" => "matrix".to_string(),
        b if b.ends_with("NiceMatrix") => format!("{}matrix", b.trim_end_matches("NiceMatrix")),
        "dmath" | "dseries" => "equation".to_string(),
        "dgroup" | "lgathered" | "rgathered" => "gathered".to_string(),
        "multlined" => "multlined".to_string(),
        b => b.to_string(),
    };
    (base, estrela)
}

/// O tipo e os delimitadores de cada ambiente.
fn tipo_do_ambiente(base: &str) -> (TipoMatriz, Option<char>, Option<char>) {
    let raiz = base.trim_end_matches("smallmatrix").trim_end_matches("matrix");
    if base.ends_with("matrix") {
        let (esq, dir) = match raiz {
            "p" => (Some('('), Some(')')),
            "b" => (Some('['), Some(']')),
            "B" => (Some('{'), Some('}')),
            "v" => (Some('|'), Some('|')),
            "V" => (Some('‖'), Some('‖')),
            _ => (None, None),
        };
        return (TipoMatriz::Matriz, esq, dir);
    }
    match base {
        "array" => (TipoMatriz::Matriz, None, None),
        "subarray" => (TipoMatriz::Pilha, None, None),
        "cases" | "dcases" | "numcases" => (TipoMatriz::Matriz, Some('{'), None),
        "rcases" | "drcases" => (TipoMatriz::Matriz, None, Some('}')),
        "aligned" | "split" | "alignedat" | "align" | "alignat" | "flalign" | "xalignat" => (TipoMatriz::Alinhado, None, None),
        "multline" | "multlined" => (TipoMatriz::Multlinha, None, None),
        "eqnarray" => (TipoMatriz::Eqnarray, None, None),
        _ => (TipoMatriz::Reunido, None, None),
    }
}

/// O primo seguido de mais primos não é "expoente duplo".
fn primos(sup: Option<&[No]>) -> bool {
    sup.is_some_and(|s| s.iter().all(|n| matches!(n.nucleo, Nucleo::Car('′', _))))
}

/// Comandos de texto comuns dentro de `\text{}` (acentos, espaços).
fn texto_de_comandos(t: &str) -> String {
    let mut s = t.replace("\\ ", " ").replace('~', "\u{a0}").replace("\\,", "\u{2009}").replace("\\quad", "\u{2003}").replace("\\qquad", "\u{2003}\u{2003}");
    for (de, para) in [("\\'a", "á"), ("\\'e", "é"), ("\\'i", "í"), ("\\'o", "ó"), ("\\'u", "ú"), ("\\~a", "ã"), ("\\~o", "õ"), ("\\^a", "â"), ("\\^e", "ê"), ("\\^o", "ô"), ("\\c{c}", "ç"), ("\\c c", "ç"), ("\\`a", "à"), ("--", "–"), ("\\%", "%"), ("\\&", "&"), ("\\_", "_"), ("\\#", "#")] {
        s = s.replace(de, para);
    }
    s.replace(['{', '}'], "")
}

/// Unidades do `siunitx` (as comuns) em texto romano.
fn unidade_si(u: &str) -> Vec<No> {
    let mut texto = String::new();
    let mut l = Vec::new();
    let mut resto = u;
    let empurrar = |texto: &mut String, l: &mut Vec<No>| {
        if !texto.is_empty() {
            l.push(No::ord(Nucleo::Texto(std::mem::take(texto), Alfabeto::Romano)));
        }
    };
    while !resto.is_empty() {
        if let Some(r) = resto.strip_prefix('\\') {
            let fim = r.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(r.len());
            let nome = &r[..fim];
            resto = &r[fim..];
            let s = match nome {
                "meter" | "metre" => "m",
                "second" => "s",
                "gram" => "g",
                "kilogram" => "kg",
                "newton" => "N",
                "joule" => "J",
                "watt" => "W",
                "volt" => "V",
                "ampere" => "A",
                "ohm" => "Ω",
                "hertz" => "Hz",
                "kelvin" => "K",
                "celsius" | "degreeCelsius" => "°C",
                "pascal" => "Pa",
                "mole" => "mol",
                "candela" => "cd",
                "coulomb" => "C",
                "farad" => "F",
                "henry" => "H",
                "tesla" => "T",
                "weber" => "Wb",
                "siemens" => "S",
                "liter" | "litre" => "L",
                "minute" => "min",
                "hour" => "h",
                "percent" => "%",
                "degree" => "°",
                "kilo" => "k",
                "mega" => "M",
                "giga" => "G",
                "milli" => "m",
                "micro" => "μ",
                "nano" => "n",
                "centi" => "c",
                "per" => "/",
                "squared" | "cubed" => {
                    empurrar(&mut texto, &mut l);
                    if let Some(u) = l.last_mut() {
                        let n = if nome == "squared" { '2' } else { '3' };
                        u.sup = Some(vec![car(n, Alfabeto::Romano)]);
                    }
                    continue;
                }
                _ => nome,
            };
            // Entre duas unidades, um espaço fino (N m); depois de prefixo (k, m…) ou de `/`, nada.
            let prefixo = matches!(nome, "kilo" | "mega" | "giga" | "milli" | "micro" | "nano" | "centi");
            if s != "/" && !texto.is_empty() && !prefixo && !texto.ends_with(['k', 'M', 'G', 'm', 'μ', 'n', 'c', '/']) {
                texto.push('\u{2009}');
            }
            texto.push_str(s);
        } else {
            let c = resto.chars().next().unwrap_or(' ');
            resto = &resto[c.len_utf8()..];
            if c == '^' {
                empurrar(&mut texto, &mut l);
                let d: String = resto.chars().take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '{' || *c == '}').collect();
                resto = &resto[d.len()..];
                if let Some(u) = l.last_mut() {
                    u.sup = Some(vec![No::ord(Nucleo::Texto(d.replace(['{', '}'], "").replace('-', "−"), Alfabeto::Romano))]);
                }
            } else if c == '.' || c == '~' {
                texto.push('\u{2009}');
            } else if c != '{' && c != '}' {
                texto.push(c);
            }
        }
    }
    empurrar(&mut texto, &mut l);
    l
}

/// `{lcr|l}` → alinhamentos e os traços verticais.
fn colunas_do_array(espec: &str) -> (Vec<Alinh>, Vec<usize>) {
    let mut cols = Vec::new();
    let mut tracos = Vec::new();
    let mut cs = espec.chars().peekable();
    while let Some(c) = cs.next() {
        match c {
            'l' => cols.push(Alinh::Esq),
            'c' => cols.push(Alinh::Centro),
            'r' => cols.push(Alinh::Dir),
            'p' | 'm' | 'b' | 'X' => {
                cols.push(Alinh::Esq);
                if cs.peek() == Some(&'{') {
                    for d in cs.by_ref() {
                        if d == '}' {
                            break;
                        }
                    }
                }
            }
            '|' => tracos.push(cols.len()),
            '@' | '!' | '>' | '<' if cs.peek() == Some(&'{') => {
                let mut nivel = 0;
                for d in cs.by_ref() {
                    if d == '{' {
                        nivel += 1;
                    } else if d == '}' {
                        nivel -= 1;
                        if nivel == 0 {
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    (cols, tracos)
}

/// Reparte a lista do corpo de um ambiente em linhas e células (pelas marcas `&` e `\\`).
pub fn montar_matriz(corpo: Vec<No>, tipo: TipoMatriz, colunas: &[Alinh]) -> Matriz {
    let mut m = Matriz { tipo: Some(tipo), colunas: colunas.to_vec(), sep: 1.0, ..Matriz::default() };
    let mut linha: Vec<Vec<No>> = vec![Vec::new()];
    let mut rotulo = None;
    let mut numero: Option<Numero> = None;
    for no in corpo {
        match no.nucleo {
            Nucleo::SepColuna if no.sup.is_none() => linha.push(Vec::new()),
            Nucleo::SepLinha(extra) => {
                m.linhas.push(std::mem::replace(&mut linha, vec![Vec::new()]));
                m.extra.push(extra);
                m.rotulos.push(rotulo.take());
                m.numeros.push(numero.take().unwrap_or(Numero::Automatico));
            }
            Nucleo::Hline => m.hlines.push(m.linhas.len()),
            Nucleo::Rotulo(r) => rotulo = Some(r),
            Nucleo::Tag(t) => numero = Some(Numero::Tag(t)),
            Nucleo::Intertexto(t) => {
                // `\intertext` no meio da linha (sem o `\\` antes): fecha a linha ali.
                if linha.iter().any(|c| !c.is_empty()) {
                    m.linhas.push(std::mem::replace(&mut linha, vec![Vec::new()]));
                    m.extra.push(0.0);
                    m.rotulos.push(rotulo.take());
                    m.numeros.push(numero.take().unwrap_or(Numero::Automatico));
                }
                m.intertextos.push((m.linhas.len(), t));
            }
            Nucleo::SemNumero => numero = Some(Numero::Nenhum),
            _ => linha.last_mut().expect("sempre há uma célula").push(no),
        }
    }
    // A última linha vazia (um `\\` no fim) não conta.
    let vazia = linha.len() == 1 && linha[0].is_empty();
    if !vazia || m.linhas.is_empty() {
        m.linhas.push(linha);
        m.extra.push(0.0);
        m.rotulos.push(rotulo);
        m.numeros.push(numero.unwrap_or(Numero::Automatico));
    } else if let Some(n) = numero {
        // `\tag` depois do último `\\`: vale para a linha anterior.
        if let Some(u) = m.numeros.last_mut() {
            *u = n;
        }
    }
    m
}

/// Decide o número de cada linha conforme o ambiente (com estrela = sem número, salvo `\tag`).
fn numerar(m: &mut Matriz, base: &str, estrela: bool) {
    let numerado = matches!(base, "equation" | "align" | "gather" | "multline" | "eqnarray" | "flalign" | "alignat" | "xalignat") && !estrela;
    let n = m.numeros.len();
    for (k, num) in m.numeros.iter_mut().enumerate() {
        let explicito = matches!(num, Numero::Tag(_)) || (*num == Numero::Nenhum);
        if matches!(num, Numero::Tag(_)) {
            continue;
        }
        if !numerado {
            *num = Numero::Nenhum;
            continue;
        }
        if explicito {
            continue;
        }
        // multline e equation: um número só, na última linha.
        if matches!(base, "multline" | "equation") && k + 1 != n {
            *num = Numero::Nenhum;
        }
    }
    if matches!(base, "multline" | "equation") && numerado {
        // Um \tag ou \nonumber em qualquer linha vale para o ambiente inteiro.
        let especial = m.numeros.iter().find(|x| matches!(x, Numero::Tag(_))).cloned().or_else(|| m.numeros.contains(&Numero::Nenhum).then_some(Numero::Nenhum).filter(|_| n == 1));
        if let (Some(e), Some(u)) = (especial, m.numeros.last_mut()) {
            *u = e;
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn fracao_expoente_e_comando_desconhecido() {
        let l = analisar("\\frac{a}{b}^2 + \\foo");
        assert_eq!(l.len(), 3);
        assert!(matches!(l[0].nucleo, Nucleo::Fracao { .. }));
        assert!(l[0].sup.is_some());
        assert_eq!(l[1].classe, Classe::Bin);
        assert!(matches!(l[2].nucleo, Nucleo::Cor(COR_ERRO, _)));
    }

    #[test]
    fn escapes_dentro_do_text() {
        let d = format!("{:?}", analisar("\\text{59,3\\% R\\& D}"));
        assert!(d.contains("59,3% R& D") && !d.contains("\\\\%"), "{d}");
    }

    #[test]
    fn texto_quebrado_nao_trava() {
        for t in ["\\frac{", "x^", "\\left(", "\\begin{pmatrix} a & b", "}}}", "\\sqrt[", "{{{{", "\\", "a_{b^{c_{d"] {
            let _ = analisar(t);
        }
        let fundo = "{".repeat(500);
        let _ = analisar(&fundo);
    }

    #[test]
    fn align_com_tag_e_nonumber() {
        let l = analisar("\\begin{align} a &= b \\label{x}\\\\ c &= d \\nonumber \\\\ e &= f \\tag{*} \\end{align}");
        let Nucleo::Matriz(m) = &l[0].nucleo else { panic!("{l:?}") };
        assert_eq!(m.linhas.len(), 3);
        assert_eq!(m.numeros, vec![Numero::Automatico, Numero::Nenhum, Numero::Tag("*".into())]);
        assert_eq!(m.rotulos[0].as_deref(), Some("x"));
        assert_eq!(m.linhas[0].len(), 2);
    }

    #[test]
    fn not_igual_vira_diferente() {
        let l = analisar("a \\not= b");
        assert!(matches!(l[1].nucleo, Nucleo::Car('≠', _)));
    }
}
