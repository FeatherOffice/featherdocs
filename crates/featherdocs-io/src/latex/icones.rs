//! Os ícones do `fontawesome5`/`fontawesome6` (`\faCircleInfo`…) como desenhos nossos, na grade de 24
//! do catálogo de ícones da suíte, na cor do texto em volta. Só os que os modelos usam.

/// O SVG do ícone `\nome` na cor `cor` (`#rrggbb`); `None` se não temos esse ícone.
pub(super) fn svg(nome: &str, cor: &str) -> Option<String> {
    let traco = |d: &str, c: &str, w: f32| format!(r#"<path d="{d}" fill="none" stroke="{c}" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round"/>"#);
    let corpo = match nome {
        "faCircleInfo" | "faInfoCircle" => format!(r#"<circle cx="12" cy="12" r="10.5" fill="{cor}"/>{}"#, traco("M12 11v6 M12 7.4h.01", "#ffffff", 2.6)),
        "faCircleExclamation" | "faExclamationCircle" => format!(r#"<circle cx="12" cy="12" r="10.5" fill="{cor}"/>{}"#, traco("M12 6.5v6.5 M12 17h.01", "#ffffff", 2.6)),
        "faTriangleExclamation" | "faExclamationTriangle" => format!(r#"<path d="M12 2.5L1.5 21h21Z" fill="{cor}" stroke="{cor}" stroke-width="2" stroke-linejoin="round"/>{}"#, traco("M12 9v5.5 M12 17.8h.01", "#ffffff", 2.4)),
        "faExclamation" => traco("M12 3.5v11 M12 20h.01", cor, 3.2),
        "faLightbulb" => traco("M9.5 18h5 M10.5 21h3 M12 3a6 6 0 0 0-3.6 10.8c.7.5 1.1 1.3 1.1 2.1v.1h5v-.1c0-.8.4-1.6 1.1-2.1A6 6 0 0 0 12 3z", cor, 2.0),
        _ => return None,
    };
    Some(format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">{corpo}</svg>"#))
}
