//! `feather-math`: fórmulas matemáticas em LaTeX compostas como o TeX compõe, em Rust puro.
//!
//! Entrada: o texto LaTeX da fórmula (o de dentro de `$…$`, `\[…\]` ou um ambiente inteiro como
//! `\begin{align}…\end{align}`). Saída: uma [`Formula`] com glifos posicionados da fonte
//! matemática embutida (Latin Modern Math) e retângulos (barra de fração, traço da raiz,
//! linhas de matriz), com largura, altura e profundidade em pontos para alinhar à linha de base
//! do texto. Quem desenha pede o contorno de cada glifo com [`contorno`].
//!
//! O algoritmo é o do apêndice G do TeXbook (estilos D/T/S/SS, classes de átomo e o espaço
//! entre elas, expoente e índice, limites, frações, raízes, delimitadores que crescem,
//! acentos), com as medidas da tabela OpenType `MATH` da fonte.
//!
//! Comum a toda a suíte: o FeatherDocs usa hoje; FeatherSlide e FeatherSheet podem usar.

mod alfabetos;
pub mod analise;
pub mod arvore;
mod caixa;
mod composicao;
mod fonte;
mod matriz;
pub mod omml;
mod omml_leitura;
mod simbolos;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub use arvore::Numero;
pub use fonte::Contorno;

use arvore::{Nivel, Nucleo, TipoMatriz};
use composicao::{hlista, Ctx};

/// Um glifo da fonte matemática: `(x, y)` é a origem dele na linha de base (em pontos, desde a
/// origem da fórmula; `y` cresce para baixo). `cor` 0 = a cor do texto.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Glifo {
    pub id: u16,
    pub x: f32,
    pub y: f32,
    pub tamanho: f32,
    pub cor: u32,
}

/// Um retângulo cheio (barra de fração, traço da raiz, linha de matriz): `y` é o topo dele.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Retangulo {
    pub x: f32,
    pub y: f32,
    pub largura: f32,
    pub altura: f32,
    pub cor: u32,
}

/// Um traço reto de espessura `espessura` (o risco do `\cancel`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Traco {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub espessura: f32,
    pub cor: u32,
}

/// Fórmula no meio do texto (estilo de texto) ou em destaque (estilo display, com ambientes de
/// várias linhas e numeração).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Modo {
    Linha,
    Exibida,
}

/// Uma linha de equação em destaque: onde fica a linha de base dela (desde a linha de base da
/// fórmula, que é a da primeira linha) e o número que leva.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LinhaDeEquacao {
    pub base: f32,
    pub numero: Numero,
    pub rotulo: Option<String>,
}

/// A fórmula composta. A origem é a ponta esquerda da linha de base.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Formula {
    pub largura: f32,
    pub altura: f32,
    pub profundidade: f32,
    pub glifos: Vec<Glifo>,
    pub retangulos: Vec<Retangulo>,
    pub tracos: Vec<Traco>,
    /// Só no modo exibido: uma entrada por linha da equação.
    pub linhas: Vec<LinhaDeEquacao>,
}

/// Compõe `latex` no corpo `tamanho` (pontos).
pub fn compor(latex: &str, modo: Modo, tamanho: f32) -> Formula {
    let nos = analise::analisar(latex);
    let tamanho = if tamanho.is_finite() && tamanho > 0.0 { tamanho } else { 10.0 };
    match modo {
        Modo::Linha => {
            let cx = hlista(&nos, &Ctx::novo(Nivel::T, tamanho));
            Formula { largura: cx.w, altura: cx.h, profundidade: cx.d, glifos: cx.glifos, retangulos: cx.rets, tracos: cx.tracos, linhas: Vec::new() }
        }
        Modo::Exibida => exibir(&nos, tamanho),
    }
}

/// O ambiente de equação do topo (o `align` inteiro), se a fórmula é um.
fn ambiente_do_topo(nos: &[arvore::No]) -> Option<&arvore::Matriz> {
    let mut atomos = nos.iter().filter(|n| !matches!(n.nucleo, Nucleo::EspacoEm(w) if w == 0.0));
    let no = atomos.next()?;
    if atomos.next().is_some() || no.sup.is_some() || no.sub.is_some() {
        return None;
    }
    match &no.nucleo {
        Nucleo::Matriz(m) if matches!(m.tipo, Some(TipoMatriz::Alinhado | TipoMatriz::Reunido | TipoMatriz::Multlinha | TipoMatriz::Eqnarray)) && m.esq.is_none() && m.dir.is_none() => Some(m),
        _ => None,
    }
}

fn exibir(nos: &[arvore::No], tamanho: f32) -> Formula {
    let ctx = Ctx::novo(Nivel::D, tamanho);
    if let Some(m) = ambiente_do_topo(nos) {
        let (cx, linhas) = matriz::compor(m, &ctx, true);
        let linhas = linhas.into_iter().map(|l| LinhaDeEquacao { base: l.base, numero: l.numero, rotulo: l.rotulo }).collect();
        return Formula { largura: cx.w, altura: cx.h, profundidade: cx.d, glifos: cx.glifos, retangulos: cx.rets, tracos: cx.tracos, linhas };
    }
    let cx = hlista(nos, &ctx);
    let numero = nos.iter().rev().find_map(|n| match &n.nucleo {
        Nucleo::Tag(t) => Some(Numero::Tag(t.clone())),
        _ => None,
    });
    let rotulo = nos.iter().find_map(|n| match &n.nucleo {
        Nucleo::Rotulo(r) => Some(r.clone()),
        _ => None,
    });
    let linhas = vec![LinhaDeEquacao { base: 0.0, numero: numero.unwrap_or_default(), rotulo }];
    Formula { largura: cx.w, altura: cx.h, profundidade: cx.d, glifos: cx.glifos, retangulos: cx.rets, tracos: cx.tracos, linhas }
}

/// Quantos números automáticos a equação em destaque consome (`align` de 3 linhas = 3).
pub fn quantos_numeros(latex: &str) -> usize {
    let nos = analise::analisar(latex);
    ambiente_do_topo(&nos).map_or(0, |m| m.numeros.iter().filter(|n| **n == Numero::Automatico).count())
}

/// Os números de cada linha e os rótulos (`\label`), sem compor.
pub fn numeros_das_linhas(latex: &str) -> Vec<(Numero, Option<String>)> {
    let nos = analise::analisar(latex);
    match ambiente_do_topo(&nos) {
        Some(m) => m.numeros.iter().cloned().zip(m.rotulos.iter().cloned()).collect(),
        None => Vec::new(),
    }
}

/// Os comandos que o compositor não conhece (saem em vermelho com o próprio nome) e os caracteres que a
/// fonte não tem (saem como caixinha vazia), na ordem.
pub fn desconhecidos(latex: &str) -> Vec<String> {
    fn visitar(nos: &[arvore::No], v: &mut Vec<String>) {
        for n in nos {
            for s in [&n.sup, &n.sub].into_iter().flatten() {
                visitar(s, v);
            }
            match &n.nucleo {
                Nucleo::Car(c, a) if fonte::fonte().glifo(alfabetos::estilizar(*c, *a)).or_else(|| fonte::fonte().glifo(*c)).is_none() => {
                    v.push(format!("(sem glifo: {c} U+{:04X})", *c as u32));
                }
                Nucleo::Texto(t, _) => v.extend(t.chars().filter(|c| !c.is_whitespace() && fonte::fonte().glifo(*c).is_none()).map(|c| format!("(sem glifo: {c} U+{:04X})", c as u32))),
                Nucleo::Cor(c, l) if *c == analise::COR_ERRO => v.extend(l.iter().filter_map(|x| match &x.nucleo {
                    Nucleo::Texto(t, _) => Some(t.clone()),
                    _ => None,
                })),
                Nucleo::Cor(_, l) | Nucleo::Grupo(l) | Nucleo::Achatado(l) | Nucleo::Caixa(l) => visitar(l, v),
                Nucleo::Fantasma { corpo, .. } | Nucleo::Acento { corpo, .. } | Nucleo::Barra { corpo, .. } | Nucleo::Chave { corpo, .. } => visitar(corpo, v),
                Nucleo::Fracao { num, den, .. } => {
                    visitar(num, v);
                    visitar(den, v);
                }
                Nucleo::Raiz { indice, corpo } => {
                    visitar(indice.as_deref().unwrap_or(&[]), v);
                    visitar(corpo, v);
                }
                Nucleo::Delimitado { partes, .. } => partes.iter().for_each(|p| visitar(p, v)),
                Nucleo::Matriz(m) => m.linhas.iter().flatten().for_each(|c| visitar(c, v)),
                Nucleo::Negado(b) => visitar(std::slice::from_ref(b.as_ref()), v),
                Nucleo::Lap(l, _) | Nucleo::Intertexto(l) => visitar(l, v),
                Nucleo::Riscado { corpo, valor, .. } => {
                    visitar(corpo, v);
                    visitar(valor.as_deref().unwrap_or(&[]), v);
                }
                _ => {}
            }
        }
    }
    let mut v = Vec::new();
    visitar(&analise::analisar(latex), &mut v);
    v
}

type Chave = (String, Modo, u32);

/// Como [`compor`], guardando o resultado (o desenho e a medida pedem a mesma fórmula muitas vezes).
pub fn compor_guardado(latex: &str, modo: Modo, tamanho: f32) -> Arc<Formula> {
    static GUARDA: OnceLock<Mutex<HashMap<Chave, Arc<Formula>>>> = OnceLock::new();
    let guarda = GUARDA.get_or_init(|| Mutex::new(HashMap::new()));
    let chave = (latex.to_string(), modo, tamanho.to_bits());
    if let Some(f) = guarda.lock().ok().and_then(|g| g.get(&chave).cloned()) {
        return f;
    }
    let f = Arc::new(compor(latex, modo, tamanho));
    if let Ok(mut g) = guarda.lock() {
        if g.len() > 4096 {
            g.clear();
        }
        g.insert(chave, Arc::clone(&f));
    }
    f
}

/// A largura da fórmula, em pontos.
pub fn largura(latex: &str, modo: Modo, tamanho: f32) -> f32 {
    compor_guardado(latex, modo, tamanho).largura
}

/// Entrega o contorno do glifo `id` no corpo `tamanho`, com a origem em `(x, y)` (pontos, `y` para baixo).
pub fn contorno<C: Contorno>(id: u16, tamanho: f32, x: f32, y: f32, destino: &mut C) {
    fonte::fonte().contorno(id, tamanho, x, y, destino);
}

/// Os bytes da fonte embutida (para quem precisar registrá-la, como o exportador de PDF).
pub fn bytes_da_fonte() -> &'static [u8] {
    include_bytes!("../assets/latinmodern-math.otf")
}

struct CaminhoSvg(String);

impl Contorno for CaminhoSvg {
    fn mover(&mut self, x: f32, y: f32) {
        self.0.push_str(&format!("M{x:.2} {y:.2}"));
    }
    fn linha(&mut self, x: f32, y: f32) {
        self.0.push_str(&format!("L{x:.2} {y:.2}"));
    }
    fn quadratica(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.push_str(&format!("Q{x1:.2} {y1:.2} {x:.2} {y:.2}"));
    }
    fn cubica(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.push_str(&format!("C{x1:.2} {y1:.2} {x2:.2} {y2:.2} {x:.2} {y:.2}"));
    }
    fn fechar(&mut self) {
        self.0.push('Z');
    }
}

fn cor_svg(cor: u32, padrao: &str) -> String {
    if cor == 0 {
        padrao.to_string()
    } else {
        format!("#{:06x}", cor >> 8)
    }
}

/// A fórmula como SVG (contornos, sem depender de fonte instalada). `margem` em pontos.
pub fn svg(f: &Formula, cor: &str, margem: f32) -> String {
    let (w, h) = (f.largura + 2.0 * margem, f.altura + f.profundidade + 2.0 * margem);
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}pt\" height=\"{h:.2}pt\" viewBox=\"{:.2} {:.2} {w:.2} {h:.2}\">",
        -margem,
        -f.altura - margem
    );
    for g in &f.glifos {
        let mut c = CaminhoSvg(String::new());
        contorno(g.id, g.tamanho, g.x, g.y, &mut c);
        if !c.0.is_empty() {
            s.push_str(&format!("<path fill=\"{}\" d=\"{}\"/>", cor_svg(g.cor, cor), c.0));
        }
    }
    for r in &f.retangulos {
        s.push_str(&format!("<rect fill=\"{}\" x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\"/>", cor_svg(r.cor, cor), r.x, r.y, r.largura, r.altura));
    }
    for t in &f.tracos {
        s.push_str(&format!("<line stroke=\"{}\" stroke-width=\"{:.2}\" x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>", cor_svg(t.cor, cor), t.espessura, t.x1, t.y1, t.x2, t.y2));
    }
    s.push_str("</svg>");
    s
}
