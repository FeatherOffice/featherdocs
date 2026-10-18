//! O índice remissivo: entradas marcadas no texto (`Marca::Indice`, invisíveis) e o bloco gerado
//! com cada termo, os subtermos e as páginas, em ordem alfabética e agrupado pela inicial. As
//! páginas só se sabem depois de paginar; o motor compõe o índice de novo quando elas mudam.

use crate::bibliografia::dobrar;
use crate::cursor::Cursor;
use crate::layout::ParagrafoGerado;
use crate::modelo::{Atrib, Conteudo, Marca, Trecho, NEGRITO, OBJETO};
use crate::sessao::{Mudanca, Sessao};
use crate::texto::inserir_objeto;

/// Um termo (ou subtermo) do índice e as páginas onde ele foi marcado.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntradaDeIndice {
    pub termo: String,
    pub sub: String,
    pub paginas: Vec<usize>,
}

/// As páginas em texto: "3, 5, 12".
fn paginas_em_texto(p: &[usize]) -> String {
    p.iter().map(usize::to_string).collect::<Vec<_>>().join(", ")
}

/// As posições (em caracteres) logo depois de cada ocorrência de `termo` em `texto` que ainda não
/// tem uma entrada do mesmo termo logo em seguida.
fn fins_de_ocorrencias(t: &[Trecho], termo: &str) -> Vec<usize> {
    let chars: Vec<char> = crate::texto::plano(t).chars().collect();
    let alvo: Vec<char> = termo.chars().collect();
    if alvo.is_empty() || alvo.len() > chars.len() {
        return Vec::new();
    }
    (0..=chars.len() - alvo.len())
        .filter(|&i| chars[i..i + alvo.len()] == alvo[..])
        .map(|i| i + alvo.len())
        .filter(|&fim| !(chars.get(fim) == Some(&OBJETO) && matches!(crate::texto::objeto_em(t, fim), Some(Marca::Indice { termo: x, .. }) if x == termo)))
        .collect()
}

impl Sessao {
    /// As entradas marcadas no documento, com as páginas, em ordem alfabética (sem acento nem caixa).
    pub(crate) fn coletar_indice(&self) -> Vec<EntradaDeIndice> {
        let mut v: Vec<EntradaDeIndice> = Vec::new();
        for (i, b) in self.doc.blocos.iter().enumerate() {
            let tabela = matches!(b.conteudo, Conteudo::Tabela { .. });
            for t in b.conteudo.textos() {
                let mut pos = 0;
                for tr in t {
                    if let Some(Marca::Indice { termo, sub }) = tr.marca.as_deref() {
                        let pagina = if tabela { self.pagina_do_bloco(b.id).map_or(1, |p| p + 1) } else { self.pagina_do_caractere(i, pos) };
                        match v.iter_mut().find(|e| &e.termo == termo && &e.sub == sub) {
                            Some(e) if !e.paginas.contains(&pagina) => e.paginas.push(pagina),
                            Some(_) => {}
                            None => v.push(EntradaDeIndice { termo: termo.clone(), sub: sub.clone(), paginas: vec![pagina] }),
                        }
                    }
                    pos += tr.texto.chars().count();
                }
            }
        }
        for e in &mut v {
            e.paginas.sort_unstable();
        }
        v.sort_by_key(|e| (dobrar(&e.termo), dobrar(&e.sub)));
        v
    }

    /// Depois de paginar: refaz as entradas do índice; `true` se mudaram e há índice para recompor.
    pub(crate) fn atualizar_indice(&mut self) -> bool {
        if !self.doc.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::IndiceRemissivo)) {
            return false;
        }
        let novo = self.coletar_indice();
        if novo == self.indice {
            return false;
        }
        self.indice = novo;
        true
    }

    /// Os parágrafos do índice: a inicial em negrito, cada termo com as páginas, os subtermos recuados.
    pub(crate) fn paragrafos_do_indice(&self) -> Vec<ParagrafoGerado> {
        let pendente = |recuo: f32| Atrib { recuo_esq: recuo + 18.0, primeira: -18.0, entrelinha: 1.0, ..Atrib::default() };
        let mut v = Vec::new();
        let mut letra = String::new();
        let mut termo_posto = String::new();
        for e in &self.indice {
            let inicial = dobrar(&e.termo).chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
            if inicial != letra {
                letra.clone_from(&inicial);
                v.push(ParagrafoGerado { texto: vec![Trecho { texto: inicial, estilo: NEGRITO, ..Trecho::default() }], atrib: Atrib::default(), antes: 10.0 });
            }
            if e.sub.is_empty() {
                v.push(ParagrafoGerado { texto: vec![Trecho::simples(&format!("{}, {}", e.termo, paginas_em_texto(&e.paginas)))], atrib: pendente(0.0), antes: 2.0 });
                termo_posto.clone_from(&e.termo);
                continue;
            }
            if termo_posto != e.termo {
                v.push(ParagrafoGerado { texto: vec![Trecho::simples(&e.termo)], atrib: pendente(0.0), antes: 2.0 });
                termo_posto.clone_from(&e.termo);
            }
            v.push(ParagrafoGerado { texto: vec![Trecho::simples(&format!("{}, {}", e.sub, paginas_em_texto(&e.paginas)))], atrib: pendente(18.0), antes: 1.0 });
        }
        v
    }

    /// Referências → Marcar entrada: o `termo` (e o `sub`termo) no fim da seleção; com `todas`, depois
    /// de toda ocorrência do termo no documento. Devolve quantas entradas novas.
    pub fn indice_marcar(&mut self, termo: &str, sub: &str, todas: bool) -> Result<(Mudanca, usize), String> {
        let termo = termo.trim();
        if termo.is_empty() {
            return Err("Escreva o termo da entrada.".to_string());
        }
        let marca = Marca::Indice { termo: termo.to_string(), sub: sub.trim().to_string() };
        if !todas {
            let (_, fim) = self.selecao_ordenada();
            self.cursor = Cursor::em(fim);
            return Ok((self.objeto_inserir(marca)?, 1));
        }
        let tem = |b: &crate::modelo::Bloco| b.conteudo.textos().iter().any(|t| !fins_de_ocorrencias(t, termo).is_empty());
        let (Some(i0), Some(i1)) = (self.doc.blocos.iter().position(tem), self.doc.blocos.iter().rposition(tem)) else {
            return Err(format!("\"{termo}\" não aparece no texto."));
        };
        let mut t = self.trabalho(i0, i1);
        let mut n = 0;
        for b in &mut t.blocos {
            for texto in b.conteudo.textos_mut() {
                for fim in fins_de_ocorrencias(texto, termo).into_iter().rev() {
                    inserir_objeto(texto, fim, marca.clone());
                    n += 1;
                }
            }
        }
        let cursor = self.cursor;
        Ok((self.fechar_trabalho(t, cursor, None), n))
    }

    /// Referências → Inserir índice: o índice remissivo no cursor.
    pub fn indice_inserir(&mut self) -> Mudanca {
        self.inserir_conteudo(Conteudo::IndiceRemissivo)
    }

    /// O ponto `desl` do bloco `i` (para os testes).
    #[cfg(test)]
    pub(crate) fn ponto(&self, i: usize, desl: usize) -> Option<crate::cursor::Pos> {
        self.doc.blocos.get(i).map(|b| crate::cursor::Pos { bloco: b.id, celula: None, desl })
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::inline::{analisar, texto_puro};
    use crate::modelo::Documento;

    #[test]
    fn marcar_todas_as_ocorrencias_e_o_indice_com_paginas() {
        let mut d = Documento::vazio();
        d.acrescentar(Conteudo::Paragrafo { texto: analisar("O gato e a árvore.") });
        d.acrescentar(Conteudo::QuebraDePagina);
        d.acrescentar(Conteudo::Paragrafo { texto: analisar("Outro gato, e o Ábaco.") });
        let mut s = Sessao::com_documento(d);
        let (_, n) = s.indice_marcar("gato", "", true).unwrap();
        assert_eq!(n, 2);
        let (_, n) = s.indice_marcar("gato", "", true).unwrap_err_or_zero();
        assert_eq!(n, 0, "não marca de novo o que já tem entrada");
        s.cursor = Cursor::em(s.ponto(2, 22).unwrap());
        s.indice_marcar("Ábaco", "", false).unwrap();
        s.cursor = Cursor::em(s.ponto(0, 17).unwrap());
        s.indice_marcar("árvore", "folhas", false).unwrap();
        assert_eq!(texto_puro(match &s.documento().blocos[0].conteudo { Conteudo::Paragrafo { texto } => texto, _ => panic!() }), "O gato e a árvore.");
        s.cursor = Cursor::em(s.ponto(2, 0).unwrap());
        s.indice_inserir();
        let linhas: Vec<String> = s.paragrafos_do_indice().iter().map(|p| texto_puro(&p.texto)).collect();
        assert_eq!(linhas, ["A", "Ábaco, 2", "árvore", "folhas, 1", "G", "gato, 1, 2"]);
        assert!(s.indice_marcar(" ", "", false).is_err());
        assert!(s.indice_marcar("xilofone", "", true).is_err());
    }

    trait ErroOuZero {
        fn unwrap_err_or_zero(self) -> (Option<Mudanca>, usize);
    }

    impl ErroOuZero for Result<(Mudanca, usize), String> {
        fn unwrap_err_or_zero(self) -> (Option<Mudanca>, usize) {
            match self {
                Ok((m, n)) => (Some(m), n),
                Err(_) => (None, 0),
            }
        }
    }
}
