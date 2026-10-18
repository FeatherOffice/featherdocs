//! CHT-016 — exemplo real Docs + Charts + persistência FDOZ.
 //! Uso: cargo run --manifest-path featherdocs/crates/featherdocs-io/Cargo.toml \
 //!   --example cht016_grafico_docs -j 2 -- /tmp/featheroffice-cht016
use featherdocs_core::modelo::{Conteudo, Documento, Trecho};
use featherdocs_core::sessao::Sessao;
use featherdocs_io::{base64, fdoz, grafico_docs::{self, Tipo}};

fn trecho(s: &str) -> Vec<Trecho> {
    vec![Trecho { texto: s.to_string(), ..Trecho::default() }]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pasta = std::env::args().nth(1).unwrap_or_else(|| "/tmp/featheroffice-cht016".into());
    std::fs::create_dir_all(&pasta)?;

    let mut doc = Documento::vazio();
    doc.acrescentar(Conteudo::Tabela {
        cabecalho: true,
        linhas: vec![
            vec![trecho("Mês"), trecho("Consumo"), trecho("Meta")],
            vec![trecho("Janeiro"), trecho("12,5"), trecho("10")],
            vec![trecho("Fevereiro"), trecho("15"), trecho("13")],
            vec![trecho("Março"), trecho("19"), trecho("16")],
        ],
    });
    let mut sessao = Sessao::com_documento(doc);
    grafico_docs::inserir_da_tabela(&mut sessao, Tipo::Colunas)?;
    let imagem_id = sessao.documento().blocos.iter().find(|b| {
        matches!(b.conteudo, Conteudo::Imagem { .. })
    }).ok_or("o gráfico não foi inserido")?.id;
    let origem = match &sessao.documento().blocos.iter().find(|b| b.id == imagem_id).unwrap().conteudo {
        Conteudo::Imagem { origem, .. } => origem.clone(),
        _ => unreachable!(),
    };
    if !grafico_docs::eh_grafico(&origem) {
        return Err("o SVG não manteve os dados editáveis".into());
    }
    let (mime, svg) = base64::de_data_url(&origem).ok_or("SVG sem data URL")?;
    if mime != "image/svg+xml" || !svg.starts_with(b"<svg") {
        return Err("gráfico SVG não foi renderizado corretamente".into());
    }
    std::fs::write(format!("{pasta}/grafico.svg"), &svg)?;

    let id_antes = imagem_id;
    sessao.ir_para_bloco(imagem_id);
    grafico_docs::atualizar_no_cursor(&mut sessao)?;
    if !sessao.documento().blocos.iter().any(|b| b.id == id_antes) {
        return Err("a atualização alterou a identidade do bloco".into());
    }
    let bytes = fdoz::gravar(sessao.documento())?;
    let lido = fdoz::ler(&bytes)?;
    if lido != *sessao.documento() {
        return Err("round-trip FDOZ divergente".into());
    }
    std::fs::write(format!("{pasta}/graficos.fdoz"), &bytes)?;
    eprintln!("CHT-016: OK; gráfico editável e round-trip FDOZ preservados em {pasta}");
    Ok(())
}
