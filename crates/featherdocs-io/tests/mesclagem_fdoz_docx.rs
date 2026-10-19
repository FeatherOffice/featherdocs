//! Persiste as mesclagens no formato nativo Arrow e exporta gridSpan OOXML.
use featherdocs_core::modelo::{Conteudo, Documento, MesclagemTabela, Trecho};
use featherdocs_core::sessao::Sessao;
use featherdocs_io::{docx, fdoz};
use std::io::{Cursor, Read};

fn modelo() -> (Documento, u64) {
    let mut documento = Documento::vazio();
    let id = documento.acrescentar(Conteudo::Tabela {
        cabecalho: true,
        linhas: vec![
            vec![vec![Trecho::simples("FeatherWorks")], Vec::new(), vec![Trecho::simples("Valor")]],
            vec![vec![Trecho::simples("Outro")], Vec::new(), Vec::new()],
        ],
    });
    (documento, id)
}

#[test]
fn fdoz_mesclagem_arrow_roundtrip_exato_e_documento_legado() {
    let (original, id) = modelo();
    let antigo = fdoz::gravar(&original).unwrap();
    assert_eq!(fdoz::ler(&antigo).unwrap(), original);

    let mut sessao = Sessao::com_documento(original.clone());
    let m = MesclagemTabela { linha: 0, coluna: 0, colunas: 2 };
    sessao.tabela_mesclagens_definir(id, vec![m], (0, 0)).unwrap();
    let esperado = sessao.documento().clone();
    let bytes = fdoz::gravar(&esperado).unwrap();
    assert_eq!(fdoz::ler(&bytes).unwrap(), esperado);
    assert_eq!(fdoz::ler(&bytes).unwrap().blocos[0].mesclagens, vec![m]);
    sessao.desfazer().unwrap();
    assert_eq!(sessao.documento(), &original);
}

#[test]
fn fdoz_duas_mesclagens_independentes_preservam_posicao() {
    let (documento, id) = modelo();
    let mut sessao = Sessao::com_documento(documento);
    let regioes = vec![
        MesclagemTabela { linha: 0, coluna: 0, colunas: 2 },
        MesclagemTabela { linha: 1, coluna: 1, colunas: 2 },
    ];
    sessao.tabela_mesclagens_definir(id, regioes.clone(), (0, 0)).unwrap();
    let bytes = fdoz::gravar(sessao.documento()).unwrap();
    assert_eq!(fdoz::ler(&bytes).unwrap().blocos[0].mesclagens, regioes);
}

#[test]
fn docx_emite_gridspan_e_celula_so_uma_vez() {
    let (documento, id) = modelo();
    let mut sessao = Sessao::com_documento(documento);
    sessao.tabela_mesclagens_definir(id, vec![
        MesclagemTabela { linha: 0, coluna: 0, colunas: 2 },
    ], (0, 0)).unwrap();
    let docx = docx::gravar(sessao.documento()).unwrap();
    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml").unwrap().read_to_string(&mut xml).unwrap();
    assert!(xml.contains("<w:gridSpan w:val=\"2\"/>"));
    assert_eq!(xml.matches("<w:gridSpan w:val=\"2\"/>").count(), 1);
    assert!(xml.contains("FeatherWorks"));
    assert!(xml.contains("Valor"));
}

#[test]
fn rejeita_malformacao_e_mescla_que_encobre_trechos() {
    let (documento, id) = modelo();
    let mut sessao = Sessao::com_documento(documento.clone());
    for regioes in [
        vec![MesclagemTabela { linha: 0, coluna: 0, colunas: 4 }],
        vec![MesclagemTabela { linha: 0, coluna: 0, colunas: 2 },
             MesclagemTabela { linha: 0, coluna: 1, colunas: 2 }],
        vec![MesclagemTabela { linha: 0, coluna: 1, colunas: 2 }],
    ] {
        assert!(sessao.tabela_mesclagens_definir(id, regioes, (0, 0)).is_err());
        assert_eq!(sessao.documento(), &documento);
    }
}
