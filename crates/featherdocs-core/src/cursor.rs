//! O cursor e a seleção: posições dentro do texto do documento. Quem guarda o
//! cursor é o motor (Rust); a interface só desenha o que ele manda e traduz mouse
//! e teclado em pedidos.

use serde::Serialize;

use crate::alvo::Cel;

/// Um ponto no texto: o bloco, a célula (se for tabela) e o caractere (0 = antes do primeiro).
/// Num bloco sem texto (régua, imagem…), o cursor fica em `desl: 0`, "no" bloco.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Pos {
    pub bloco: u64,
    pub celula: Cel,
    pub desl: usize,
}

/// O cursor (âncora = onde a seleção começou, foco = onde está o cursor) e o que acompanha.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Cursor {
    pub ancora: Pos,
    pub foco: Pos,
    /// Posição horizontal que as setas ↑ ↓ procuram, lembrada enquanto só se anda na vertical.
    pub x_desejado: Option<f32>,
    /// Formatação ligada com o cursor parado (Ctrl+B sem seleção), que vale para o próximo texto digitado.
    pub pendente: Option<(u8, crate::modelo::Fmt)>,
}

impl Cursor {
    /// Um cursor sem seleção em `p`.
    pub fn em(p: Pos) -> Self {
        Self { ancora: p, foco: p, x_desejado: None, pendente: None }
    }

    /// Não há texto selecionado.
    pub fn vazio(&self) -> bool {
        self.ancora == self.foco
    }
}

/// Para onde as setas e companhia levam o cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Movimento {
    Esquerda,
    Direita,
    PalavraEsquerda,
    PalavraDireita,
    Cima,
    Baixo,
    InicioDaLinha,
    FimDaLinha,
    InicioDoDocumento,
    FimDoDocumento,
    PaginaAcima,
    PaginaAbaixo,
}
