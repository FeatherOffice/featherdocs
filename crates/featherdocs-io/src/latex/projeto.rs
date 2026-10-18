//! O projeto LaTeX inteiro: o `.tex` principal mais os arquivos ao lado dele (`\input`, o `.bib`,
//! as imagens), lidos de uma pasta ou de um `.zip` (o "Baixar fonte" do Overleaf). As imagens
//! entram no documento como `data:` URL, que assim não depende mais dos arquivos de origem.

use super::avisos::{Coletor, TipoDeAviso};
use super::{grupo, opcional, Leitura};
use featherdocs_core::modelo::{Conteudo, Documento};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

/// Um arquivo do projeto pelo nome relativo ao `.tex` principal (`None` = não existe).
pub type Arquivos<'a> = &'a dyn Fn(&str) -> Option<Vec<u8>>;

/// Teto de cada arquivo e do projeto descompactado: o `.zip` vem de fora e não é confiado.
const LIMITE_ARQUIVO: u64 = 64 * 1024 * 1024;
const LIMITE_TOTAL: u64 = 512 * 1024 * 1024;

/// O nome normalizado de um caminho relativo (`./a/../b.tex` → `b.tex`); `None` se sai do projeto.
fn normalizar(nome: &str) -> Option<String> {
    let nome = nome.trim().replace('\\', "/");
    let mut partes: Vec<&str> = Vec::new();
    for p in nome.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                partes.pop()?;
            }
            p => partes.push(p),
        }
    }
    (!partes.is_empty()).then(|| partes.join("/"))
}

/// Os arquivos de uma pasta, só abaixo dela.
pub fn da_pasta(pasta: &std::path::Path) -> impl Fn(&str) -> Option<Vec<u8>> + '_ {
    move |nome| {
        let p = pasta.join(normalizar(nome)?);
        let f = std::fs::File::open(p).ok()?;
        if f.metadata().ok()?.len() > LIMITE_ARQUIVO {
            return None;
        }
        let mut bytes = Vec::new();
        f.take(LIMITE_ARQUIVO).read_to_end(&mut bytes).ok()?;
        Some(bytes)
    }
}

/// Lê um `.zip` de projeto: acha o `.tex` principal e lê com os outros arquivos do pacote.
/// Devolve a leitura e o texto do `.tex` principal (para o painel de código).
pub fn ler_zip(bytes: &[u8]) -> Result<(Leitura, String), String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| format!("ZIP inválido: {e}"))?;
    let mut arquivos: HashMap<String, Vec<u8>> = HashMap::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(|e| format!("ZIP inválido: {e}"))?;
        if f.is_dir() {
            continue;
        }
        let Some(nome) = normalizar(f.name()) else { continue };
        // O tamanho declarado pode mentir: o teto vale para o que sai de fato.
        let mut dados = Vec::new();
        (&mut f).take(LIMITE_ARQUIVO + 1).read_to_end(&mut dados).map_err(|e| format!("ZIP inválido: {e}"))?;
        if dados.len() as u64 > LIMITE_ARQUIVO {
            return Err(format!("O arquivo {nome} do ZIP passa de 64 MiB."));
        }
        total += dados.len() as u64;
        if total > LIMITE_TOTAL {
            return Err("O ZIP descompactado passa de 512 MiB.".to_string());
        }
        arquivos.insert(nome, dados);
    }
    // O Overleaf às vezes põe tudo numa pasta só: os nomes ficam relativos a ela.
    let principal = principal(&arquivos).ok_or("O ZIP não tem um .tex com \\documentclass e \\begin{document}.")?;
    let base = principal.rsplit_once('/').map_or(String::new(), |(p, _)| format!("{p}/"));
    let texto = crate::texto_utf8(&arquivos[&principal])?;
    let ler = |nome: &str| normalizar(&format!("{base}{nome}")).and_then(|n| arquivos.get(&n).cloned());
    Ok((super::ler_projeto(&texto, &ler), texto))
}

/// Os arquivos do projeto: os do ZIP, ou (com uma pasta) os da pasta do `.tex` que um projeto
/// LaTeX costuma ter, sem descer em subpastas. Em ordem, o principal primeiro.
pub fn arquivos_do_projeto(caminho: &Path) -> Vec<String> {
    const TIPOS: [&str; 12] = ["tex", "bib", "cls", "sty", "bst", "png", "jpg", "jpeg", "pdf", "svg", "eps", "gif"];
    let mut v: Vec<String> = if caminho.extension().is_some_and(|e| e.eq_ignore_ascii_case("zip")) {
        std::fs::File::open(caminho)
            .ok()
            .and_then(|f| zip::ZipArchive::new(f).ok())
            .map(|z| z.file_names().filter(|n| !n.ends_with('/')).filter_map(normalizar).collect())
            .unwrap_or_default()
    } else {
        let principal = caminho.file_name().map(|n| n.to_string_lossy().into_owned());
        caminho
            .parent()
            .and_then(|p| std::fs::read_dir(p).ok())
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| Some(n) == principal.as_ref() || n.rsplit_once('.').is_some_and(|(_, x)| TIPOS.contains(&x.to_ascii_lowercase().as_str())))
            .collect()
    };
    // Uma pasta de Downloads pode ter milhares: o painel mostra os primeiros.
    v.truncate(500);
    v.sort_by_key(|n| (!n.to_ascii_lowercase().ends_with(".tex"), n.to_lowercase()));
    v
}

/// O `.tex` principal: o que tem `\documentclass` e `\begin{document}`; entre vários, o `main.tex`,
/// depois o mais raso e o maior.
fn principal(arquivos: &HashMap<String, Vec<u8>>) -> Option<String> {
    let mut candidatos: Vec<&String> = arquivos
        .iter()
        .filter(|(n, b)| {
            n.to_ascii_lowercase().ends_with(".tex") && {
                let t = String::from_utf8_lossy(b);
                t.contains("\\documentclass") && t.contains("\\begin{document}")
            }
        })
        .map(|(n, _)| n)
        .collect();
    candidatos.sort_by_key(|n| (!n.rsplit('/').next().is_some_and(|f| f.eq_ignore_ascii_case("main.tex")), n.matches('/').count(), usize::MAX - arquivos[*n].len(), (*n).clone()));
    candidatos.first().map(|n| (*n).clone())
}

/// Troca `\input{x}`/`\include{x}` pelo conteúdo de `x.tex` (até 8 níveis); o que falta vai para o aviso.
pub(super) fn incluir(fonte: &str, arquivos: Arquivos, coletor: &Coletor, nivel: u8) -> String {
    let mut saida = String::with_capacity(fonte.len());
    let mut resto = fonte;
    while let Some((i, n)) = ["\\input", "\\include", "\\lstinputlisting"].iter().filter_map(|c| resto.find(c).map(|i| (i, c.len()))).min() {
        saida.push_str(&resto[..i]);
        let depois = &resto[i + n..];
        // `\lstinputlisting[opções]{arquivo}`: o código do arquivo num `lstlisting` com as mesmas opções.
        if resto[i..].starts_with("\\lstinputlisting") {
            let (opcoes, o) = opcional(depois).unwrap_or(("", 0));
            if let Some((nome, k)) = grupo(&depois[o..]) {
                match arquivos(nome.trim()) {
                    Some(b) => saida.push_str(&format!("\\begin{{lstlisting}}[{opcoes}]\n{}\n\\end{{lstlisting}}", String::from_utf8_lossy(&b).trim_end())),
                    None => coletor.anotar(TipoDeAviso::Arquivo, nome.trim()),
                }
                resto = &depois[o + k..];
                continue;
            }
        }
        // `\includegraphics`, `\inputencoding`…: outro comando que só começa igual.
        let Some((nome, k)) = depois.starts_with('{').then(|| grupo(depois)).flatten() else {
            saida.push_str(&resto[i..i + n]);
            resto = depois;
            continue;
        };
        let nome = nome.trim();
        let achado = [nome.to_string(), format!("{nome}.tex")].into_iter().find_map(|n| arquivos(&n));
        match achado {
            Some(b) if nivel < 8 => saida.push_str(&incluir(&String::from_utf8_lossy(&b), arquivos, coletor, nivel + 1)),
            _ => coletor.anotar(TipoDeAviso::Arquivo, &format!("{nome}.tex")),
        }
        resto = &depois[k..];
    }
    saida.push_str(resto);
    saida
}

/// Troca a classe e os pacotes que vêm no projeto (`minhaclasse.cls`, `utils.sty`) pelo que eles
/// definem: a classe vira a base dela (`\LoadClass{article}`) com o código dela logo depois, e o
/// `\usepackage{utils}` vira o conteúdo do `.sty`. Os `\if…` de motor e de opção ficam no ramo falso
/// (o do pdfLaTeX no `\ifPDFTeX`).
pub(super) fn classes_e_pacotes(fonte: &str, arquivos: Arquivos) -> String {
    let mut feitos = std::collections::HashSet::new();
    let mut texto = fonte.to_string();
    if let Some(i) = texto.find("\\documentclass") {
        let depois = &texto[i + 14..];
        let (opcoes, k) = opcional(depois).map_or(("", 0), |(o, k)| (o, k));
        if let Some((nome, g)) = grupo(&depois[k..]) {
            if let Some(b) = arquivos(&format!("{}.cls", nome.trim())) {
                let mut codigo = preparar(&String::from_utf8_lossy(&b));
                let mut base = "article".to_string();
                for carregar in ["\\LoadClassWithOptions", "\\LoadClass"] {
                    if let Some(j) = codigo.find(carregar) {
                        let resto = &codigo[j + carregar.len()..];
                        let o = opcional(resto).map_or(0, |(_, k)| k);
                        if let Some((b, g2)) = grupo(&resto[o..]) {
                            base = b.trim().to_string();
                            codigo.replace_range(j..j + carregar.len() + o + g2, "");
                        }
                        break;
                    }
                }
                let novo = format!("\\documentclass[{opcoes}]{{{base}}} {} ", pacotes_locais(&codigo, arquivos, &mut feitos, 0));
                texto.replace_range(i..i + 14 + k + g, &novo);
            }
        }
    }
    pacotes_locais(&texto, arquivos, &mut feitos, 0)
}

/// Troca cada `\usepackage{x}` cujo `x.sty` está no projeto pelo conteúdo dele (até 8 níveis, cada um
/// uma vez só); os outros nomes da mesma lista continuam no `\usepackage`.
fn pacotes_locais(texto: &str, arquivos: Arquivos, feitos: &mut std::collections::HashSet<String>, nivel: u8) -> String {
    let mut saida = String::with_capacity(texto.len());
    let mut resto = texto;
    while let Some(i) = resto.find("\\usepackage") {
        saida.push_str(&resto[..i]);
        let depois = &resto[i + 11..];
        let (opcoes, k) = opcional(depois).map_or((None, 0), |(o, k)| (Some(o), k));
        let Some((nomes, g)) = grupo(&depois[k..]) else {
            saida.push_str("\\usepackage");
            resto = depois;
            continue;
        };
        let mut outros = Vec::new();
        for nome in nomes.split(',').map(str::trim).filter(|n| !n.is_empty()) {
            match arquivos(&format!("{nome}.sty")) {
                Some(b) if nivel < 8 => {
                    if feitos.insert(nome.to_string()) {
                        saida.push_str(&pacotes_locais(&preparar(&String::from_utf8_lossy(&b)), arquivos, feitos, nivel + 1));
                        saida.push(' ');
                    }
                }
                _ => outros.push(nome),
            }
        }
        if !outros.is_empty() {
            saida.push_str(&format!("\\usepackage{}{{{}}}", opcoes.map_or(String::new(), |o| format!("[{o}]")), outros.join(",")));
        }
        resto = &depois[k + g..];
    }
    saida.push_str(resto);
    saida
}

/// O código de um `.cls`/`.sty` como se estivesse no preâmbulo: sem comentários, `\RequirePackage`
/// como `\usepackage`, sem as declarações da classe e com os `\if…` de motor resolvidos.
fn preparar(codigo: &str) -> String {
    let mut s = String::with_capacity(codigo.len());
    for linha in codigo.lines() {
        // O `%` não escapado corta o resto da linha (e a quebra dela).
        let mut corte = None;
        let mut barra = false;
        for (k, c) in linha.char_indices() {
            if c == '%' && !barra {
                corte = Some(k);
                break;
            }
            barra = c == '\\' && !barra;
        }
        match corte {
            Some(k) => {
                s.push_str(&linha[..k]);
                // `\centering%` + `Modelo` na linha de baixo são dois tokens: o espaço os mantém separados.
                if linha[..k].trim_end_matches(|c: char| c.is_ascii_alphabetic() || c == '@').ends_with('\\') {
                    s.push(' ');
                }
            }
            None => {
                s.push_str(linha);
                s.push('\n');
            }
        }
    }
    let mut s = s.replace("\\RequirePackageWithOptions", "\\usepackage").replace("\\RequirePackage", "\\usepackage");
    // As declarações da classe não mudam o documento.
    for (cmd, grupos) in [("\\NeedsTeXFormat", 1), ("\\ProvidesClass", 1), ("\\ProvidesPackage", 1), ("\\ProvidesFile", 1), ("\\DeclareOption*", 1), ("\\DeclareOption", 2), ("\\PassOptionsToPackage", 2), ("\\PassOptionsToClass", 2), ("\\ExecuteOptions", 1)] {
        let mut de = 0;
        while let Some(i) = s[de..].find(cmd).map(|i| i + de) {
            let mut k = i + cmd.len();
            for _ in 0..grupos {
                k += grupo(&s[k..]).map_or(0, |(_, g)| g);
            }
            k += opcional(&s[k..]).map_or(0, |(_, o)| o);
            // As opções do `hyperref` (o `hidelinks`) ficam: o leitor as usa nas cores dos links.
            if s[i..k].ends_with("{hyperref}") {
                de = k;
                continue;
            }
            s.replace_range(i..k, "");
            de = i;
        }
    }
    s = s.replace("\\ProcessOptions\\relax", "").replace("\\ProcessOptions*", "").replace("\\ProcessOptions", "").replace("\\makeatletter", "").replace("\\makeatother", "");
    // `\AtBeginDocument{x}` e `\IfFontExistsTF{fonte}{sim}{não}`: o código de dentro / o ramo sem a fonte.
    while let Some(i) = s.find("\\AtBeginDocument") {
        let Some((corpo, g)) = grupo(&s[i + 16..]) else { break };
        let corpo = corpo.to_string();
        s.replace_range(i..i + 16 + g, &corpo);
    }
    while let Some(i) = s.find("\\IfFontExistsTF") {
        let mut k = i + 15;
        let mut partes = Vec::new();
        for _ in 0..3 {
            let Some((p, g)) = grupo(&s[k..]) else { break };
            partes.push(p.to_string());
            k += g;
        }
        let nao = partes.get(2).cloned().unwrap_or_default();
        s.replace_range(i..k, &nao);
    }
    // Numa linha só: o código entra no lugar do `\usepackage` sem mudar a linha do que vem depois
    // (os avisos apontam a linha do `.tex`). A linha em branco vira `\par`, que é o que ela é.
    let s = arroba_como_letra(&condicionais(&s));
    let mut linha = String::with_capacity(s.len());
    for (k, bloco) in s.split("\n\n").enumerate() {
        if k > 0 && !bloco.trim().is_empty() {
            linha.push_str("\\par ");
        }
        linha.push_str(&bloco.replace('\n', " "));
    }
    linha
}

/// Num `.cls`/`.sty` o `@` é letra (`\show@instituicao` é um comando só): fora dali não é, então o
/// nome vira só letras (`\showArrobainstituicao`).
fn arroba_como_letra(s: &str) -> String {
    let mut saida = String::with_capacity(s.len());
    let mut nome = false;
    for c in s.chars() {
        match c {
            '\\' => nome = true,
            '@' if nome => {
                saida.push_str("Arroba");
                continue;
            }
            c if c.is_ascii_alphabetic() => {}
            _ => nome = false,
        }
        saida.push(c);
    }
    saida
}

/// Resolve `\ifPDFTeX`, `\ifXeTeX`, `\ifLuaTeX` e `\if@twoside` (e os `\if…` dentro deles): fica o
/// ramo do pdfLaTeX (o motor padrão, o que os modelos usam no PDF que trazem), em página de um lado só. Os outros `\if…` ficam como estão.
fn condicionais(s: &str) -> String {
    const CONHECIDOS: [(&str, bool); 6] = [("ifPDFTeX", true), ("ifpdftex", true), ("ifXeTeX", false), ("ifxetex", false), ("ifLuaTeX", false), ("if@twoside", false)];
    let nome_em = |r: &str| -> String { r[1..].chars().take_while(|c| c.is_ascii_alphabetic() || *c == '@').collect() };
    let mut saida = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let r = &s[i..];
        if r.starts_with('\\') {
            let nome = nome_em(r);
            if let Some(&(_, verdade)) = CONHECIDOS.iter().find(|(n, _)| *n == nome) {
                // Acha o `\else` e o `\fi` deste `\if`, pulando os `\if…` de dentro.
                let corpo_ini = i + 1 + nome.len();
                let (mut prof, mut j, mut senao, mut fim) = (0, corpo_ini, None, None);
                while j < s.len() {
                    if s[j..].starts_with('\\') {
                        let n = nome_em(&s[j..]);
                        if n.starts_with("if") && n.len() > 2 {
                            prof += 1;
                        } else if n == "fi" {
                            if prof == 0 {
                                fim = Some(j);
                                break;
                            }
                            prof -= 1;
                        } else if n == "else" && prof == 0 {
                            senao = Some(j);
                        }
                        // `\{`, `\ç`: só a barra; o caractere depois segue pelo caminho normal.
                        j += 1 + n.len();
                        continue;
                    }
                    j += s[j..].chars().next().map_or(1, char::len_utf8);
                }
                if let Some(fim) = fim {
                    let ramo = match (verdade, senao) {
                        (true, Some(e)) => &s[corpo_ini..e],
                        (true, None) => &s[corpo_ini..fim],
                        (false, Some(e)) => &s[e + 5..fim],
                        (false, None) => "",
                    };
                    saida.push_str(&condicionais(ramo));
                    i = fim + 3;
                    continue;
                }
            }
        }
        let c = r.chars().next().unwrap_or(' ');
        saida.push(c);
        i += c.len_utf8();
    }
    saida
}

/// As fontes do `.bib` de fora (`\bibliography{a,b}` ou `\addbibresource{a.bib}`).
pub(super) fn fontes_de_fora(doc: &mut Documento, fonte: &str, arquivos: Arquivos, coletor: &Coletor) {
    for comando in ["\\bibliography{", "\\addbibresource"] {
        let mut resto = fonte;
        while let Some(i) = resto.find(comando) {
            let mut depois = &resto[i + comando.len() - usize::from(comando.ends_with('{'))..];
            if let Some((_, k)) = opcional(depois) {
                depois = &depois[k..];
            }
            let Some((nomes, k)) = grupo(depois) else { break };
            for nome in nomes.split(',').map(str::trim).filter(|n| !n.is_empty() && !n.contains("\\jobname")) {
                let arquivo = if nome.ends_with(".bib") { nome.to_string() } else { format!("{nome}.bib") };
                match arquivos(&arquivo) {
                    Some(b) => {
                        for f in crate::bib::ler(&String::from_utf8_lossy(&b)) {
                            if !doc.bibliografia.fontes.iter().any(|x| x.chave == f.chave) {
                                doc.bibliografia.fontes.push(f);
                            }
                        }
                    }
                    None => coletor.anotar(TipoDeAviso::Arquivo, &arquivo),
                }
            }
            resto = &depois[k..];
        }
    }
}

/// As imagens com caminho relativo viram `data:` URL; a que não está no projeto vai para o aviso.
pub(super) fn embutir_imagens(doc: &mut Documento, arquivos: Arquivos, coletor: &Coletor) {
    for b in &mut doc.blocos {
        let Conteudo::Imagem { origem, .. } = &b.conteudo else { continue };
        if origem.is_empty() || origem.contains("://") || origem.starts_with("data:") {
            continue;
        }
        let origem = origem.clone();
        // O `\includegraphics` aceita o nome sem extensão.
        let nomes = std::iter::once(origem.clone()).chain(["png", "jpg", "jpeg", "svg", "gif", "webp"].iter().map(|e| format!("{origem}.{e}")));
        match nomes.into_iter().find_map(|n| arquivos(&n).map(|b| (n, b))) {
            Some((n, bytes)) => {
                if n.ends_with(".svg") {
                    coletor.anotar(TipoDeAviso::Erro, &format!("Unknown graphics extension .svg (o pdfLaTeX não lê SVG): {origem}"));
                }
                // Sem a altura no código, ela sai da proporção da imagem (senão a página a deforma).
                if let (Some((w, h)), Conteudo::Imagem { largura, altura, .. }) = (pixels(&bytes), &mut b.conteudo) {
                    if *largura <= 0.0 {
                        // O tamanho natural do pdfLaTeX: a resolução gravada no PNG, ou um pixel por ponto.
                        // Largura negativa é o `scale=` do `\includegraphics`.
                        let escala = if *largura < 0.0 { -*largura } else { 1.0 };
                        *largura = w as f32 * pontos_por_pixel(&bytes) * escala;
                    }
                    if *altura <= 0.0 {
                        *altura = *largura * h as f32 / w as f32;
                    }
                }
                if let Conteudo::Imagem { origem, .. } = &mut b.conteudo {
                    *origem = crate::base64::para_data_url(tipo(&n), &bytes);
                }
            }
            None => coletor.anotar(TipoDeAviso::Arquivo, &origem),
        }
    }
    // O logo do cabeçalho e do rodapé (`\fancyhead[C]{\pgfuseimage{logo}}`).
    let x = &mut doc.extras;
    for f in [&mut x.cabecalho, &mut x.rodape, &mut x.cabecalho_primeira, &mut x.rodape_primeira] {
        if f.imagem.is_empty() || f.imagem.starts_with("data:") {
            continue;
        }
        let nomes = std::iter::once(f.imagem.clone()).chain(["png", "jpg", "jpeg"].iter().map(|e| format!("{}.{e}", f.imagem)));
        match nomes.into_iter().find_map(|n| arquivos(&n).map(|b| (n, b))) {
            Some((n, bytes)) => {
                if let Some((w, h)) = pixels(&bytes) {
                    let proporcao = h as f32 / w as f32;
                    match (f.imagem_largura > 0.0, f.imagem_altura > 0.0) {
                        (true, false) => f.imagem_altura = f.imagem_largura * proporcao,
                        (false, true) => f.imagem_largura = f.imagem_altura / proporcao,
                        (false, false) => {
                            f.imagem_largura = w as f32 * pontos_por_pixel(&bytes);
                            f.imagem_altura = f.imagem_largura * proporcao;
                        }
                        (true, true) => {}
                    }
                }
                f.imagem = crate::base64::para_data_url(tipo(&n), &bytes);
            }
            None => {
                coletor.anotar(TipoDeAviso::Arquivo, &f.imagem);
                f.imagem.clear();
            }
        }
    }
    sem_escala(doc);
}

/// A imagem que ficou sem arquivo perde o `scale=` (a largura negativa): sai no tamanho natural. A do
/// cabeçalho sem arquivo sai (seria um retângulo cinza).
pub(super) fn sem_escala(doc: &mut Documento) {
    for b in &mut doc.blocos {
        if let Conteudo::Imagem { largura, .. } = &mut b.conteudo {
            *largura = largura.max(0.0);
        }
    }
    let x = &mut doc.extras;
    for f in [&mut x.cabecalho, &mut x.rodape, &mut x.cabecalho_primeira, &mut x.rodape_primeira] {
        if !f.imagem.starts_with("data:") {
            f.imagem.clear();
        }
    }
}

/// Largura e altura em pixels pelo cabeçalho do PNG (IHDR) ou do JPEG (SOF); `None` se não é nenhum.
/// Pontos por pixel pela resolução do bloco `pHYs` do PNG (300 dpi = 0,24); 1 sem ela.
fn pontos_por_pixel(b: &[u8]) -> f32 {
    let mut i = 8;
    while b.starts_with(b"\x89PNG") && i + 8 <= b.len() {
        let n = u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as usize;
        if &b[i + 4..i + 8] == b"pHYs" && i + 17 <= b.len() && b[i + 16] == 1 {
            let por_metro = u32::from_be_bytes([b[i + 8], b[i + 9], b[i + 10], b[i + 11]]) as f32;
            if por_metro > 0.0 {
                return 72.0 / (por_metro * 0.0254);
            }
        }
        if &b[i + 4..i + 8] == b"IDAT" {
            break;
        }
        i += 12 + n;
    }
    1.0
}

fn pixels(b: &[u8]) -> Option<(u32, u32)> {
    if b.starts_with(b"\x89PNG\r\n\x1a\n") && b.len() >= 24 {
        let n = |i: usize| u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
        return Some((n(16), n(20))).filter(|(w, h)| *w > 0 && *h > 0);
    }
    if !b.starts_with(&[0xFF, 0xD8]) {
        return None;
    }
    let mut i = 2;
    while i + 9 < b.len() {
        if b[i] != 0xFF {
            return None;
        }
        let marca = b[i + 1];
        let tam = usize::from(u16::from_be_bytes([b[i + 2], b[i + 3]]));
        // SOF0…SOF15, menos DHT (C4), JPG (C8) e DAC (CC).
        if (0xC0..=0xCF).contains(&marca) && !matches!(marca, 0xC4 | 0xC8 | 0xCC) {
            let h = u32::from(u16::from_be_bytes([b[i + 5], b[i + 6]]));
            let w = u32::from(u16::from_be_bytes([b[i + 7], b[i + 8]]));
            return Some((w, h)).filter(|(w, h)| *w > 0 && *h > 0);
        }
        i += 2 + tam;
    }
    None
}

/// Os bytes de `nome` (um dos [`arquivos_do_projeto`]), do `.zip` ou da pasta do `.tex`, para o
/// painel mostrar; `None` se não existe ou passa de 64 MiB.
pub fn ler_arquivo(caminho: &Path, nome: &str) -> Option<Vec<u8>> {
    let nome = normalizar(nome)?;
    let mut dados = Vec::new();
    if caminho.extension().is_some_and(|e| e.eq_ignore_ascii_case("zip")) {
        let mut z = zip::ZipArchive::new(std::fs::File::open(caminho).ok()?).ok()?;
        let i = (0..z.len()).find(|&i| z.by_index(i).ok().and_then(|f| normalizar(f.name())).as_ref() == Some(&nome))?;
        z.by_index(i).ok()?.take(LIMITE_ARQUIVO + 1).read_to_end(&mut dados).ok()?;
    } else {
        // Da pasta só os arquivos dela mesma, como a lista mostra.
        if nome.contains('/') {
            return None;
        }
        std::fs::File::open(caminho.parent()?.join(&nome)).ok()?.take(LIMITE_ARQUIVO + 1).read_to_end(&mut dados).ok()?;
    }
    (dados.len() as u64 <= LIMITE_ARQUIVO).then_some(dados)
}

/// A `data:` URL de uma imagem do projeto, pelo nome do arquivo.
pub fn data_url_da_imagem(nome: &str, bytes: &[u8]) -> String {
    crate::base64::para_data_url(tipo(nome), bytes)
}

fn tipo(nome: &str) -> &'static str {
    match nome.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        _ => "image/png",
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::io::Write;

    fn zip_de(arquivos: &[(&str, &[u8])]) -> Vec<u8> {
        let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (n, b) in arquivos {
            z.start_file(*n, zip::write::SimpleFileOptions::default()).unwrap();
            z.write_all(b).unwrap();
        }
        z.finish().unwrap().into_inner()
    }

    #[test]
    fn zip_do_overleaf_traz_input_bib_e_imagens() {
        let tex = b"\\documentclass{article}\n\\begin{document}\n\\input{cap1}\n\\begin{figure}\\includegraphics{fig}\\caption{A}\\end{figure}\n\\cite{k1}\n\\bibliography{refs}\n\\end{document}\n";
        let zip = zip_de(&[("proj/main.tex", tex), ("proj/cap1.tex", b"Texto do cap\\'itulo."), ("proj/fig.png", b"\x89PNG"), ("proj/refs.bib", b"@book{k1, author={Silva, Ana}, title={T}, year={2020}}")]);
        let (l, texto) = ler_zip(&zip).unwrap();
        assert!(texto.starts_with("\\documentclass"));
        let d = &l.documento;
        assert!(d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Paragrafo { texto } if featherdocs_core::inline::texto_puro(texto).contains("capítulo"))));
        assert!(d.blocos.iter().any(|b| matches!(&b.conteudo, Conteudo::Imagem { origem, .. } if origem.starts_with("data:image/png;base64,"))));
        assert_eq!(d.bibliografia.fontes.len(), 1);
        assert!(l.avisos.iter().all(|a| a.tipo != TipoDeAviso::Arquivo), "{:?}", l.avisos);
    }

    #[test]
    fn caminho_que_sai_do_projeto_nao_e_lido() {
        assert_eq!(normalizar("../segredo"), None);
        assert_eq!(normalizar("./a/../b.tex").as_deref(), Some("b.tex"));
    }
}
