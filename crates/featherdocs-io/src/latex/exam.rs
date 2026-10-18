//! A classe `exam` (provas e listas): as questões, as partes e as alternativas viram as listas do
//! `enumitem`; o espaço da resposta (`solutionorlines`, `solutionorbox`, `\fillin`) vira o desenho
//! dele; o `\gradetable` vira a tabela de pontos. Sem `\printanswers` a resposta não aparece.

use super::{dimensao, grupo, opcional, Classe};

/// O corpo com os comandos da `exam` trocados pelo que o leitor já entende.
pub(super) fn reescrever(corpo: &str, preambulo: &str, classe: Classe) -> String {
    let respostas = preambulo.contains("\\printanswers") || corpo.contains("\\printanswers");
    let (um, varios) = argumentos(preambulo, "\\pointpoints", 2).map_or(("point".to_string(), "points".to_string()), |g| (g[0].clone(), g[1].clone()));
    let comprimento = |nome: &str, padrao: f32| {
        [format!("\\setlength{{\\{nome}}}"), format!("\\setlength\\{nome}")]
            .iter()
            .find_map(|c| preambulo.rfind(c.as_str()).and_then(|i| grupo(&preambulo[i + c.len()..])).and_then(|(v, _)| dimensao(v, classe)))
            .unwrap_or(padrao)
    };
    let (passo, linha_do_fillin) = (comprimento("linefillheight", 18.07), comprimento("fillinlinelength", 72.27));
    let tabela = tabela_de_pontos(corpo, preambulo);

    let mut s = corpo.to_string();
    for nome in ["solutionorlines", "solutionordottedlines", "solutionorbox", "solution"] {
        s = trocar_ambiente(&s, nome, |altura, resposta| {
            if respostas {
                format!("\\par\\noindent\\textbf{{Resposta:}}\\enspace {resposta}\\par ")
            } else if nome == "solution" {
                String::new()
            } else {
                // Logo abaixo do texto da questão (uma linha nova do mesmo item, sem vão de parágrafo).
                let h = altura.and_then(|a| dimensao(a, classe)).unwrap_or(72.27);
                if nome == "solutionorbox" {
                    format!("\\hfill\\\\\\featherexamcaixa{{{h}}}")
                } else {
                    format!("\\hfill\\\\\\featherexamlinhas{{{h}}}{{{passo}}}")
                }
            }
        });
    }
    s = trocar_comando(&s, "fillin", |r| {
        let (resposta, a) = opcional(r).map_or(("", 0), |(o, k)| (o, k));
        let (largura, b) = opcional(&r[a..]).map_or((None, 0), |(o, k)| (Some(o), k));
        let largura = largura.and_then(|l| dimensao(l, classe)).unwrap_or(linha_do_fillin);
        let texto = if respostas { format!("\\underline{{{resposta}}}") } else { format!("\\rule{{{largura}pt}}{{0.4pt}}") };
        (texto, a + b)
    });
    s = trocar_comando(&s, "question", |r| {
        let (pontos, k) = opcional(r).map_or(("", 0), |(o, k)| (o.trim(), k));
        let rotulo = match pontos.replace(',', ".").parse::<f32>() {
            Ok(p) => format!("({pontos} {})\\enspace ", if p == 1.0 { &um } else { &varios }),
            Err(_) => String::new(),
        };
        (format!("\\item {rotulo}"), k)
    });
    // O `\topsep` da classe sem o `\partopsep` (a `exam` o zera).
    let questoes = format!("enumerate}}[label=\\arabic*.,resume,topsep={}pt]", (classe.tamanhos[4] * 0.8).round());
    for (ambiente, lista) in [
        ("questions", questoes.as_str()),
        ("parts", "enumerate}[label=(\\alph*),topsep=0pt]"),
        ("subparts", "enumerate}[label=\\roman*.,topsep=0pt]"),
        ("choices", "enumerate}[label=\\Alph*.,topsep=0pt]"),
        ("oneparchoices", "enumerate}[label=\\Alph*.,topsep=0pt]"),
        ("checkboxes", "itemize}[label=\u{25EF},topsep=0pt]"),
        ("oneparcheckboxes", "itemize}[label=\u{25EF},topsep=0pt]"),
    ] {
        let (tipo, _) = lista.split_once('}').unwrap_or((lista, ""));
        s = s.replace(&format!("\\begin{{{ambiente}}}"), &format!("\\begin{{{lista}")).replace(&format!("\\end{{{ambiente}}}"), &format!("\\end{{{tipo}}}"));
    }
    for item in ["part", "subpart", "choice", "correctchoice", "CorrectChoice"] {
        s = trocar_comando(&s, item, |_| ("\\item ".to_string(), 0));
    }
    // `\fullwidth{…}`: fora do recuo das questões (a lista fecha e a numeração continua depois).
    s = trocar_comando(&s, "fullwidth", |r| match grupo(r) {
        Some((g, k)) => (format!("\\end{{enumerate}}\\par\\noindent {{{g}}}\\par\\begin{{enumerate}}[label=\\arabic*.,resume]"), k),
        None => (String::new(), 0),
    });
    // `\\gradetable[h][questions]`: as duas opções são lidas e ignoradas (a tabela sai sempre deitada).
    s = trocar_comando(&s, "gradetable", |r| {
        let a = opcional(r).map_or(0, |(_, k)| k);
        (tabela.clone(), a + opcional(&r[a..]).map_or(0, |(_, k)| k))
    });
    for nada in ["shadedsolutions", "framedsolutions", "unframedsolutions", "printanswers", "noprintanswers", "addpoints", "noaddpoints"] {
        s = trocar_comando(&s, nada, |_| (String::new(), 0));
    }
    s
}

/// `\comando{a}{b}…` no texto: os `n` grupos.
fn argumentos(texto: &str, comando: &str, n: usize) -> Option<Vec<String>> {
    let mut r = &texto[texto.rfind(comando)? + comando.len()..];
    let mut saida = Vec::new();
    for _ in 0..n {
        let (g, k) = grupo(r)?;
        saida.push(g.trim().to_string());
        r = &r[k..];
    }
    Some(saida)
}

/// Troca cada `\nome` (não seguido de letra) pelo que `f` devolve a partir do texto de depois; `f`
/// diz também quantos bytes de depois consumiu.
fn trocar_comando(s: &str, nome: &str, mut f: impl FnMut(&str) -> (String, usize)) -> String {
    let marca = format!("\\{nome}");
    let mut saida = String::with_capacity(s.len());
    let mut resto = s;
    while let Some(i) = resto.find(&marca) {
        let depois = &resto[i + marca.len()..];
        saida.push_str(&resto[..i]);
        if depois.starts_with(|c: char| c.is_ascii_alphabetic() || c == '@') {
            saida.push_str(&marca);
            resto = depois;
            continue;
        }
        let (texto, k) = f(depois);
        saida.push_str(&texto);
        resto = &depois[k..];
    }
    saida.push_str(resto);
    saida
}

/// Troca cada `\begin{nome}[altura] … \end{nome}` pelo que `f` faz da altura e do conteúdo.
fn trocar_ambiente(s: &str, nome: &str, f: impl Fn(Option<&str>, &str) -> String) -> String {
    let (abre, fecha) = (format!("\\begin{{{nome}}}"), format!("\\end{{{nome}}}"));
    let mut saida = String::with_capacity(s.len());
    let mut resto = s;
    while let Some(i) = resto.find(&abre) {
        let depois = &resto[i + abre.len()..];
        let Some(fim) = depois.find(&fecha) else { break };
        let dentro = &depois[..fim];
        let (altura, k) = opcional(dentro).map_or((None, 0), |(o, k)| (Some(o), k));
        // A linha em branco antes do ambiente não abre parágrafo: a resposta fica colada à questão.
        saida.push_str(resto[..i].trim_end());
        saida.push_str(&f(altura, dentro[k..].trim()));
        resto = &depois[fim + fecha.len()..];
    }
    saida.push_str(resto);
    saida
}

/// O `\gradetable[h]`: uma coluna por questão, com os pontos de cada uma e o total.
fn tabela_de_pontos(corpo: &str, preambulo: &str) -> String {
    let mut pontos: Vec<f32> = Vec::new();
    for (i, _) in corpo.match_indices("\\question") {
        let depois = &corpo[i + 9..];
        if depois.starts_with(|c: char| c.is_ascii_alphabetic()) {
            continue;
        }
        pontos.push(opcional(depois).and_then(|(o, _)| o.trim().replace(',', ".").parse().ok()).unwrap_or(0.0));
    }
    let palavra = |cmd: &str, padrao: &str| argumentos(preambulo, cmd, 1).map_or(padrao.to_string(), |g| g[0].clone());
    let numero = |p: f32| if p.fract() == 0.0 { format!("{p:.0}") } else { format!("{p}").replace('.', ",") };
    let n = pontos.len();
    // Cada questão numa coluna de `\cellwidth` (2 em), centrada.
    let mut t = format!("\\begin{{tabular}}{{|l|{}c|}}\\hline\n", ">{\\centering}p{2em}|".repeat(n));
    t.push_str(&palavra("\\hqword", "Question"));
    (1..=n).for_each(|i| t.push_str(&format!(" & {i}")));
    t.push_str(&format!(" & {}\\\\ \\hline\n{}", palavra("\\htword", "Total"), palavra("\\hpword", "Points")));
    pontos.iter().for_each(|p| t.push_str(&format!(" & {}", numero(*p))));
    t.push_str(&format!(" & {}\\\\ \\hline\n{}", numero(pontos.iter().sum()), palavra("\\hsword", "Score")));
    t.push_str(&" & ".repeat(n + 1));
    t.push_str("\\\\ \\hline\n\\end{tabular}");
    t
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn questoes_partes_e_alternativas_viram_listas() {
        let pre = "\\pointpoints{ponto}{pontos}\\hqword{Questões}";
        let corpo = "\\begin{questions}\\question[2] A?\\begin{parts}\\part B\\end{parts}\\question C\\begin{choices}\\choice x\\correctchoice y\\end{choices}\\end{questions}\\gradetable[h]";
        let s = reescrever(corpo, pre, Classe::default());
        assert!(s.starts_with("\\begin{enumerate}[label=\\arabic*.,resume,topsep=8pt]\\item (2 pontos)\\enspace  A?\\begin{enumerate}[label=(\\alph*),topsep=0pt]\\item  B\\end{enumerate}"), "{s}");
        assert!(s.contains("\\item  C\\begin{enumerate}[label=\\Alph*.,topsep=0pt]\\item  x\\item  y\\end{enumerate}"), "{s}");
        assert!(s.contains("Questões & 1 & 2 & Total") && s.contains(" & 2 & 0 & 2\\\\"), "{s}");
    }

    #[test]
    fn espaco_da_resposta_sem_e_com_gabarito() {
        let corpo = "\\begin{solutionorlines}[2cm]Azul.\\end{solutionorlines}\\fillin[V][1cm]";
        let s = reescrever(corpo, "\\setlength\\linefillheight{.6cm}", Classe::default());
        assert!(s.contains("\\\\\\featherexamlinhas{56.69") && s.contains("\\rule{28.34") && !s.contains("Azul"), "{s}");
        let s = reescrever(corpo, "\\printanswers", Classe::default());
        assert!(s.contains("\\textbf{Resposta:}\\enspace Azul.") && s.contains("\\underline{V}"), "{s}");
    }
}
