//! A ponte de tabela com o FeatherSheet: células em CSV (UTF-8, vírgula, aspas
//! duplas quando preciso). O FeatherDocs grava a tabela num arquivo temporário,
//! o FeatherSheet o abre e salva, e o FeatherDocs lê as células de volta. Nenhum
//! app importa código do outro: o contrato é o arquivo.

/// Células → texto CSV.
pub fn gravar(linhas: &[Vec<String>]) -> String {
    let mut s = String::new();
    for l in linhas {
        let cels: Vec<String> = l.iter().map(|c| campo(c)).collect();
        s.push_str(&cels.join(","));
        s.push('\n');
    }
    s
}

fn campo(c: &str) -> String {
    if c.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", c.replace('"', "\"\""))
    } else {
        c.to_string()
    }
}

/// Texto CSV → células. Aceita `,` ou `;` (o que o FeatherSheet/Excel gravar em português) e aspas.
pub fn ler(texto: &str) -> Vec<Vec<String>> {
    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
    let primeira = texto.lines().next().unwrap_or("");
    let sep = if primeira.matches(';').count() > primeira.matches(',').count() { ';' } else { ',' };
    let mut linhas: Vec<Vec<String>> = Vec::new();
    let mut linha: Vec<String> = Vec::new();
    let mut atual = String::new();
    let mut aspas = false;
    let mut cs = texto.chars().peekable();
    while let Some(c) = cs.next() {
        match (aspas, c) {
            (true, '"') if cs.peek() == Some(&'"') => {
                atual.push('"');
                cs.next();
            }
            (true, '"') => aspas = false,
            (true, _) => atual.push(c),
            (false, '"') => aspas = true,
            (false, c) if c == sep => linha.push(std::mem::take(&mut atual)),
            (false, '\r') => {}
            (false, '\n') => {
                linha.push(std::mem::take(&mut atual));
                linhas.push(std::mem::take(&mut linha));
            }
            (false, _) => atual.push(c),
        }
    }
    if !atual.is_empty() || !linha.is_empty() {
        linha.push(atual);
        linhas.push(linha);
    }
    linhas
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn ida_e_volta_com_virgula_e_aspas() {
        let l = vec![vec!["a,b".to_string(), "diz \"oi\"".to_string()], vec!["1".to_string(), String::new()]];
        assert_eq!(ler(&gravar(&l)), l);
    }

    #[test]
    fn ponto_e_virgula_do_excel_brasileiro() {
        assert_eq!(ler("a;b\n1;2\n"), vec![vec!["a", "b"], vec!["1", "2"]]);
    }
}
