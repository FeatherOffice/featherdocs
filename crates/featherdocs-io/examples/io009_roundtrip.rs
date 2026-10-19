//! IO-009: FDoz ZIP/Arrow -> ler -> regravar -> reler.
//! cargo run --manifest-path featherdocs/crates/featherdocs-io/Cargo.toml --example io009_roundtrip -- /tmp/io009.fdoz
use featherdocs_core::{inline::analisar, modelo::{Conteudo, Documento}};
use featherdocs_io::fdoz;

fn executar(destino: &std::path::Path) -> Result<(), String> {
    let mut documento = Documento::vazio();
    documento.meta.titulo = "Documento ⚡ UTF-8".into();
    documento.meta.autor = "Equipe FeatherWorks".into();
    documento.acrescentar(Conteudo::Titulo {
        nivel: 1,
        texto: analisar("Documento ⚡ UTF-8"),
    });
    documento.acrescentar(Conteudo::Paragrafo {
        texto: analisar("Texto em português: ação e café"),
    });
    documento.acrescentar(Conteudo::Tabela {
        cabecalho: true,
        linhas: vec![
            vec![analisar("Mês"), analisar("Valor")],
            vec![analisar("Janeiro"), analisar("150,25")],
            vec![analisar("Fevereiro"), analisar("-18,00")],
        ],
    });
    let original = documento;
    let bytes = fdoz::gravar(&original)?;
    if !bytes.starts_with(b"PK\x03\x04") {
        return Err("FDoz não começa com cabeçalho ZIP".into());
    }
    let recuperada = fdoz::ler(&bytes)?;
    if recuperada != original {
        return Err("FDoz alterou campos no primeiro round-trip".into());
    }
    let novamente = fdoz::gravar(&recuperada)?;
    let segunda = fdoz::ler(&novamente)?;
    if segunda != original {
        return Err("FDoz alterou campos ao regravar o arquivo".into());
    }
    if fdoz::ler(b"lixo, nao zip").is_ok() {
        return Err("FDoz aceitou bytes inválidos".into());
    }
    std::fs::write(destino, &novamente).map_err(|e| e.to_string())?;
    eprintln!("IO-009 fdoz: {} bytes, {} blocos, round-trip semantico OK", novamente.len(), segunda.blocos.len());
    Ok(())
}

fn main() {
    let destino = std::env::args().nth(1).unwrap_or_else(|| "/tmp/io009.fdoz".into());
    if let Err(e) = executar(std::path::Path::new(&destino)) {
        eprintln!("ERRO IO-009 fdoz: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn io009_fdoz_arquivo_e_roundtrip_real() {
        let arquivo = std::env::temp_dir().join(format!("io009-{}-docs.fdoz", std::process::id()));
        executar(&arquivo).unwrap();
        assert!(fdoz::ler(&std::fs::read(&arquivo).unwrap()).is_ok());
        std::fs::remove_file(arquivo).unwrap();
    }
}
