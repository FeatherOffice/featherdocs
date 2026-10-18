//! Mesclagem horizontal real FeatherDocs: segurança, seleção, layout e undo.
use featherdocs_core::modelo::{Conteudo, Documento, MesclagemTabela, Trecho, NEGRITO};
use featherdocs_core::sessao::Sessao;
use featherdocs_core::layout::{compor_em_mesclada, Deco, Entorno};
use featherdocs_tabelas::{
    docs_mesclar_celulas, docs_desmesclar_celulas, docs_editar_estrutura,
};

fn texto(s: &str) -> Vec<Trecho> {
    if s.is_empty() { Vec::new() } else { vec![Trecho::simples(s)] }
}
fn fixture(direita: &str) -> (Sessao, u64) {
    let mut d = Documento::vazio();
    let id = d.acrescentar(Conteudo::Tabela {
        cabecalho: false,
        linhas: vec![vec![texto("Origem"), texto(direita), Vec::new()]],
    });
    (Sessao::com_documento(d), id)
}
fn mesclas(s: &Sessao, id: u64) -> Vec<MesclagemTabela> {
    s.documento().blocos.iter().find(|b| b.id == id).unwrap().mesclagens.clone()
}

#[test]
fn mescla_duas_celulas_vazias_a_direita_com_undo_refazer() {
    let (mut s, id) = fixture("");
    let antes = s.documento().clone();
    let versao = s.versao();
    docs_mesclar_celulas(&mut s).unwrap();
    assert!(s.versao() > versao);
    let m = MesclagemTabela { linha: 0, coluna: 0, colunas: 2 };
    assert_eq!(mesclas(&s, id), vec![m]);
    assert_eq!(s.documento().blocos[0].conteudo, antes.blocos[0].conteudo);
    s.desfazer().unwrap();
    assert_eq!(s.documento(), &antes);
    s.refazer().unwrap();
    assert_eq!(mesclas(&s, id), vec![m]);
}

#[test]
fn celula_com_conteudo_nao_pode_ser_encoberta() {
    let (mut s, id) = fixture("Texto que não pode sumir");
    let antes = s.documento().clone();
    let versao = s.versao();
    assert!(docs_mesclar_celulas(&mut s).is_err());
    assert_eq!(mesclas(&s, id), Vec::<MesclagemTabela>::new());
    assert_eq!(s.versao(), versao);
    assert_eq!(s.documento(), &antes);
}

#[test]
fn desmesclar_conserva_texto_e_tem_undo_proprio() {
    let (mut s, id) = fixture("");
    docs_mesclar_celulas(&mut s).unwrap();
    let apos_mesclar = s.documento().clone();
    docs_desmesclar_celulas(&mut s).unwrap();
    assert!(mesclas(&s, id).is_empty());
    assert_eq!(s.documento().blocos[0].conteudo, apos_mesclar.blocos[0].conteudo);
    s.desfazer().unwrap();
    assert_eq!(s.documento(), &apos_mesclar);
    s.refazer().unwrap();
    assert!(mesclas(&s, id).is_empty());
}

#[test]
fn repetir_mesclar_amplia_ate_a_borda_sem_perder_conteudo() {
    let (mut s, id) = fixture("");
    docs_mesclar_celulas(&mut s).unwrap();
    docs_mesclar_celulas(&mut s).unwrap();
    assert_eq!(mesclas(&s, id), vec![
        MesclagemTabela { linha: 0, coluna: 0, colunas: 3 },
    ]);
    let antes = s.documento().clone();
    let versao = s.versao();
    assert!(docs_mesclar_celulas(&mut s).is_err());
    assert_eq!(s.documento(), &antes);
    assert_eq!(s.versao(), versao);
}

#[test]
fn mesclagem_respeita_final_da_linha_e_a_posicao_na_tabela() {
    let mut d = Documento::vazio();
    d.acrescentar(Conteudo::Tabela {
        cabecalho: false, linhas: vec![vec![texto("Só")]],
    });
    let mut s = Sessao::com_documento(d);
    let antes = s.documento().clone();
    assert!(docs_mesclar_celulas(&mut s).is_err());
    assert_eq!(s.documento(), &antes);
}

#[test]
fn desenho_elimina_divisoria_e_alarga_celula_principal() {
    let (mut s, id) = fixture("");
    let bloco = &s.documento().blocos[0];
    let antes = compor_em_mesclada(
        &bloco.conteudo, &bloco.atrib, 300.0, &Entorno::default(), &[],
    );
    docs_mesclar_celulas(&mut s).unwrap();
    let bloco = s.documento().blocos.iter().find(|b| b.id == id).unwrap();
    let depois = compor_em_mesclada(
        &bloco.conteudo, &bloco.atrib, 300.0, &Entorno::default(), &bloco.mesclagens,
    );
    let verticais = |linha: &featherdocs_core::layout::Linha| linha.deco.iter().filter(|d|
        matches!(d, Deco::Traco { x1, x2, .. } if x1 == x2)
    ).count();
    assert_eq!(verticais(&antes.linhas[0]), 4);
    assert_eq!(verticais(&depois.linhas[0]), 3);
    assert!(s.total_de_paginas() >= 1);
}

#[test]
fn bordas_e_formatacao_inline_nao_sao_modificadas() {
    let mut d = Documento::vazio();
    let mut rico = Trecho::simples("Negrito");
    rico.estilo = NEGRITO;
    rico.link = Some("https://example.org".into());
    let id = d.acrescentar(Conteudo::Tabela {
        cabecalho: true, linhas: vec![vec![vec![rico.clone()], vec![]]],
    });
    let mut s = Sessao::com_documento(d);
    let old = s.documento().blocos[0].conteudo.clone();
    docs_mesclar_celulas(&mut s).unwrap();
    assert_eq!(s.documento().blocos[0].conteudo, old);
    assert_eq!(mesclas(&s, id)[0].colunas, 2);
}

#[test]
fn estrutura_nao_pode_apagar_mesclagem_ao_inserir_coluna() {
    let (mut s, id) = fixture("");
    docs_mesclar_celulas(&mut s).unwrap();
    let antes = s.documento().clone();
    assert!(docs_editar_estrutura(&mut s, "coluna+").is_err());
    assert!(docs_editar_estrutura(&mut s, "linha-").is_err());
    assert_eq!(s.documento(), &antes);
    assert_eq!(mesclas(&s, id).len(), 1);
}

#[test]
fn cursor_dentro_da_area_coberta_e_redirecionado_para_ancora() {
    let (mut s, _) = fixture("");
    docs_mesclar_celulas(&mut s).unwrap();
    // largura útil A4 com duas margens: ~523 pt; coluna 1 começa após 1/3.
    let cfg = s.documento().pagina;
    let p = s.pos_na_pagina(0, cfg.margem_esq + cfg.largura_util() * 0.5,
        cfg.margem_sup + 18.0).unwrap();
    assert_eq!(p.celula, Some((0, 0)));
}
