//! APP-007: gera FDOZ, DOCX e SVG usando o mesmo gráfico FeatherDocs/Charts.
//! Uso (na raiz do repo):
//! cargo run --manifest-path featherdocs/crates/featherdocs-io/Cargo.toml \
//!   --example app007_roundtrip_docs -j 2 -- /tmp/featheroffice-app007

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use featherdocs_core::modelo::{Conteudo, Documento, Trecho};
use featherdocs_core::sessao::Sessao;
use featherdocs_io::{base64, docx, fdoz, grafico_docs::{self, Tipo}};

fn texto(s: &str) -> Vec<Trecho> {
    vec![Trecho::simples(s)]
}

fn main() -> Result<(), Box<dyn Error>> {
    let pasta: PathBuf = std::env::args_os()
        .nth(1).map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/featheroffice-app007"));
    fs::create_dir_all(&pasta)?;

    let mut documento = Documento::vazio();
    documento.meta.titulo = "Gráfico FeatherDocs — QA APP-007".into();
    documento.acrescentar(Conteudo::Paragrafo {
        texto: texto("Medições por mês — comparação com meta."),
    });
    let tabela_id = documento.acrescentar(Conteudo::Tabela {
        cabecalho: true,
        linhas: vec![
            vec![texto("Mês"), texto("Consumo"), texto("Meta")],
            vec![texto("Janeiro"), texto("12,5"), texto("10")],
            vec![texto("Fevereiro"), texto("15,2"), texto("13")],
            vec![texto("Março"), texto("19"), texto("16")],
        ],
    });
    let mut sessao = Sessao::com_documento(documento);
    sessao.ir_para_bloco(tabela_id);
    grafico_docs::inserir_da_tabela(&mut sessao, Tipo::Barras)?;

    let original = sessao.documento().clone();
    let fdoz_bytes = fdoz::gravar(&original)?;
    let fdoz_lido = fdoz::ler(&fdoz_bytes)?;
    if original != fdoz_lido {
        return Err("FDOZ não preservou a árvore de blocos original".into());
    }
    fs::write(pasta.join("app007-grafico.fdoz"), &fdoz_bytes)?;

    let docx_bytes = docx::gravar(&original)?;
    let docx_lido = docx::ler(&docx_bytes)?;
    let (imagem_id, origem) = docx_lido.blocos.iter().find_map(|b| match &b.conteudo {
        Conteudo::Imagem { origem, .. } => Some((b.id, origem.clone())),
        _ => None,
    }).ok_or("DOCX não contém o gráfico")?;
    if !grafico_docs::eh_grafico(&origem) {
        return Err("DOCX perdeu o vínculo editável do gráfico".into());
    }
    let (mime, svg) = base64::de_data_url(&origem)
        .ok_or("DOCX perdeu a data URL da imagem")?;
    if mime != "image/svg+xml" || !svg.windows(4).any(|parte| parte == b"<svg") {
        return Err("DOCX não preservou a imagem SVG".into());
    }
    fs::write(pasta.join("app007-grafico.svg"), &svg)?;
    fs::write(pasta.join("app007-grafico.docx"), &docx_bytes)?;

    // O gráfico deve continuar editável depois de reabrir o DOCX.
    let mut reaberta = Sessao::com_documento(docx_lido);
    reaberta.ir_para_bloco(imagem_id);
    grafico_docs::atualizar_no_cursor(&mut reaberta)?;
    let atualizada = reaberta.documento().blocos.iter()
        .find(|b| b.id == imagem_id)
        .ok_or("a atualização trocou o ID da imagem")?;
    if !matches!(&atualizada.conteudo,
        Conteudo::Imagem { origem, .. } if grafico_docs::eh_grafico(origem))
    {
        return Err("DOCX perdeu a atualização do gráfico".into());
    }

    println!("APP-007 OK: FDOZ, DOCX, SVG e atualização após reabertura.");
    println!("Arquivos para inspeção: {}", pasta.display());
    Ok(())
}
