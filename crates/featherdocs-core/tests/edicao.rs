//! A edição de ponta a ponta, como a interface a usa: digitar, mover, selecionar,
//! formatar, desfazer — sempre pelos comandos públicos da sessão.

use featherdocs_core::alvo::texto_alvo;
use featherdocs_core::cursor::Movimento;
use featherdocs_core::inline::analisar;
use featherdocs_core::modelo::{Conteudo, Documento, NEGRITO};
use featherdocs_core::sessao::Sessao;
use featherdocs_core::texto::plano;

fn textos(s: &Sessao) -> Vec<String> {
    s.documento().blocos.iter().map(|b| texto_alvo(&b.conteudo, None).map(|t| plano(&t)).unwrap_or_else(|| "·".into())).collect()
}

fn longo(n: usize) -> Sessao {
    let mut d = Documento::vazio();
    d.acrescentar(Conteudo::sumario(2));
    for i in 0..n {
        d.acrescentar(Conteudo::Titulo { nivel: 1, texto: analisar(&format!("Capítulo {i}")) });
        for _ in 0..8 {
            d.acrescentar(Conteudo::Paragrafo { texto: analisar(&"texto corrido de teste ".repeat(30)) });
        }
    }
    Sessao::com_documento(d)
}

#[test]
fn documento_novo_tem_uma_pagina_e_o_cursor_no_comeco() {
    let s = Sessao::default();
    assert_eq!(s.total_de_paginas(), 1);
    assert_eq!(s.estado().blocos, 1);
    let v = s.visao(0, 0);
    assert!(v.cursor.is_some());
    assert!(v.retangulos.is_empty());
}

#[test]
fn escrever_um_texto_com_enter_e_formatar() {
    let mut s = Sessao::default();
    s.inserir_texto("# Relatório");
    assert_eq!(textos(&s), vec!["# Relatório"], "sem espaço depois do # ainda é texto");
    let mut s = Sessao::default();
    s.inserir_texto("#");
    s.inserir_texto(" ");
    s.inserir_texto("Relatório");
    s.tecla("Enter", false, false, false);
    s.inserir_texto("Um texto ");
    s.tecla("b", true, false, false);
    s.inserir_texto("forte");
    s.tecla("b", true, false, false);
    s.inserir_texto(" e normal.");
    assert_eq!(textos(&s), vec!["Relatório", "Um texto forte e normal."]);
    let Conteudo::Paragrafo { texto } = &s.documento().blocos[1].conteudo else { panic!() };
    assert_eq!(texto.iter().find(|t| t.estilo == NEGRITO).map(|t| t.texto.as_str()), Some("forte"));
    assert_eq!(s.estado().palavras, 6);
}

#[test]
fn desfazer_e_refazer_levam_o_cursor_junto() {
    let mut s = Sessao::default();
    s.inserir_texto("abc");
    s.tecla("Enter", false, false, false);
    s.inserir_texto("def");
    s.tecla("z", true, false, false);
    s.tecla("z", true, false, false);
    assert_eq!(textos(&s), vec!["abc"]);
    assert_eq!(s.cursor().foco.desl, 3);
    s.tecla("y", true, false, false);
    s.tecla("y", true, false, false);
    assert_eq!(textos(&s), vec!["abc", "def"]);
    assert!(s.estado().sujo);
    s.marcar_gravado();
    assert!(!s.estado().sujo);
}

#[test]
fn documento_grande_sumario_atualiza_ao_digitar_um_titulo() {
    let mut s = longo(40);
    assert!(s.total_de_paginas() > 10);
    let antes = s.quadro(0, 0).unwrap();
    s.mover(Movimento::InicioDoDocumento, false);
    s.mover(Movimento::Baixo, false); // sumário → primeiro título
    s.mover(Movimento::FimDaLinha, false);
    s.inserir_texto(" novo");
    assert_ne!(antes, s.quadro(0, 0).unwrap(), "o sumário mudou");
    assert_eq!(s.estrutura()[0].texto, "Capítulo 0 novo");
}

#[test]
fn digitar_no_fim_de_um_documento_grande_so_repinta_as_ultimas_paginas() {
    let mut s = longo(60);
    s.mover(Movimento::FimDoDocumento, false);
    let m = s.inserir_texto("fim");
    assert!(m.primeira_pagina + 3 >= s.total_de_paginas(), "primeira página mudada = {}", m.primeira_pagina);
}

#[test]
fn cursor_visivel_depois_de_cada_acao_e_selecao_com_retangulos() {
    let mut s = longo(2);
    s.mover(Movimento::FimDoDocumento, false);
    s.mover(Movimento::Cima, true);
    s.mover(Movimento::Cima, true);
    let ult = s.total_de_paginas() - 1;
    let v = s.visao(0, ult);
    assert!(v.cursor.is_some());
    assert!(!v.retangulos.is_empty());
    assert!(v.formato.tem_selecao);
    assert!(v.retangulos.iter().all(|r| r.w > 0.0 && r.h > 0.0));
}

#[test]
fn tabela_vai_e_volta_pelo_csv() {
    let mut s = Sessao::default();
    s.inserir_bloco("tabela");
    let id = s.cursor().foco.bloco;
    let mut c = s.tabela_celulas(id).unwrap();
    c[1][1] = "42".into();
    s.tabela_substituir(id, &c).unwrap();
    assert_eq!(s.tabela_celulas(id).unwrap()[1][1], "42");
}

#[test]
fn trocar_o_papel_repagina() {
    let mut s = longo(5);
    assert!((s.estado().largura - 595.28).abs() < 0.01);
    s.papel_definir("carta").unwrap();
    assert!((s.estado().largura - 612.0).abs() < 0.01);
    assert!(s.papel_definir("a0").is_err());
}

#[test]
fn margens_maiores_repaginam_e_as_absurdas_sao_recusadas() {
    let mut s = longo(5);
    let antes = s.estado().paginas;
    s.margens_definir(144.0, 144.0, 72.0, 72.0).unwrap();
    let e = s.estado();
    assert!((e.margem_esq - 144.0).abs() < 0.01 && (e.margem_dir - 144.0).abs() < 0.01);
    assert!(e.paginas >= antes, "texto mais estreito não pode caber em menos páginas");
    assert!(s.margens_definir(-1.0, 72.0, 72.0, 72.0).is_err());
    assert!(s.margens_definir(300.0, 300.0, 72.0, 72.0).is_err());
    assert!((s.estado().margem_esq - 144.0).abs() < 0.01, "o pedido recusado não muda nada");
}

#[test]
fn clique_em_cada_pagina_poe_o_cursor_num_texto_valido() {
    let mut s = longo(6);
    for p in 0..s.total_de_paginas() {
        s.clicar(p, 200.0, 300.0, false, 1);
        let f = s.cursor().foco;
        assert!(s.geometria(&f).is_some(), "página {p}");
    }
}

#[test]
fn papel_orientacao_e_medida_propria_mantem_as_margens() {
    let mut s = featherdocs_core::sessao::Sessao::default();
    s.margens_definir(36.0, 36.0, 36.0, 36.0).unwrap();
    s.papel_definir("oficio").unwrap();
    assert_eq!(s.estado().papel, "oficio");
    assert_eq!(s.estado().margem_esq, 36.0, "trocar o papel não zera as margens");
    s.orientacao_definir(true).unwrap();
    let e = s.estado();
    assert!(e.paisagem && e.largura > e.altura && e.papel == "oficio");
    s.papel_definir("a4").unwrap();
    assert!(s.estado().paisagem, "o papel novo segue a orientação");
    s.orientacao_definir(false).unwrap();
    s.papel_personalizado(500.0, 700.0).unwrap();
    assert_eq!(s.estado().papel, "personalizado");
    assert!(s.papel_personalizado(100.0, 700.0).is_err());
}
