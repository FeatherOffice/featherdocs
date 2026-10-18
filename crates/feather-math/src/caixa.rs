//! A caixa do TeX (largura, altura, profundidade) com o que ela desenha já posicionado
//! em relação à própria origem (ponta esquerda da linha de base; `y` cresce para baixo).

use crate::{Glifo, Retangulo, Traco};

#[derive(Clone, Debug, Default)]
pub struct Caixa {
    pub w: f32,
    pub h: f32,
    pub d: f32,
    pub glifos: Vec<Glifo>,
    pub rets: Vec<Retangulo>,
    /// Traços inclinados (`\cancel`).
    pub tracos: Vec<Traco>,
    /// Correção itálica do último glifo (para o expoente não colar na letra inclinada).
    pub italico: f32,
    /// Ponto de encaixe do acento, quando a caixa é um glifo só.
    pub acento: Option<f32>,
    /// Um caractere só: o expoente não usa a "queda" da base composta (regra 18a do TeX).
    pub simples: bool,
    /// Operador grande (∑, ∫): o índice recua a correção itálica.
    pub op_grande: bool,
}

impl Caixa {
    pub fn vazia() -> Self {
        Self::default()
    }

    pub fn espaco(w: f32) -> Self {
        Self { w, ..Self::default() }
    }

    /// Junta o desenho de `o` deslocado de `(dx, dy)`; medidas ficam por conta de quem chama.
    pub fn anexar(&mut self, o: &Caixa, dx: f32, dy: f32) {
        self.glifos.extend(o.glifos.iter().map(|g| Glifo { x: g.x + dx, y: g.y + dy, ..*g }));
        self.rets.extend(o.rets.iter().map(|r| Retangulo { x: r.x + dx, y: r.y + dy, ..*r }));
        self.tracos.extend(o.tracos.iter().map(|t| Traco { x1: t.x1 + dx, y1: t.y1 + dy, x2: t.x2 + dx, y2: t.y2 + dy, ..*t }));
    }

    /// Põe `o` à direita, na mesma linha de base.
    pub fn seguir(&mut self, o: &Caixa) {
        let x = self.w;
        self.anexar(o, x, 0.0);
        self.w += o.w;
        self.h = self.h.max(o.h);
        self.d = self.d.max(o.d);
        self.italico = o.italico;
    }

    /// Sobe (`dy` < 0) ou desce a caixa inteira, ajustando altura e profundidade.
    pub fn deslocar_vertical(&mut self, dy: f32) {
        for g in &mut self.glifos {
            g.y += dy;
        }
        for r in &mut self.rets {
            r.y += dy;
        }
        for t in &mut self.tracos {
            t.y1 += dy;
            t.y2 += dy;
        }
        self.h -= dy;
        self.d += dy;
    }

    pub fn retangulo(&mut self, x: f32, y: f32, largura: f32, altura: f32, cor: u32) {
        self.rets.push(Retangulo { x, y, largura, altura, cor });
    }

    /// Pinta tudo que ainda não tem cor própria.
    pub fn colorir(&mut self, cor: u32) {
        for g in &mut self.glifos {
            if g.cor == 0 {
                g.cor = cor;
            }
        }
        for r in &mut self.rets {
            if r.cor == 0 {
                r.cor = cor;
            }
        }
        for t in &mut self.tracos {
            if t.cor == 0 {
                t.cor = cor;
            }
        }
    }
}
