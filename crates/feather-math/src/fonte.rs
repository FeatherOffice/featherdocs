//! A fonte matemática embutida (Latin Modern Math) e o que se lê dela: o glifo de
//! cada caractere, as métricas, o contorno e a tabela OpenType `MATH` — constantes,
//! correção itálica, ponto de encaixe do acento, variantes de tamanho e montagem.
//!
//! Tudo que sai daqui está em **em** (unidade da fonte ÷ unidades por em): quem
//! compõe multiplica pelo corpo do estilo atual.

use std::sync::OnceLock;
use ttf_parser::{math, Face, GlyphId};

static DADOS: &[u8] = include_bytes!("../assets/latinmodern-math.otf");

/// As constantes da tabela `MATH`, já em em. Os nomes seguem a especificação OpenType.
#[derive(Clone, Copy, Debug, Default)]
pub struct Constantes {
    pub script_percent: f32,
    pub script_script_percent: f32,
    pub display_operator_min_height: f32,
    pub axis_height: f32,
    pub accent_base_height: f32,
    pub subscript_shift_down: f32,
    pub subscript_top_max: f32,
    pub subscript_baseline_drop_min: f32,
    pub superscript_shift_up: f32,
    pub superscript_shift_up_cramped: f32,
    pub superscript_bottom_min: f32,
    pub superscript_baseline_drop_max: f32,
    pub sub_superscript_gap_min: f32,
    pub superscript_bottom_max_with_subscript: f32,
    pub space_after_script: f32,
    pub upper_limit_gap_min: f32,
    pub upper_limit_baseline_rise_min: f32,
    pub lower_limit_gap_min: f32,
    pub lower_limit_baseline_drop_min: f32,
    pub stack_top_shift_up: f32,
    pub stack_top_display_style_shift_up: f32,
    pub stack_bottom_shift_down: f32,
    pub stack_bottom_display_style_shift_down: f32,
    pub stack_gap_min: f32,
    pub stack_display_style_gap_min: f32,
    pub stretch_stack_gap_above_min: f32,
    pub stretch_stack_gap_below_min: f32,
    pub fraction_numerator_shift_up: f32,
    pub fraction_numerator_display_style_shift_up: f32,
    pub fraction_denominator_shift_down: f32,
    pub fraction_denominator_display_style_shift_down: f32,
    pub fraction_numerator_gap_min: f32,
    pub fraction_num_display_style_gap_min: f32,
    pub fraction_rule_thickness: f32,
    pub fraction_denominator_gap_min: f32,
    pub fraction_denom_display_style_gap_min: f32,
    pub overbar_vertical_gap: f32,
    pub overbar_rule_thickness: f32,
    pub overbar_extra_ascender: f32,
    pub underbar_vertical_gap: f32,
    pub underbar_rule_thickness: f32,
    pub underbar_extra_descender: f32,
    pub radical_vertical_gap: f32,
    pub radical_display_style_vertical_gap: f32,
    pub radical_rule_thickness: f32,
    pub radical_extra_ascender: f32,
    pub radical_kern_before_degree: f32,
    pub radical_kern_after_degree: f32,
    pub radical_degree_bottom_raise_percent: f32,
    /// Sobreposição mínima entre as peças de uma montagem.
    pub min_connector_overlap: f32,
}

/// Uma peça de montagem (delimitador que cresce): glifo, conectores e se se repete.
#[derive(Clone, Copy, Debug)]
pub struct Peca {
    pub glifo: u16,
    pub inicio: f32,
    pub fim: f32,
    pub avanco: f32,
    pub extensor: bool,
}

pub struct Fonte {
    face: Face<'static>,
    upem: f32,
    pub c: Constantes,
}

/// A fonte, lida uma vez só.
pub fn fonte() -> &'static Fonte {
    static F: OnceLock<Fonte> = OnceLock::new();
    F.get_or_init(|| {
        let face = Face::parse(DADOS, 0).expect("a Latin Modern Math embutida é uma fonte válida");
        let upem = f32::from(face.units_per_em());
        let c = constantes(&face, upem);
        Fonte { face, upem, c }
    })
}

fn constantes(face: &Face, upem: f32) -> Constantes {
    let Some(m) = face.tables().math.and_then(|m| m.constants) else { return Constantes::default() };
    let v = |x: math::MathValue| f32::from(x.value) / upem;
    let overlap = face.tables().math.and_then(|m| m.variants).map_or(0.0, |v| f32::from(v.min_connector_overlap) / upem);
    Constantes {
        script_percent: f32::from(m.script_percent_scale_down()) / 100.0,
        script_script_percent: f32::from(m.script_script_percent_scale_down()) / 100.0,
        display_operator_min_height: f32::from(m.display_operator_min_height()) / upem,
        axis_height: v(m.axis_height()),
        accent_base_height: v(m.accent_base_height()),
        subscript_shift_down: v(m.subscript_shift_down()),
        subscript_top_max: v(m.subscript_top_max()),
        subscript_baseline_drop_min: v(m.subscript_baseline_drop_min()),
        superscript_shift_up: v(m.superscript_shift_up()),
        superscript_shift_up_cramped: v(m.superscript_shift_up_cramped()),
        superscript_bottom_min: v(m.superscript_bottom_min()),
        superscript_baseline_drop_max: v(m.superscript_baseline_drop_max()),
        sub_superscript_gap_min: v(m.sub_superscript_gap_min()),
        superscript_bottom_max_with_subscript: v(m.superscript_bottom_max_with_subscript()),
        space_after_script: v(m.space_after_script()),
        upper_limit_gap_min: v(m.upper_limit_gap_min()),
        upper_limit_baseline_rise_min: v(m.upper_limit_baseline_rise_min()),
        lower_limit_gap_min: v(m.lower_limit_gap_min()),
        lower_limit_baseline_drop_min: v(m.lower_limit_baseline_drop_min()),
        stack_top_shift_up: v(m.stack_top_shift_up()),
        stack_top_display_style_shift_up: v(m.stack_top_display_style_shift_up()),
        stack_bottom_shift_down: v(m.stack_bottom_shift_down()),
        stack_bottom_display_style_shift_down: v(m.stack_bottom_display_style_shift_down()),
        stack_gap_min: v(m.stack_gap_min()),
        stack_display_style_gap_min: v(m.stack_display_style_gap_min()),
        stretch_stack_gap_above_min: v(m.stretch_stack_gap_above_min()),
        stretch_stack_gap_below_min: v(m.stretch_stack_gap_below_min()),
        fraction_numerator_shift_up: v(m.fraction_numerator_shift_up()),
        fraction_numerator_display_style_shift_up: v(m.fraction_numerator_display_style_shift_up()),
        fraction_denominator_shift_down: v(m.fraction_denominator_shift_down()),
        fraction_denominator_display_style_shift_down: v(m.fraction_denominator_display_style_shift_down()),
        fraction_numerator_gap_min: v(m.fraction_numerator_gap_min()),
        fraction_num_display_style_gap_min: v(m.fraction_num_display_style_gap_min()),
        fraction_rule_thickness: v(m.fraction_rule_thickness()),
        fraction_denominator_gap_min: v(m.fraction_denominator_gap_min()),
        fraction_denom_display_style_gap_min: v(m.fraction_denom_display_style_gap_min()),
        overbar_vertical_gap: v(m.overbar_vertical_gap()),
        overbar_rule_thickness: v(m.overbar_rule_thickness()),
        overbar_extra_ascender: v(m.overbar_extra_ascender()),
        underbar_vertical_gap: v(m.underbar_vertical_gap()),
        underbar_rule_thickness: v(m.underbar_rule_thickness()),
        underbar_extra_descender: v(m.underbar_extra_descender()),
        radical_vertical_gap: v(m.radical_vertical_gap()),
        radical_display_style_vertical_gap: v(m.radical_display_style_vertical_gap()),
        radical_rule_thickness: v(m.radical_rule_thickness()),
        radical_extra_ascender: v(m.radical_extra_ascender()),
        radical_kern_before_degree: v(m.radical_kern_before_degree()),
        radical_kern_after_degree: v(m.radical_kern_after_degree()),
        radical_degree_bottom_raise_percent: f32::from(m.radical_degree_bottom_raise_percent()) / 100.0,
        min_connector_overlap: overlap,
    }
}

/// Quem recebe o contorno de um glifo (coordenadas já em pontos, `y` para baixo).
pub trait Contorno {
    fn mover(&mut self, x: f32, y: f32);
    fn linha(&mut self, x: f32, y: f32);
    fn quadratica(&mut self, x1: f32, y1: f32, x: f32, y: f32);
    fn cubica(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32);
    fn fechar(&mut self);
}

struct Ponte<'a, C: Contorno> {
    destino: &'a mut C,
    escala: f32,
    x: f32,
    y: f32,
}

impl<C: Contorno> ttf_parser::OutlineBuilder for Ponte<'_, C> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.destino.mover(self.x + x * self.escala, self.y - y * self.escala);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.destino.linha(self.x + x * self.escala, self.y - y * self.escala);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let s = self.escala;
        self.destino.quadratica(self.x + x1 * s, self.y - y1 * s, self.x + x * s, self.y - y * s);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let s = self.escala;
        self.destino.cubica(self.x + x1 * s, self.y - y1 * s, self.x + x2 * s, self.y - y2 * s, self.x + x * s, self.y - y * s);
    }
    fn close(&mut self) {
        self.destino.fechar();
    }
}

impl Fonte {
    /// O glifo de um caractere (`None` se a fonte não tem).
    pub fn glifo(&self, c: char) -> Option<u16> {
        self.face.glyph_index(c).map(|g| g.0).filter(|&g| g != 0)
    }

    pub fn avanco(&self, g: u16) -> f32 {
        f32::from(self.face.glyph_hor_advance(GlyphId(g)).unwrap_or(0)) / self.upem
    }

    /// Altura acima e profundidade abaixo da linha de base, pela caixa do desenho.
    pub fn altura_profundidade(&self, g: u16) -> (f32, f32) {
        match self.face.glyph_bounding_box(GlyphId(g)) {
            Some(r) => (f32::from(r.y_max) / self.upem, -f32::from(r.y_min) / self.upem),
            None => (0.0, 0.0),
        }
    }

    /// Onde o desenho começa e termina na horizontal (acento combinante tem avanço zero).
    pub fn extensao_x(&self, g: u16) -> (f32, f32) {
        match self.face.glyph_bounding_box(GlyphId(g)) {
            Some(r) => (f32::from(r.x_min) / self.upem, f32::from(r.x_max) / self.upem),
            None => (0.0, 0.0),
        }
    }

    /// Correção itálica do glifo (0 se a tabela não diz).
    pub fn italico(&self, g: u16) -> f32 {
        self.info()
            .and_then(|i| i.italic_corrections)
            .and_then(|v| v.get(GlyphId(g)))
            .map_or(0.0, |v| f32::from(v.value) / self.upem)
    }

    /// Onde o acento se apoia em cima do glifo (do lado esquerdo), se a tabela diz.
    pub fn encaixe_do_acento(&self, g: u16) -> Option<f32> {
        self.info()
            .and_then(|i| i.top_accent_attachments)
            .and_then(|v| v.get(GlyphId(g)))
            .map(|v| f32::from(v.value) / self.upem)
    }

    fn info(&self) -> Option<math::GlyphInfo<'static>> {
        self.face.tables().math.and_then(|m| m.glyph_info)
    }

    fn construcao(&self, g: u16, vertical: bool) -> Option<math::GlyphConstruction<'static>> {
        let v = self.face.tables().math.and_then(|m| m.variants)?;
        let c = if vertical { v.vertical_constructions } else { v.horizontal_constructions };
        c.get(GlyphId(g))
    }

    /// As variantes de tamanho prontas, da menor para a maior: (glifo, medida em em).
    pub fn variantes(&self, g: u16, vertical: bool) -> Vec<(u16, f32)> {
        self.construcao(g, vertical)
            .map(|c| c.variants.into_iter().map(|v| (v.variant_glyph.0, f32::from(v.advance_measurement) / self.upem)).collect())
            .unwrap_or_default()
    }

    /// As peças para montar o glifo em qualquer tamanho (de baixo para cima / da esquerda para a direita).
    pub fn montagem(&self, g: u16, vertical: bool) -> Option<(Vec<Peca>, f32)> {
        let a = self.construcao(g, vertical)?.assembly?;
        let pecas = a
            .parts
            .into_iter()
            .map(|p| Peca {
                glifo: p.glyph_id.0,
                inicio: f32::from(p.start_connector_length) / self.upem,
                fim: f32::from(p.end_connector_length) / self.upem,
                avanco: f32::from(p.full_advance) / self.upem,
                extensor: p.part_flags.extender(),
            })
            .collect();
        Some((pecas, f32::from(a.italics_correction.value) / self.upem))
    }

    /// Entrega o contorno do glifo `g` no corpo `tamanho`, com a origem (linha de base) em `(x, y)`.
    pub fn contorno<C: Contorno>(&self, g: u16, tamanho: f32, x: f32, y: f32, destino: &mut C) {
        let mut p = Ponte { destino, escala: tamanho / self.upem, x, y };
        self.face.outline_glyph(GlyphId(g), &mut p);
    }
}
