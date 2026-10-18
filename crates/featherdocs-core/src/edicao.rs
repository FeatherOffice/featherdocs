//! Do bloco para a "fonte" que o usuário edita e de volta, e a troca de tipo.
//! É aqui que mora o atalho Markdown (`# `, `- `, `1. `, `> `, `---`…): o usuário
//! digita a marca e o bloco vira o tipo certo — a marca crua nunca fica no documento.

use crate::inline::{serializar, texto_puro};
use crate::modelo::{Celula, Conteudo, InicioDeSecao, Trecho};

/// O texto corrido de um bloco que tem texto em linha.
pub fn trechos_de(c: &Conteudo) -> Option<&Vec<Trecho>> {
    match c {
        Conteudo::Titulo { texto, .. } | Conteudo::Paragrafo { texto } | Conteudo::Item { texto, .. } | Conteudo::Citacao { texto } => Some(texto),
        _ => None,
    }
}

/// O nome do tipo (o mesmo que `tipo_definir` aceita).
pub fn nome_do_tipo(c: &Conteudo) -> String {
    match c {
        Conteudo::Titulo { nivel, .. } => format!("titulo{nivel}"),
        Conteudo::Paragrafo { .. } => "paragrafo".into(),
        Conteudo::Item { ordenado: true, .. } => "item_ordenado".into(),
        Conteudo::Item { .. } => "item".into(),
        Conteudo::Citacao { .. } => "citacao".into(),
        Conteudo::Codigo { .. } => "codigo".into(),
        Conteudo::Tabela { .. } => "tabela".into(),
        Conteudo::Imagem { .. } => "imagem".into(),
        Conteudo::Equacao { .. } => "equacao".into(),
        Conteudo::Sumario { .. } => "sumario".into(),
        Conteudo::Regua => "regua".into(),
        Conteudo::QuebraDePagina => "quebra_de_pagina".into(),
        Conteudo::QuebraDeColuna => "quebra_de_coluna".into(),
        Conteudo::QuebraDeSecao { inicio } => match inicio {
            InicioDeSecao::ProximaPagina => "secao_proxima",
            InicioDeSecao::Continua => "secao_continua",
            InicioDeSecao::PaginaPar => "secao_par",
            InicioDeSecao::PaginaImpar => "secao_impar",
        }
        .into(),
        Conteudo::Bibliografia { .. } => "bibliografia".into(),
        Conteudo::IndiceRemissivo => "indice_remissivo".into(),
    }
}

fn celula_para_fonte(c: &Celula) -> String {
    serializar(c).replace('|', "\\|")
}

/// A tabela como texto de barras (`| a | b |`), com a linha `---` depois do cabeçalho.
pub fn tabela_para_fonte(cabecalho: bool, linhas: &[Vec<Celula>]) -> String {
    let mut saida = Vec::new();
    for (i, l) in linhas.iter().enumerate() {
        let cels: Vec<String> = l.iter().map(celula_para_fonte).collect();
        saida.push(format!("| {} |", cels.join(" | ")));
        if i == 0 && cabecalho {
            saida.push(format!("|{}", " --- |".repeat(l.len().max(1))));
        }
    }
    saida.join("\n")
}

/// A fonte editável de um bloco.
pub fn fonte_de(c: &Conteudo) -> String {
    match c {
        Conteudo::Codigo { codigo, .. } => codigo.clone(),
        Conteudo::Equacao { latex } => latex.clone(),
        Conteudo::Tabela { cabecalho, linhas } => tabela_para_fonte(*cabecalho, linhas),
        Conteudo::Imagem { origem, legenda, .. } => format!("![{}]({origem})", serializar(legenda)),
        Conteudo::Sumario { niveis, .. } => format!("niveis: {niveis}"),
        _ => trechos_de(c).map(|t| serializar(t)).unwrap_or_default(),
    }
}

/// O mesmo conteúdo em outro tipo. Os nomes: `paragrafo`, `titulo1`…`titulo6`, `item`,
/// `item_ordenado`, `citacao`, `codigo`, `equacao`, `tabela`, `regua`, `sumario`, `quebra_de_pagina`,
/// `quebra_de_coluna`, `secao_proxima`, `secao_continua`, `secao_par`, `secao_impar`.
pub fn converter(c: &Conteudo, tipo: &str) -> Result<Conteudo, String> {
    let texto = trechos_de(c).cloned().unwrap_or_else(|| vec![Trecho::simples(&fonte_de(c))]);
    let nivel = if let Conteudo::Item { nivel, .. } = c { *nivel } else { 0 };
    Ok(match tipo {
        "paragrafo" => Conteudo::Paragrafo { texto },
        "item" => Conteudo::Item { ordenado: false, nivel, texto },
        "item_ordenado" => Conteudo::Item { ordenado: true, nivel, texto },
        "citacao" => Conteudo::Citacao { texto },
        "codigo" => Conteudo::Codigo { linguagem: String::new(), codigo: texto_puro(&texto) },
        "equacao" => Conteudo::Equacao { latex: texto_puro(&texto) },
        "tabela" => tabela_nova(texto),
        "regua" => Conteudo::Regua,
        "quebra_de_pagina" => Conteudo::QuebraDePagina,
        "quebra_de_coluna" => Conteudo::QuebraDeColuna,
        "secao_proxima" => Conteudo::QuebraDeSecao { inicio: InicioDeSecao::ProximaPagina },
        "secao_continua" => Conteudo::QuebraDeSecao { inicio: InicioDeSecao::Continua },
        "secao_par" => Conteudo::QuebraDeSecao { inicio: InicioDeSecao::PaginaPar },
        "secao_impar" => Conteudo::QuebraDeSecao { inicio: InicioDeSecao::PaginaImpar },
        "sumario" => Conteudo::sumario(3),
        "bibliografia" => Conteudo::Bibliografia { todas: false },
        "indice_remissivo" => Conteudo::IndiceRemissivo,
        "imagem" => Conteudo::Imagem { origem: texto_puro(&texto), legenda: Vec::new(), largura: 0.0, altura: 0.0 },
        t if t.starts_with("titulo") => {
            let n: u8 = t[6..].parse().ok().filter(|n| (1..=6).contains(n)).ok_or_else(|| format!("Nível de título inválido: \"{t}\"."))?;
            Conteudo::Titulo { nivel: n, texto }
        }
        outro => return Err(format!("Não conheço o tipo de bloco \"{outro}\".")),
    })
}

/// Uma tabela 3×3 com cabeçalho, a primeira célula com o texto dado.
fn tabela_nova(primeira: Vec<Trecho>) -> Conteudo {
    let mut linhas: Vec<Vec<Celula>> = vec![vec![Vec::new(); 3]; 3];
    linhas[0][0] = primeira;
    Conteudo::Tabela { cabecalho: true, linhas }
}

/// Uma tabela só de texto puro (o que vem de uma planilha).
pub fn tabela_de_texto(linhas: &[Vec<String>], cabecalho: bool) -> Conteudo {
    let linhas = linhas.iter().map(|l| l.iter().map(|c| if c.is_empty() { Vec::new() } else { vec![Trecho::simples(c)] }).collect()).collect();
    Conteudo::Tabela { cabecalho, linhas }
}

/// As células de uma tabela em texto puro (o que vai para uma planilha).
pub fn tabela_em_texto(linhas: &[Vec<Celula>]) -> Vec<Vec<String>> {
    linhas.iter().map(|l| l.iter().map(|c| texto_puro(c)).collect()).collect()
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::inline::analisar;

    #[test]
    fn converter_titulo_para_paragrafo_mantem_o_texto() {
        let t = Conteudo::Titulo { nivel: 1, texto: analisar("Oi **você**") };
        let p = converter(&t, "paragrafo").unwrap();
        assert_eq!(fonte_de(&p), "Oi **você**");
        assert!(converter(&t, "titulo9").is_err());
    }

    #[test]
    fn tabela_para_texto_de_barras() {
        let linhas = vec![vec![analisar("a"), analisar("b")], vec![analisar("1"), analisar("**2**")]];
        assert_eq!(tabela_para_fonte(true, &linhas), "| a | b |\n| --- | --- |\n| 1 | **2** |");
    }

    #[test]
    fn converter_codigo_em_paragrafo_conta_os_mesmos_caracteres() {
        let c = Conteudo::Codigo { linguagem: String::new(), codigo: "a\nb".into() };
        let p = converter(&c, "paragrafo").unwrap();
        assert_eq!(crate::alvo::tamanho_alvo(&p, None), 3);
    }
}
