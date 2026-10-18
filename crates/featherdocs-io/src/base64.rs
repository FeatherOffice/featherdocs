//! API histórica Base64 do FeatherDocs.
//!
//! O algoritmo comum agora mora em `featheroffice-image::base64`.
//! Reexporta os mesmos nomes, preservando chamadas de FDOZ, DOCX,
//! renderização de gráficos SVG e consumidores antigos.

pub use feather_image::base64::{codificar, de_data_url, decodificar, para_data_url};

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn api_legada_preserva_data_url_e_unicode() {
        let html = "<svg>café Ω</svg>";
        let encoded = codificar(html.as_bytes());
        assert_eq!(decodificar(&encoded), Some(html.as_bytes().to_vec()));
        let url = para_data_url("image/svg+xml", html.as_bytes());
        assert_eq!(de_data_url(&url), Some(("image/svg+xml".into(), html.as_bytes().to_vec())));
        assert!(de_data_url("data:image/svg+xml,nao-base64").is_none());
    }
}
