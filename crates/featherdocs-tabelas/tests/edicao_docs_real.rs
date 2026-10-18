//! TBL-012: operações chamadas pela GUI sobre Sessao real do FeatherDocs.
use featherdocs_core::modelo::{Conteudo, Documento, Fmt, Marca, Trecho, NEGRITO};
use featherdocs_core::sessao::Sessao;
use featherdocs_tabelas::{docs_editar_celula, docs_editar_estrutura};

fn texto(s: &str) -> Vec<Trecho> {
    if s.is_empty() { Vec::new() } else { vec![Trecho::simples(s)] }
}

fn tabela(s: &Sessao, id: u64) -> Conteudo {
    s.documento().blocos.iter().find(|b| b.id == id).unwrap().conteudo.clone()
}

fn linhas(c: &Conteudo) -> &Vec<Vec<Vec<Trecho>>> {
    let Conteudo::Tabela { linhas, .. } = c else { panic!("não é tabela") };
    linhas
}

fn sessao() -> (Sessao, u64) {
    let mut documento = Documento::vazio();
    let id = documento.acrescentar(Conteudo::Tabela {
        cabecalho: true,
        linhas: vec![
            vec![texto("Primeiro"), texto("A")],
            vec![texto("Segundo"), texto("B")],
            vec![texto("Terceiro"), texto("C")],
        ],
    });
    (Sessao::com_documento(documento), id)
}

#[test]
fn linha_mais_usa_tables_e_pagina_com_undo_refazer() {
    let (mut s, id) = sessao();
    let antes = s.documento().clone();
    let versao = s.versao();
    let mudou = docs_editar_estrutura(&mut s, "linha+").unwrap();
    assert!(mudou.versao > versao);
    assert_eq!(linhas(&tabela(&s, id)).len(), 4);
    assert!(linhas(&tabela(&s, id))[1].iter().all(Vec::is_empty));
    assert_eq!(linhas(&tabela(&s, id))[2][0], texto("Segundo"));
    assert!(s.total_de_paginas() > 0);
    let apos = s.documento().clone();
    s.desfazer().unwrap();
    assert_eq!(s.documento(), &antes);
    s.refazer().unwrap();
    assert_eq!(s.documento(), &apos);
}

#[test]
fn colunas_mais_menos_preservam_trechos_ricos_e_cabecalho() {
    let mut documento = Documento::vazio();
    let mut rico = Trecho::simples("Preço");
    rico.estilo = NEGRITO;
    rico.link = Some("https://example.com".into());
    rico.fmt = Fmt { tamanho: 14.0, ..Fmt::default() };
    let id = documento.acrescentar(Conteudo::Tabela {
        cabecalho: true,
        linhas: vec![vec![vec![rico.clone()], texto("R$")]],
    });
    let mut s = Sessao::com_documento(documento);
    docs_editar_estrutura(&mut s, "coluna+").unwrap();
    let atual = tabela(&s, id);
    assert_eq!(linhas(&atual)[0][0], vec![rico.clone()]);
    assert!(linhas(&atual)[0][1].is_empty());
    assert_eq!(linhas(&atual)[0][2], texto("R$"));
    assert!(matches!(atual, Conteudo::Tabela { cabecalho: true, .. }));
    docs_editar_estrutura(&mut s, "coluna-").unwrap();
    let atual = tabela(&s, id);
    assert_eq!(linhas(&atual)[0][0], vec![rico.clone()]);
    assert_eq!(linhas(&atual)[0][1], texto("R$"));
    s.desfazer().unwrap();
    assert_eq!(linhas(&tabela(&s, id))[0][0], vec![rico]);
}

#[test]
fn formatacao_marcador_e_revisoes_se_deslocam_sem_perda() {
    let mut documento = Documento::vazio();
    let mut nota = Trecho::objeto(Marca::Nota {
        fim: false,
        texto: vec![Trecho::simples("Referência da nota")],
    });
    nota.estilo = NEGRITO;
    // O motor numera as notas ao compor; já nasce com o rótulo que ele daria.
    nota.rotulo = "1".into();
    let id = documento.acrescentar(Conteudo::Tabela {
        cabecalho: false,
        linhas: vec![vec![vec![nota.clone()], texto("A")]],
    });
    let mut s = Sessao::com_documento(documento);
    docs_editar_estrutura(&mut s, "linha+").unwrap();
    assert_eq!(linhas(&tabela(&s, id))[0][0], vec![nota.clone()]);
    docs_editar_estrutura(&mut s, "coluna+").unwrap();
    assert_eq!(linhas(&tabela(&s, id))[0][0], vec![nota]);
}

#[test]
fn atributos_do_bloco_nao_sao_perdidos_na_migracao() {
    let (mut s, id) = sessao();
    let original = s.documento().blocos[0].clone();
    docs_editar_estrutura(&mut s, "coluna+").unwrap();
    let novo = &s.documento().blocos[0];
    assert_eq!(novo.id, original.id);
    assert_eq!(novo.atrib, original.atrib);
    assert_eq!(novo.id, id);
    assert_eq!(s.cursor().foco.celula, Some((0, 1)));
}

#[test]
fn edicao_de_celula_simples_preserva_estilo_do_trecho() {
    let mut doc = Documento::vazio();
    let mut fonte = Trecho::simples("Antigo");
    fonte.estilo = NEGRITO;
    fonte.fmt.cor = 0x123456ff;
    let id = doc.acrescentar(Conteudo::Tabela { cabecalho: true, linhas: vec![vec![vec![fonte.clone()]]] });
    let mut s = Sessao::com_documento(doc);
    docs_editar_celula(&mut s, id, 0, 0, "Novo").unwrap();
    let mut esperado = fonte;
    esperado.texto = "Novo".into();
    assert_eq!(linhas(&tabela(&s, id))[0][0], vec![esperado]);
    s.desfazer().unwrap();
    assert_eq!(linhas(&tabela(&s, id))[0][0][0].texto, "Antigo");
}

#[test]
fn editar_multiplos_trechos_formatados_e_recusado_sem_mutacao() {
    let mut d = Documento::vazio();
    let mut primeiro = Trecho::simples("ABC");
    primeiro.estilo = NEGRITO;
    let id = d.acrescentar(Conteudo::Tabela {
        cabecalho: false, linhas: vec![vec![vec![primeiro, Trecho::simples("DEF")]]],
    });
    let mut s = Sessao::com_documento(d);
    let antes = s.documento().clone();
    let versao = s.versao();
    assert!(docs_editar_celula(&mut s, id, 0, 0, "Mudar").is_err());
    assert_eq!(s.documento(), &antes);
    assert_eq!(s.versao(), versao);
}

#[test]
fn linha_coluna_exclusoes_comparam_resultado_de_comando_nativo() {
    for acao in ["linha-", "coluna-"] {
        let (mut comum, id) = sessao();
        let (mut legado, outro_id) = sessao();
        assert_eq!(id, outro_id);
        docs_editar_estrutura(&mut comum, acao).unwrap();
        legado.tabela_excluir(if acao == "linha-" { "linha" } else { "coluna" }).unwrap();
        assert_eq!(tabela(&comum, id), tabela(&legado, id), "ação: {acao}");
    }
}

#[test]
fn ultima_linha_exclui_tabela_como_comportamento_anterior() {
    let mut d = Documento::vazio();
    d.acrescentar(Conteudo::Tabela { cabecalho: false, linhas: vec![vec![texto("sozinha")]] });
    let mut s = Sessao::com_documento(d);
    docs_editar_estrutura(&mut s, "linha-").unwrap();
    assert!(matches!(s.documento().blocos[0].conteudo, Conteudo::Paragrafo { .. }));
    s.desfazer().unwrap();
    assert!(matches!(s.documento().blocos[0].conteudo, Conteudo::Tabela { .. }));
}

#[test]
fn rejeicoes_nao_mutam_documento_ou_historico() {
    let (mut s, id) = sessao();
    let antes = s.documento().clone();
    let versao = s.versao();
    assert!(docs_editar_estrutura(&mut s, "tabela_explodir").is_err());
    assert!(docs_editar_celula(&mut s, id, 10, 1, "fora").is_err());
    assert!(docs_editar_celula(&mut s, u64::MAX, 0, 0, "ausente").is_err());
    assert_eq!(s.documento(), &antes);
    assert_eq!(s.versao(), versao);
}

#[test]
fn limite_de_200_linhas_e_30_colunas_nao_permite_mais() {
    let (mut s, id) = sessao();
    let mut linhas_grandes = vec![vec![Vec::new(); 30]; 200];
    linhas_grandes[0][0] = texto("conservado");
    let conteudo = Conteudo::Tabela { cabecalho: true, linhas: linhas_grandes };
    s.tabela_conteudo_substituir(id, conteudo, (0, 0)).unwrap();
    let antes = s.documento().clone();
    assert!(docs_editar_estrutura(&mut s, "linha+").is_err());
    assert!(docs_editar_estrutura(&mut s, "coluna+").is_err());
    assert_eq!(s.documento(), &antes);
}

#[test]
fn tabela_grande_repagina_e_undo_restabelece_numero_de_paginas() {
    let mut doc = Documento::vazio();
    let mut grade = vec![vec![Vec::new(); 2]; 110];
    for (i, linha) in grade.iter_mut().enumerate() {
        linha[0] = texto(&format!("Linha {i}: conteúdo para quebrar entre páginas"));
        linha[1] = texto("Valor");
    }
    let id = doc.acrescentar(Conteudo::Tabela { cabecalho: true, linhas: grade });
    let mut s = Sessao::com_documento(doc);
    let paginas_antes = s.total_de_paginas();
    assert!(paginas_antes > 1);
    let antigo = tabela(&s, id);
    docs_editar_estrutura(&mut s, "linha+").unwrap();
    assert_eq!(linhas(&tabela(&s, id)).len(), 111);
    assert!(s.total_de_paginas() > 1);
    s.desfazer().unwrap();
    assert_eq!(tabela(&s, id), antigo);
    assert_eq!(s.total_de_paginas(), paginas_antes);
}
