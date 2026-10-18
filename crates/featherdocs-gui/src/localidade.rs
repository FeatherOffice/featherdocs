//! Textos da interface nativa mantidos no JSON do aplicativo.
use std::sync::OnceLock;
pub fn tr(chave: &str) -> &'static str {
    static DADOS: OnceLock<serde_json::Value> = OnceLock::new();
    DADOS
        .get_or_init(|| {
            serde_json::from_str(include_str!("../dados/pt-BR.json")).expect("Localidade inválida")
        })
        .get(chave)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Texto indisponível")
}
