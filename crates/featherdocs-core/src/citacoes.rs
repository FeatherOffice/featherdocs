//! Fontes, citações e a lista de referências do documento aberto. A formatação de cada estilo
//! está em `bibliografia`; aqui ficam o cadastro das fontes, a citação no cursor e o bloco gerado.

use std::collections::HashMap;

use crate::bibliografia::{citacao, estilo_do_nome, lista, numerico};
use crate::cursor::Pos;
use crate::layout::ParagrafoGerado;
use crate::modelo::{Atrib, Bibliografia, Conteudo, EstiloDeCitacao, Fonte, Marca};
use crate::sessao::{Mudanca, Sessao};

/// A chave de uma fonte: letras, números e `_ - : .`, sem espaço (é o que o BibTeX aceita).
fn chave_valida(c: &str) -> Result<String, String> {
    let c = c.trim();
    if c.is_empty() || !c.chars().all(|x| x.is_alphanumeric() || "_-:.".contains(x)) {
        return Err("A chave da fonte não pode ficar vazia e só tem letras, números e _ - : . (sem espaço).".to_string());
    }
    Ok(c.to_string())
}

/// As chaves citadas no documento, na ordem em que aparecem pela primeira vez.
pub fn chaves_citadas(blocos: &[crate::modelo::Bloco]) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for t in blocos.iter().flat_map(|b| b.conteudo.textos()).flatten() {
        if let Some(Marca::Citacao { chaves, .. }) = t.marca.as_deref() {
            for c in chaves {
                if !v.contains(c) {
                    v.push(c.clone());
                }
            }
        }
    }
    v
}

/// O número de cada fonte nos estilos numéricos (a ordem da primeira citação); vazio nos outros.
pub(crate) fn numeros_das_fontes(e: EstiloDeCitacao, citadas: &[String]) -> HashMap<String, usize> {
    if !numerico(e) {
        return HashMap::new();
    }
    citadas.iter().enumerate().map(|(k, c)| (c.clone(), k + 1)).collect()
}

/// O rótulo de uma citação no estilo do documento.
pub(crate) fn rotulo_da_citacao(b: &Bibliografia, numeros: &HashMap<String, usize>, chaves: &[String], pagina: &str) -> String {
    citacao(b.estilo, chaves, pagina, &b.fontes, numeros)
}

impl Sessao {
    /// As fontes e o estilo (para a caixa "Gerenciar fontes").
    pub fn bibliografia(&self) -> &Bibliografia {
        &self.doc.bibliografia
    }

    /// Os parágrafos da lista de referências, com o recuo de cada estilo (o pendente da APA e da
    /// Chicago, o número para fora nos numéricos).
    pub(crate) fn paragrafos_da_bibliografia(&self, todas: bool) -> Vec<ParagrafoGerado> {
        let b = &self.doc.bibliografia;
        let recuo = match b.estilo {
            EstiloDeCitacao::Apa | EstiloDeCitacao::Chicago => 36.0,
            EstiloDeCitacao::Vancouver | EstiloDeCitacao::Ieee => 24.0,
            EstiloDeCitacao::Abnt => 0.0,
        };
        let atrib = Atrib { recuo_esq: recuo, primeira: -recuo, entrelinha: 1.0, ..Atrib::default() };
        lista(b.estilo, &b.fontes, &chaves_citadas(&self.doc.blocos), todas).into_iter().map(|texto| ParagrafoGerado { texto, atrib, antes: 8.0 }).collect()
    }

    /// Cadastra uma fonte nova ou troca a de chave `antiga` (vazia = nova). Mudar a chave muda também
    /// as citações que a usam.
    pub fn fonte_salvar(&mut self, mut fonte: Fonte, antiga: &str) -> Result<Mudanca, String> {
        fonte.chave = chave_valida(&fonte.chave)?;
        let fontes = &self.doc.bibliografia.fontes;
        if fonte.chave != antiga && fontes.iter().any(|f| f.chave == fonte.chave) {
            return Err(format!("Já existe uma fonte com a chave \"{}\".", fonte.chave));
        }
        let nova = fonte.chave.clone();
        match self.doc.bibliografia.fontes.iter_mut().find(|f| f.chave == antiga && !antiga.is_empty()) {
            Some(f) => *f = fonte,
            None => self.doc.bibliografia.fontes.push(fonte),
        }
        if !antiga.is_empty() && antiga != nova {
            if let Some(m) = self.trocar_chave_nas_citacoes(antiga, &nova) {
                return Ok(m);
            }
        }
        Ok(self.fim_da_edicao_sem_blocos())
    }

    /// Troca a chave `de` por `para` em toda citação (um passo de desfazer); `None` se nenhuma a usa.
    fn trocar_chave_nas_citacoes(&mut self, de: &str, para: &str) -> Option<Mudanca> {
        let usa = |b: &crate::modelo::Bloco| b.conteudo.textos().into_iter().flatten().any(|t| matches!(t.marca.as_deref(), Some(Marca::Citacao { chaves, .. }) if chaves.iter().any(|c| c == de)));
        let i0 = self.doc.blocos.iter().position(usa)?;
        let i1 = self.doc.blocos.iter().rposition(usa)?;
        let mut t = self.trabalho(i0, i1);
        for b in &mut t.blocos {
            for tr in b.conteudo.textos_mut().into_iter().flatten() {
                if let Some(Marca::Citacao { chaves, .. }) = tr.marca.as_deref_mut() {
                    for c in chaves.iter_mut().filter(|c| *c == de) {
                        *c = para.to_string();
                    }
                }
            }
        }
        let cursor = self.cursor;
        Some(self.fechar_trabalho(t, cursor, None))
    }

    /// Tira uma fonte do cadastro (as citações dela passam a mostrar `?chave?`).
    pub fn fonte_remover(&mut self, chave: &str) -> Mudanca {
        let antes = self.doc.bibliografia.fontes.len();
        self.doc.bibliografia.fontes.retain(|f| f.chave != chave);
        if antes == self.doc.bibliografia.fontes.len() {
            return self.sem_mudanca();
        }
        self.fim_da_edicao_sem_blocos()
    }

    /// Junta fontes ao cadastro (de um `.bib`): a de chave repetida troca a que havia. Devolve quantas.
    pub fn fontes_importar(&mut self, fontes: Vec<Fonte>) -> (Mudanca, usize) {
        let n = fontes.len();
        for f in fontes {
            match self.doc.bibliografia.fontes.iter_mut().find(|x| x.chave == f.chave) {
                Some(x) => *x = f,
                None => self.doc.bibliografia.fontes.push(f),
            }
        }
        (self.fim_da_edicao_sem_blocos(), n)
    }

    /// Referências → Estilo: `abnt`, `apa`, `vancouver`, `ieee`, `chicago`.
    pub fn estilo_citacao_definir(&mut self, nome: &str) -> Result<Mudanca, String> {
        let e = estilo_do_nome(nome).ok_or_else(|| format!("Não conheço o estilo de citação \"{nome}\"."))?;
        if self.doc.bibliografia.estilo == e {
            return Ok(self.sem_mudanca());
        }
        self.doc.bibliografia.estilo = e;
        Ok(self.fim_da_edicao_sem_blocos())
    }

    fn conferir_chaves(&self, chaves: &[String]) -> Result<Vec<String>, String> {
        if chaves.is_empty() {
            return Err("Escolha ao menos uma fonte.".to_string());
        }
        for c in chaves {
            if !self.doc.bibliografia.fontes.iter().any(|f| &f.chave == c) {
                return Err(format!("Não há fonte com a chave \"{c}\"."));
            }
        }
        Ok(chaves.to_vec())
    }

    /// Referências → Inserir citação: as fontes `chaves` (e a página citada) no cursor.
    pub fn citacao_inserir(&mut self, chaves: &[String], pagina: &str) -> Result<Mudanca, String> {
        let chaves = self.conferir_chaves(chaves)?;
        self.objeto_inserir(Marca::Citacao { chaves, pagina: pagina.trim().to_string() })
    }

    /// Troca as fontes e a página da citação em `p`.
    pub fn citacao_editar(&mut self, p: &Pos, chaves: &[String], pagina: &str) -> Result<Mudanca, String> {
        if !matches!(self.objeto_em(p), Some(Marca::Citacao { .. })) {
            return Err("Aqui não há citação.".to_string());
        }
        let chaves = self.conferir_chaves(chaves)?;
        self.objeto_trocar(p, Marca::Citacao { chaves, pagina: pagina.trim().to_string() })
    }

    /// Referências → Bibliografia: a lista de referências no cursor (`todas`: também as não citadas).
    pub fn bibliografia_inserir(&mut self, todas: bool) -> Mudanca {
        self.inserir_conteudo(Conteudo::Bibliografia { todas })
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::cursor::Cursor;
    use crate::inline::{analisar, texto_puro};
    use crate::modelo::{Autor, Documento};

    fn sessao() -> Sessao {
        let mut d = Documento::vazio();
        d.acrescentar(Conteudo::Paragrafo { texto: analisar("Segundo o autor") });
        d.acrescentar(Conteudo::Paragrafo { texto: analisar("E outro") });
        let mut s = Sessao::com_documento(d);
        for (chave, sobrenome, ano) in [("silva2023", "Silva", "2023"), ("lima2020", "Lima", "2020")] {
            let f = Fonte { chave: chave.into(), autores: vec![Autor { sobrenome: sobrenome.into(), nome: "Ana".into() }], titulo: format!("Livro de {sobrenome}"), ano: ano.into(), ..Fonte::default() };
            s.fonte_salvar(f, "").unwrap();
        }
        s
    }

    fn ir(s: &mut Sessao, bloco: usize, desl: usize) {
        let id = s.documento().blocos[bloco].id;
        s.cursor = Cursor::em(Pos { bloco: id, celula: None, desl });
    }

    fn rotulos(s: &Sessao) -> Vec<String> {
        s.documento().blocos.iter().flat_map(|b| b.conteudo.textos().into_iter().flatten().filter(|t| t.marca.is_some()).map(|t| t.rotulo.clone()).collect::<Vec<_>>()).collect()
    }

    #[test]
    fn citar_trocar_estilo_e_a_lista_acompanha() {
        let mut s = sessao();
        ir(&mut s, 0, 15);
        s.citacao_inserir(&["silva2023".into()], "4").unwrap();
        ir(&mut s, 1, 7);
        s.citacao_inserir(&["lima2020".into(), "silva2023".into()], "").unwrap();
        assert_eq!(rotulos(&s), ["(SILVA, 2023, p. 4)", "(LIMA, 2020; SILVA, 2023)"]);
        s.estilo_citacao_definir("ieee").unwrap();
        assert_eq!(rotulos(&s), ["[1, p. 4]", "[2], [1]"]);
        s.bibliografia_inserir(false);
        let refs = s.paragrafos_da_bibliografia(false);
        assert!(texto_puro(&refs[0].texto).starts_with("[1] A. Silva"));
        assert!(s.citacao_inserir(&["nao_existe".into()], "").is_err());
        assert!(s.estilo_citacao_definir("mla").is_err());
    }

    #[test]
    fn mudar_a_chave_muda_as_citacoes_e_remover_deixa_a_marca_de_erro() {
        let mut s = sessao();
        ir(&mut s, 0, 15);
        s.citacao_inserir(&["silva2023".into()], "").unwrap();
        let mut f = s.bibliografia().fontes[0].clone();
        f.chave = "silva_novo".into();
        s.fonte_salvar(f, "silva2023").unwrap();
        assert!(matches!(s.objeto_em(&Pos { desl: 15, ..s.cursor.foco }), Some(Marca::Citacao { chaves, .. }) if chaves == ["silva_novo"]));
        assert!(s.fonte_salvar(Fonte { chave: "lima2020".into(), ..Fonte::default() }, "silva_novo").is_err(), "chave repetida");
        assert!(s.fonte_salvar(Fonte { chave: "tem espaço".into(), ..Fonte::default() }, "").is_err());
        s.fonte_remover("silva_novo");
        assert_eq!(rotulos(&s), ["(?silva_novo?)"]);
    }
}
