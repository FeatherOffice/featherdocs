//! Comandos de pacotes além do núcleo e do `amsmath`: `mathtools` (`\prescript`, `\mathclap`,
//! `\xlongequal`…), `physics` (`\abs`, `\dv`, `\ket`, `\qty`, `\mqty`…), `braket`, `tensor`,
//! `cancel` e `\genfrac`. A maioria é um molde em LaTeX que o próprio analisador lê de novo.

use super::{car, Analisador};
use crate::arvore::{Alfabeto, Alinh, Classe, Nivel, No, Nucleo};
use crate::simbolos;

/// Troca `#1`, `#2`… pelos argumentos.
fn preencher(molde: &str, args: &[String]) -> String {
    let mut t = molde.to_string();
    for (k, a) in args.iter().enumerate().rev() {
        t = t.replace(&format!("#{}", k + 1), a);
    }
    t
}

/// `^a_b` → `{}^a{}_b`: cada índice do `\tensor` numa coluna própria.
fn escalonar(arg: &str) -> String {
    let mut s = String::new();
    let mut nivel = 0;
    for c in arg.chars() {
        match c {
            '{' => nivel += 1,
            '}' => nivel -= 1,
            '^' | '_' if nivel == 0 => s.push_str("{}"),
            _ => {}
        }
        s.push(c);
    }
    s
}

/// O par de delimitadores de `\qty(…)`, `\mqty[…]`: quem abre, quem fecha e o nome do ambiente.
fn par_de(c: char) -> Option<(char, &'static str, &'static str, &'static str)> {
    Some(match c {
        '(' => (')', "\\left(", "\\right)", "pmatrix"),
        '[' => (']', "\\left[", "\\right]", "bmatrix"),
        '{' => ('}', "\\left\\{", "\\right\\}", "matrix"),
        '|' => ('|', "\\left|", "\\right|", "vmatrix"),
        _ => return None,
    })
}

impl Analisador {
    /// Os comandos de pacote; `None` = não é um deles.
    pub(super) fn comando_pacote(&mut self, n: &str) -> Option<No> {
        if let Some(no) = self.physics(n) {
            return Some(no);
        }
        Some(match n {
            "prescript" => {
                let (sup, sub, base) = (self.cru(), self.cru(), self.cru());
                self.molde(&format!("{{}}^{{{sup}}}_{{{sub}}}{{{base}}}"))
            }
            "sideset" => {
                let (antes, depois) = (self.cru(), self.cru());
                let op = self.atomo().unwrap_or_else(|| No::ord(Nucleo::Vazio));
                let mut l = self.sub(&format!("{{}}{antes}"), self.alf);
                l.push(No::novo(Classe::Op, op.nucleo));
                l.extend(self.sub(&format!("{{}}{depois}"), self.alf));
                No::novo(Classe::Op, Nucleo::Grupo(l))
            }
            "tensor" | "indices" => {
                if self.espiar() == Some('*') {
                    self.i += 1;
                }
                let base = if n == "tensor" { self.cru() } else { String::new() };
                let indices = escalonar(&self.cru());
                self.molde(&format!("{{{base}}}{indices}"))
            }
            "mathclap" | "mathllap" | "mathrlap" | "clap" | "llap" | "rlap" => {
                self.opcional();
                let alinh = match n.trim_start_matches("math") {
                    "llap" => Alinh::Dir,
                    "rlap" => Alinh::Esq,
                    _ => Alinh::Centro,
                };
                No::ord(Nucleo::Lap(self.argumento(), alinh))
            }
            "lefteqn" => {
                let mut l = vec![No::ord(Nucleo::Estilo(Nivel::D))];
                l.extend(self.argumento());
                No::ord(Nucleo::Lap(l, Alinh::Esq))
            }
            "smashoperator" => {
                self.opcional();
                No::novo(Classe::Op, Nucleo::Grupo(self.argumento()))
            }
            "shoveleft" | "shoveright" => No::ord(Nucleo::Grupo(self.argumento())),
            "cancel" | "bcancel" | "xcancel" => {
                let tipo = match n {
                    "cancel" => 1,
                    "bcancel" => 2,
                    _ => 3,
                };
                No::ord(Nucleo::Riscado { corpo: self.argumento(), tipo, valor: None })
            }
            "cancelto" => {
                let valor = self.argumento();
                No::ord(Nucleo::Riscado { corpo: self.argumento(), tipo: 4, valor: Some(valor) })
            }
            "genfrac" => self.genfrac(),
            "xlongequal" | "xtwoheadrightarrow" | "xtwoheadleftarrow" | "xLongrightarrow" | "xLongleftarrow" => {
                let c = match n {
                    "xlongequal" => '=',
                    "xtwoheadrightarrow" => '↠',
                    "xtwoheadleftarrow" => '↞',
                    "xLongrightarrow" => '⇒',
                    _ => '⇐',
                };
                self.seta_rotulada(c)
            }
            "idotsint" => {
                let l = vec![car('∫', Alfabeto::Romano), No::ord(Nucleo::Espaco(-3.0)), car('⋯', Alfabeto::Romano), No::ord(Nucleo::Espaco(-3.0)), car('∫', Alfabeto::Romano)];
                let l = l.into_iter().map(|mut x| {
                    if matches!(x.nucleo, Nucleo::Car('∫', _)) {
                        x.classe = Classe::Op;
                    }
                    x
                }).collect();
                let mut no = No::novo(Classe::Op, Nucleo::Grupo(l));
                no.limites = Some(false);
                no
            }
            "varlimsup" | "varliminf" | "varinjlim" | "varprojlim" => {
                let lim = vec![No::ord(Nucleo::Texto("lim".into(), Alfabeto::Romano))];
                let nucleo = match n {
                    "varlimsup" => Nucleo::Barra { corpo: lim, cima: true },
                    "varliminf" => Nucleo::Barra { corpo: lim, cima: false },
                    "varinjlim" => Nucleo::Acento { c: '→', corpo: lim, largo: true, baixo: true },
                    _ => Nucleo::Acento { c: '←', corpo: lim, largo: true, baixo: true },
                };
                No::novo(Classe::Op, nucleo)
            }
            "MoveEqLeft" | "ArrowBetweenLines" | "SwapAboveDisplaySkip" | "displaybreak" | "allowdisplaybreaks" => {
                self.opcional();
                No::ord(Nucleo::EspacoEm(0.0))
            }
            "rule" => {
                // `\rule[desce]{largura}{altura}`: um retângulo cheio apoiado na linha de base.
                let sobe = self.opcional().and_then(|t| super::medida_em(&t)).unwrap_or(0.0);
                let w = super::medida_em(&self.cru()).unwrap_or(0.0);
                let h = super::medida_em(&self.cru()).unwrap_or(0.0);
                No::ord(Nucleo::Regra { largura: w, altura: h, sobe })
            }
            "hdotsfor" => {
                self.opcional();
                self.cru();
                No::novo(Classe::Interno, Nucleo::Car('⋯', Alfabeto::Romano))
            }
            "matrix" | "pmatrix" if self.proximo_e('{') => self.matriz_plain(n),
            _ => return None,
        })
    }

    fn proximo_e(&mut self, c: char) -> bool {
        let salvo = self.i;
        self.pular_brancos();
        let sim = self.espiar() == Some(c);
        self.i = salvo;
        sim
    }

    /// Lê de novo um LaTeX montado (molde) como um grupo.
    fn molde(&mut self, latex: &str) -> No {
        No::ord(Nucleo::Grupo(self.sub(latex, self.alf)))
    }

    fn genfrac(&mut self) -> No {
        let esq = simbolos::delimitador(self.cru().trim()).flatten();
        let dir = simbolos::delimitador(self.cru().trim()).flatten();
        let espessura = self.cru();
        let barra = !espessura.trim().starts_with('0');
        let nivel = match self.cru().trim() {
            "0" => Some(Nivel::D),
            "1" => Some(Nivel::T),
            "2" => Some(Nivel::S),
            "3" => Some(Nivel::SS),
            _ => None,
        };
        let num = self.argumento();
        let den = self.argumento();
        No::ord(Nucleo::Fracao { num, den, barra, nivel, esq, dir })
    }

    /// `\xlongequal[sob]{sobre}` e companhia: a mesma montagem do `\xrightarrow`.
    fn seta_rotulada(&mut self, c: char) -> No {
        let embaixo = self.opcional().map(|t| self.sub(&t, self.alf));
        let em_cima = self.argumento();
        let mut no = No::novo(Classe::Rel, Nucleo::Chave { c, corpo: Vec::new(), cima: true });
        no.limites = Some(true);
        no.sup = Some(em_cima);
        no.sub = embaixo;
        no
    }

    /// `\matrix{a & b \cr c & d}` (plain TeX): o `\cr` vale `\\`.
    fn matriz_plain(&mut self, n: &str) -> No {
        let corpo = self.cru().replace("\\cr", "\\\\");
        let env = if n == "pmatrix" { "pmatrix" } else { "matrix" };
        self.molde(&format!("\\begin{{{env}}}{corpo}\\end{{{env}}}"))
    }

    /// O conteúdo até o delimitador que fecha `fecha`, com os de dentro balanceados (para `\qty(…)`).
    fn ate_fechar(&mut self, abre: char, fecha: char) -> String {
        let inicio = self.i;
        let mut nivel = 0;
        while let Some(c) = self.espiar() {
            if c == '\\' {
                self.i += 2;
                continue;
            }
            if c == fecha && nivel == 0 {
                break;
            }
            if c == abre && abre != fecha {
                nivel += 1;
            } else if c == fecha {
                nivel -= 1;
            }
            self.i += 1;
        }
        let fim = self.i.min(self.s.len());
        self.i = (self.i + 1).min(self.s.len());
        self.s[inicio..fim].iter().collect()
    }

    /// O pacote `physics` (e os mesmos nomes do `braket` e do `\DeclarePairedDelimiter` comum).
    fn physics(&mut self, n: &str) -> Option<No> {
        let delimitado = |esq: &str, dir: &str| format!("\\left{esq} #1 \\right{dir}");
        let molde = match n {
            "abs" => delimitado("|", "|"),
            "norm" => delimitado("\\|", "\\|"),
            "ket" | "Ket" => delimitado("|", "\\rangle"),
            "bra" | "Bra" => delimitado("\\langle", "|"),
            "expval" | "ev" => delimitado("\\langle", "\\rangle"),
            "Set" => delimitado("\\{", "\\}"),
            "order" => "\\mathcal{O}\\left( #1 \\right)".to_string(),
            "comm" => "\\left[ #1, #2 \\right]".to_string(),
            "acomm" | "anticommutator" => "\\left\\{ #1, #2 \\right\\}".to_string(),
            "braket" => {
                let a = self.cru();
                let t = if self.proximo_e('{') {
                    let b = self.cru();
                    format!("\\left\\langle {a} \\middle| {b} \\right\\rangle")
                } else {
                    format!("\\left\\langle {a} \\right\\rangle")
                };
                return Some(self.molde(&t));
            }
            "dv" | "pdv" | "odv" => return Some(self.derivada(n)),
            "dd" => {
                let ordem = self.opcional().map(|o| format!("^{{{o}}}")).unwrap_or_default();
                if self.proximo_e('{') {
                    let x = self.cru();
                    return Some(self.molde(&format!("\\,\\mathrm{{d}}{ordem}{x}")));
                }
                return Some(self.molde(&format!("\\mathrm{{d}}{ordem}")));
            }
            "grad" => return Some(self.molde("\\nabla")),
            "curl" => return Some(self.molde("\\nabla\\times")),
            "laplacian" => return Some(self.molde("\\nabla^2")),
            "Tr" | "Rank" | "Res" | "erfc" | "Pf" => {
                let mut no = No::novo(Classe::Op, Nucleo::Texto(n.to_string(), Alfabeto::Romano));
                no.limites = Some(false);
                return Some(no);
            }
            "qty" | "mqty" | "pmqty" | "bmqty" | "vmqty" => {
                self.pular_brancos();
                let abre = self.espiar()?;
                if n == "qty" && abre == '{' {
                    return None; // o `\qty{3}{m}` do siunitx
                }
                let (fecha, esq, dir, env) = par_de(abre)?;
                self.i += 1;
                let dentro = self.ate_fechar(abre, fecha);
                let t = match n {
                    "qty" => format!("{esq} {dentro} {dir}"),
                    "mqty" => format!("\\begin{{{env}}}{dentro}\\end{{{env}}}"),
                    "pmqty" => format!("\\begin{{pmatrix}}{dentro}\\end{{pmatrix}}"),
                    "bmqty" => format!("\\begin{{bmatrix}}{dentro}\\end{{bmatrix}}"),
                    _ => format!("\\begin{{vmatrix}}{dentro}\\end{{vmatrix}}"),
                };
                return Some(self.molde(&t));
            }
            _ => return None,
        };
        // `\abs*{x}`, `\abs[\big]{x}`: estrela = crescer (já é o padrão aqui); tamanho fixo pelo opcional.
        if self.espiar() == Some('*') {
            self.i += 1;
        }
        let tamanho = self.opcional();
        let nargs = if molde.contains("#2") { 2 } else { 1 };
        let args: Vec<String> = (0..nargs).map(|_| self.cru()).collect();
        let mut t = preencher(&molde, &args);
        if let Some(tam) = tamanho.filter(|t| !t.trim().is_empty()) {
            let tam = tam.trim().to_string();
            t = t.replace("\\left", &format!("{tam}l")).replace("\\right", &format!("{tam}r")).replace("\\middle", &format!("{tam}m"));
        }
        Some(self.molde(&t))
    }

    /// `\dv[n]{f}{x}` e `\pdv{f}{x}{y}`: as derivadas do `physics`.
    fn derivada(&mut self, n: &str) -> No {
        let d = if n == "pdv" { "\\partial" } else { "\\mathrm{d}" };
        let ordem = self.opcional();
        let f = self.cru();
        let x = if self.proximo_e('{') { Some(self.cru()) } else { None };
        let y = if n == "pdv" && self.proximo_e('{') { Some(self.cru()) } else { None };
        let pot = |o: &Option<String>| o.as_ref().map(|o| format!("^{{{o}}}")).unwrap_or_default();
        let t = match (x, y) {
            (Some(x), Some(y)) => format!("\\frac{{{d}^2 {f}}}{{{d} {x}\\,{d} {y}}}"),
            (Some(x), None) => format!("\\frac{{{d}{} {f}}}{{{d} {x}{}}}", pot(&ordem), pot(&ordem)),
            (None, _) => format!("\\frac{{{d}{}}}{{{d} {f}{}}}", pot(&ordem), pot(&ordem)),
        };
        self.molde(&t)
    }
}
