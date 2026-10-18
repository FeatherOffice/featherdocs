//! Comandos em linha dos pacotes mais usados (`siunitx`, `\text`, `\textsc`, `\fontsize`, caixas…).

use super::Inline;

impl Inline<'_> {
    /// Comandos em linha além dos básicos; devolve quantos bytes consumiu depois do nome.
    pub(super) fn extra(&mut self, _nome: &str, _depois: &str) -> Option<usize> {
        None
    }
}
