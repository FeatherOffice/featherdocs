//! LaTeX (o subconjunto de documento) para a árvore de blocos: seções, listas
//! (aninhadas), tabelas `tabular`, figuras, fórmulas (`$…$`, `\[…\]`, `equation`,
//! `align`…), código (`verbatim`, `lstlisting`, `minted`), citações e o texto em
//! linha (`\textbf`, `\emph`, `\texttt`, `\href`, acentos, travessões).
//!
//! Não é um compilador completo de TeX, mas expande macros do usuário (`\newcommand`,
//! `\renewcommand`), reconhece cores (`xcolor`, `\definecolor`, `\color`, `\textcolor`, `\colorbox`),
//! `minipage` e `tabularx`.

use std::collections::HashMap;

use featherdocs_core::inline::texto_puro;
use featherdocs_core::modelo::{
    Alinhamento, Atrib, Bloco, Celula, Conteudo, Documento, EstiloDeCitacao, Faixas, Fmt, Marca, Numeracao, Trecho, CODIGO, ITALICO, MATEMATICA, NEGRITO, SERIFA, SUBLINHADO, TACHADO,
};

#[derive(Clone, Debug, Default)]
struct MacroDef {
    args: usize,
    corpo: String,
}

mod avisos;
mod linha;
mod pacotes;
mod tabelas;

pub use avisos::{Aviso, Leitura, TipoDeAviso};

/// Lê um arquivo `.tex`.
pub fn ler(fonte: &str) -> Documento {
    ler_com_avisos(fonte).documento
}

/// Lê um `.tex` e diz o que não entendeu (o aviso de compatibilidade). Sem pasta, o `\input` não
/// é resolvido (e entra no aviso).
pub fn ler_com_avisos(fonte: &str) -> Leitura {
    ler_na_pasta(fonte, None)
}

/// Como `ler_com_avisos`, resolvendo `\input{…}`/`\include{…}` com os arquivos de `pasta` (a do `.tex`).
pub fn ler_na_pasta(fonte: &str, pasta: Option<&std::path::Path>) -> Leitura {
    let coletor = avisos::Coletor::default();
    let fonte_inteira = pacotes::incluir_arquivos(fonte, pasta, &coletor);
    let documento = ler_texto(&fonte_inteira, &coletor);
    Leitura { avisos: coletor.consolidar(&fonte_inteira), documento }
}

fn ler_texto(fonte: &str, coletor: &avisos::Coletor) -> Documento {
    let limpo = sem_comentarios(fonte);
    let (preambulo, corpo) = separar(&limpo);
    pacotes::avisar_preambulo(preambulo, coletor);
    let (cores, macros) = extrair_definicoes(&limpo);
    let mut doc = Documento::vazio();
    doc.meta.titulo = argumento_de(preambulo, "\\title", &cores).unwrap_or_default();
    doc.meta.autor = argumento_de(preambulo, "\\author", &cores).unwrap_or_default();
    let corpo_expandido = expandir_macros(corpo.to_string(), &macros);
    let classe = Classe::do_preambulo(preambulo);
    configurar_pagina(&mut doc, preambulo);
    let (itemsep, cor_marcador) = listas_do_preambulo(preambulo, &cores, classe);
    let ctx = Ctx {
        capitulo: corpo_expandido.contains("\\chapter"),
        titulo: doc.meta.titulo.clone(),
        autor: doc.meta.autor.clone(),
        classe,
        parskip: comprimento_definido(preambulo, "\\parskip", classe),
        itemsep,
        cor_marcador,
        cores,
        // O LaTeX justifica o texto por padrão; `\raggedright` no preâmbulo (ou `ragged2e`) tira.
        alinhamento: if preambulo.contains("\\raggedright") || preambulo.contains("\\RaggedRight") { Alinhamento::Esquerda } else { Alinhamento::Justificado },
        avisos: coletor.clone(),
        ..Ctx::default()
    };
    let capitulo = ctx.capitulo;
    for mut b in Analisador::novo(&corpo_expandido, ctx).executar() {
        b.id = doc.proximo_id;
        doc.proximo_id += 1;
        doc.blocos.push(b);
    }
    numerar_como_o_fonte(&mut doc, &corpo_expandido, capitulo);
    ler_fontes(&mut doc, &limpo, preambulo);
    if corpo_expandido.contains("\\nocite{*}") {
        for b in &mut doc.blocos {
            if let Conteudo::Bibliografia { todas } = &mut b.conteudo {
                *todas = true;
            }
        }
    }
    doc
}

/// As fontes do `filecontents` (o `.bib` dentro do próprio `.tex`) e o estilo do `biblatex` ou do
/// `\bibliographystyle`. Um `.bib` de fora não dá para ler daqui (o leitor só recebe o texto).
fn ler_fontes(doc: &mut Documento, fonte: &str, preambulo: &str) {
    for abre in ["\\begin{filecontents*}", "\\begin{filecontents}"] {
        if let Some(i) = fonte.find(abre) {
            let depois = &fonte[i + abre.len()..];
            let fim = depois.find("\\end{filecontents").unwrap_or(depois.len());
            doc.bibliografia.fontes.extend(crate::bib::ler(&depois[..fim]));
        }
    }
    let p = preambulo.to_lowercase();
    let estilo = if p.contains("biblatex-chicago") || p.contains("style=chicago") {
        Some(EstiloDeCitacao::Chicago)
    } else if p.contains("style=abnt") || p.contains("abntex2") {
        Some(EstiloDeCitacao::Abnt)
    } else if p.contains("style=apa") || p.contains("{apalike}") || p.contains("{apa}") {
        Some(EstiloDeCitacao::Apa)
    } else if p.contains("style=ieee") || p.contains("{ieeetr}") || p.contains("{ieeetran}") {
        Some(EstiloDeCitacao::Ieee)
    } else if p.contains("style=vancouver") || p.contains("{vancouver}") || p.contains("{unsrt}") {
        Some(EstiloDeCitacao::Vancouver)
    } else {
        None
    };
    if let Some(e) = estilo {
        doc.bibliografia.estilo = e;
    }
}

/// `\section{…}` numera e `\section*{…}` não: com alguma divisão sem estrela, a numeração dos títulos
/// liga (de capítulos, se há `\chapter`) e os títulos com estrela ficam fora dela. As divisões do fonte
/// e os títulos do documento saem na mesma ordem.
fn numerar_como_o_fonte(doc: &mut Documento, corpo: &str, capitulo: bool) {
    let estrelas: Vec<bool> = corpo
        .match_indices('\\')
        .filter_map(|(i, _)| {
            let (nome, n) = comando(&corpo[i..]);
            DIVISOES.iter().any(|d| d.0 == nome).then(|| corpo[i..i + n].ends_with('*'))
        })
        .collect();
    if estrelas.iter().all(|e| *e) {
        return;
    }
    doc.estilos.numeracao = if capitulo { Numeracao::Capitulo } else { Numeracao::Decimal };
    let titulos = doc.blocos.iter_mut().filter(|b| matches!(b.conteudo, Conteudo::Titulo { .. }));
    for (b, estrela) in titulos.zip(estrelas) {
        b.atrib.sem_numero = estrela;
    }
}

/// Um comprimento do TeX em pontos (`8pt`, `13mm`, `1.4em`, `-2pt`); `None` se não é um.
fn dimensao(s: &str, classe: Classe) -> Option<f32> {
    let s = s.trim();
    let fim = s.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == ',')).unwrap_or(s.len());
    let n: f32 = s[..fim].replace(',', ".").parse().ok()?;
    let corpo = if classe.corpo > 0.0 { classe.corpo } else { 10.0 };
    let unidade = s[fim..].trim_start();
    let fator = match unidade.get(..2).unwrap_or(unidade) {
        "pt" | "bp" | "" => 1.0,
        "mm" => 72.0 / 25.4,
        "cm" => 72.0 / 2.54,
        "in" => 72.0,
        "em" => corpo,
        "ex" => corpo * 0.43,
        "pc" => 12.0,
        _ => return None,
    };
    Some(n * fator)
}

/// `\setlength{\nome}{valor}` no preâmbulo.
fn comprimento_definido(preambulo: &str, nome: &str, classe: Classe) -> Option<f32> {
    let mut resto = preambulo;
    let mut achado = None;
    while let Some(i) = resto.find("\\setlength") {
        resto = &resto[i + 10..];
        if let Some((alvo, k)) = grupo(resto) {
            if alvo.trim() == nome {
                achado = grupo(&resto[k..]).and_then(|(v, _)| dimensao(v, classe));
            }
        }
    }
    achado
}

/// As opções `chave=valor` de um `[…]` ou `{…}`.
fn opcoes_chave_valor(s: &str) -> Vec<(String, String)> {
    let mut saida = Vec::new();
    let (mut prof, mut atual) = (0, String::new());
    for c in s.chars().chain(std::iter::once(',')) {
        match c {
            '{' => prof += 1,
            '}' => prof -= 1,
            ',' if prof == 0 => {
                let (k, v) = atual.split_once('=').unwrap_or((atual.as_str(), ""));
                saida.push((k.trim().to_string(), v.trim().to_string()));
                atual.clear();
                continue;
            }
            _ => {}
        }
        atual.push(c);
    }
    saida
}

/// Papel e margens (`geometry`, `a4paper`/`letterpaper`) e o estilo de página (`\pagestyle{empty}`).
fn configurar_pagina(doc: &mut Documento, preambulo: &str) {
    let classe = Classe::do_preambulo(preambulo);
    let mut opcoes = Vec::new();
    if let Some(i) = preambulo.find("\\documentclass") {
        opcoes.extend(opcional(&preambulo[i + 14..]).map(|(o, _)| opcoes_chave_valor(o)).unwrap_or_default());
    }
    for (marca, abre) in [("{geometry}", None), ("\\geometry", Some('{'))] {
        let Some(i) = preambulo.find(marca) else { continue };
        let texto = match abre {
            Some(_) => grupo(&preambulo[i + marca.len()..]).map(|(g, _)| g),
            // `\usepackage[opções]{geometry}`: as opções vêm antes do nome.
            None => preambulo[..i].rfind('[').map(|a| &preambulo[a + 1..preambulo[..i].rfind(']').unwrap_or(a + 1).max(a + 1)]),
        };
        opcoes.extend(texto.map(opcoes_chave_valor).unwrap_or_default());
    }
    let p = &mut doc.pagina;
    for (k, v) in opcoes {
        let d = || dimensao(&v, classe);
        match k.as_str() {
            "a4paper" => (p.largura, p.altura) = (595.28, 841.89),
            "letterpaper" => (p.largura, p.altura) = (612.0, 792.0),
            "top" | "tmargin" => p.margem_sup = d().unwrap_or(p.margem_sup),
            "bottom" | "bmargin" => p.margem_inf = d().unwrap_or(p.margem_inf),
            "left" | "lmargin" | "inner" => p.margem_esq = d().unwrap_or(p.margem_esq),
            "right" | "rmargin" | "outer" => p.margem_dir = d().unwrap_or(p.margem_dir),
            "margin" => {
                if let Some(m) = d() {
                    (p.margem_sup, p.margem_inf, p.margem_esq, p.margem_dir) = (m, m, m, m);
                }
            }
            "hmargin" => {
                if let Some(m) = d() {
                    (p.margem_esq, p.margem_dir) = (m, m);
                }
            }
            "vmargin" => {
                if let Some(m) = d() {
                    (p.margem_sup, p.margem_inf) = (m, m);
                }
            }
            _ => {}
        }
    }
    let estilo = preambulo.find("\\pagestyle").and_then(|i| grupo(&preambulo[i + 10..])).map(|(g, _)| g.trim().to_string());
    if matches!(estilo.as_deref(), Some("empty")) {
        doc.extras.cabecalho = Faixas::default();
        doc.extras.rodape = Faixas::default();
    }
}

/// O `\setlist[itemize]{…}` do `enumitem`: o espaço entre itens e a cor do marcador (`label=\textcolor{x}{…}`).
fn listas_do_preambulo(preambulo: &str, cores: &HashMap<String, u32>, classe: Classe) -> (Option<f32>, u32) {
    let Some(i) = preambulo.find("\\setlist") else { return (None, 0) };
    let resto = &preambulo[i + 8..];
    let resto = opcional(resto).map_or(resto, |(_, k)| &resto[k..]);
    let Some((g, _)) = grupo(resto) else { return (None, 0) };
    let mut itemsep = None;
    let mut cor = 0;
    for (k, v) in opcoes_chave_valor(g) {
        match k.as_str() {
            "itemsep" => itemsep = dimensao(&v, classe),
            "label" => {
                if let Some(j) = v.find("\\textcolor") {
                    cor = grupo(&v[j + 10..]).and_then(|(c, _)| resolver_cor(cores, c)).unwrap_or(0);
                } else if let Some(j) = v.find("\\color") {
                    cor = grupo(&v[j + 6..]).and_then(|(c, _)| resolver_cor(cores, c)).unwrap_or(0);
                }
            }
            _ => {}
        }
    }
    (itemsep, cor)
}

/// Corta o que vem depois de um `%` não escapado, linha a linha.
fn sem_comentarios(fonte: &str) -> String {
    let mut saida = String::with_capacity(fonte.len());
    for linha in fonte.lines() {
        let mut anterior = ' ';
        let mut fim = linha.len();
        for (i, c) in linha.char_indices() {
            if c == '%' && anterior != '\\' {
                fim = i;
                break;
            }
            anterior = c;
        }
        saida.push_str(&linha[..fim]);
        saida.push('\n');
    }
    saida
}

fn separar(src: &str) -> (&str, &str) {
    let Some(ini) = src.find("\\begin{document}") else { return ("", src) };
    let corpo = &src[ini + "\\begin{document}".len()..];
    (&src[..ini], corpo.rfind("\\end{document}").map_or(corpo, |f| &corpo[..f]))
}

/// O argumento `{…}` do primeiro `comando` encontrado, já como texto puro.
fn argumento_de(src: &str, comando: &str, cores: &HashMap<String, u32>) -> Option<String> {
    let i = src.find(comando)? + comando.len();
    let (g, _) = grupo(&src[i..])?;
    Some(texto_puro(&inline(g, cores)).trim().to_string())
}

/// `{…}` no começo de `s` (espaços antes são aceitos): o interior e quantos bytes consumiu.
fn grupo(s: &str) -> Option<(&str, usize)> {
    delimitado(s, '{', '}')
}

/// `[…]` opcional.
fn opcional(s: &str) -> Option<(&str, usize)> {
    delimitado(s, '[', ']')
}

fn delimitado(s: &str, abre: char, fecha: char) -> Option<(&str, usize)> {
    let inicio = s.len() - s.trim_start().len();
    let resto = &s[inicio..];
    if !resto.starts_with(abre) {
        return None;
    }
    let mut prof = 0;
    let mut escapou = false;
    for (i, c) in resto.char_indices() {
        if escapou {
            escapou = false;
        } else if c == '\\' {
            escapou = true;
        } else if c == abre {
            prof += 1;
        } else if c == fecha {
            prof -= 1;
            if prof == 0 {
                return Some((&resto[1..i], inicio + i + 1));
            }
        }
    }
    None
}

/// O nome do comando que começa em `s` (`s` começa com `\`) e o tamanho total, com a barra e o `*`.
fn comando(s: &str) -> (&str, usize) {
    let resto = &s[1..];
    let letras = resto.chars().take_while(char::is_ascii_alphabetic).count();
    if letras == 0 {
        let n = resto.chars().next().map_or(0, char::len_utf8);
        return (&resto[..n], 1 + n);
    }
    let estrela = usize::from(resto[letras..].starts_with('*'));
    (&resto[..letras], 1 + letras + estrela)
}

fn parsear_cor(modelo: &str, espec: &str) -> Option<u32> {
    let mod_limpo = modelo.trim().to_ascii_uppercase();
    let esp = espec.trim();
    if mod_limpo == "HTML" || mod_limpo == "HEX" {
        let hex = esp.trim_start_matches('#');
        if hex.len() == 6 {
            let v = u32::from_str_radix(hex, 16).ok()?;
            return Some((v << 8) | 0xFF);
        }
    } else if mod_limpo == "RGB" {
        let partes: Vec<&str> = esp.split(',').collect();
        if partes.len() == 3 {
            let r = partes[0].trim().parse::<u8>().ok()?;
            let g = partes[1].trim().parse::<u8>().ok()?;
            let b = partes[2].trim().parse::<u8>().ok()?;
            return Some(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xFF);
        }
    } else if mod_limpo == "GRAY" {
        let g = esp.parse::<f32>().ok()?;
        let v = (g.clamp(0.0, 1.0) * 255.0) as u8;
        return Some(((v as u32) << 24) | ((v as u32) << 16) | ((v as u32) << 8) | 0xFF);
    }
    None
}

fn resolver_cor(cores: &HashMap<String, u32>, nome: &str) -> Option<u32> {
    let limpo = nome.trim();
    if let Some(&c) = cores.get(limpo) {
        return Some(c);
    }
    match limpo.to_ascii_lowercase().as_str() {
        "white" | "branco" => Some(0xFFFFFFFF),
        "black" | "preto" => Some(0x000000FF),
        "red" | "vermelho" => Some(0xE03030FF),
        "green" | "verde" => Some(0x20A040FF),
        "blue" | "azul" => Some(0x2060D0FF),
        "darkblue" => Some(0x1A3A5CFF),
        "midblue" => Some(0x2E6DA4FF),
        "lightblue" => Some(0xD6E8F7FF),
        "lightgray" | "lightgrey" => Some(0xF4F6F9FF),
        "gray" | "grey" => Some(0x808080FF),
        "graytext" => Some(0x555555FF),
        "darktext" => Some(0x1C1C1CFF),
        _ => {
            let hex = limpo.trim_start_matches('#');
            if hex.len() == 6 {
                if let Ok(v) = u32::from_str_radix(hex, 16) {
                    return Some((v << 8) | 0xFF);
                }
            }
            None
        }
    }
}

fn extrair_definicoes(src: &str) -> (HashMap<String, u32>, HashMap<String, MacroDef>) {
    let mut cores = HashMap::new();
    let mut macros = HashMap::new();
    let mut i = 0;
    while i < src.len() {
        let r = &src[i..];
        if r.starts_with("\\definecolor") {
            let mut p = 12;
            if let Some((nome, k1)) = grupo(&r[p..]) {
                p += k1;
                if let Some((modelo, k2)) = grupo(&r[p..]) {
                    p += k2;
                    if let Some((espec, k3)) = grupo(&r[p..]) {
                        if let Some(c) = parsear_cor(modelo, espec) {
                            cores.insert(nome.trim().to_string(), c);
                        }
                        i += p + k3;
                        continue;
                    }
                }
            }
        } else if r.starts_with("\\newcommand") || r.starts_with("\\renewcommand") {
            let n = if r.starts_with("\\newcommand") { 11 } else { 13 };
            let mut p = n;
            let mut nome_cmd = None;
            if let Some((cmd_grp, k1)) = grupo(&r[p..]) {
                nome_cmd = Some(cmd_grp.trim().trim_start_matches('\\').to_string());
                p += k1;
            } else {
                let depois = r[p..].trim_start();
                p += r[p..].len() - depois.len();
                if depois.starts_with('\\') {
                    let (cmd, k) = comando(depois);
                    nome_cmd = Some(cmd.to_string());
                    p += k;
                }
            }
            if let Some(nome) = nome_cmd {
                let mut num_args = 0;
                if let Some((opt, k_opt)) = opcional(&r[p..]) {
                    num_args = opt.trim().parse::<usize>().unwrap_or(0);
                    p += k_opt;
                }
                if let Some((corpo, k_corpo)) = grupo(&r[p..]) {
                    macros.insert(nome, MacroDef { args: num_args, corpo: corpo.to_string() });
                    i += p + k_corpo;
                    continue;
                }
            }
        }
        let c = r.chars().next().unwrap_or(' ');
        i += c.len_utf8();
    }
    (cores, macros)
}

fn expandir_macros(mut texto: String, macros: &HashMap<String, MacroDef>) -> String {
    if macros.is_empty() {
        return texto;
    }
    for _ in 0..10 {
        let mut mudou = false;
        let mut saida = String::with_capacity(texto.len());
        let mut i = 0;
        while i < texto.len() {
            let resto = &texto[i..];
            if resto.starts_with('\\') {
                let (nome, n) = comando(resto);
                if let Some(m) = macros.get(nome) {
                    let mut pos = i + n;
                    let mut args = Vec::with_capacity(m.args);
                    let mut falhou = false;
                    for _ in 0..m.args {
                        if let Some((arg, k)) = grupo(&texto[pos..]) {
                            args.push(arg);
                            pos += k;
                        } else {
                            falhou = true;
                            break;
                        }
                    }
                    if !falhou {
                        mudou = true;
                        let mut corpo_exp = m.corpo.clone();
                        for (idx, arg) in args.iter().enumerate() {
                            corpo_exp = corpo_exp.replace(&format!("#{}", idx + 1), arg);
                        }
                        saida.push_str(&corpo_exp);
                        i = pos;
                        continue;
                    }
                }
            }
            let c = resto.chars().next().unwrap_or(' ');
            saida.push(c);
            i += c.len_utf8();
        }
        texto = saida;
        if !mudou {
            break;
        }
    }
    texto
}

// ---------------------------------------------------------------------------
// Texto em linha
// ---------------------------------------------------------------------------

/// O estado das declarações em vigor (`\bfseries`, `\small`, `\color{…}`): vale até o fim do grupo
/// `{…}` ou do ambiente em que foi feita, e atravessa parágrafos, como no TeX.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Estado {
    estilo: u8,
    cor: u32,
    /// Corpo em pontos; 0 = o normal do documento.
    tamanho: f32,
}

/// A classe do documento: o corpo (`10pt`, `11pt`, `12pt`) e a família padrão.
#[derive(Clone, Copy, Debug)]
struct Classe {
    /// Os corpos de `\tiny` a `\Huge` nesta classe.
    tamanhos: [f32; 10],
    /// O corpo do texto normal, quando a classe o diz (0 = o do FeatherDocs).
    corpo: f32,
    /// `SERIFA` se a família padrão é a romana (o padrão do LaTeX), 0 se é `\sfdefault`.
    familia: u8,
}

impl Default for Classe {
    fn default() -> Self {
        Self { tamanhos: TAMANHOS_10PT, corpo: 0.0, familia: 0 }
    }
}

const TAMANHOS_10PT: [f32; 10] = [5.0, 7.0, 8.0, 9.0, 10.0, 12.0, 14.4, 17.28, 20.74, 24.88];
const TAMANHOS_11PT: [f32; 10] = [6.0, 8.0, 9.0, 10.0, 10.95, 12.0, 14.4, 17.28, 20.74, 24.88];
const TAMANHOS_12PT: [f32; 10] = [6.0, 8.0, 10.0, 10.95, 12.0, 14.4, 17.28, 20.74, 24.88, 24.88];
const DECLARACOES_DE_TAMANHO: [&str; 10] = ["tiny", "scriptsize", "footnotesize", "small", "normalsize", "large", "Large", "LARGE", "huge", "Huge"];

impl Classe {
    /// A classe pelo `\documentclass[…]` e pelo preâmbulo (`\renewcommand{\familydefault}{\sfdefault}`).
    fn do_preambulo(preambulo: &str) -> Self {
        let opcoes = preambulo.find("\\documentclass").and_then(|i| opcional(&preambulo[i + 14..])).map_or("", |(o, _)| o);
        let tem = |o: &str| opcoes.split(',').any(|x| x.trim() == o);
        let (tamanhos, corpo) = if tem("12pt") {
            (TAMANHOS_12PT, 12.0)
        } else if tem("11pt") {
            (TAMANHOS_11PT, 0.0)
        } else if tem("10pt") {
            (TAMANHOS_10PT, 10.0)
        } else {
            (TAMANHOS_10PT, 0.0)
        };
        let sem_serifa = preambulo.contains("\\sfdefault") || preambulo.contains("\\sffamily");
        Self { tamanhos, corpo, familia: if sem_serifa { 0 } else { SERIFA } }
    }

    /// O corpo de uma declaração de tamanho; `Some(0)` = o normal.
    fn tamanho_de(&self, nome: &str) -> Option<f32> {
        let i = DECLARACOES_DE_TAMANHO.iter().position(|d| *d == nome)?;
        Some(if i == 4 { 0.0 } else { self.tamanhos[i] })
    }
}

/// Texto em linha de LaTeX para trechos, sem contexto (título e autor do preâmbulo).
fn inline(src: &str, cores: &HashMap<String, u32>) -> Vec<Trecho> {
    let ctx = Ctx { cores: cores.clone(), ..Ctx::default() };
    let mut e = Estado::default();
    inline_em(src, &ctx, &mut e)
}

/// Texto em linha no contexto do documento: começa no estado `e` e o deixa como as declarações do texto o deixaram.
fn inline_em(src: &str, ctx: &Ctx, e: &mut Estado) -> Vec<Trecho> {
    let mut i = Inline { saida: Vec::new(), e: *e, link: None, ctx };
    i.corre(src);
    *e = i.e;
    aparar(i.saida)
}

/// Sem espaço nas pontas nem em volta de tabulação (`\hfill`).
fn aparar(mut saida: Vec<Trecho>) -> Vec<Trecho> {
    while saida.last().is_some_and(|t| t.texto.trim().is_empty() && !t.texto.contains('\t')) {
        saida.pop();
    }
    if let Some(p) = saida.first_mut() {
        p.texto = p.texto.trim_start().to_string();
    }
    if let Some(u) = saida.last_mut() {
        u.texto = u.texto.trim_end_matches(' ').to_string();
    }
    for k in 0..saida.len() {
        if let Some(t) = saida[k].texto.find('\t') {
            let (antes, depois) = saida[k].texto.split_at(t);
            saida[k].texto = format!("{}\t{}", antes.trim_end_matches(' '), depois[1..].trim_start_matches(' '));
            if saida[k].texto.starts_with('\t') && k > 0 {
                let a = saida[k - 1].texto.trim_end_matches(' ').len();
                saida[k - 1].texto.truncate(a);
            }
            if saida[k].texto.ends_with('\t') {
                if let Some(prox) = saida.get_mut(k + 1) {
                    prox.texto = prox.texto.trim_start_matches(' ').to_string();
                }
            }
        }
    }
    saida.retain(|t| !t.texto.is_empty());
    saida
}

struct Inline<'a> {
    saida: Vec<Trecho>,
    e: Estado,
    link: Option<String>,
    ctx: &'a Ctx,
}

impl Inline<'_> {
    fn texto(&mut self, t: &str, extra: u8) {
        if t.is_empty() {
            return;
        }
        let estilo = self.e.estilo | self.ctx.classe.familia | extra;
        let tamanho = if self.e.tamanho > 0.0 { self.e.tamanho } else { self.ctx.classe.corpo };
        let fmt = Fmt { cor: self.e.cor, tamanho, ..Fmt::default() };
        if let Some(u) = self.saida.last_mut() {
            if u.marca.is_none() && u.estilo == estilo && u.link == self.link && u.fmt == fmt {
                u.texto.push_str(t);
                return;
            }
        }
        self.saida.push(Trecho { texto: t.to_string(), estilo, link: self.link.clone(), fmt, ..Trecho::default() });
    }

    /// O mesmo que `texto`, mas sem espaço duplo.
    fn texto_colapsado(&mut self, t: &str) {
        if t == " " && self.saida.last().is_some_and(|u| u.texto.ends_with(' ')) {
            return;
        }
        self.texto(t, 0);
    }

    /// Corre `arg` num grupo: o que ele declarar não vaza.
    fn grupo(&mut self, arg: &str) {
        let antes = self.e;
        self.corre(arg);
        self.e = antes;
    }

    fn com_estilo(&mut self, bit: u8, arg: &str) {
        let antes = self.e;
        self.e.estilo |= bit;
        self.corre(arg);
        self.e = antes;
    }

    fn corre(&mut self, src: &str) {
        let mut i = 0;
        while i < src.len() {
            let resto = &src[i..];
            let c = resto.chars().next().unwrap_or(' ');
            i += match c {
                '\\' => self.barra(resto),
                '$' => self.matematica(resto),
                '~' => {
                    self.texto("\u{a0}", 0);
                    1
                }
                '{' => {
                    if let Some((dentro, k)) = delimitado(resto, '{', '}') {
                        self.grupo(dentro);
                        k
                    } else {
                        1
                    }
                }
                '}' => 1,
                '-' | '`' | '\'' => self.pontuacao(resto),
                c if c.is_whitespace() => {
                    let n = resto.len() - resto.trim_start().len();
                    self.texto_colapsado(" ");
                    n
                }
                _ => {
                    let n = resto.find(['\\', '$', '~', '{', '}', '-', '`', '\'']).filter(|&n| n > 0).unwrap_or(c.len_utf8());
                    let n = resto[..n].find(char::is_whitespace).filter(|&k| k > 0).unwrap_or(n);
                    self.texto(&resto[..n], 0);
                    n
                }
            };
        }
    }

    fn pontuacao(&mut self, resto: &str) -> usize {
        for (de, para) in [("---", "—"), ("--", "–"), ("``", "“"), ("''", "”")] {
            if resto.starts_with(de) {
                self.texto(para, 0);
                return de.len();
            }
        }
        let c = resto.chars().next().unwrap_or(' ');
        self.texto(&c.to_string(), 0);
        c.len_utf8()
    }

    fn matematica(&mut self, resto: &str) -> usize {
        let abre = if resto.starts_with("$$") { 2 } else { 1 };
        let corpo = &resto[abre..];
        match corpo.find(&resto[..abre]) {
            Some(f) => {
                // `$$…$$` no meio do parágrafo: fica no texto, mas no estilo display (o feather-math compõe).
                let formula = corpo[..f].trim();
                if abre == 2 {
                    self.texto(&format!("\\displaystyle {formula}"), MATEMATICA);
                } else {
                    self.texto(formula, MATEMATICA);
                }
                abre + f + abre
            }
            None => {
                self.texto("$", 0);
                1
            }
        }
    }

    /// `\(…\)`: fórmula em linha, como `$…$`; `resto` começa logo depois do `\(`.
    fn matematica_parenteses(&mut self, resto: &str) -> usize {
        match resto.find("\\)") {
            Some(f) => {
                self.texto(resto[..f].trim(), MATEMATICA);
                f + 2
            }
            None => 0,
        }
    }

    /// Declarações (valem até o fim do grupo): tamanho, série, forma e família. `true` se era uma.
    fn declaracao(&mut self, nome: &str) -> bool {
        if let Some(t) = self.ctx.classe.tamanho_de(nome) {
            self.e.tamanho = t;
            return true;
        }
        let e = &mut self.e.estilo;
        match nome {
            "bfseries" | "bf" => *e |= NEGRITO,
            "mdseries" => *e &= !NEGRITO,
            "itshape" | "it" | "slshape" | "sl" | "em" => *e |= ITALICO,
            "upshape" => *e &= !ITALICO,
            "ttfamily" | "tt" => *e |= CODIGO,
            "normalfont" | "rm" => *e &= !(NEGRITO | ITALICO | CODIGO),
            _ => return false,
        }
        true
    }

    /// Trata um comando em linha; devolve quantos bytes consumiu.
    fn barra(&mut self, resto: &str) -> usize {
        let (nome, n) = comando(resto);
        let depois = &resto[n..];
        if nome == "(" {
            return n + self.matematica_parenteses(depois);
        }
        if nome == "ensuremath" {
            if let Some((g, k)) = grupo(depois) {
                self.texto(g.trim(), MATEMATICA);
                return n + k;
            }
        }
        if nome == "\\" {
            let k = opcional(depois).map_or(0, |(_, len)| len);
            self.texto_colapsado(" ");
            return n + k;
        }
        if matches!(nome, "vspace" | "hspace" | "setlength") {
            let mut k = 0;
            if let Some((_, len)) = opcional(&depois[k..]) {
                k += len;
            }
            if let Some((_, len)) = grupo(&depois[k..]) {
                k += len;
            }
            if nome == "setlength" {
                if let Some((_, len)) = grupo(&depois[k..]) {
                    k += len;
                }
            }
            return n + k;
        }
        if self.declaracao(nome) {
            return n;
        }
        if let Some(r) = self.simbolo(nome, depois) {
            return n + r;
        }
        if let Some(r) = acento(nome, depois) {
            self.texto(&r.0, 0);
            return n + r.1;
        }
        if nome == "color" {
            if let Some((arg, g)) = grupo(depois) {
                if let Some(c) = resolver_cor(&self.ctx.cores, arg) {
                    self.e.cor = c;
                }
                return n + g;
            }
        }
        if nome == "textcolor" {
            if let Some(k) = self.textcolor(depois) {
                return n + k;
            }
        }
        if matches!(nome, "cite" | "citep" | "citet" | "parencite" | "textcite" | "autocite" | "footcite" | "citeonline" | "apud") {
            if let Some(k) = self.citacao(depois) {
                return n + k;
            }
        }
        if let Some(k) = self.extra(nome, depois) {
            return n + k;
        }
        let Some((arg, g)) = grupo(depois) else {
            self.ctx.avisos.comando(nome);
            return n;
        };
        if self.com_argumento(nome, arg, &depois[g..]) {
            return n + g + extra_depois(nome, &depois[g..]);
        }
        // Comando desconhecido: o texto do argumento continua saindo, e o aviso diz o que faltou.
        self.ctx.avisos.comando(nome);
        self.grupo(arg);
        n + g
    }

    /// `\cite[antes][p.~4]{chave1,chave2}`: a citação vira objeto (a última opção é a página).
    fn citacao(&mut self, depois: &str) -> Option<usize> {
        let mut k = 0;
        let mut pagina = "";
        while let Some((o, len)) = opcional(&depois[k..]) {
            pagina = o;
            k += len;
        }
        let (chaves, g) = grupo(&depois[k..])?;
        let pagina = pagina.trim().trim_start_matches("pp.").trim_start_matches("p.").trim_start_matches('~').trim().replace('~', " ");
        let chaves = chaves.split(',').map(str::trim).filter(|c| !c.is_empty()).map(str::to_string).collect();
        self.saida.push(Trecho { estilo: self.e.estilo | self.ctx.classe.familia, ..Trecho::objeto(Marca::Citacao { chaves, pagina }) });
        Some(k + g)
    }

    /// `\textcolor[modelo]{cor}{texto}`.
    fn textcolor(&mut self, depois: &str) -> Option<usize> {
        let (modelo, offset) = opcional(depois).map_or((None, 0), |(o, k)| (Some(o), k));
        let (cor_nome, g1) = grupo(&depois[offset..])?;
        let (texto, g2) = grupo(&depois[offset + g1..])?;
        let antes = self.e;
        let c = match modelo {
            Some(m) => parsear_cor(m, cor_nome),
            None => resolver_cor(&self.ctx.cores, cor_nome),
        };
        if let Some(cor) = c {
            self.e.cor = cor;
        }
        self.corre(texto);
        self.e = antes;
        Some(offset + g1 + g2)
    }

    /// Comandos sem argumento que viram um texto.
    fn simbolo(&mut self, nome: &str, depois: &str) -> Option<usize> {
        let t = match nome {
            "newline" | "linebreak" => " ",
            "ldots" | "dots" | "textellipsis" => "…",
            "LaTeX" => "LaTeX",
            "TeX" => "TeX",
            "S" => "§",
            "copyright" => "©",
            "textbackslash" => "\\",
            "textbullet" => "•",
            "textbar" => "|",
            "textendash" => "–",
            "textemdash" => "—",
            "quad" | "qquad" | " " | "," | ";" | "!" | ":" | "enspace" | "thinspace" => " ",
            "hfill" | "hfil" => "\t",
            "&" | "%" | "$" | "#" | "_" | "{" | "}" => nome,
            "today" | "noindent" | "indent" | "centering" | "raggedright" | "raggedleft" | "par" | "label" | "protect" | "relax" | "strut" | "null" | "nobreak" | "/" => "",
            _ => return None,
        };
        if t == "\t" {
            self.texto(t, 0);
        } else {
            self.texto_colapsado(t);
        }
        Some(if nome == "label" { grupo(depois).map_or(0, |g| g.1) } else { 0 })
    }

    /// Comandos com um argumento `{…}`. `true` se tratou (e já consumiu o argumento).
    fn com_argumento(&mut self, nome: &str, arg: &str, depois: &str) -> bool {
        match nome {
            "textbf" | "mathbf" => self.com_estilo(NEGRITO, arg),
            "emph" | "textit" | "textsl" | "mathit" => self.com_estilo(ITALICO, arg),
            "underline" | "uline" => self.com_estilo(SUBLINHADO, arg),
            "sout" | "st" => self.com_estilo(TACHADO, arg),
            "textsf" => {
                let antes = self.e;
                self.e.estilo &= !SERIFA;
                self.corre(arg);
                self.e = antes;
            }
            "texttt" | "code" | "verb" | "lstinline" => self.texto(arg, CODIGO),
            "MakeUppercase" | "uppercase" => self.trocar_caixa(arg, true),
            "MakeLowercase" | "lowercase" => self.trocar_caixa(arg, false),
            "url" => self.com_link(arg, arg),
            "href" => {
                let texto = grupo(depois).map_or("", |g| g.0);
                self.com_link(arg, texto);
            }
            "cite" | "citep" | "citet" | "ref" | "eqref" | "autoref" | "pageref" => self.texto(&format!("[{arg}]"), 0),
            "footnote" | "endnote" => {
                let mut nota = Inline { saida: Vec::new(), e: Estado::default(), link: None, ctx: self.ctx };
                nota.corre(arg);
                let texto = aparar(nota.saida);
                self.saida.push(Trecho { estilo: self.e.estilo | self.ctx.classe.familia, ..Trecho::objeto(Marca::Nota { fim: nome == "endnote", texto }) });
            }
            "index" => {
                let (termo, sub) = arg.split_once('!').unwrap_or((arg, ""));
                self.saida.push(Trecho::objeto(Marca::Indice { termo: termo.trim().to_string(), sub: sub.trim().to_string() }));
            }
            "label" | "phantom" | "hphantom" | "vphantom" => {}
            _ => return false,
        }
        true
    }

    /// `\MakeUppercase{…}`: o argumento é lido normalmente (comandos e tudo) e só o texto que ele
    /// produziu muda de caixa.
    fn trocar_caixa(&mut self, arg: &str, maiusculas: bool) {
        let n = self.saida.len();
        let ja = self.saida.last().map_or(0, |t| t.texto.len());
        self.grupo(arg);
        let trocar = |t: &str| if maiusculas { t.to_uppercase() } else { t.to_lowercase() };
        for (k, t) in self.saida.iter_mut().enumerate().skip(n.saturating_sub(1)) {
            if t.estilo & MATEMATICA != 0 {
                continue;
            }
            if k + 1 == n {
                let (fica, muda) = t.texto.split_at(ja.min(t.texto.len()));
                t.texto = format!("{fica}{}", trocar(muda));
            } else {
                t.texto = trocar(&t.texto);
            }
        }
    }

    fn com_link(&mut self, url: &str, texto: &str) {
        let antes = self.link.replace(url.to_string());
        self.grupo(texto);
        self.link = antes;
    }
}

/// `\href{url}{texto}` consome o segundo grupo também.
fn extra_depois(nome: &str, depois: &str) -> usize {
    if nome == "href" {
        grupo(depois).map_or(0, |g| g.1)
    } else {
        0
    }
}

/// Acentos: `\'a`, `\'{a}`, `\~a`, `\c{c}`, `\c c`… Devolve o caractere e quantos bytes leu depois do comando.
fn acento(nome: &str, depois: &str) -> Option<(String, usize)> {
    let marca = match nome {
        "'" => '\u{301}',
        "`" => '\u{300}',
        "^" => '\u{302}',
        "\"" => '\u{308}',
        "~" => '\u{303}',
        "c" => '\u{327}',
        _ => return None,
    };
    let (base, lido) = if let Some((g, n)) = grupo(depois) { (g.chars().next()?, n) } else {
        let t = depois.trim_start();
        (t.chars().next()?, depois.len() - t.len() + t.chars().next()?.len_utf8())
    };
    Some((compor(base, marca), lido))
}

/// Letra + acento já composta (á, ç, ñ…); sem forma pronta, devolve só a letra.
fn compor(base: char, marca: char) -> String {
    let tabela = [
        ("aáàâäã", 'a'),
        ("eéèêë~", 'e'),
        ("iíìîï~", 'i'),
        ("oóòôöõ", 'o'),
        ("uúùûü~", 'u'),
        ("AÁÀÂÄÃ", 'A'),
        ("EÉÈÊË~", 'E'),
        ("IÍÌÎÏ~", 'I'),
        ("OÓÒÔÖÕ", 'O'),
        ("UÚÙÛÜ~", 'U'),
    ];
    let posicao = match marca {
        '\u{301}' => 1,
        '\u{300}' => 2,
        '\u{302}' => 3,
        '\u{308}' => 4,
        '\u{303}' => 5,
        _ => 0,
    };
    if marca == '\u{327}' {
        return match base {
            'c' => "ç".into(),
            'C' => "Ç".into(),
            _ => base.to_string(),
        };
    }
    if marca == '\u{303}' && (base == 'n' || base == 'N') {
        return if base == 'n' { "ñ".into() } else { "Ñ".into() };
    }
    for (letras, b) in tabela {
        if b == base && posicao > 0 {
            return letras.chars().nth(posicao).filter(|&c| c != '~').map_or_else(|| base.to_string(), |c| c.to_string());
        }
    }
    base.to_string()
}

// ---------------------------------------------------------------------------
// Blocos
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
struct Ctx {
    capitulo: bool,
    nivel_lista: u8,
    titulo: String,
    autor: String,
    fundo: u32,
    alinhamento: Alinhamento,
    cores: HashMap<String, u32>,
    classe: Classe,
    /// As declarações em vigor neste grupo.
    estado: Estado,
    /// `\parskip` e o `itemsep` do `enumitem`, quando o documento os define.
    parskip: Option<f32>,
    itemsep: Option<f32>,
    cor_marcador: u32,
    /// `\vspace` em espera: vira o espaço antes do próximo bloco.
    vspace: f32,
    /// O que o leitor não entendeu (compartilhado com os analisadores filhos).
    avisos: avisos::Coletor,
}

/// A entrelinha do LaTeX (`\baselineskip` = 1,2 × o corpo) no múltiplo do FeatherDocs (1,4 × o corpo = 1,15).
const ENTRELINHA_LATEX: f32 = 1.2 / 1.4 * 1.15;

struct Analisador<'a> {
    s: &'a str,
    i: usize,
    ctx: Ctx,
    saida: Vec<Bloco>,
}

/// Níveis das divisões: `(comando, nível sem \chapter, nível com \chapter)`.
const DIVISOES: [(&str, u8, u8); 6] = [("chapter", 1, 1), ("section", 1, 2), ("subsection", 2, 3), ("subsubsection", 3, 4), ("paragraph", 4, 5), ("subparagraph", 5, 6)];

/// Comandos que somem sozinhos, com quantos argumentos `{…}` levam junto.
const IGNORADOS: [(&str, usize); 25] = [
    ("documentclass", 1),
    ("usepackage", 1),
    ("label", 1),
    ("hspace", 1),
    ("setlength", 2),
    ("pagestyle", 1),
    ("thispagestyle", 1),
    ("date", 1),
    ("bibliographystyle", 1),
    ("bibliography", 1),
    ("noindent", 0),
    ("indent", 0),
    ("bigskip", 0),
    ("medskip", 0),
    ("smallskip", 0),
    ("par", 0),
    ("newline", 0),
    ("vfill", 0),
    ("relax", 0),
    ("clearpage", 0),
    ("pagebreak", 0),
    ("setlist", 1),
    ("dimexpr", 0),
    ("arrayrulecolor", 1),
    ("microtype", 0),
];

impl<'a> Analisador<'a> {
    fn novo(s: &'a str, ctx: Ctx) -> Self {
        Self { s, i: 0, ctx, saida: Vec::new() }
    }

    fn push_bloco(&mut self, c: Conteudo) {
        let atrib = self.atrib_para(&c);
        self.saida.push(Bloco { id: 0, conteudo: c, atrib, mesclagens: Vec::new() });
    }

    /// Os atributos que o contexto dá ao bloco: fundo, alinhamento, entrelinha, espaços
    /// (`\parskip`, `itemsep`, o `\vspace` em espera), cor do marcador e da régua.
    fn atrib_para(&mut self, c: &Conteudo) -> Atrib {
        let mut a = Atrib { fundo: self.ctx.fundo, alinhamento: self.ctx.alinhamento, ..Atrib::default() };
        let texto = matches!(c, Conteudo::Titulo { .. } | Conteudo::Paragrafo { .. } | Conteudo::Item { .. } | Conteudo::Citacao { .. });
        if texto {
            a.entrelinha = ENTRELINHA_LATEX;
        }
        match c {
            Conteudo::Paragrafo { .. } => a.depois = self.ctx.parskip,
            Conteudo::Item { .. } => {
                a.depois = self.ctx.itemsep;
                a.cor_marcador = self.ctx.cor_marcador;
            }
            Conteudo::Regua => {
                a.cor_borda = self.ctx.estado.cor;
                a.antes = Some(0.0);
                a.depois = Some(0.0);
            }
            _ => {}
        }
        if self.ctx.vspace != 0.0 && (texto || matches!(c, Conteudo::Regua)) {
            a.antes = Some(self.ctx.vspace.max(0.0));
            self.ctx.vspace = 0.0;
        }
        a
    }

    /// Texto em linha com as declarações em vigor, que ficam como o texto as deixou.
    fn inline_aqui(&mut self, src: &str) -> Vec<Trecho> {
        let mut e = self.ctx.estado;
        let t = inline_em(src, &self.ctx, &mut e);
        self.ctx.estado = e;
        t
    }

    fn resto(&self) -> &'a str {
        &self.s[self.i..]
    }

    fn executar(mut self) -> Vec<Bloco> {
        loop {
            let n = self.resto().len() - self.resto().trim_start().len();
            self.i += n;
            if self.i >= self.s.len() {
                break;
            }
            if !self.bloco() {
                self.paragrafo();
            }
        }
        self.saida
    }

    /// Tenta tratar um bloco que começa aqui; `false` se é texto corrido.
    fn bloco(&mut self) -> bool {
        let r = self.resto();
        if r.starts_with("$$") {
            return self.equacao_de("$$", "$$");
        }
        if r.starts_with('{') {
            return self.grupo_com_blocos();
        }
        if !r.starts_with('\\') {
            return false;
        }
        if r.starts_with("\\colorbox") {
            let mut p = 9;
            let mut modelo = None;
            if let Some((opt, k_opt)) = opcional(&r[p..]) {
                modelo = Some(opt);
                p += k_opt;
            }
            if let Some((cor_nome, k1)) = grupo(&r[p..]) {
                p += k1;
                if let Some((conteudo, k2)) = grupo(&r[p..]) {
                    self.i += p + k2;
                    let cor = match modelo {
                        Some(m) => parsear_cor(m, cor_nome),
                        None => resolver_cor(&self.ctx.cores, cor_nome),
                    }.unwrap_or(0);
                    let mut ctx_novo = self.ctx.clone();
                    ctx_novo.fundo = cor;
                    if conteudo.contains("\\centering") {
                        ctx_novo.alinhamento = Alinhamento::Centro;
                    }
                    let filhos = Analisador::novo(conteudo, ctx_novo).executar();
                    let n = filhos.len();
                    // A caixa é uma faixa só: entre os blocos de dentro não há espaço.
                    for (k, mut f) in filhos.into_iter().enumerate() {
                        if f.atrib.fundo == 0 {
                            f.atrib.fundo = cor;
                        }
                        if k > 0 {
                            f.atrib.antes = Some(0.0);
                        }
                        if k + 1 < n {
                            f.atrib.depois = Some(0.0);
                        }
                        self.saida.push(f);
                    }
                    return true;
                }
            }
        }
        let (nome, n) = comando(r);
        if nome == "[" {
            return self.equacao_de("\\[", "\\]");
        }
        if let Some(feito) = self.declaracao_de_bloco(nome, n) {
            return feito;
        }
        if let Some((_, a, b)) = DIVISOES.iter().find(|d| d.0 == nome) {
            return self.divisao(n, if self.ctx.capitulo { *b } else { *a });
        }
        match nome {
            "begin" => self.ambiente(n),
            "maketitle" => {
                self.i += n;
                self.titulo_do_documento();
                true
            }
            "newpage" => self.simples(n, Conteudo::QuebraDePagina),
            "printbibliography" | "bibliography" | "printindex" | "nocite" => self.referencias(nome, n),
            "tableofcontents" => self.simples(n, Conteudo::sumario(3)),
            "listoffigures" => self.simples(n, Conteudo::Sumario { niveis: 1, sem_numeros: false, sem_pontilhado: false, lista: 1 }),
            "listoftables" => self.simples(n, Conteudo::Sumario { niveis: 1, sem_numeros: false, sem_pontilhado: false, lista: 2 }),
            "hrule" => {
                self.i += n;
                let r = self.resto();
                let mut skip = 0;
                for (idx, c) in r.char_indices() {
                    if c == '\n' || c == '}' || c == '\\' {
                        break;
                    }
                    skip = idx + c.len_utf8();
                }
                self.i += skip;
                self.push_bloco(Conteudo::Regua);
                true
            }
            "includegraphics" => self.imagem_solta(n),
            "caption" | "captionof" => self.legenda_solta(n),
            "title" | "author" => self.guardar_meta(nome, n),
            _ => self.ignorado(nome, n),
        }
    }

    /// `{…}` que tem bloco dentro (`{\color{x}\hrule}`): é um grupo, o que ele declara não vaza.
    /// Sem bloco dentro, é texto (`false`).
    fn grupo_com_blocos(&mut self) -> bool {
        let Some((dentro, k)) = grupo(self.resto()) else { return false };
        if !tem_bloco(dentro) {
            return false;
        }
        let filhos = Analisador::novo(dentro, self.ctx.clone()).executar();
        self.saida.extend(filhos);
        self.i += k;
        true
    }

    /// `\vspace`, `\centering`, `\raggedright`, `\raggedleft` entre blocos.
    fn declaracao_de_bloco(&mut self, nome: &str, n: usize) -> Option<bool> {
        match nome {
            "vspace" | "vspace*" => {
                let resto = &self.resto()[n..];
                let (g, k) = grupo(resto)?;
                if let Some(d) = dimensao(g, self.ctx.classe) {
                    self.ctx.vspace += d;
                }
                self.i += n + k;
            }
            "centering" => {
                self.ctx.alinhamento = Alinhamento::Centro;
                self.i += n;
            }
            "raggedright" => {
                self.ctx.alinhamento = Alinhamento::Esquerda;
                self.i += n;
            }
            "raggedleft" => {
                self.ctx.alinhamento = Alinhamento::Direita;
                self.i += n;
            }
            _ => return None,
        }
        Some(true)
    }

    fn simples(&mut self, n: usize, c: Conteudo) -> bool {
        self.i += n;
        self.push_bloco(c);
        true
    }

    /// `\printbibliography[…]` (ou `\bibliography{…}`) e `\printindex` viram as listas geradas;
    /// o `\nocite{…}` só some (o `\nocite{*}` liga "todas" depois, em `ler`).
    fn referencias(&mut self, nome: &str, n: usize) -> bool {
        self.i += n;
        while let Some((_, k)) = opcional(self.resto()) {
            self.i += k;
        }
        if nome != "printbibliography" && nome != "printindex" {
            if let Some((_, k)) = grupo(self.resto()) {
                self.i += k;
            }
        }
        match nome {
            "printindex" => self.push_bloco(Conteudo::IndiceRemissivo),
            "nocite" => {}
            _ => self.push_bloco(Conteudo::Bibliografia { todas: false }),
        }
        true
    }

    fn ignorado(&mut self, nome: &str, n: usize) -> bool {
        let Some((_, args)) = IGNORADOS.iter().find(|d| d.0 == nome) else { return false };
        self.i += n;
        while let Some((_, k)) = opcional(self.resto()) {
            self.i += k;
        }
        for _ in 0..*args {
            if let Some((_, k)) = grupo(self.resto()) {
                self.i += k;
            }
        }
        true
    }

    fn guardar_meta(&mut self, nome: &str, n: usize) -> bool {
        let Some((g, k)) = grupo(&self.resto()[n..]) else { return false };
        let t = texto_puro(&inline(g, &self.ctx.cores)).trim().to_string();
        if nome == "title" {
            self.ctx.titulo = t;
        } else {
            self.ctx.autor = t;
        }
        self.i += n + k;
        true
    }

    fn titulo_do_documento(&mut self) {
        if !self.ctx.titulo.is_empty() {
            self.push_bloco(Conteudo::Titulo { nivel: 1, texto: vec![Trecho::simples(&self.ctx.titulo)] });
        }
        if !self.ctx.autor.is_empty() {
            self.push_bloco(Conteudo::Paragrafo { texto: vec![Trecho { texto: self.ctx.autor.clone(), estilo: ITALICO, link: None, ..Trecho::default() }] });
        }
    }

    fn divisao(&mut self, n: usize, nivel: u8) -> bool {
        let mut p = n;
        if let Some((_, k)) = opcional(&self.resto()[p..]) {
            p += k;
        }
        let Some((g, k)) = grupo(&self.resto()[p..]) else { return false };
        let mut ctx = self.ctx.clone();
        ctx.classe.corpo = 0.0;
        let texto = inline_em(g, &ctx, &mut Estado::default());
        self.push_bloco(Conteudo::Titulo { nivel, texto });
        self.i += p + k;
        true
    }

    fn equacao_de(&mut self, abre: &str, fecha: &str) -> bool {
        let corpo = &self.resto()[abre.len()..];
        let Some(f) = corpo.find(fecha) else { return false };
        self.push_bloco(Conteudo::Equacao { latex: limpar_formula(&corpo[..f]) });
        self.i += abre.len() + f + fecha.len();
        true
    }

    fn imagem_solta(&mut self, n: usize) -> bool {
        let mut p = n;
        if let Some((_, k)) = opcional(&self.resto()[p..]) {
            p += k;
        }
        let Some((g, k)) = grupo(&self.resto()[p..]) else { return false };
        self.push_bloco(Conteudo::Imagem { origem: g.trim().to_string(), legenda: Vec::new(), largura: 0.0, altura: 0.0 });
        self.i += p + k;
        true
    }

    fn legenda_solta(&mut self, n: usize) -> bool {
        let mut p = n;
        if let Some((_, k)) = opcional(&self.resto()[p..]) {
            p += k;
        }
        let Some((g, k)) = grupo(&self.resto()[p..]) else { return false };
        let mut texto = self.inline_aqui(g);
        for t in &mut texto {
            t.estilo |= ITALICO;
        }
        self.push_bloco(Conteudo::Paragrafo { texto });
        self.i += p + k;
        true
    }

    /// Texto corrido até uma linha em branco, o começo de outro bloco ou o fim de parágrafo
    /// (`\par`, `\\`, `\newline`) fora de grupo.
    fn paragrafo(&mut self) {
        let inicio = self.i;
        let (fim, proximo) = self.fim_do_paragrafo(inicio);
        let texto = self.inline_aqui(&self.s[inicio..fim]);
        self.i = proximo;
        if !texto.is_empty() {
            self.push_bloco(Conteudo::Paragrafo { texto });
        }
    }

    /// Onde o parágrafo que começa em `inicio` termina e onde a leitura continua.
    fn fim_do_paragrafo(&mut self, inicio: usize) -> (usize, usize) {
        let mut j = inicio;
        let mut prof = 0i32;
        while j < self.s.len() {
            let r = &self.s[j..];
            if prof == 0 && j > inicio && (r.starts_with("\n\n") || r.starts_with("\n\r\n") || ComecaBloco::sim(r)) {
                return (j, j);
            }
            // Fórmula em linha: o `\\`, o `&` e o `\begin{pmatrix}` de dentro dela são da fórmula, não do parágrafo.
            if let Some(n) = fim_da_formula_em_linha(r) {
                j += n;
                continue;
            }
            if r.starts_with('\\') {
                let (nome, n) = comando(r);
                if prof == 0 && matches!(nome, "\\" | "par" | "newline" | "linebreak") {
                    let mut k = n;
                    if let Some((d, len)) = opcional(&r[k..]) {
                        self.ctx.vspace += dimensao(d, self.ctx.classe).unwrap_or(0.0);
                        k += len;
                    }
                    return (j, j + k);
                }
                j += n.max(1);
                continue;
            }
            match r.chars().next() {
                Some('{') => prof += 1,
                Some('}') => prof -= 1,
                _ => {}
            }
            j += r.chars().next().map_or(1, char::len_utf8);
        }
        if j == inicio {
            j = inicio + self.s[inicio..].chars().next().map_or(1, char::len_utf8);
        }
        (j, j)
    }
}

/// Há bloco no nível de cima deste texto (fora de grupos)?
fn tem_bloco(s: &str) -> bool {
    let mut prof = 0;
    let mut j = 0;
    while j < s.len() {
        let r = &s[j..];
        if prof == 0 && ComecaBloco::sim(r) {
            return true;
        }
        if r.starts_with('\\') {
            j += comando(r).1.max(1);
            continue;
        }
        match r.chars().next() {
            Some('{') => prof += 1,
            Some('}') => prof -= 1,
            _ => {}
        }
        j += r.chars().next().map_or(1, char::len_utf8);
    }
    false
}

/// O que, no meio do texto, abre um bloco novo.
struct ComecaBloco;

impl ComecaBloco {
    fn sim(r: &str) -> bool {
        if r.starts_with("$$") {
            return true;
        }
        if r.starts_with("\\\\") {
            return false;
        }
        if !r.starts_with('\\') {
            return false;
        }
        let (nome, _) = comando(r);
        DIVISOES.iter().any(|d| d.0 == nome)
            || matches!(
                nome,
                "begin" | "[" | "newpage" | "maketitle" | "tableofcontents" | "listoffigures" | "listoftables" | "includegraphics" | "caption" | "hrule" | "end" | "colorbox" | "printbibliography" | "printindex" | "nocite"
            )
    }
}

/// Se `r` começa uma fórmula em linha (`$…$`, `$$…$$`, `\(…\)`), quantos bytes ela ocupa até o fechamento.
fn fim_da_formula_em_linha(r: &str) -> Option<usize> {
    let (abre, fecha) = if r.starts_with("$$") {
        ("$$", "$$")
    } else if r.starts_with('$') {
        ("$", "$")
    } else if r.starts_with("\\(") {
        ("\\(", "\\)")
    } else {
        return None;
    };
    r[abre.len()..].find(fecha).map(|f| abre.len() + f + fecha.len())
}

fn limpar_formula(f: &str) -> String {
    let mut s = f.to_string();
    while let Some(i) = s.find("\\label{") {
        let Some((_, k)) = grupo(&s[i + 6..]) else { break };
        s.replace_range(i..i + 6 + k, "");
    }
    s.trim().to_string()
}

// ---------------------------------------------------------------------------
// Ambientes (`\begin{…}` … `\end{…}`)
// ---------------------------------------------------------------------------

/// O interior do ambiente `nome` que abre em `s` (logo depois do `\begin{nome}`) e o total de bytes até depois do `\end`.
fn corpo_do_ambiente<'a>(s: &'a str, nome: &str) -> Option<(&'a str, usize)> {
    let abre = format!("\\begin{{{nome}}}");
    let fecha = format!("\\end{{{nome}}}");
    let mut prof = 1;
    let mut j = 0;
    while j < s.len() {
        let r = &s[j..];
        if r.starts_with(&abre) {
            prof += 1;
            j += abre.len();
        } else if r.starts_with(&fecha) {
            prof -= 1;
            if prof == 0 {
                return Some((&s[..j], j + fecha.len()));
            }
            j += fecha.len();
        } else {
            j += r.chars().next().map_or(1, char::len_utf8);
        }
    }
    None
}

impl Analisador<'_> {
    fn ambiente(&mut self, n: usize) -> bool {
        let Some((nome, k)) = grupo(&self.resto()[n..]) else { return false };
        let mut apos = &self.resto()[n + k..];
        let mut extra_p = 0;
        if nome == "minipage" {
            while let Some((_, opt_len)) = opcional(&apos[extra_p..]) {
                extra_p += opt_len;
            }
            if let Some((_, grp_len)) = grupo(&apos[extra_p..]) {
                extra_p += grp_len;
            }
        } else if nome == "tabularx" {
            if let Some((_, grp_len)) = grupo(&apos[extra_p..]) {
                extra_p += grp_len;
            }
        }
        apos = &apos[extra_p..];
        let Some((corpo, total)) = corpo_do_ambiente(apos, nome) else { return false };
        self.i += n + k + extra_p + total;
        let base = nome.trim_end_matches('*');
        match base {
            "itemize" | "enumerate" | "description" => self.lista(corpo, base == "enumerate"),
            "verbatim" | "Verbatim" | "lstlisting" | "minted" => self.codigo(corpo, base),
            // O ambiente fica inteiro no LaTeX da equação: o feather-math sabe numerar (`equation`, `align`
            // sem estrela), alinhar no `&` e ler `\label`, `\tag` e `\nonumber`.
            "displaymath" | "math" => self.push_bloco(Conteudo::Equacao { latex: limpar_formula(corpo) }),
            "split" => self.push_bloco(Conteudo::Equacao { latex: format!("\\begin{{aligned}}{}\\end{{aligned}}", corpo.trim()) }),
            "equation" | "align" | "eqnarray" | "multline" | "gather" | "flalign" | "alignat" | "xalignat" | "dmath" | "dgroup" | "dseries" | "empheq" => {
                self.push_bloco(Conteudo::Equacao { latex: format!("\\begin{{{nome}}}{}\\end{{{nome}}}", corpo.trim()) })
            }
            "quote" | "quotation" | "abstract" => self.citacao(corpo),
            "tabular" | "tabularx" | "longtable" => self.tabular(corpo),
            "figure" => self.figura(corpo),
            "center" | "flushleft" | "flushright" | "raggedright" => {
                let mut ctx = self.ctx.clone();
                ctx.alinhamento = match base {
                    "center" => Alinhamento::Centro,
                    "flushright" => Alinhamento::Direita,
                    _ => Alinhamento::Esquerda,
                };
                let filhos = Analisador::novo(corpo, ctx).executar();
                self.saida.extend(filhos);
            }
            "minipage" => {
                let mut ctx_mini = self.ctx.clone();
                if corpo.contains("\\centering") {
                    ctx_mini.alinhamento = Alinhamento::Centro;
                }
                let filhos = Analisador::novo(corpo, ctx_mini).executar();
                self.saida.extend(filhos);
            }
            _ => self.ambiente_extra(base, corpo),
        }
        true
    }

    fn citacao(&mut self, corpo: &str) {
        for mut c in Analisador::novo(corpo, self.ctx.clone()).executar() {
            if let Conteudo::Paragrafo { texto } = c.conteudo {
                c.conteudo = Conteudo::Citacao { texto };
            }
            self.saida.push(c);
        }
    }

    fn codigo(&mut self, corpo: &str, base: &str) {
        let mut corpo = corpo;
        let mut linguagem = String::new();
        if base == "minted" {
            if let Some((l, k)) = grupo(corpo) {
                linguagem = l.to_string();
                corpo = &corpo[k..];
            }
        } else if base == "lstlisting" {
            if let Some((o, k)) = opcional(corpo) {
                linguagem = o.split(',').find_map(|p| p.trim().strip_prefix("language=")).unwrap_or("").to_string();
                corpo = &corpo[k..];
            }
        }
        self.push_bloco(Conteudo::Codigo { linguagem, codigo: corpo.trim_matches('\n').trim_end().to_string() });
    }

    fn figura(&mut self, corpo: &str) {
        let origem = corpo.find("\\includegraphics").and_then(|p| {
            let r = &corpo[p + "\\includegraphics".len()..];
            let r = opcional(r).map_or(r, |(_, k)| &r[k..]);
            grupo(r).map(|(g, _)| g.trim().to_string())
        });
        let legenda = corpo.find("\\caption").and_then(|p| grupo(&corpo[p + "\\caption".len()..])).map(|(g, _)| inline_em(g, &self.ctx, &mut self.ctx.estado.clone())).unwrap_or_default();
        match origem {
            Some(origem) => self.push_bloco(Conteudo::Imagem { origem, legenda, largura: 0.0, altura: 0.0 }),
            None => self.saida.extend(Analisador::novo(corpo, self.ctx.clone()).executar()),
        }
    }

    fn tabular(&mut self, corpo: &str) {
        let mut corpo = corpo;
        while let Some((_, k)) = opcional(corpo) {
            corpo = &corpo[k..];
        }
        let mut colunas = "";
        while let Some((g, k)) = grupo(corpo) {
            if colunas.is_empty() {
                colunas = g;
            }
            corpo = &corpo[k..];
            if !corpo.trim_start().starts_with('{') {
                break;
            }
        }
        let tem_regua = ["\\hline", "\\toprule", "\\midrule", "\\bottomrule", "\\cline"].iter().any(|r| corpo.contains(r)) || colunas.contains('|');
        let cabecalho = corpo.contains("\\midrule") || corpo.matches("\\hline").count() >= 2 && corpo.find("\\hline").is_some_and(|p| p < 3);
        let linhas = linhas_de_tabela(corpo, &self.ctx);
        if linhas.len() == 1 && !tem_regua && linhas[0].len() > 1 {
            // Uma linha só, sem réguas: é arrumação de texto (nome à esquerda, data à direita), não tabela.
            let texto = linha_como_texto(&linhas[0], colunas.trim_end_matches(['@', '{', '}']).ends_with('r'));
            self.push_bloco(Conteudo::Paragrafo { texto });
        } else if !linhas.is_empty() {
            self.push_bloco(Conteudo::Tabela { cabecalho, linhas });
        }
    }

    fn lista(&mut self, corpo: &str, ordenado: bool) {
        let nivel = self.ctx.nivel_lista;
        let (antes, itens) = itens_de(corpo);
        // `\begin{itemize}\small`: o que vem antes do primeiro `\item` vale para a lista toda.
        let mut e = self.ctx.estado;
        inline_em(antes, &self.ctx, &mut e);
        let ctx = Ctx { nivel_lista: nivel + 1, estado: e, vspace: 0.0, ..self.ctx.clone() };
        for item in itens {
            let mut filhos = Analisador::novo(item, ctx.clone()).executar().into_iter();
            let (texto, outro) = match filhos.next() {
                Some(Bloco { conteudo: Conteudo::Paragrafo { texto }, .. }) => (texto, None),
                Some(outro) => (Vec::new(), Some(outro)),
                None => (Vec::new(), None),
            };
            self.push_bloco(Conteudo::Item { ordenado, nivel: nivel.min(5), texto });
            self.saida.extend(outro);
            self.saida.extend(filhos);
        }
    }
}

/// Separa o corpo de uma lista nos `\item` do nível dele (ignora os de listas aninhadas); devolve
/// também o que vem antes do primeiro item.
fn itens_de(corpo: &str) -> (&str, Vec<&str>) {
    let mut itens = Vec::new();
    let mut prof = 0;
    let mut inicio: Option<usize> = None;
    let mut j = 0;
    while j < corpo.len() {
        let r = &corpo[j..];
        if r.starts_with("\\begin{") {
            prof += 1;
        } else if r.starts_with("\\end{") {
            prof -= 1;
        } else if prof == 0 && r.starts_with("\\item") && !r[5..].starts_with(|c: char| c.is_ascii_alphabetic()) {
            if let Some(a) = inicio {
                itens.push(&corpo[a..j]);
            }
            let mut ini = j + 5;
            if let Some((_, k)) = opcional(&corpo[ini..]) {
                ini += k;
            }
            inicio = Some(ini);
            j = ini;
            continue;
        }
        j += r.chars().next().map_or(1, char::len_utf8);
    }
    if let Some(a) = inicio {
        itens.push(&corpo[a..]);
    }
    let antes = corpo.find("\\item").map_or(corpo, |i| &corpo[..i]);
    (antes, itens)
}

/// Linhas (`\\`) e células (`&`) de um `tabular`, sem as réguas.
fn linhas_de_tabela(corpo: &str, ctx: &Ctx) -> Vec<Vec<Celula>> {
    let mut limpo = corpo.to_string();
    for regua in ["\\hline", "\\toprule", "\\midrule", "\\bottomrule"] {
        limpo = limpo.replace(regua, "");
    }
    limpo
        .split("\\\\")
        .filter(|l| !l.trim().is_empty())
        .map(|l| dividir_celulas(l).iter().map(|c| inline_em(c, ctx, &mut ctx.estado.clone())).collect())
        .collect()
}

/// As células de uma linha como texto corrido: a última vai para a direita (`\t`) se a coluna é `r`.
fn linha_como_texto(celulas: &[Celula], ultima_a_direita: bool) -> Vec<Trecho> {
    let mut saida: Vec<Trecho> = Vec::new();
    for (k, c) in celulas.iter().enumerate() {
        if k > 0 {
            let sep = if ultima_a_direita && k + 1 == celulas.len() { "\t" } else { " " };
            saida.push(Trecho { texto: sep.into(), ..c.first().cloned().unwrap_or_default() });
            if let Some(t) = saida.last_mut() {
                t.link = None;
            }
        }
        saida.extend(c.iter().cloned());
    }
    saida
}

fn dividir_celulas(linha: &str) -> Vec<String> {
    let mut celulas = vec![String::new()];
    let mut anterior = ' ';
    for c in linha.chars() {
        if c == '&' && anterior != '\\' {
            celulas.push(String::new());
        } else if let Some(u) = celulas.last_mut() {
            u.push(c);
        }
        anterior = c;
    }
    celulas
}

#[cfg(test)]
mod testes {
    use super::*;
    use featherdocs_core::edicao::nome_do_tipo;

    fn tipos(d: &Documento) -> Vec<String> {
        d.blocos.iter().map(|b| nome_do_tipo(&b.conteudo)).collect()
    }

    #[test]
    fn documento_completo() {
        let d = ler(
            "\\documentclass{article}\n\\title{Meu \\textbf{artigo}}\\author{Ana}\n\\begin{document}\n\\maketitle\n\\section{Introdu\\c{c}\\~ao}\nTexto com \\emph{\\'enfase} e $x^2$.\n\n\\subsection*{Sub}\nOutro.\n\\end{document}",
        );
        assert_eq!(d.meta.titulo, "Meu artigo");
        assert_eq!(tipos(&d), vec!["titulo1", "paragrafo", "titulo1", "paragrafo", "titulo2", "paragrafo"]);
        let Conteudo::Titulo { texto, .. } = &d.blocos[2].conteudo else { panic!() };
        assert_eq!(texto_puro(texto), "Introdução");
    }

    #[test]
    fn listas_aninhadas() {
        let d = ler("\\begin{itemize}\\item a \\item b\n\\begin{enumerate}\\item c\\end{enumerate}\n\\item d\\end{itemize}");
        let v: Vec<(bool, u8)> = d.blocos.iter().map(|b| if let Conteudo::Item { ordenado, nivel, .. } = b.conteudo { (ordenado, nivel) } else { (false, 99) }).collect();
        assert_eq!(v, vec![(false, 0), (false, 0), (true, 1), (false, 0)]);
    }

    #[test]
    fn formulas_codigo_e_tabela() {
        let d = ler("\\[ E = mc^2 \\]\n\\begin{equation}\\label{a} x \\end{equation}\n\\begin{verbatim}\nint a;\n\\end{verbatim}\n\\begin{tabular}{|l|l|}\\hline A & B \\\\ \\hline 1 & 2 \\\\ \\hline\\end{tabular}");
        assert_eq!(tipos(&d), vec!["equacao", "equacao", "codigo", "tabela"]);
        assert!(matches!(&d.blocos[1].conteudo, Conteudo::Equacao { latex } if latex == "\\begin{equation}\\label{a} x\\end{equation}"));
        let Conteudo::Tabela { cabecalho, linhas } = &d.blocos[3].conteudo else { panic!() };
        assert!(*cabecalho);
        assert_eq!(linhas.len(), 2);
    }

    #[test]
    fn figura_com_legenda() {
        let d = ler("\\begin{figure}\\includegraphics[width=3cm]{img/a.png}\\caption{Uma figura}\\end{figure}");
        let Conteudo::Imagem { origem, legenda, .. } = &d.blocos[0].conteudo else { panic!() };
        assert_eq!(origem, "img/a.png");
        assert_eq!(texto_puro(legenda), "Uma figura");
    }

    #[test]
    fn comentarios_e_pontuacao() {
        let d = ler("Antes % some\ndepois -- e --- ``aspas'' 50\\% \\& mais.\n");
        let Conteudo::Paragrafo { texto } = &d.blocos[0].conteudo else { panic!() };
        assert_eq!(texto_puro(texto), "Antes depois – e — “aspas” 50% & mais.");
    }

    #[test]
    fn link_e_codigo_em_linha() {
        let d = ler("Veja \\href{http://x.org}{o site} e \\texttt{ls -l}.");
        let Conteudo::Paragrafo { texto } = &d.blocos[0].conteudo else { panic!() };
        assert_eq!(texto[1].link.as_deref(), Some("http://x.org"));
        assert!(texto.iter().any(|t| t.estilo & CODIGO != 0 && t.texto == "ls -l"));
    }

    #[test]
    fn capitulos_empurram_as_secoes() {
        let d = ler("\\chapter{A}\\section{B}");
        assert!(matches!(d.blocos[0].conteudo, Conteudo::Titulo { nivel: 1, .. }));
        assert!(matches!(d.blocos[1].conteudo, Conteudo::Titulo { nivel: 2, .. }));
    }

    #[test]
    fn lixo_nao_trava() {
        for f in ["\\begin{itemize}", "\\section{", "$$ sem fim", "\\href{a", "{{{", "\\", "%"] {
            let _ = ler(f);
        }
    }

    #[test]
    fn exportar_e_reler() {
        let md = featherdocs_core::inline::analisar("Texto **forte** e *ênfase*.");
        let mut d = Documento::vazio();
        d.acrescentar(Conteudo::Titulo { nivel: 1, texto: featherdocs_core::inline::analisar("Intro") });
        d.acrescentar(Conteudo::Paragrafo { texto: md });
        let volta = ler(&crate::exportar::latex(&d));
        assert_eq!(crate::exportar::texto_simples(&d), crate::exportar::texto_simples(&volta));
    }

    #[test]
    fn notas_e_indice_viram_objetos_e_voltam() {
        let d = ler("\\documentclass{article}\\begin{document}\nGato\\index{gato!felino}\\footnote{Um \\textbf{bicho}.} e cão\\endnote{Do fim.}.\n\\end{document}");
        let Conteudo::Paragrafo { texto } = &d.blocos[0].conteudo else { panic!("{:?}", d.blocos) };
        let marcas: Vec<&Marca> = texto.iter().filter_map(|t| t.marca.as_deref()).collect();
        assert!(matches!(marcas[0], Marca::Indice { termo, sub } if termo == "gato" && sub == "felino"));
        assert!(matches!(marcas[1], Marca::Nota { fim: false, texto } if featherdocs_core::inline::serializar(texto) == "Um **bicho**."), "{marcas:?}");
        assert!(matches!(marcas[2], Marca::Nota { fim: true, .. }));
        assert_eq!(featherdocs_core::inline::texto_puro(texto), "Gato e cão.");
        let tex = crate::exportar::latex(&d);
        let volta = ler(&tex);
        let Conteudo::Paragrafo { texto: t2 } = &volta.blocos[0].conteudo else { panic!() };
        assert_eq!(t2.iter().filter(|t| t.marca.is_some()).count(), 3, "{tex}");
    }

    #[test]
    fn citacoes_fontes_e_listas_geradas_vao_e_voltam_pelo_latex() {
        let tex = r"\begin{filecontents*}{\jobname.bib}
@book{silva2023, author={Silva, Ana}, title={Livro}, year={2023}}
\end{filecontents*}
\documentclass{article}
\usepackage[style=apa]{biblatex}
\begin{document}
Como diz \cite[p.~4]{silva2023} e \parencite{a, b}.

\nocite{*}
\printbibliography[heading=bibintoc]

\printindex
\end{document}";
        let d = ler(tex);
        assert_eq!(d.bibliografia.estilo, EstiloDeCitacao::Apa);
        assert_eq!(d.bibliografia.fontes.len(), 1);
        let Conteudo::Paragrafo { texto } = &d.blocos[0].conteudo else { panic!("{:?}", d.blocos) };
        let m: Vec<&Marca> = texto.iter().filter_map(|t| t.marca.as_deref()).collect();
        assert!(matches!(m[0], Marca::Citacao { chaves, pagina } if chaves == &["silva2023"] && pagina == "4"), "{m:?}");
        assert!(matches!(m[1], Marca::Citacao { chaves, .. } if chaves == &["a", "b"]));
        let tipos: Vec<String> = d.blocos.iter().map(|b| featherdocs_core::edicao::nome_do_tipo(&b.conteudo)).collect();
        assert_eq!(tipos, ["paragrafo", "bibliografia", "indice_remissivo"]);
        assert!(matches!(d.blocos[1].conteudo, Conteudo::Bibliografia { todas: true }));
        let volta = ler(&crate::exportar::latex(&d));
        assert_eq!(volta.bibliografia, d.bibliografia);
        assert_eq!(volta.blocos.len(), 3);
    }

    #[test]
    fn secao_sem_estrela_liga_a_numeracao_e_a_com_estrela_fica_fora() {
        let d = ler("\\begin{document}\\section*{Prefácio}\\section{Um}\\subsection{Dois}\\end{document}");
        assert_eq!(d.estilos.numeracao, Numeracao::Decimal);
        let fora: Vec<bool> = d.blocos.iter().map(|b| b.atrib.sem_numero).collect();
        assert_eq!(fora, [true, false, false]);
        assert_eq!(ler("\\section*{A}").estilos.numeracao, Numeracao::Nenhuma);
        assert_eq!(ler("\\chapter{A}\\section{B}").estilos.numeracao, Numeracao::Capitulo);
    }

    #[test]
    fn numeracao_vai_e_volta_pelo_latex() {
        let d = ler("\\begin{document}\\section*{Prefácio}\\section{Um}\\subsection{Dois}\\end{document}");
        let tex = crate::exportar::latex(&d);
        assert!(tex.contains("\\section*{Pref") && tex.contains("\\section{Um}") && tex.contains("\\subsection{Dois}"), "{tex}");
        let volta = ler(&tex);
        assert_eq!(volta.estilos.numeracao, d.estilos.numeracao);
        assert_eq!(volta.blocos.iter().map(|b| b.atrib.sem_numero).collect::<Vec<_>>(), [true, false, false]);
        let mut sem = d.clone();
        sem.estilos.numeracao = Numeracao::Nenhuma;
        assert!(crate::exportar::latex(&sem).contains("\\section*{Um}"));
    }

    #[test]
    fn cores_macros_e_colorbox() {
        let tex = r#"
\definecolor{darkblue}{HTML}{1A3A5C}
\definecolor{midblue}{HTML}{2E6DA4}
\newcommand{\destaque}[1]{\textbf{\color{midblue}#1}}
\begin{document}
\colorbox{darkblue}{
  \begin{minipage}{10cm}
    \centering
    \color{white}Meu Nome
  \end{minipage}
}
\destaque{Importante}
\end{document}
"#;
        let d = ler(tex);
        assert!(!d.blocos.is_empty());
        // O primeiro bloco tem fundo darkblue (0x1A3A5CFF)
        assert_eq!(d.blocos[0].atrib.fundo, 0x1A3A5CFF);
        assert_eq!(d.blocos[0].atrib.alinhamento, Alinhamento::Centro);
        // O texto "Meu Nome" é branco (0xFFFFFFFF)
        if let Conteudo::Paragrafo { texto } = &d.blocos[0].conteudo {
            assert!(texto.iter().any(|t| t.fmt.cor == 0xFFFFFFFF && t.texto.contains("Meu Nome")));
        } else {
            panic!("esperado parágrafo");
        }
        // O macro \destaque virou negrito com midblue (0x2E6DA4FF)
        if let Conteudo::Paragrafo { texto } = &d.blocos[1].conteudo {
            assert!(texto.iter().any(|t| t.fmt.cor == 0x2E6DA4FF && (t.estilo & NEGRITO != 0)));
        } else {
            panic!("esperado parágrafo");
        }
    }

    /// Um resumo do `curriculo_marcelo.tex` da raiz (o que o leitor tem que acertar).
    const CURRICULO: &str = r#"
\documentclass[10pt, a4paper]{article}
\usepackage[utf8]{inputenc}
\usepackage[T1]{fontenc}
\usepackage[portuguese]{babel}
\usepackage[a4paper, top=13mm, bottom=13mm, left=17mm, right=17mm]{geometry}
\setlength{\parindent}{0pt}
\setlength{\parskip}{0pt}
\pagestyle{empty}
\usepackage{helvet}
\renewcommand{\familydefault}{\sfdefault}
\usepackage[dvipsnames, table]{xcolor}
\definecolor{darkblue}{HTML}{1A3A5C}
\definecolor{midblue}{HTML}{2E6DA4}
\definecolor{lightblue}{HTML}{D6E8F7}
\definecolor{lightgray}{HTML}{F4F6F9}
\definecolor{graytext}{HTML}{555555}
\definecolor{darktext}{HTML}{1C1C1C}
\usepackage{tabularx}
\usepackage{colortbl}
\usepackage{array}
\usepackage{enumitem}
\newcommand{\secline}{
  \vspace{1pt}
  {\color{midblue}\hrule height 0.8pt}
  \vspace{4pt}
}
\newcommand{\seccao}[1]{
  \vspace{6pt}
  {\large\bfseries\color{darkblue}\MakeUppercase{#1}}
  \par\secline
}
\newcommand{\expheader}[3]{
  \vspace{4pt}
  \noindent
  \colorbox{lightgray}{
    \begin{tabularx}{\dimexpr\textwidth-2\fboxsep\relax}{@{}X r@{}}
      \textbf{\color{midblue}#1}\;{\color{graytext}|\ #2} &
      {\small\itshape\color{graytext}#3}
    \end{tabularx}
  }
  \par\vspace{2pt}
}
\newcommand{\edu}[3]{
  \noindent\textbf{#1} --- {\color{midblue}#2} \hfill {\small\itshape\color{graytext}#3}\par
  \vspace{2pt}
}
\newcommand{\skill}[2]{
  \item \textbf{\color{midblue}#1:} {\color{darktext}#2}
}
\begin{document}
\noindent
\colorbox{darkblue}{
  \begin{minipage}{\dimexpr\textwidth-2\fboxsep\relax}
    \vspace{8pt}
    \centering
    \textbf{\LARGE\color{white}Marcelo A. Gonçalves}\\[3pt]
    {\color{lightblue}Engenheiro Eletricista · Cientista de Dados \& Automação}
    \vspace{8pt}
  \end{minipage}
}\par

\seccao{Perfil Profissional}
Engenheiro Eletricista com \textbf{CREA ativo}.

\seccao{Experiência Profissional}
\expheader{SANDECH}{Engenheiro}{Jan 2026 -- Atualmente}
\begin{itemize}
  \item \textbf{Automação com Python e VBA:} redução do lead time.
\end{itemize}

\seccao{Habilidades Técnicas}
\begin{itemize}
  \skill{Linguagens}{Python (Pandas, SQLAlchemy, Flask), SQL, VBA, JavaScript}
\end{itemize}
\end{document}
"#;

    #[test]
    fn curriculo_marcelo_completo() {
        let tex = CURRICULO;
        let d = ler(tex);
        let par = |b: &Bloco| match &b.conteudo {
            Conteudo::Paragrafo { texto } | Conteudo::Item { texto, .. } => texto.clone(),
            _ => Vec::new(),
        };
        // Papel e margens do geometry; sem número de página (\pagestyle{empty}).
        assert!((d.pagina.margem_esq - 17.0 * 72.0 / 25.4).abs() < 0.01);
        assert!((d.pagina.margem_sup - 13.0 * 72.0 / 25.4).abs() < 0.01);
        assert!(d.extras.rodape.vazia());
        // A faixa do cabeçalho: o \\ parte em dois parágrafos, os dois com o fundo e centralizados.
        let (nome, sub) = (&d.blocos[0], &d.blocos[1]);
        for b in [nome, sub] {
            assert_eq!(b.atrib.fundo, 0x1A3A5CFF);
            assert_eq!(b.atrib.alinhamento, Alinhamento::Centro);
        }
        let t = par(nome);
        assert_eq!(texto_puro(&t), "Marcelo A. Gonçalves");
        assert!(t.iter().all(|t| t.fmt.cor == 0xFFFFFFFF && t.fmt.tamanho == 17.28 && t.estilo & NEGRITO != 0), "{t:?}");
        assert!(par(sub).iter().any(|t| t.fmt.cor == 0xD6E8F7FF && t.texto.contains("Engenheiro Eletricista") && t.fmt.tamanho == 10.0));
        assert_eq!(sub.atrib.antes, Some(0.0));
        // Seção: \large (12 pt na classe 10pt), negrito, darkblue e em maiúsculas sem estragar comandos.
        let secao = d.blocos.iter().find(|b| texto_puro(&par(b)) == "PERFIL PROFISSIONAL").expect("seção");
        assert!(par(secao).iter().all(|t| t.fmt.tamanho == 12.0 && t.estilo & NEGRITO != 0 && t.fmt.cor == 0x1A3A5CFF));
        // A régua da seção na cor midblue.
        assert!(d.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::Regua) && b.atrib.cor_borda == 0x2E6DA4FF));
        // A tarja da experiência: um parágrafo com fundo, o período encostado à direita (tabulação).
        let tarja = d.blocos.iter().find(|b| b.atrib.fundo == 0xF4F6F9FF).expect("tarja");
        let t = texto_puro(&par(tarja));
        assert!(t.starts_with("SANDECH | Engenheiro\tJan 2026"), "{t:?}");
        // O texto corrido sai justificado, como no LaTeX; o cabeçalho continua centralizado.
        let perfil = d.blocos.iter().find(|b| texto_puro(&par(b)).starts_with("Engenheiro Eletricista com")).expect("perfil");
        assert_eq!(perfil.atrib.alinhamento, Alinhamento::Justificado);
                // O item da habilidade: texto em darktext.
        assert!(d.blocos.iter().any(|b| matches!(b.conteudo, Conteudo::Item { .. }) && par(b).iter().any(|t| t.fmt.cor == 0x1C1C1CFF)));
    }
}
