//! A composição: lista de átomos → caixas, pelo apêndice G do TeXbook, com as medidas
//! tiradas da tabela OpenType `MATH` (como fazem XeTeX/LuaTeX e o MathML Core). Nenhum
//! número inventado: o que não vem da tabela é parâmetro do próprio LaTeX (em em, com o
//! valor em pontos do corpo de 10 pt no comentário).

use crate::alfabetos::estilizar;
use crate::arvore::{Alfabeto, Classe, Nivel, No, Nucleo};
use crate::caixa::Caixa;
use crate::fonte::{fonte, Peca};
use crate::Glifo;

/// `\nulldelimiterspace` = 1,2 pt.
pub const NULLDELIMITERSPACE: f32 = 0.12;
/// `\delimiterfactor` = 901 e `\delimitershortfall` = 5 pt.
const DELIMITERFACTOR: f32 = 0.901;
const DELIMITERSHORTFALL: f32 = 0.5;
/// `\fboxsep` = 3 pt e `\fboxrule` = 0,4 pt (`\boxed`).
const FBOXSEP: f32 = 0.3;
const FBOXRULE: f32 = 0.04;

/// O estado da composição: estilo, se está "apertado" (cramped), o corpo do texto e a cor.
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    pub nivel: Nivel,
    pub apertado: bool,
    /// Corpo do texto em pontos (o do estilo display/text).
    pub base: f32,
    pub cor: u32,
}

impl Ctx {
    pub fn novo(nivel: Nivel, base: f32) -> Self {
        Self { nivel, apertado: false, base, cor: 0 }
    }

    /// O corpo do estilo atual, em pontos.
    pub fn tam(&self) -> f32 {
        let c = &fonte().c;
        match self.nivel {
            Nivel::D | Nivel::T => self.base,
            Nivel::S => self.base * c.script_percent,
            Nivel::SS => self.base * c.script_script_percent,
        }
    }

    /// Uma constante da fonte (em em) no corpo atual.
    pub fn pt(&self, em: f32) -> f32 {
        em * self.tam()
    }

    fn com(&self, nivel: Nivel, apertado: bool) -> Self {
        Self { nivel, apertado, ..*self }
    }

    /// O estilo do expoente.
    pub fn sup(&self) -> Self {
        let n = if self.nivel <= Nivel::T { Nivel::S } else { Nivel::SS };
        self.com(n, self.apertado)
    }

    /// O estilo do índice (sempre apertado).
    pub fn sub(&self) -> Self {
        self.com(self.sup().nivel, true)
    }

    /// Numerador: D→T, T→S, S→SS.
    pub fn num(&self) -> Self {
        let n = match self.nivel {
            Nivel::D => Nivel::T,
            Nivel::T => Nivel::S,
            _ => Nivel::SS,
        };
        self.com(n, self.apertado)
    }

    pub fn den(&self) -> Self {
        self.com(self.num().nivel, true)
    }

    pub fn apertar(&self) -> Self {
        self.com(self.nivel, true)
    }

    pub fn display(&self) -> bool {
        self.nivel == Nivel::D
    }
}

// ───────────────────────────── glifos ─────────────────────────────

/// A caixa de um glifo pelo número.
pub fn glifo(g: u16, ctx: &Ctx) -> Caixa {
    let f = fonte();
    let t = ctx.tam();
    let (h, d) = f.altura_profundidade(g);
    Caixa {
        w: f.avanco(g) * t,
        h: h * t,
        d: d * t,
        glifos: vec![Glifo { id: g, x: 0.0, y: 0.0, tamanho: t, cor: ctx.cor }],
        italico: f.italico(g) * t,
        acento: f.encaixe_do_acento(g).map(|a| a * t),
        simples: true,
        ..Caixa::default()
    }
}

/// O glifo de um caractere (o quadrado vazio da fonte, se ela não tem).
pub fn id_do_caractere(c: char) -> u16 {
    fonte().glifo(c).unwrap_or(0)
}

pub fn caractere(c: char, ctx: &Ctx) -> Caixa {
    match fonte().glifo(c) {
        Some(g) => glifo(g, ctx),
        None => tofu(ctx),
    }
}

/// O caractere que a fonte não tem: uma caixinha vazia, para o leitor ver que falta algo.
pub fn tofu(ctx: &Ctx) -> Caixa {
    let t = ctx.tam();
    let (w, h, e) = (0.5 * t, 0.7 * t, 0.04 * t);
    let mut cx = Caixa { w: w + 0.1 * t, h, simples: true, ..Caixa::vazia() };
    let x = 0.05 * t;
    cx.retangulo(x, -h, w, e, ctx.cor);
    cx.retangulo(x, -e, w, e, ctx.cor);
    cx.retangulo(x, -h, e, h, ctx.cor);
    cx.retangulo(x + w - e, -h, e, h, ctx.cor);
    cx
}

/// Texto em pé (nome de função, `\text`): os glifos lado a lado, sem espaço matemático.
pub fn texto(s: &str, alf: Alfabeto, ctx: &Ctx) -> Caixa {
    let mut cx = Caixa::vazia();
    let espaco = fonte().glifo(' ').map_or(0.333, |g| fonte().avanco(g));
    for c in s.chars() {
        match c {
            ' ' | '\u{a0}' => cx.seguir(&Caixa::espaco(espaco * ctx.tam())),
            '\u{2009}' => cx.seguir(&Caixa::espaco(ctx.tam() / 6.0)),
            '\u{2003}' => cx.seguir(&Caixa::espaco(ctx.tam())),
            _ => {
                let alfa = if alf == Alfabeto::Romano { c } else { estilizar(c, alf) };
                match fonte().glifo(alfa).or_else(|| fonte().glifo(c)) {
                    Some(g) => cx.seguir(&glifo(g, ctx)),
                    None => cx.seguir(&tofu(ctx)),
                }
            }
        }
    }
    cx.simples = s.chars().count() == 1;
    cx
}

// ───────────────────────────── listas e espaços ─────────────────────────────

/// O espaço entre duas classes, em mu, no estilo `nivel` (tabela do cap. 18 do TeXbook).
pub fn espaco_entre(a: Classe, b: Classe, nivel: Nivel) -> f32 {
    use Classe::*;
    const fn idx(c: Classe) -> usize {
        match c {
            Ord => 0,
            Op => 1,
            Bin => 2,
            Rel => 3,
            Abre => 4,
            Fecha => 5,
            Pont => 6,
            Interno => 7,
        }
    }
    // 1 fino, 2 médio, 3 grosso; negativo = só em display/text.
    const T: [[i8; 8]; 8] = [
        [0, 1, -2, -3, 0, 0, 0, -1],
        [1, 1, 0, -3, 0, 0, 0, -1],
        [-2, -2, 0, 0, -2, 0, 0, -2],
        [-3, -3, 0, 0, -3, 0, 0, -3],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, -2, -3, 0, 0, 0, -1],
        [-1, -1, 0, -1, -1, -1, -1, -1],
        [-1, 1, -2, -3, -1, 0, -1, -1],
    ];
    let v = T[idx(a)][idx(b)];
    if v < 0 && nivel > Nivel::T {
        return 0.0;
    }
    match v.abs() {
        1 => 3.0,
        2 => 4.0,
        3 => 5.0,
        _ => 0.0,
    }
}

enum Item {
    Atomo(Classe, Caixa, Nivel),
    Kern(f32),
}

/// Regras 5 e 6 do apêndice G: o binário sem operandos vira ordinário.
fn reclassificar(itens: &mut [Item]) {
    let mut anterior: Option<usize> = None;
    for k in 0..itens.len() {
        let Item::Atomo(c, _, _) = itens[k] else { continue };
        let antes = anterior.map(|a| match itens[a] {
            Item::Atomo(c, _, _) => c,
            Item::Kern(_) => Classe::Ord,
        });
        if c == Classe::Bin && matches!(antes, None | Some(Classe::Bin | Classe::Op | Classe::Rel | Classe::Abre | Classe::Pont)) {
            if let Item::Atomo(c, _, _) = &mut itens[k] {
                *c = Classe::Ord;
            }
        }
        if matches!(c, Classe::Rel | Classe::Fecha | Classe::Pont) && antes == Some(Classe::Bin) {
            if let Some(Item::Atomo(c, _, _)) = anterior.map(|a| &mut itens[a]) {
                *c = Classe::Ord;
            }
        }
        anterior = Some(k);
    }
    if let Some(Item::Atomo(c @ Classe::Bin, _, _)) = anterior.map(|a| &mut itens[a]) {
        *c = Classe::Ord;
    }
}

/// Uma lista horizontal de átomos, com o espaçamento do TeX entre eles.
pub fn hlista(nos: &[No], ctx0: &Ctx) -> Caixa {
    let mut ctx = *ctx0;
    let mut itens = Vec::with_capacity(nos.len());
    for no in nos {
        if no.eh_marca() {
            match no.nucleo {
                Nucleo::Estilo(n) => ctx.nivel = n,
                Nucleo::Espaco(mu) => itens.push(Item::Kern(mu * ctx.tam() / 18.0)),
                Nucleo::EspacoEm(e) => itens.push(Item::Kern(e * ctx.base)),
                _ => {}
            }
            continue;
        }
        itens.push(Item::Atomo(no.classe, atomo(no, &ctx), ctx.nivel));
    }
    reclassificar(&mut itens);
    let mut saida = Caixa::vazia();
    let mut anterior: Option<Classe> = None;
    let mut n_atomos = 0;
    for item in &itens {
        match item {
            Item::Kern(w) => saida.seguir(&Caixa::espaco(*w)),
            Item::Atomo(c, cx, nivel) => {
                if let Some(a) = anterior {
                    let mu = espaco_entre(a, *c, *nivel);
                    if mu > 0.0 {
                        let t = ctx0.com(*nivel, false).tam();
                        saida.seguir(&Caixa::espaco(mu * t / 18.0));
                    }
                }
                saida.seguir(cx);
                anterior = Some(*c);
                n_atomos += 1;
            }
        }
    }
    if n_atomos == 1 && itens.len() == 1 {
        if let Some(Item::Atomo(_, cx, _)) = itens.first() {
            saida.acento = cx.acento;
            saida.simples = cx.simples;
            saida.op_grande = cx.op_grande;
            saida.italico = cx.italico;
        }
    }
    saida
}

// ───────────────────────────── átomos ─────────────────────────────

fn limites_valem(no: &No, ctx: &Ctx) -> bool {
    no.classe == Classe::Op && no.limites.unwrap_or(ctx.display()) || (no.limites == Some(true))
}

/// Um átomo: o núcleo e os scripts (ao lado ou, nos operadores com limites, em cima e embaixo).
pub fn atomo(no: &No, ctx: &Ctx) -> Caixa {
    if let Nucleo::Chave { c, corpo, cima } = &no.nucleo {
        if corpo.is_empty() {
            return seta_com_rotulos(*c, no, ctx, *cima);
        }
    }
    let mut nucleo = nucleo(no, ctx);
    if no.sup.is_none() && no.sub.is_none() {
        return nucleo;
    }
    // `^\prime`: o primo da fonte já é de expoente (no alto); vai no corpo da base, sem subir.
    if let Some(sup) = no.sup.as_deref().filter(|s| !s.is_empty() && s.iter().all(|n| matches!(n.nucleo, Nucleo::Car('′' | '″' | '‴' | '⁗', _)))) {
        nucleo.seguir(&hlista(sup, ctx));
        nucleo.simples = false;
        return match no.sub.as_deref() {
            Some(sub) => scripts(nucleo, None, Some(sub), ctx),
            None => nucleo,
        };
    }
    let sup = no.sup.as_deref();
    let sub = no.sub.as_deref();
    if limites_valem(no, ctx) {
        limites(nucleo, sup, sub, ctx)
    } else {
        scripts(nucleo, sup, sub, ctx)
    }
}

fn nucleo(no: &No, ctx: &Ctx) -> Caixa {
    match &no.nucleo {
        Nucleo::Car(c, alf) => {
            let alvo = estilizar(*c, *alf);
            let Some(g) = fonte().glifo(alvo).or_else(|| fonte().glifo(*c)) else { return tofu(ctx) };
            if no.classe == Classe::Op && !fonte().variantes(g, true).is_empty() {
                operador_grande(g, ctx)
            } else {
                glifo(g, ctx)
            }
        }
        Nucleo::Texto(s, alf) => texto(s, *alf, ctx),
        Nucleo::Grupo(l) => hlista(l, ctx),
        Nucleo::Vazio => Caixa { simples: true, ..Caixa::vazia() },
        Nucleo::Fracao { num, den, barra, nivel, esq, dir } => fracao(num, den, *barra, *nivel, (*esq, *dir), ctx),
        Nucleo::Raiz { indice, corpo } => raiz(indice.as_deref(), corpo, ctx),
        Nucleo::Delimitado { esq, dir, partes, meios } => delimitado(*esq, *dir, partes, meios, ctx),
        Nucleo::Grande { c, altura } => grande(*c, *altura, ctx),
        Nucleo::Acento { c, corpo, largo, baixo } => acento(*c, corpo, *largo, *baixo, ctx),
        Nucleo::Barra { corpo, cima } => barra(corpo, *cima, ctx),
        Nucleo::Chave { c, corpo, cima } => chave(*c, corpo, *cima, ctx),
        Nucleo::Matriz(m) => crate::matriz::matriz(m, ctx).0,
        Nucleo::Cor(cor, l) => {
            let mut c2 = *ctx;
            c2.cor = *cor;
            let mut cx = hlista(l, &c2);
            cx.colorir(*cor);
            cx
        }
        Nucleo::Fantasma { corpo, largura, altura } => {
            let cx = hlista(corpo, ctx);
            Caixa {
                w: if *largura { cx.w } else { 0.0 },
                h: if *altura { cx.h } else { 0.0 },
                d: if *altura { cx.d } else { 0.0 },
                ..Caixa::vazia()
            }
        }
        Nucleo::Achatado(l) => {
            let mut cx = hlista(l, ctx);
            cx.h = 0.0;
            cx.d = 0.0;
            cx
        }
        Nucleo::Caixa(l) => emoldurar(hlista(l, ctx), ctx),
        Nucleo::Negado(n) => negado(n, ctx),
        Nucleo::Lap(l, alinh) => {
            let cx = hlista(l, ctx);
            let dx = match alinh {
                crate::arvore::Alinh::Esq => 0.0,
                crate::arvore::Alinh::Centro => -cx.w / 2.0,
                crate::arvore::Alinh::Dir => -cx.w,
            };
            let mut sem = Caixa { h: cx.h, d: cx.d, ..Caixa::vazia() };
            sem.anexar(&cx, dx, 0.0);
            sem
        }
        Nucleo::Riscado { corpo, tipo, valor } => riscado(corpo, *tipo, valor.as_deref(), ctx),
        Nucleo::Regra { largura, altura, sobe } => {
            let (w, h, y0) = (largura * ctx.base, altura * ctx.base, sobe * ctx.base);
            let mut cx = Caixa { w, h: (h + y0).max(0.0), d: (-y0).max(0.0), ..Caixa::vazia() };
            cx.retangulo(0.0, -(h + y0), w, h, ctx.cor);
            cx
        }
        _ => Caixa::vazia(),
    }
}

/// `\sum`, `\int`…: no display, a primeira variante com altura ≥ `DisplayOperatorMinHeight`;
/// centrado no eixo.
fn operador_grande(g: u16, ctx: &Ctx) -> Caixa {
    let f = fonte();
    let mut escolhido = g;
    if ctx.display() {
        let minimo = f.c.display_operator_min_height;
        let vs = f.variantes(g, true);
        escolhido = vs.iter().find(|(_, m)| *m >= minimo).or(vs.last()).map_or(g, |v| v.0);
        if escolhido == g && vs.len() > 1 {
            escolhido = vs[1].0;
        }
    }
    let mut cx = glifo(escolhido, ctx);
    let eixo = ctx.pt(f.c.axis_height);
    let desce = (cx.h - cx.d) / 2.0 - eixo;
    cx.deslocar_vertical(desce);
    cx.op_grande = true;
    cx.simples = true;
    cx
}

/// Expoente e índice ao lado da base (regra 18 do apêndice G, com as constantes da tabela MATH).
fn scripts(base: Caixa, sup: Option<&[No]>, sub: Option<&[No]>, ctx: &Ctx) -> Caixa {
    let c = &fonte().c;
    let p = |v: f32| ctx.pt(v);
    let sup_cx = sup.map(|l| hlista(l, &ctx.sup()));
    let sub_cx = sub.map(|l| hlista(l, &ctx.sub()));
    // O operador grande já virou caixa centrada no eixo: vale a "queda" (regra 18a).
    let composto = !base.simples || base.op_grande;
    let mut u = 0.0f32;
    let mut v = 0.0f32;
    if let Some(s) = &sup_cx {
        let sobe = if ctx.apertado { c.superscript_shift_up_cramped } else { c.superscript_shift_up };
        u = p(sobe).max(s.d + p(c.superscript_bottom_min));
        if composto {
            u = u.max(base.h - p(c.superscript_baseline_drop_max));
        }
    }
    if let Some(s) = &sub_cx {
        v = p(c.subscript_shift_down);
        if composto {
            v = v.max(base.d + p(c.subscript_baseline_drop_min));
        }
        if sup_cx.is_none() {
            v = v.max(s.h - p(c.subscript_top_max));
        }
    }
    if let (Some(s), Some(i)) = (&sup_cx, &sub_cx) {
        let vao = (u - s.d) - (i.h - v);
        if vao < p(c.sub_superscript_gap_min) {
            v += p(c.sub_superscript_gap_min) - vao;
            let falta = p(c.superscript_bottom_max_with_subscript) - (u - s.d);
            if falta > 0.0 {
                u += falta;
                v -= falta;
            }
        }
    }
    let mut cx = base.clone();
    cx.acento = None;
    cx.simples = false;
    let mut fim = base.w;
    if let Some(s) = &sup_cx {
        // OpenType: no operador grande o avanço já cobre o desenho e só o índice recua a correção itálica.
        let x = if base.op_grande { base.w } else { base.w + base.italico };
        cx.anexar(s, x, -u);
        cx.h = cx.h.max(u + s.h);
        cx.d = cx.d.max(s.d - u);
        fim = fim.max(x + s.w);
    }
    if let Some(i) = &sub_cx {
        let x = if base.op_grande { base.w - base.italico } else { base.w };
        cx.anexar(i, x, v);
        cx.d = cx.d.max(v + i.d);
        cx.h = cx.h.max(i.h - v);
        fim = fim.max(x + i.w);
    }
    cx.w = fim + p(c.space_after_script);
    cx.italico = 0.0;
    cx
}

/// Limites em cima e embaixo (∑ no display, `\lim`, `\overbrace`).
fn limites(op: Caixa, sup: Option<&[No]>, sub: Option<&[No]>, ctx: &Ctx) -> Caixa {
    let c = &fonte().c;
    let p = |v: f32| ctx.pt(v);
    let sup_cx = sup.map(|l| hlista(l, &ctx.sup()));
    let sub_cx = sub.map(|l| hlista(l, &ctx.sub()));
    let delta = op.italico;
    let w = op.w.max(sup_cx.as_ref().map_or(0.0, |s| s.w)).max(sub_cx.as_ref().map_or(0.0, |s| s.w));
    let mut cx = Caixa { w, h: op.h, d: op.d, ..Caixa::vazia() };
    cx.anexar(&op, (w - op.w) / 2.0, 0.0);
    if let Some(s) = &sup_cx {
        let u = p(c.upper_limit_baseline_rise_min).max(p(c.upper_limit_gap_min) + s.d);
        cx.anexar(s, (w - s.w) / 2.0 + delta / 2.0, -(op.h + u));
        cx.h = op.h + u + s.h;
    }
    if let Some(s) = &sub_cx {
        let v = p(c.lower_limit_baseline_drop_min).max(p(c.lower_limit_gap_min) + s.h);
        cx.anexar(s, (w - s.w) / 2.0 - delta / 2.0, op.d + v);
        cx.d = op.d + v + s.d;
    }
    cx
}

// ───────────────────────────── frações e raízes ─────────────────────────────

fn fracao(num: &[No], den: &[No], barra: bool, nivel: Option<Nivel>, (esq, dir): (Option<char>, Option<char>), ctx0: &Ctx) -> Caixa {
    let ctx = match nivel {
        Some(n) => Ctx { nivel: n, ..*ctx0 },
        None => *ctx0,
    };
    let c = &fonte().c;
    let p = |v: f32| ctx.pt(v);
    let n = hlista(num, &ctx.num());
    let d = hlista(den, &ctx.den());
    let disp = ctx.display();
    let eixo = p(c.axis_height);
    let (u, v, espessura) = if barra {
        let th = p(c.fraction_rule_thickness);
        let (sobe, desce, vn, vd) = if disp {
            (c.fraction_numerator_display_style_shift_up, c.fraction_denominator_display_style_shift_down, c.fraction_num_display_style_gap_min, c.fraction_denom_display_style_gap_min)
        } else {
            (c.fraction_numerator_shift_up, c.fraction_denominator_shift_down, c.fraction_numerator_gap_min, c.fraction_denominator_gap_min)
        };
        let u = p(sobe).max(eixo + th / 2.0 + p(vn) + n.d);
        let v = p(desce).max(d.h + p(vd) - (eixo - th / 2.0));
        (u, v, th)
    } else {
        let (sobe, desce, vao) = if disp {
            (c.stack_top_display_style_shift_up, c.stack_bottom_display_style_shift_down, c.stack_display_style_gap_min)
        } else {
            (c.stack_top_shift_up, c.stack_bottom_shift_down, c.stack_gap_min)
        };
        let (mut u, mut v) = (p(sobe), p(desce));
        let real = (u - n.d) - (d.h - v);
        if real < p(vao) {
            let meio = (p(vao) - real) / 2.0;
            u += meio;
            v += meio;
        }
        (u, v, 0.0)
    };
    let w = n.w.max(d.w);
    let folga = if esq.is_none() && dir.is_none() { ctx.base * NULLDELIMITERSPACE } else { 0.0 };
    let mut cx = Caixa { w: w + 2.0 * folga, h: u + n.h, d: v + d.d, ..Caixa::vazia() };
    cx.anexar(&n, folga + (w - n.w) / 2.0, -u);
    cx.anexar(&d, folga + (w - d.w) / 2.0, v);
    if barra {
        cx.retangulo(folga, -(eixo + espessura / 2.0), w, espessura, ctx.cor);
    }
    if esq.is_some() || dir.is_some() {
        // Regra 15e do apêndice G: delimitador de tamanho `delim1` (2,39 em) no display e `delim2`
        // (1,01 em) fora dele — os parâmetros σ20/σ21 da cmsy10, que a tabela MATH não tem.
        let alvo = ctx.base * if disp { 2.39 } else { 1.01 } * if ctx.nivel > Nivel::T { fonte().c.script_percent } else { 1.0 };
        let mut com = Caixa::vazia();
        com.seguir(&delimitador_no_eixo(esq, alvo, &ctx));
        com.seguir(&cx);
        com.seguir(&delimitador_no_eixo(dir, alvo, &ctx));
        return com;
    }
    cx
}

fn raiz(indice: Option<&[No]>, corpo: &[No], ctx: &Ctx) -> Caixa {
    let c = &fonte().c;
    let p = |v: f32| ctx.pt(v);
    let b = hlista(corpo, &ctx.apertar());
    let th = p(c.radical_rule_thickness);
    let mut vao = p(if ctx.display() { c.radical_display_style_vertical_gap } else { c.radical_vertical_gap });
    let alvo = b.h + b.d + vao + th;
    let mut r = esticar_vertical('√', alvo, ctx);
    let total = r.h + r.d;
    if total > alvo {
        vao += (total - alvo) / 2.0;
    }
    // O topo do radical fica no topo do traço.
    let topo = b.h + vao + th;
    let desce = r.h - topo;
    r.deslocar_vertical(desce);
    let mut cx = Caixa::vazia();
    if let Some(ind) = indice {
        let i = hlista(ind, &ctx.com(Nivel::SS, true));
        let antes = p(c.radical_kern_before_degree);
        let depois = p(c.radical_kern_after_degree);
        let sobe = c.radical_degree_bottom_raise_percent * (r.h + r.d);
        let y = r.d - sobe;
        cx.anexar(&i, antes, y);
        cx.w = (antes + i.w + depois).max(0.0);
        cx.h = cx.h.max(i.h - y);
    }
    let x = cx.w;
    cx.anexar(&r, x, 0.0);
    let xr = x + r.w;
    cx.retangulo(xr, -topo, b.w, th, ctx.cor);
    cx.anexar(&b, xr, 0.0);
    cx.w = xr + b.w;
    cx.h = cx.h.max(topo + p(c.radical_extra_ascender));
    cx.d = cx.d.max(b.d).max(r.d);
    cx
}

// ───────────────────────────── delimitadores ─────────────────────────────

/// A altura total que um delimitador precisa para cobrir `h`/`d` (regra 19 do apêndice G).
pub fn altura_do_delimitador(h: f32, d: f32, ctx: &Ctx) -> f32 {
    let eixo = ctx.pt(fonte().c.axis_height);
    let delta = (h - eixo).max(d + eixo);
    (2.0 * delta * DELIMITERFACTOR).max(2.0 * delta - ctx.base * DELIMITERSHORTFALL)
}

/// O delimitador na altura total `alvo`, centrado no eixo; `None` = o espaço nulo (`\left.`).
pub fn delimitador_no_eixo(c: Option<char>, alvo: f32, ctx: &Ctx) -> Caixa {
    let Some(c) = c else { return Caixa::espaco(ctx.base * NULLDELIMITERSPACE) };
    let mut cx = esticar_vertical(c, alvo, ctx);
    let eixo = ctx.pt(fonte().c.axis_height);
    let desce = (cx.h - cx.d) / 2.0 - eixo;
    cx.deslocar_vertical(desce);
    cx.simples = false;
    cx.acento = None;
    cx
}

/// O glifo `c` com altura total (h + d) de pelo menos `alvo`: a primeira variante que cabe, ou
/// a montagem com as peças da tabela MATH.
pub fn esticar_vertical(c: char, alvo: f32, ctx: &Ctx) -> Caixa {
    esticar(c, alvo, ctx, true)
}

pub fn esticar_horizontal(c: char, alvo: f32, ctx: &Ctx) -> Caixa {
    esticar(c, alvo, ctx, false)
}

fn esticar(c: char, alvo: f32, ctx: &Ctx, vertical: bool) -> Caixa {
    let f = fonte();
    let g = id_do_caractere(c);
    let t = ctx.tam();
    let medida = |cx: &Caixa| if vertical { cx.h + cx.d } else { cx.w };
    let base = glifo(g, ctx);
    if medida(&base) >= alvo {
        return base;
    }
    let vs = f.variantes(g, vertical);
    for (v, m) in &vs {
        if m * t >= alvo {
            return glifo(*v, ctx);
        }
    }
    if let Some((pecas, ital)) = f.montagem(g, vertical) {
        if let Some(cx) = montar(&pecas, alvo, ctx, vertical, ital * t) {
            return cx;
        }
    }
    vs.last().map_or(base, |(v, _)| glifo(*v, ctx))
}

/// Monta as peças: repete os extensores até passar do alvo e distribui a sobreposição.
fn montar(pecas: &[Peca], alvo: f32, ctx: &Ctx, vertical: bool, italico: f32) -> Option<Caixa> {
    let t = ctx.tam();
    let minimo = fonte().c.min_connector_overlap * t;
    if pecas.is_empty() {
        return None;
    }
    let mut escolhidas: Vec<Peca> = Vec::new();
    for repeticoes in 0..200 {
        escolhidas = pecas.iter().flat_map(|p| std::iter::repeat(*p).take(if p.extensor { repeticoes } else { 1 })).collect();
        if escolhidas.is_empty() {
            continue;
        }
        let soma: f32 = escolhidas.iter().map(|p| p.avanco * t).sum();
        let maximo = soma - minimo * (escolhidas.len() as f32 - 1.0);
        if maximo >= alvo {
            break;
        }
    }
    let n = escolhidas.len();
    let soma: f32 = escolhidas.iter().map(|p| p.avanco * t).sum();
    let mut sobre = if n > 1 { ((soma - alvo) / (n as f32 - 1.0)).max(minimo) } else { 0.0 };
    // A sobreposição não passa do conector mais curto.
    let teto = escolhidas.windows(2).map(|w| w[0].fim.min(w[1].inicio) * t).fold(f32::INFINITY, f32::min);
    if teto.is_finite() {
        sobre = sobre.min(teto.max(minimo));
    }
    let mut cx = Caixa::vazia();
    let mut pos = 0.0f32;
    let mut alto = 0.0f32;
    for (k, p) in escolhidas.iter().enumerate() {
        let gl = glifo(p.glifo, ctx);
        if vertical {
            // `pos` = do pé da montagem até o pé (desenho) desta peça.
            let y = -(pos + gl.d);
            cx.anexar(&gl, 0.0, y);
            cx.w = cx.w.max(gl.w);
            alto = alto.max(pos + gl.d + gl.h);
        } else {
            cx.anexar(&gl, pos, 0.0);
            cx.h = cx.h.max(gl.h);
            cx.d = cx.d.max(gl.d);
        }
        pos += p.avanco * t;
        if k + 1 < n {
            pos -= sobre;
        }
    }
    if vertical {
        cx.h = alto.max(pos);
        cx.d = 0.0;
    } else {
        cx.w = pos;
    }
    cx.italico = italico;
    Some(cx)
}

fn delimitado(esq: Option<char>, dir: Option<char>, partes: &[Vec<No>], meios: &[Option<char>], ctx: &Ctx) -> Caixa {
    let caixas: Vec<Caixa> = partes.iter().map(|l| hlista(l, ctx)).collect();
    let h = caixas.iter().map(|c| c.h).fold(0.0, f32::max);
    let d = caixas.iter().map(|c| c.d).fold(0.0, f32::max);
    let alvo = altura_do_delimitador(h, d, ctx);
    let mut cx = Caixa::vazia();
    cx.seguir(&delimitador_no_eixo(esq, alvo, ctx));
    for (k, parte) in caixas.iter().enumerate() {
        if k > 0 {
            let m = delimitador_no_eixo(meios.get(k - 1).copied().flatten(), alvo, ctx);
            // `\middle` se espaça como relação dos dois lados.
            let fino = ctx.tam() * 4.0 / 18.0;
            cx.seguir(&Caixa::espaco(fino));
            cx.seguir(&m);
            cx.seguir(&Caixa::espaco(fino));
        }
        cx.seguir(parte);
    }
    cx.seguir(&delimitador_no_eixo(dir, alvo, ctx));
    cx.simples = false;
    cx
}

/// `\big(` e companhia: altura total fixa (1,2 / 1,8 / 2,4 / 3,0 em, como o `\big` do plain TeX).
fn grande(c: Option<char>, altura: f32, ctx: &Ctx) -> Caixa {
    let alvo = altura * ctx.base * if ctx.nivel > Nivel::T { fonte().c.script_percent } else { 1.0 };
    delimitador_no_eixo(c, alvo * DELIMITERFACTOR, ctx)
}

// ───────────────────────────── acentos e barras ─────────────────────────────

fn acento(c: char, corpo: &[No], largo: bool, baixo: bool, ctx: &Ctx) -> Caixa {
    let f = fonte();
    let cf = &f.c;
    let b = hlista(corpo, &ctx.apertar());
    let mut a = if largo { esticar_horizontal(c, b.w, ctx) } else { caractere(c, ctx) };
    // Ponto de encaixe do acento (o da tabela, ou o meio do desenho, já que o combinante tem avanço 0).
    let g_acento = a.glifos.first().map(|g| g.id);
    let encaixe_a = match (a.glifos.len(), g_acento) {
        (1, Some(g)) => f.encaixe_do_acento(g).map(|v| v * ctx.tam()).unwrap_or_else(|| {
            let (x0, x1) = f.extensao_x(g);
            (x0 + x1) / 2.0 * ctx.tam()
        }),
        _ => a.w / 2.0,
    };
    let encaixe_b = if b.simples { b.acento.unwrap_or(b.w / 2.0 + b.italico / 2.0) } else { b.w / 2.0 };
    let x = encaixe_b - encaixe_a;
    let mut cx = Caixa { w: b.w, h: b.h, d: b.d, italico: b.italico, simples: b.simples, ..Caixa::vazia() };
    cx.anexar(&b, 0.0, 0.0);
    if baixo {
        let y = b.d + ctx.pt(cf.underbar_vertical_gap) + a.h;
        if !combinante(c) {
            let w = b.w.max(a.w);
            let mut nova = Caixa { w, h: b.h, d: y + a.d, ..Caixa::vazia() };
            nova.anexar(&b, (w - b.w) / 2.0, 0.0);
            nova.anexar(&a, (w - a.w) / 2.0, y);
            return nova;
        }
        cx.anexar(&a, x, y);
        cx.d = y + a.d;
    } else if combinante(c) {
        // O acento da fonte foi desenhado para letra da altura `AccentBaseHeight`: sobe o que passar.
        let sobe = b.h - b.h.min(ctx.pt(cf.accent_base_height));
        a.deslocar_vertical(-sobe);
        cx.anexar(&a, x, 0.0);
        cx.h = cx.h.max(a.h);
    } else {
        // Seta esticada (`\overrightarrow`): por cima da base, com o vão de uma pilha esticada; se ela
        // for mais larga que a base, a caixa cresce e a base fica no meio (a seta não invade o vizinho).
        let y = -(b.h + ctx.pt(cf.stretch_stack_gap_below_min) + a.d);
        let w = b.w.max(a.w);
        let mut nova = Caixa { w, h: -y + a.h, d: b.d, italico: b.italico, ..Caixa::vazia() };
        nova.anexar(&b, (w - b.w) / 2.0, 0.0);
        nova.anexar(&a, (w - a.w) / 2.0, y);
        return nova;
    }
    cx.acento = None;
    cx
}

/// Caractere combinante (os acentos de verdade); as setas sobre a base não são.
fn combinante(c: char) -> bool {
    matches!(c as u32, 0x300..=0x36F | 0x20D0..=0x20FF)
}

fn barra(corpo: &[No], cima: bool, ctx: &Ctx) -> Caixa {
    let c = &fonte().c;
    let p = |v: f32| ctx.pt(v);
    let b = hlista(corpo, &if cima { ctx.apertar() } else { *ctx });
    let mut cx = Caixa { w: b.w, h: b.h, d: b.d, ..Caixa::vazia() };
    cx.anexar(&b, 0.0, 0.0);
    if cima {
        let th = p(c.overbar_rule_thickness);
        let topo = b.h + p(c.overbar_vertical_gap) + th;
        cx.retangulo(0.0, -topo, b.w, th, ctx.cor);
        cx.h = topo + p(c.overbar_extra_ascender);
    } else {
        let th = p(c.underbar_rule_thickness);
        let y = b.d + p(c.underbar_vertical_gap);
        cx.retangulo(0.0, y, b.w, th, ctx.cor);
        cx.d = y + th + p(c.underbar_extra_descender);
    }
    cx
}

/// `\overbrace`/`\underbrace`: a chave esticada até a largura da base.
fn chave(c: char, corpo: &[No], cima: bool, ctx: &Ctx) -> Caixa {
    let cf = &fonte().c;
    let b = hlista(corpo, ctx);
    let ch = esticar_horizontal(c, b.w, ctx);
    let mut cx = Caixa { w: b.w.max(ch.w), h: b.h, d: b.d, ..Caixa::vazia() };
    let xb = (cx.w - b.w) / 2.0;
    let xc = (cx.w - ch.w) / 2.0;
    cx.anexar(&b, xb, 0.0);
    if cima {
        let y = -(b.h + ctx.pt(cf.stretch_stack_gap_below_min) + ch.d);
        cx.anexar(&ch, xc, y);
        cx.h = -y + ch.h;
    } else {
        let y = b.d + ctx.pt(cf.stretch_stack_gap_above_min) + ch.h;
        cx.anexar(&ch, xc, y);
        cx.d = y + ch.d;
    }
    cx
}

/// `\xrightarrow[embaixo]{em cima}`: a seta cresce até cobrir o rótulo.
fn seta_com_rotulos(c: char, no: &No, ctx: &Ctx, _cima: bool) -> Caixa {
    let sup = no.sup.as_deref().map(|l| hlista(l, &ctx.sup()));
    let sub = no.sub.as_deref().map(|l| hlista(l, &ctx.sub()));
    let rotulo = sup.as_ref().map_or(0.0, |s| s.w).max(sub.as_ref().map_or(0.0, |s| s.w));
    // O `\ext@arrow` do amsmath: rótulo + 0,5 em de cada lado, e no mínimo 1,8 em.
    let alvo = (rotulo + ctx.base).max(1.8 * ctx.base);
    let mut seta = if c == '=' { igual_comprido(alvo, ctx) } else { esticar_horizontal(c, alvo, ctx) };
    seta.simples = false;
    seta.italico = 0.0;
    limites(seta, no.sup.as_deref(), no.sub.as_deref(), ctx)
}

/// O `=` comprido do `\xlongequal`: os dois traços do `=` da fonte, esticados até `w`.
fn igual_comprido(w: f32, ctx: &Ctx) -> Caixa {
    let igual = caractere('=', ctx);
    let th = ctx.pt(fonte().c.fraction_rule_thickness);
    let mut cx = Caixa { w, h: igual.h, d: igual.d, ..Caixa::vazia() };
    cx.retangulo(0.0, -igual.h, w, th, ctx.cor);
    cx.retangulo(0.0, igual.d - th, w, th, ctx.cor);
    cx
}

/// `\cancel` e companhia: o risco em diagonal por cima da base (`\cancelto` com seta e o valor no alto).
fn riscado(corpo: &[No], tipo: u8, valor: Option<&[No]>, ctx: &Ctx) -> Caixa {
    let b = hlista(corpo, ctx);
    let mut cx = b.clone();
    let folga = ctx.tam() * 0.1;
    let (x0, x1, topo, pe) = (-folga / 2.0, b.w + folga / 2.0, -b.h - folga, b.d + folga);
    let th = ctx.pt(fonte().c.fraction_rule_thickness);
    let traco = |x1: f32, y1: f32, x2: f32, y2: f32| crate::Traco { x1, y1, x2, y2, espessura: th, cor: ctx.cor };
    if tipo == 1 || tipo == 3 || tipo == 4 {
        cx.tracos.push(traco(x0, pe, x1, topo));
    }
    if tipo == 2 || tipo == 3 {
        cx.tracos.push(traco(x0, topo, x1, pe));
    }
    if let (4, Some(v)) = (tipo, valor) {
        // A ponta da seta e o valor no fim do risco.
        let ponta = th * 3.0;
        cx.tracos.push(traco(x1, topo, x1 - ponta, topo + ponta * 0.4));
        cx.tracos.push(traco(x1, topo, x1 - ponta * 0.4, topo + ponta));
        let vcx = hlista(v, &ctx.sup());
        cx.anexar(&vcx, x1, topo - vcx.d);
        cx.h = cx.h.max(-topo + vcx.d + vcx.h);
        cx.w = cx.w.max(x1 + vcx.w);
    }
    cx.h = cx.h.max(-topo);
    cx.d = cx.d.max(pe);
    cx
}

fn emoldurar(b: Caixa, ctx: &Ctx) -> Caixa {
    let sep = ctx.base * FBOXSEP;
    let r = ctx.base * FBOXRULE;
    let w = b.w + 2.0 * (sep + r);
    let (h, d) = (b.h + sep + r, b.d + sep + r);
    let mut cx = Caixa { w, h, d, ..Caixa::vazia() };
    cx.anexar(&b, sep + r, 0.0);
    cx.retangulo(0.0, -h, w, r, ctx.cor);
    cx.retangulo(0.0, d - r, w, r, ctx.cor);
    cx.retangulo(0.0, -h, r, h + d, ctx.cor);
    cx.retangulo(w - r, -h, r, h + d, ctx.cor);
    cx
}

fn negado(no: &No, ctx: &Ctx) -> Caixa {
    let b = atomo(no, ctx);
    let barra = caractere('\u{338}', ctx);
    let (x0, x1) = fonte().extensao_x(barra.glifos.first().map_or(0, |g| g.id));
    let meio = (x0 + x1) / 2.0 * ctx.tam();
    let mut cx = b.clone();
    cx.anexar(&barra, b.w / 2.0 - meio, 0.0);
    cx.h = cx.h.max(barra.h);
    cx.d = cx.d.max(barra.d);
    cx
}
