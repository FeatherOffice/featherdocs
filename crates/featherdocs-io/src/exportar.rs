//! Exportar para texto legível sem depender de nenhum programa da suíte:
//! Markdown e LaTeX.

use featherdocs_core::edicao::tabela_para_fonte;
use featherdocs_core::inline::{serializar, texto_puro};
use featherdocs_core::modelo::{Bloco, Conteudo, Documento, EstiloDeCitacao, Marca, Numeracao, Trecho, CODIGO, ITALICO, MATEMATICA, NEGRITO, SUBLINHADO, TACHADO};

/// As linhas `[de, ate)` (contadas de 0) que um bloco ocupa no texto exportado.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct LinhasDoBloco {
    pub id: u64,
    pub de: usize,
    pub ate: usize,
}

/// O texto exportado e onde cada bloco foi parar nele (para o modo dividido ligar código e página).
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Mapa {
    pub texto: String,
    pub blocos: Vec<LinhasDoBloco>,
}

impl Mapa {
    fn linhas(&self) -> usize {
        self.texto.matches('\n').count()
    }

    fn solto(&mut self, s: &str) {
        self.texto.push_str(s);
    }

    fn bloco(&mut self, id: u64, s: &str) {
        let de = self.linhas();
        self.texto.push_str(s);
        self.blocos.push(LinhasDoBloco { id, de, ate: de + s.matches('\n').count() + 1 });
    }
}

/// O documento como Markdown. Sumário vira `[TOC]`; fórmula, `$$ … $$`.
/// A equação já é um ambiente de destaque inteiro (`equation`, `align`…, com ou sem estrela): no
/// LaTeX e no Markdown ela sai crua, sem `\[…\]`/`$$…$$` em volta.
fn ambiente_de_exibicao(latex: &str) -> bool {
    let Some(r) = latex.trim_start().strip_prefix("\\begin{") else { return false };
    let nome = r.split('}').next().unwrap_or("").trim_end_matches('*');
    matches!(nome, "equation" | "align" | "gather" | "multline" | "eqnarray" | "flalign" | "alignat" | "xalignat" | "dmath" | "dgroup" | "dseries" | "empheq")
}

pub fn markdown(doc: &Documento) -> String {
    markdown_com_mapa(doc).texto
}

/// Uma cópia do documento com as notas numeradas em ordem (1, 2, 3… as de rodapé e as de fim, cada
/// uma na sua conta): os rótulos que o Markdown usa nas chamadas e nas definições.
fn com_notas_numeradas(doc: &Documento) -> Documento {
    let mut d = doc.clone();
    let (mut rodape, mut fim) = (0, 0);
    for b in &mut d.blocos {
        for t in b.conteudo.textos_mut() {
            for tr in t.iter_mut() {
                if let Some(Marca::Nota { fim: f, .. }) = tr.marca.as_deref() {
                    let n = if *f { &mut fim } else { &mut rodape };
                    *n += 1;
                    tr.rotulo = n.to_string();
                }
            }
        }
    }
    d
}

/// As definições das notas no fim do Markdown: `[^1]: texto`.
fn definicoes_das_notas(doc: &Documento) -> String {
    let mut s = String::new();
    for b in &doc.blocos {
        for t in b.conteudo.textos() {
            for tr in t {
                if let Some(Marca::Nota { fim, texto }) = tr.marca.as_deref() {
                    s.push_str(&format!("[^{}{}]: {}\n", if *fim { "fim-" } else { "" }, tr.rotulo, serializar(texto)));
                }
            }
        }
    }
    s
}

/// O documento como ficaria aceitando todas as sugestões (o Markdown e o LaTeX não têm controle de
/// alterações: sai o texto final). Sem sugestões, nada muda.
fn sem_sugestoes(doc: &Documento) -> std::borrow::Cow<'_, Documento> {
    let tem = doc.blocos.iter().flat_map(|b| b.conteudo.textos()).flatten().any(|t| t.rev.is_some());
    if !tem {
        return std::borrow::Cow::Borrowed(doc);
    }
    let mut d = doc.clone();
    for t in d.blocos.iter_mut().flat_map(|b| b.conteudo.textos_mut()) {
        t.retain(|x| !matches!(x.rev.as_deref(), Some(r) if r.tipo == featherdocs_core::modelo::TipoDeRevisao::Remocao));
        for x in t.iter_mut() {
            x.rev = None;
        }
        featherdocs_core::texto::normalizar(t);
    }
    std::borrow::Cow::Owned(d)
}

/// Como `markdown`, com as linhas de cada bloco.
pub fn markdown_com_mapa(doc: &Documento) -> Mapa {
    let doc = &com_notas_numeradas(&sem_sugestoes(doc));
    let mut m = Mapa::default();
    let mut anterior_item = false;
    for b in &doc.blocos {
        let item = matches!(b.conteudo, Conteudo::Item { .. });
        if !m.texto.is_empty() && !(item && anterior_item) {
            m.solto("\n");
        }
        anterior_item = item;
        m.bloco(b.id, &bloco_markdown(&b.conteudo));
        m.solto("\n");
    }
    let notas = definicoes_das_notas(doc);
    if !notas.is_empty() {
        m.solto("\n");
        m.solto(&notas);
    }
    let fontes = fontes_em_comentario(doc);
    if !fontes.is_empty() {
        m.solto("\n");
        m.solto(&fontes);
    }
    m
}

/// O Markdown de um bloco sozinho (a "impressão digital" que o modo dividido compara).
pub fn bloco_markdown(c: &Conteudo) -> String {
    match c {
        Conteudo::Titulo { nivel, texto } => format!("{} {}", "#".repeat(usize::from((*nivel).clamp(1, 6))), serializar(texto)),
        Conteudo::Paragrafo { texto } => serializar(texto),
        Conteudo::Item { ordenado, nivel, texto } => format!("{}{} {}", "  ".repeat(usize::from(*nivel)), if *ordenado { "1." } else { "-" }, serializar(texto)),
        Conteudo::Citacao { texto } => format!("> {}", serializar(texto)),
        Conteudo::Codigo { linguagem, codigo } => format!("```{linguagem}\n{codigo}\n```"),
        Conteudo::Tabela { cabecalho, linhas } => tabela_para_fonte(*cabecalho, linhas),
        Conteudo::Imagem { origem, legenda, .. } => format!("![{}]({origem})", serializar(legenda)),
        // Ambiente inteiro (`\begin{align}…`): sai cru, como o pandoc/MathJax esperam; o resto, entre `$$`.
        Conteudo::Equacao { latex } if ambiente_de_exibicao(latex) => latex.clone(),
        Conteudo::Equacao { latex } => format!("$$\n{latex}\n$$"),
        Conteudo::Sumario { lista: 1, .. } => "[LOF]".to_string(),
        Conteudo::Sumario { lista: 2, .. } => "[LOT]".to_string(),
        Conteudo::Sumario { .. } => "[TOC]".to_string(),
        Conteudo::Regua => "---".to_string(),
        Conteudo::QuebraDePagina => "<div style=\"page-break-after: always\"></div>".to_string(),
        Conteudo::Bibliografia { todas: false } => "[REFERENCIAS]".to_string(),
        Conteudo::Bibliografia { todas: true } => "[REFERENCIAS: TODAS]".to_string(),
        Conteudo::IndiceRemissivo => "[INDICE]".to_string(),
    }
}

/// O começo do comentário que guarda as fontes no Markdown (o leitor procura por ele).
pub const MARCA_DAS_FONTES: &str = "<!-- FeatherDocs: fontes";

/// As fontes e o estilo num comentário HTML no fim do Markdown (em BibTeX), para o arquivo voltar
/// com as citações inteiras; quem lê com outro programa não vê nada.
fn fontes_em_comentario(doc: &Documento) -> String {
    let b = &doc.bibliografia;
    if b.fontes.is_empty() {
        return String::new();
    }
    format!("{MARCA_DAS_FONTES} estilo={}\n{}-->\n", nome_do_estilo(b.estilo), crate::bib::gravar(&b.fontes).replace("--", "\\-\\-"))
}

/// O nome do estilo de citação (o mesmo que o motor aceita).
pub fn nome_do_estilo(e: EstiloDeCitacao) -> &'static str {
    match e {
        EstiloDeCitacao::Abnt => "abnt",
        EstiloDeCitacao::Apa => "apa",
        EstiloDeCitacao::Vancouver => "vancouver",
        EstiloDeCitacao::Ieee => "ieee",
        EstiloDeCitacao::Chicago => "chicago",
    }
}

fn escapar_tex(t: &str) -> String {
    let mut s = String::with_capacity(t.len());
    for c in t.chars() {
        match c {
            '\\' => s.push_str("\\textbackslash{}"),
            '&' | '%' | '$' | '#' | '_' | '{' | '}' => {
                s.push('\\');
                s.push(c);
            }
            '~' => s.push_str("\\textasciitilde{}"),
            '^' => s.push_str("\\textasciicircum{}"),
            _ => s.push(c),
        }
    }
    s
}

/// Um objeto preso ao texto em LaTeX: `\footnote`, `\endnote`, `\cite`, `\index`, ou o campo.
fn objeto_tex(t: &Trecho, m: &Marca) -> String {
    match m {
        Marca::Nota { fim: false, texto } => format!("\\footnote{{{}}}", texto_tex(texto)),
        Marca::Nota { fim: true, texto } => format!("\\endnote{{{}}}", texto_tex(texto)),
        Marca::Citacao { chaves, pagina } if pagina.is_empty() => format!("\\cite{{{}}}", chaves.join(",")),
        Marca::Citacao { chaves, pagina } => format!("\\cite[p.~{}]{{{}}}", escapar_tex(pagina), chaves.join(",")),
        Marca::Indice { termo, sub } if sub.is_empty() => format!("\\index{{{}}}", escapar_tex(termo)),
        Marca::Indice { termo, sub } => format!("\\index{{{}!{}}}", escapar_tex(termo), escapar_tex(sub)),
        Marca::Comentario { .. } => String::new(),
        Marca::Campo { campo } => match campo.as_str() {
            "pagina" => "\\thepage{}".to_string(),
            "paginas" => "\\pageref{LastPage}".to_string(),
            "data" => "\\today{}".to_string(),
            _ => escapar_tex(&t.rotulo),
        },
    }
}

fn trecho_tex(t: &Trecho) -> String {
    if let Some(m) = t.marca.as_deref() {
        return objeto_tex(t, m);
    }
    let mut s = if t.estilo & MATEMATICA != 0 {
        format!("${}$", t.texto)
    } else if t.estilo & CODIGO != 0 {
        format!("\\texttt{{{}}}", escapar_tex(&t.texto))
    } else {
        escapar_tex(&t.texto)
    };
    for (bit, cmd) in [(NEGRITO, "textbf"), (ITALICO, "emph"), (SUBLINHADO, "underline"), (TACHADO, "sout")] {
        if t.estilo & bit != 0 {
            s = format!("\\{cmd}{{{s}}}");
        }
    }
    if t.fmt.cor != 0 {
        let hex = format!("{:06X}", (t.fmt.cor >> 8) & 0xFFFFFF);
        s = format!("\\textcolor[HTML]{{{hex}}}{{{s}}}");
    }
    match &t.link {
        Some(u) => format!("\\href{{{u}}}{{{s}}}"),
        None => s,
    }
}

fn texto_tex(ts: &[Trecho]) -> String {
    ts.iter().map(trecho_tex).collect()
}

fn tabela_tex(linhas: &[Vec<Vec<Trecho>>]) -> String {
    let cols = linhas.iter().map(Vec::len).max().unwrap_or(1);
    let mut s = format!("\\begin{{tabular}}{{{}}}\n\\hline\n", "l".repeat(cols));
    for l in linhas {
        let cels: Vec<String> = l.iter().map(|c| texto_tex(c)).collect();
        s.push_str(&format!("{} \\\\ \\hline\n", cels.join(" & ")));
    }
    s.push_str("\\end{tabular}");
    s
}

/// As divisões do LaTeX por nível de título; com capítulos, o nível 1 é `\\chapter`.
const DIVISOES: [&str; 7] = ["chapter", "section", "subsection", "subsubsection", "paragraph", "subparagraph", "subparagraph"];

/// O LaTeX de um bloco sozinho (a "impressão digital" que o modo dividido compara).
pub fn bloco_tex(b: &Bloco, numeracao: Numeracao) -> String {
    let c = &b.conteudo;
    let mut s = match c {
        Conteudo::Titulo { nivel, texto } => {
            let desloc = usize::from(numeracao != Numeracao::Capitulo);
            let cmd = DIVISOES[usize::from((*nivel).clamp(1, 6)) - 1 + desloc];
            // Sem numeração (no documento ou só neste título), a divisão leva estrela.
            let estrela = if numeracao == Numeracao::Nenhuma || b.atrib.sem_numero { "*" } else { "" };
            format!("\\{cmd}{estrela}{{{}}}", texto_tex(texto))
        }
        Conteudo::Paragrafo { texto } => texto_tex(texto),
        Conteudo::Item { texto, .. } => format!("\\item {}", texto_tex(texto)),
        Conteudo::Citacao { texto } => format!("\\begin{{quote}}\n{}\n\\end{{quote}}", texto_tex(texto)),
        Conteudo::Codigo { codigo, .. } => format!("\\begin{{verbatim}}\n{codigo}\n\\end{{verbatim}}"),
        Conteudo::Tabela { linhas, .. } => tabela_tex(linhas),
        Conteudo::Imagem { origem, legenda, .. } => {
            format!("\\begin{{figure}}[h]\n\\centering\n\\includegraphics[width=0.8\\textwidth]{{{origem}}}\n\\caption{{{}}}\n\\end{{figure}}", texto_tex(legenda))
        }
        Conteudo::Equacao { latex } if ambiente_de_exibicao(latex) => latex.clone(),
        Conteudo::Equacao { latex } => format!("\\[\n{latex}\n\\]"),
        Conteudo::Sumario { lista: 1, .. } => "\\listoffigures".to_string(),
        Conteudo::Sumario { lista: 2, .. } => "\\listoftables".to_string(),
        Conteudo::Sumario { .. } => "\\tableofcontents".to_string(),
        Conteudo::Regua => "\\hrule".to_string(),
        Conteudo::QuebraDePagina => "\\newpage".to_string(),
        Conteudo::Bibliografia { todas: false } => "\\printbibliography".to_string(),
        Conteudo::Bibliografia { todas: true } => "\\nocite{*}\n\\printbibliography".to_string(),
        Conteudo::IndiceRemissivo => "\\printindex".to_string(),
    };
    if b.atrib.fundo != 0 {
        let hex = format!("{:06X}", (b.atrib.fundo >> 8) & 0xFFFFFF);
        s = format!("\\noindent\\colorbox[HTML]{{{hex}}}{{{s}}}");
    }
    s
}

/// O documento como LaTeX (classe `article`, com `babel` em português).
pub fn latex(doc: &Documento) -> String {
    latex_com_mapa(doc).texto
}

/// O mapa (bloco → linhas) de um código escrito pelo usuário, que não foi gerado pelo FeatherDocs:
/// cada bloco é procurado pelas palavras dele, em ordem, sem diferença de caixa (o `\MakeUppercase`
/// e as macros mudam a forma, não as palavras). No LaTeX a busca começa depois de `\begin{document}`
/// e pula comentários. O bloco vai da linha onde foi achado até a do bloco seguinte.
pub fn mapa_de_texto(texto: &str, doc: &Documento, fonte: crate::dividido::Fonte) -> Mapa {
    let linhas: Vec<String> = texto
        .lines()
        .map(|l| if fonte == crate::dividido::Fonte::Latex && l.trim_start().starts_with('%') { String::new() } else { l.to_lowercase() })
        .collect();
    let inicio = if fonte == crate::dividido::Fonte::Latex { linhas.iter().position(|l| l.contains("\\begin{document}")).map_or(0, |p| p + 1) } else { 0 };
    let mut blocos: Vec<LinhasDoBloco> = Vec::new();
    let mut ultima = inicio;
    for b in &doc.blocos {
        let palavras = palavras_do_bloco(&b.conteudo);
        let Some(primeira) = palavras.first() else { continue };
        let tem = |k: usize, p: &str| linhas[k].contains(p) || linhas.get(k + 1).is_some_and(|l| l.contains(p));
        let achado = (ultima..linhas.len())
            .find(|&k| linhas[k].contains(primeira.as_str()) && palavras[1..].iter().all(|p| tem(k, p)))
            .or_else(|| (ultima..linhas.len()).find(|&k| linhas[k].contains(primeira.as_str())));
        if let Some(k) = achado {
            blocos.push(LinhasDoBloco { id: b.id, de: k, ate: k + 1 });
            ultima = k;
        }
    }
    for i in 0..blocos.len().saturating_sub(1) {
        let proximo = blocos[i + 1].de;
        if proximo > blocos[i].de {
            blocos[i].ate = proximo;
        }
    }
    Mapa { texto: texto.to_string(), blocos }
}

/// Até três palavras (em minúsculas) do texto de um bloco.
fn palavras_do_bloco(c: &Conteudo) -> Vec<String> {
    let texto = match c {
        Conteudo::Titulo { texto, .. } | Conteudo::Paragrafo { texto } | Conteudo::Item { texto, .. } | Conteudo::Citacao { texto } => texto_puro(texto),
        Conteudo::Codigo { codigo, .. } => codigo.clone(),
        Conteudo::Tabela { linhas, .. } => linhas.iter().flatten().map(|c| texto_puro(c)).collect::<Vec<_>>().join(" "),
        Conteudo::Equacao { latex } => latex.clone(),
        _ => String::new(),
    };
    let palavras: Vec<&str> = texto.split(|c: char| !c.is_alphanumeric()).filter(|p| !p.is_empty()).collect();
    // As de três letras ou mais distinguem melhor; um bloco só de palavras curtas ("Um", "T") usa as que tem.
    let longas: Vec<&str> = palavras.iter().copied().filter(|p| p.chars().count() >= 3).collect();
    let escolhidas = if longas.is_empty() { palavras } else { longas };
    escolhidas.into_iter().take(3).map(str::to_lowercase).collect()
}

/// Como `latex`, com as linhas de cada bloco.
pub fn latex_com_mapa(doc: &Documento) -> Mapa {
    let doc = &*sem_sugestoes(doc);
    let classe = if doc.estilos.numeracao == Numeracao::Capitulo { "report" } else { "article" };
    let usa = objetos_usados(doc);
    // As fontes vão dentro do próprio .tex (o `filecontents` grava o .bib ao compilar).
    let mut s = if usa.citacoes { format!("\\begin{{filecontents*}}[overwrite]{{\\jobname.bib}}\n{}\\end{{filecontents*}}\n", crate::bib::gravar(&doc.bibliografia.fontes)) } else { String::new() };
    s.push_str(&format!("\\documentclass[11pt]{{{classe}}}\n"));
    s.push_str("\\usepackage[utf8]{inputenc}\n\\usepackage[T1]{fontenc}\n\\usepackage[brazil]{babel}\n\\usepackage{amsmath}\n\\usepackage{graphicx}\n\\usepackage{hyperref}\n\\usepackage[normalem]{ulem}\n\\usepackage[dvipsnames, table]{xcolor}\n");
    if usa.citacoes {
        s.push_str(pacote_de_citacao(doc.bibliografia.estilo));
        s.push_str("\\addbibresource{\\jobname.bib}\n");
    }
    if usa.notas_de_fim {
        s.push_str("\\usepackage{endnotes}\n");
    }
    if usa.indice {
        s.push_str("\\usepackage{makeidx}\n\\makeindex\n");
    }
    if usa.paginas {
        s.push_str("\\usepackage{lastpage}\n");
    }
    if !doc.meta.titulo.is_empty() {
        s.push_str(&format!("\\title{{{}}}\n", escapar_tex(&doc.meta.titulo)));
    }
    if !doc.meta.autor.is_empty() {
        s.push_str(&format!("\\author{{{}}}\n", escapar_tex(&doc.meta.autor)));
    }
    s.push_str("\\begin{document}\n");
    if !doc.meta.titulo.is_empty() {
        s.push_str("\\maketitle\n");
    }
    let mut m = Mapa { texto: s, blocos: Vec::new() };
    let mut dentro: Option<&str> = None;
    for b in &doc.blocos {
        let ambiente = match &b.conteudo {
            Conteudo::Item { ordenado: true, .. } => Some("enumerate"),
            Conteudo::Item { .. } => Some("itemize"),
            _ => None,
        };
        if dentro != ambiente {
            if let Some(a) = dentro {
                m.solto(&format!("\\end{{{a}}}\n"));
            }
            if let Some(a) = ambiente {
                m.solto(&format!("\\begin{{{a}}}\n"));
            }
            dentro = ambiente;
        }
        m.bloco(b.id, &bloco_tex(b, doc.estilos.numeracao));
        m.solto("\n\n");
    }
    if let Some(a) = dentro {
        m.solto(&format!("\\end{{{a}}}\n"));
    }
    if usa.notas_de_fim {
        m.solto("\\theendnotes\n");
    }
    m.solto("\\end{document}\n");
    m
}

/// O `biblatex` no estilo do documento.
fn pacote_de_citacao(e: EstiloDeCitacao) -> &'static str {
    match e {
        EstiloDeCitacao::Abnt => "\\usepackage[style=abnt]{biblatex}\n",
        EstiloDeCitacao::Apa => "\\usepackage[style=apa]{biblatex}\n",
        EstiloDeCitacao::Vancouver => "\\usepackage[style=vancouver]{biblatex}\n",
        EstiloDeCitacao::Ieee => "\\usepackage[style=ieee]{biblatex}\n",
        EstiloDeCitacao::Chicago => "\\usepackage[authordate]{biblatex-chicago}\n",
    }
}

/// Os pacotes que os objetos do documento pedem.
#[derive(Default)]
struct Usados {
    notas_de_fim: bool,
    indice: bool,
    paginas: bool,
    citacoes: bool,
}

fn objetos_usados(doc: &Documento) -> Usados {
    let mut u = Usados { citacoes: !doc.bibliografia.fontes.is_empty(), ..Usados::default() };
    for b in &doc.blocos {
        match b.conteudo {
            Conteudo::Bibliografia { .. } => u.citacoes = true,
            Conteudo::IndiceRemissivo => u.indice = true,
            _ => {}
        }
    }
    for m in doc.blocos.iter().flat_map(|b| b.conteudo.textos()).flatten().filter_map(|t| t.marca.as_deref()) {
        match m {
            Marca::Nota { fim: true, .. } => u.notas_de_fim = true,
            Marca::Indice { .. } => u.indice = true,
            Marca::Campo { campo } if campo == "paginas" => u.paginas = true,
            Marca::Citacao { .. } => u.citacoes = true,
            _ => {}
        }
    }
    u
}

/// O texto puro do documento (para busca e contagem fora do motor).
pub fn texto_simples(doc: &Documento) -> String {
    doc.blocos
        .iter()
        .filter_map(|b| match &b.conteudo {
            Conteudo::Titulo { texto, .. } | Conteudo::Paragrafo { texto } | Conteudo::Item { texto, .. } | Conteudo::Citacao { texto } => Some(texto_puro(texto)),
            Conteudo::Codigo { codigo, .. } => Some(codigo.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}
