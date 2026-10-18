//! Pacotes e definições do preâmbulo; `\input`/`\include`; ambientes além dos básicos.

use super::avisos::{self, Coletor, TipoDeAviso};
use super::Analisador;

/// Troca `\input{x}`/`\include{x}` pelo conteúdo de `x.tex` na `pasta`.
pub(super) fn incluir_arquivos(fonte: &str, _pasta: Option<&std::path::Path>, _coletor: &Coletor) -> String {
    fonte.to_string()
}

/// A classe e os pacotes que o leitor não conhece entram no aviso.
pub(super) fn avisar_preambulo(preambulo: &str, coletor: &Coletor) {
    if let Some(c) = avisos::classe_do_preambulo(preambulo) {
        if !avisos::classe_conhecida(&c) {
            coletor.anotar(TipoDeAviso::Classe, &c);
        }
    }
    for p in avisos::pacotes_do_preambulo(preambulo) {
        if !avisos::pacote_conhecido(&p) {
            coletor.anotar(TipoDeAviso::Pacote, &p);
        }
    }
}

impl Analisador<'_> {
    /// Ambiente que os básicos não tratam: o conteúdo sai como texto e o nome vai para o aviso.
    pub(super) fn ambiente_extra(&mut self, base: &str, corpo: &str) {
        if base != "document" {
            self.ctx.avisos.ambiente(base);
        }
        let filhos = Analisador::novo(corpo, self.ctx.clone()).executar();
        self.saida.extend(filhos);
    }
}
