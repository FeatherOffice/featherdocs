//! Um pedaço do TikZ vira desenho SVG: nós (`\node (a) [right=of b] {texto};`, com `draw`, `fill`,
//! `rounded corners`, `circle`, `minimum width/height`, `font`, linhas com `\\`), linhas e setas
//! (`\draw[->] (a) -- (b);`, `-|`, `|-`, entre nós, âncoras como `a.east` ou coordenadas), `node`
//! no meio do caminho, `rectangle`, `circle`, `thick`, `dashed`, `node distance`, `x=`/`y=`,
//! `every node/.style` e estilos com nome (`estilo/.style`).
//! Do preâmbulo: os estilos do `\tikzset` (também com parâmetro, `quadrado=#1`), os `pics` e as
//! variáveis do `\tikzmath`; coordenadas com conta (`(\col+\dist*2,\lin)`).
//! shortcut: sem `\foreach`, curvas (`to[bend]`, `..controls..`) e `\coordinate`; o que não for lido
//! vai para o aviso pelo chamador.

use std::collections::HashMap;
use std::fmt::Write;

use super::{delimitado, grupo, opcional};

/// 1 cm em pontos.
const CM: f32 = 28.452_756;
/// O corpo do texto dos nós (o `\normalsize` de 10 pt).
const CORPO: f32 = 10.0;
/// O `inner sep` padrão (0,3333 em).
const FOLGA: f32 = 3.333;
/// A espessura padrão das linhas do TikZ.
const FINA: f32 = 0.4;

#[derive(Clone, Default)]
struct Estilo {
    borda: bool,
    fundo: Option<u32>,
    cor: Option<u32>,
    texto: Option<u32>,
    arredondado: bool,
    circulo: bool,
    largura_min: f32,
    altura_min: f32,
    /// 0 = a padrão (`FINA`).
    espessura: f32,
    tracejado: Option<&'static str>,
    /// 0 = `CORPO`.
    fonte: f32,
    /// Ponta no início e no fim (`<-`, `->`, `-{Stealth}`).
    pontas: (bool, bool),
}

struct No {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    linhas: Vec<String>,
    estilo: Estilo,
}

/// O que o preâmbulo define para os desenhos: os estilos do `\tikzset` (como opções), os `pics`
/// (nome → código) e as variáveis do `\tikzmath`, das mais longas para as mais curtas.
#[derive(Debug, Default)]
pub(super) struct Definicoes {
    estilos: String,
    pics: HashMap<String, String>,
    variaveis: Vec<(String, f32)>,
}

/// As `Definicoes` do preâmbulo.
pub(super) fn definicoes(preambulo: &str) -> Definicoes {
    let sem_comentarios: String = preambulo.lines().map(|l| l.split('%').next().unwrap_or("")).collect::<Vec<_>>().join("\n");
    let mut d = Definicoes::default();
    for (i, _) in sem_comentarios.match_indices("\\tikzmath") {
        let Some((g, _)) = grupo(&sem_comentarios[i + 9..]) else { continue };
        for frase in g.split(';') {
            let Some((nome, conta)) = frase.split_once('=') else { continue };
            let nome = nome.trim();
            if nome.starts_with('\\') && !nome.contains(' ') {
                if let Some(v) = calcular(&trocar_variaveis(conta, &d.variaveis)) {
                    d.variaveis.push((nome.to_string(), v));
                    d.variaveis.sort_by_key(|(n, _)| std::cmp::Reverse(n.len()));
                }
            }
        }
    }
    for (i, _) in sem_comentarios.match_indices("\\tikzset") {
        let Some((g, _)) = grupo(&sem_comentarios[i + 8..]) else { continue };
        for (k, v) in opcoes(g) {
            // `pics/nome/.style n args={2}{code={…}}` ou `pics/nome/.style={code={…}}`.
            if let Some(nome) = k.strip_prefix("pics/").and_then(|r| r.split_once("/.style")).map(|(n, _)| n.trim()) {
                // O `opcoes` já tirou a chave de fora: `{2}{code=…}` chega como `2}{code=…`.
                let corpo = if k.contains("args") { v.split_once("}{").map_or(v.as_str(), |(_, r)| r) } else { v.as_str() };
                if let Some((_, codigo)) = opcoes(corpo).into_iter().find(|(c, _)| c == "code") {
                    d.pics.insert(nome.to_string(), codigo);
                }
            } else if k.ends_with("/.style") {
                d.estilos.push_str(&format!("{k}={{{v}}},"));
            }
        }
    }
    d.estilos = trocar_variaveis(&d.estilos, &d.variaveis);
    d
}

/// O corpo de um desenho com o que o preâmbulo define: os estilos na frente (como opções do
/// `tikzpicture`), cada `\pic{nome={a}{b}}` trocado pelo código dele e as variáveis pelos valores.
pub(super) fn com_definicoes(corpo: &str, d: &Definicoes) -> String {
    let mut corpo = corpo.trim_start().to_string();
    while let Some(i) = corpo.find("\\pic") {
        let mut j = i + 4;
        if let Some((_, k)) = opcional(&corpo[j..]) {
            j += k;
        }
        let Some((g, k)) = grupo(corpo[j..].trim_start()) else { break };
        let fim = j + (corpo[j..].len() - corpo[j..].trim_start().len()) + k;
        let (nome, args) = g.split_once('=').unwrap_or((g, ""));
        let mut codigo = d.pics.get(nome.trim()).cloned().unwrap_or_default();
        let mut resto = args.trim();
        let mut n = 1;
        while let Some((a, k)) = grupo(resto) {
            codigo = codigo.replace(&format!("#{n}"), a);
            resto = resto[k..].trim_start();
            n += 1;
        }
        corpo.replace_range(i..fim, &codigo);
    }
    let corpo = trocar_variaveis(&corpo, &d.variaveis);
    match opcional(&corpo) {
        Some((o, k)) => format!("[{}{o}]{}", d.estilos, &corpo[k..]),
        None if d.estilos.is_empty() => corpo,
        None => format!("[{}]{corpo}", d.estilos.trim_end_matches(',')),
    }
}

fn trocar_variaveis(s: &str, variaveis: &[(String, f32)]) -> String {
    let mut s = s.to_string();
    for (nome, v) in variaveis {
        s = s.replace(nome.as_str(), &v.to_string());
    }
    s
}

/// Uma conta com `+ - * /` e parênteses (`(2.2+1.2*2)`); `None` se não é uma.
fn calcular(s: &str) -> Option<f32> {
    fn soma(s: &[u8], i: &mut usize) -> Option<f32> {
        let mut v = produto(s, i)?;
        loop {
            match pula(s, i) {
                Some(b'+') => { *i += 1; v += produto(s, i)?; }
                Some(b'-') => { *i += 1; v -= produto(s, i)?; }
                _ => return Some(v),
            }
        }
    }
    fn produto(s: &[u8], i: &mut usize) -> Option<f32> {
        let mut v = fator(s, i)?;
        loop {
            match pula(s, i) {
                Some(b'*') => { *i += 1; v *= fator(s, i)?; }
                Some(b'/') => { *i += 1; v /= fator(s, i)?; }
                _ => return Some(v),
            }
        }
    }
    fn fator(s: &[u8], i: &mut usize) -> Option<f32> {
        match pula(s, i)? {
            b'(' => {
                *i += 1;
                let v = soma(s, i)?;
                (pula(s, i)? == b')').then(|| *i += 1)?;
                Some(v)
            }
            b'-' => { *i += 1; Some(-fator(s, i)?) }
            _ => {
                let ini = *i;
                while *i < s.len() && (s[*i].is_ascii_digit() || s[*i] == b'.') {
                    *i += 1;
                }
                std::str::from_utf8(&s[ini..*i]).ok()?.parse().ok()
            }
        }
    }
    fn pula(s: &[u8], i: &mut usize) -> Option<u8> {
        while *i < s.len() && s[*i].is_ascii_whitespace() {
            *i += 1;
        }
        s.get(*i).copied()
    }
    let b = s.as_bytes();
    let mut i = 0;
    let v = soma(b, &mut i)?;
    (pula(b, &mut i).is_none()).then_some(v)
}

/// Um comprimento do TikZ em pontos (sem unidade = cm).
fn medida(s: &str) -> Option<f32> {
    let s = s.trim();
    let fim = s.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).unwrap_or(s.len());
    let v: f32 = s[..fim].parse().ok()?;
    Some(v * match s[fim..].trim() {
        "" | "cm" => CM,
        "mm" => CM / 10.0,
        "pt" => 1.0,
        "in" => 72.27,
        "em" => CORPO,
        "ex" => CORPO * 0.43,
        _ => return None,
    })
}

/// As opções `chave=valor` separadas por vírgula (o `=` e a vírgula valem só fora de chaves).
fn opcoes(s: &str) -> Vec<(String, String)> {
    let mut saida = Vec::new();
    let mut prof = 0;
    let mut atual = String::new();
    let mut igual = None;
    for c in s.chars().chain(std::iter::once(',')) {
        match c {
            '{' => prof += 1,
            '}' => prof -= 1,
            _ => {}
        }
        if c == ',' && prof == 0 {
            let (k, v) = igual.map_or((atual.as_str(), ""), |i| (&atual[..i], &atual[i + 1..]));
            let k = k.trim().to_string();
            if !k.is_empty() {
                let v = v.trim();
                let v = v.strip_prefix('{').and_then(|v| v.strip_suffix('}')).unwrap_or(v);
                saida.push((k, v.trim().to_string()));
            }
            atual.clear();
            igual = None;
        } else {
            if c == '=' && prof == 0 && igual.is_none() {
                igual = Some(atual.len());
            }
            atual.push(c);
        }
    }
    saida
}

/// `->`, `<->`, `-{Stealth[length=2mm]}`, `-`: se há ponta no início e no fim.
fn pontas(k: &str) -> Option<(bool, bool)> {
    if k.contains(' ') {
        return None;
    }
    let mut prof = 0;
    for (i, c) in k.char_indices() {
        match c {
            '{' | '[' => prof += 1,
            '}' | ']' => prof -= 1,
            '-' if prof == 0 => return Some((i > 0, i + 1 < k.len())),
            _ => {}
        }
    }
    None
}

/// O corpo das declarações de tamanho do `font=` (classe de 10 pt).
fn fonte(v: &str) -> Option<f32> {
    [("\\tiny", 5.0), ("\\scriptsize", 7.0), ("\\footnotesize", 8.0), ("\\small", 9.0), ("\\normalsize", 10.0), ("\\large", 12.0), ("\\Large", 14.4), ("\\LARGE", 17.28)]
        .iter()
        .rev()
        .find(|(n, _)| v.split('\\').any(|p| format!("\\{}", p.trim()) == *n))
        .map(|(_, t)| *t)
}

/// O texto de uma linha do nó: os símbolos de fórmula mais comuns viram o caractere.
fn texto_da_linha(s: &str) -> String {
    const SIMBOLOS: [(&str, &str); 16] = [("times", "×"), ("cdot", "·"), ("pm", "±"), ("approx", "≈"), ("leq", "≤"), ("geq", "≥"), ("le", "≤"), ("ge", "≥"), ("to", "→"), ("rightarrow", "→"), ("degree", "°"), ("Omega", "Ω"), ("mu", "µ"), ("alpha", "α"), ("beta", "β"), ("eta", "η")];
    let mut saida = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(p) = s[i..].find('\\') {
        saida.push_str(&s[i..i + p]);
        i += p;
        let (nome, n) = super::comando(&s[i..]);
        match SIMBOLOS.iter().find(|(c, _)| *c == nome) {
            Some((_, para)) => saida.push_str(para),
            None => saida.push_str(&s[i..i + n]),
        }
        i += n.max(1);
    }
    saida.push_str(&s[i..]);
    let s = saida.replace('$', "");
    super::texto_puro(&super::inline(&s, &HashMap::new())).trim().to_string()
}

struct Desenho<'a> {
    cor: &'a dyn Fn(&str) -> Option<u32>,
    distancia: (f32, f32),
    /// O tamanho da unidade de `x` e de `y` (`x=0.5cm`).
    unidade: (f32, f32),
    todo_no: Estilo,
    estilos: HashMap<String, Vec<(String, String)>>,
    nos: HashMap<String, usize>,
    lista: Vec<No>,
    svg: String,
    pontos: Vec<(f32, f32)>,
    /// As cores das setas (cada uma tem o seu marcador).
    setas: Vec<String>,
}

impl Desenho<'_> {
    /// As opções com os estilos com nome trocados pelo que eles dizem.
    fn expandir(&self, o: &[(String, String)], prof: u8) -> Vec<(String, String)> {
        let mut saida = Vec::new();
        for (k, v) in o {
            match self.estilos.get(k) {
                Some(e) if v.is_empty() && prof < 8 => saida.extend(self.expandir(e, prof + 1)),
                // Estilo com parâmetro: `quadrado=verde` troca o `#1` dele.
                Some(e) if prof < 8 && e.iter().any(|(a, b)| a.contains("#1") || b.contains("#1")) => {
                    let com_valor: Vec<(String, String)> = e.iter().map(|(a, b)| (a.replace("#1", v), b.replace("#1", v))).collect();
                    saida.extend(self.expandir(&com_valor, prof + 1));
                }
                _ => saida.push((k.clone(), v.clone())),
            }
        }
        saida
    }

    fn estilo(&self, base: &Estilo, o: &[(String, String)]) -> Estilo {
        let mut e = base.clone();
        for (k, v) in self.expandir(o, 0) {
            match k.as_str() {
                "draw" => {
                    e.borda = v != "none";
                    if !v.is_empty() {
                        e.cor = (self.cor)(&v);
                    }
                }
                "fill" => e.fundo = (self.cor)(&v),
                "text" => e.texto = (self.cor)(&v),
                "color" => e.cor = (self.cor)(&v),
                "rounded corners" => e.arredondado = true,
                "sharp corners" => e.arredondado = false,
                "circle" => e.circulo = true,
                "rectangle" => e.circulo = false,
                "minimum width" => e.largura_min = medida(&v).unwrap_or(0.0),
                "minimum height" => e.altura_min = medida(&v).unwrap_or(0.0),
                "minimum size" => {
                    let m = medida(&v).unwrap_or(0.0);
                    (e.largura_min, e.altura_min) = (m, m);
                }
                "ultra thin" => e.espessura = 0.1,
                "very thin" => e.espessura = 0.2,
                "thin" => e.espessura = 0.4,
                "semithick" => e.espessura = 0.6,
                "thick" => e.espessura = 0.8,
                "very thick" => e.espessura = 1.2,
                "ultra thick" => e.espessura = 1.6,
                "line width" => e.espessura = medida(&v).unwrap_or(FINA),
                "dashed" => e.tracejado = Some("3 3"),
                "densely dashed" => e.tracejado = Some("3 2"),
                "loosely dashed" => e.tracejado = Some("3 6"),
                "dotted" | "densely dotted" => e.tracejado = Some("0.4 2"),
                "solid" => e.tracejado = None,
                "font" => e.fonte = fonte(&v).unwrap_or(e.fonte),
                // Um nome de cor sozinho (`\draw[red]`).
                c if v.is_empty() && (self.cor)(c).is_some() => e.cor = (self.cor)(c),
                c if v.is_empty() => e.pontas = pontas(c).unwrap_or(e.pontas),
                _ => {}
            }
        }
        e
    }

    /// `right=of a`, `below=2cm of a`, `below right=of a`: o centro do nó novo.
    fn posicao(&self, o: &[(String, String)], w: f32, h: f32) -> Option<(f32, f32)> {
        for (k, v) in o {
            let Some((dist, nome)) = v.split_once("of ").map(|(d, n)| (d.trim(), n.trim())) else { continue };
            let Some(&i) = self.nos.get(nome) else { continue };
            let a = &self.lista[i];
            let (dv, dh) = medida(dist).map_or(self.distancia, |d| (d, d));
            let (mut x, mut y) = (a.x, a.y);
            for lado in k.split_whitespace() {
                match lado {
                    "right" => x = a.x + a.w / 2.0 + dh + w / 2.0,
                    "left" => x = a.x - a.w / 2.0 - dh - w / 2.0,
                    "below" => y = a.y + a.h / 2.0 + dv + h / 2.0,
                    "above" => y = a.y - a.h / 2.0 - dv - h / 2.0,
                    _ => return None,
                }
            }
            return Some((x, y));
        }
        None
    }

    fn cor_hex(c: u32) -> String {
        format!("#{:06x}", c >> 8)
    }

    /// Um nó ainda sem lugar: o texto (uma linha por `\\`) e o tamanho.
    fn novo_no(&self, texto: &str, o: &[(String, String)]) -> No {
        let estilo = self.estilo(&self.todo_no, o);
        let corpo = if estilo.fonte > 0.0 { estilo.fonte } else { CORPO };
        let linhas: Vec<String> = texto.split("\\\\").map(texto_da_linha).collect();
        let largura_texto = linhas.iter().map(|l| featherdocs_core::medida::largura(l, featherdocs_core::modelo::SERIFA, corpo)).fold(0.0, f32::max);
        let mut w = (largura_texto + 2.0 * FOLGA).max(estilo.largura_min);
        let mut h = (corpo * 1.2 * linhas.len() as f32 + 2.0 * FOLGA).max(estilo.altura_min);
        if estilo.circulo {
            w = w.max(h);
            h = w;
        }
        No { x: 0.0, y: 0.0, w, h, linhas, estilo }
    }

    /// `\node[opções] (nome) at (x,y) {texto}` (as opções podem vir antes ou depois do nome).
    fn no(&mut self, mut r: &str) -> Option<()> {
        let mut o = Vec::new();
        let mut nome = None;
        let mut em = None;
        loop {
            r = r.trim_start();
            if let Some((g, k)) = opcional(r) {
                o.extend(opcoes(g));
                r = &r[k..];
            } else if let Some((g, k)) = delimitado(r, '(', ')') {
                nome = Some(g.trim().to_string());
                r = &r[k..];
            } else if let Some(depois) = r.strip_prefix("at") {
                let (g, k) = delimitado(depois.trim_start(), '(', ')')?;
                em = Some(self.coordenada(g)?);
                r = &depois.trim_start()[k..];
            } else {
                break;
            }
        }
        let (texto, _) = grupo(r)?;
        let mut n = self.novo_no(texto, &o);
        (n.x, n.y) = em.or_else(|| self.posicao(&o, n.w, n.h)).unwrap_or((0.0, 0.0));
        if let Some(nome) = nome {
            self.nos.insert(nome, self.lista.len());
        }
        self.lista.push(n);
        Some(())
    }

    /// Um número do TikZ: com unidade, em pontos; sem, na unidade do eixo.
    fn comprimento(s: &str, unidade: f32) -> Option<f32> {
        let s = s.trim();
        match s.parse::<f32>() {
            Ok(v) => Some(v * unidade),
            Err(_) => medida(s).or_else(|| calcular(s).map(|v| v * unidade)),
        }
    }

    /// `(1,2)`, `(1cm,2cm)`, `(nome)`, `(nome.east)`, `(a -| b)`; o y do TikZ cresce para cima, o
    /// do SVG para baixo.
    fn coordenada(&self, g: &str) -> Option<(f32, f32)> {
        let g = g.trim();
        if let Some((a, b)) = g.split_once("-|") {
            let (pa, pb) = (self.coordenada(a)?, self.coordenada(b)?);
            return Some((pb.0, pa.1));
        }
        if let Some((a, b)) = g.split_once("|-") {
            let (pa, pb) = (self.coordenada(a)?, self.coordenada(b)?);
            return Some((pa.0, pb.1));
        }
        if let Some((a, b)) = g.split_once(',') {
            return Some((Self::comprimento(a, self.unidade.0)?, -Self::comprimento(b, self.unidade.1)?));
        }
        let (nome, ancora) = g.split_once('.').unwrap_or((g, "center"));
        let n = &self.lista[*self.nos.get(nome.trim())?];
        let (dx, dy) = match ancora.trim() {
            "north" => (0.0, -1.0),
            "south" => (0.0, 1.0),
            "east" => (1.0, 0.0),
            "west" => (-1.0, 0.0),
            "north east" => (1.0, -1.0),
            "north west" => (-1.0, -1.0),
            "south east" => (1.0, 1.0),
            "south west" => (-1.0, 1.0),
            _ => (0.0, 0.0),
        };
        Some((n.x + dx * n.w / 2.0, n.y + dy * n.h / 2.0))
    }

    /// Onde a reta do centro do nó até `(px, py)` sai da borda dele.
    fn borda_do_no(n: &No, px: f32, py: f32) -> (f32, f32) {
        let (dx, dy) = (px - n.x, py - n.y);
        if dx == 0.0 && dy == 0.0 {
            return (n.x, n.y);
        }
        let t = if n.estilo.circulo {
            n.w / 2.0 / (dx * dx + dy * dy).sqrt()
        } else {
            let tx = if dx != 0.0 { n.w / 2.0 / dx.abs() } else { f32::INFINITY };
            let ty = if dy != 0.0 { n.h / 2.0 / dy.abs() } else { f32::INFINITY };
            tx.min(ty)
        };
        (n.x + dx * t.min(1.0), n.y + dy * t.min(1.0))
    }

    /// O `node[right] {h}` no meio do caminho: o rótulo encostado no ponto `(x, y)`.
    fn rotulo(&mut self, texto: &str, o: &[(String, String)], (x, y): (f32, f32)) {
        let mut n = self.novo_no(texto, o);
        let (mut cx, mut cy) = (x, y);
        for (k, v) in o {
            let lados = if k == "anchor" {
                // `anchor=west` = o nó fica à direita do ponto.
                v.split_whitespace().map(|l| match l { "west" => "right", "east" => "left", "north" => "below", "south" => "above", l => l }).collect::<Vec<_>>()
            } else if v.is_empty() {
                k.split_whitespace().collect()
            } else {
                continue;
            };
            for lado in lados {
                match lado {
                    "right" => cx = x + n.w / 2.0,
                    "left" => cx = x - n.w / 2.0,
                    "above" => cy = y - n.h / 2.0,
                    "below" => cy = y + n.h / 2.0,
                    _ => {}
                }
            }
        }
        (n.x, n.y) = (cx, cy);
        self.lista.push(n);
    }

    /// `\draw[opções] (a) -- (b) -| (1,2) rectangle (3,4);`, `circle (r)` e `node {…}`.
    fn caminho(&mut self, r: &str, preencher: bool) -> Option<()> {
        let mut r = r.trim_start();
        let mut o = Vec::new();
        if let Some((g, k)) = opcional(r) {
            o = opcoes(g);
            r = &r[k..];
        }
        let e = self.estilo(&Estilo::default(), &o);
        let traco = Self::cor_hex(e.cor.unwrap_or(0x000000FF));
        let espessura = if e.espessura > 0.0 { e.espessura } else { FINA };
        let tracejado = e.tracejado.map_or(String::new(), |t| format!(r#" stroke-dasharray="{t}""#));
        let fundo = if preencher { e.fundo.or(e.cor).map_or("black".into(), Self::cor_hex) } else { e.fundo.map_or("none".into(), Self::cor_hex) };
        // Os pontos do caminho: (índice do nó, x, y) e o que liga cada um ao anterior.
        let mut pontos: Vec<(Option<usize>, f32, f32)> = Vec::new();
        let mut ligacoes: Vec<&str> = Vec::new();
        let mut ligacao = "";
        loop {
            r = r.trim_start();
            if r.is_empty() {
                break;
            }
            if let Some((g, k)) = delimitado(r, '(', ')') {
                if ligacao == "circle" {
                    let raio = Self::comprimento(g, self.unidade.0)?;
                    let (cx, cy) = pontos.last().map(|p| (p.1, p.2))?;
                    let _ = writeln!(self.svg, r#"<circle cx="{cx}" cy="{cy}" r="{raio}" fill="{fundo}" stroke="{traco}" stroke-width="{espessura}"{tracejado}/>"#);
                    self.pontos.extend([(cx - raio, cy - raio), (cx + raio, cy + raio)]);
                    ligacao = "";
                } else {
                    let no = self.nos.get(g.trim()).copied();
                    let (x, y) = self.coordenada(g)?;
                    pontos.push((no, x, y));
                    ligacoes.push(ligacao);
                }
                r = &r[k..];
            } else if let Some(d) = r.strip_prefix("node") {
                let mut d = d.trim_start();
                let mut oo = Vec::new();
                if let Some((g, k)) = opcional(d) {
                    oo = opcoes(g);
                    d = d[k..].trim_start();
                }
                let (texto, k) = grupo(d)?;
                let ponto = pontos.last().map(|p| (p.1, p.2))?;
                self.rotulo(texto, &oo, ponto);
                r = &d[k..];
            } else if let Some(d) = r.strip_prefix("-|") {
                (ligacao, r) = ("-|", d);
            } else if let Some(d) = r.strip_prefix("|-") {
                (ligacao, r) = ("|-", d);
            } else if let Some(d) = r.strip_prefix("--").or_else(|| r.strip_prefix("to")) {
                (ligacao, r) = ("--", d);
            } else if let Some(d) = r.strip_prefix("rectangle") {
                (ligacao, r) = ("rectangle", d);
            } else if let Some(d) = r.strip_prefix("circle") {
                (ligacao, r) = ("circle", d);
            } else {
                return None;
            }
        }
        // Uma linha por trecho: de nó a nó, ela começa e acaba na borda deles.
        let mut d = String::new();
        for k in 1..pontos.len() {
            let ((na, xa, ya), (nb, xb, yb)) = (pontos[k - 1], pontos[k]);
            // `-|`: anda na horizontal e depois na vertical; `|-`, o contrário.
            let quina = match ligacoes[k] {
                "-|" => Some((xb, ya)),
                "|-" => Some((xa, yb)),
                _ => None,
            };
            let (alvo_a, alvo_b) = quina.map_or(((xb, yb), (xa, ya)), |q| (q, q));
            let (xa, ya) = na.map_or((xa, ya), |i| Self::borda_do_no(&self.lista[i], alvo_a.0, alvo_a.1));
            let (xb, yb) = nb.map_or((xb, yb), |i| Self::borda_do_no(&self.lista[i], alvo_b.0, alvo_b.1));
            if ligacoes[k] == "rectangle" {
                let _ = writeln!(self.svg, r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{fundo}" stroke="{traco}" stroke-width="{espessura}"{tracejado}/>"#, xa.min(xb), ya.min(yb), (xb - xa).abs(), (yb - ya).abs());
            } else {
                // Trechos seguidos formam uma linha só (a ponta fica só no fim dela).
                if d.is_empty() || !d.ends_with(&format!("L{xa} {ya}")) {
                    let _ = write!(d, "M{xa} {ya}");
                }
                if let Some((qx, qy)) = quina {
                    let _ = write!(d, "L{qx} {qy}");
                }
                let _ = write!(d, "L{xb} {yb}");
            }
            self.pontos.extend([(xa, ya), (xb, yb)]);
        }
        if !d.is_empty() {
            let (inicio, fim) = e.pontas;
            if (inicio || fim) && !self.setas.contains(&traco) {
                self.setas.push(traco.clone());
            }
            let id = &traco[1..];
            let fim = if fim { format!(r#" marker-end="url(#seta-{id})""#) } else { String::new() };
            let inicio = if inicio { format!(r#" marker-start="url(#volta-{id})""#) } else { String::new() };
            let preenche = if preencher { fundo.as_str() } else { "none" };
            let _ = writeln!(self.svg, r#"<path d="{d}" fill="{preenche}" stroke="{traco}" stroke-width="{espessura}" stroke-linejoin="round"{tracejado}{fim}{inicio}/>"#);
        }
        Some(())
    }
}

/// Os comandos do desenho separados pelo `;` (fora de chaves e colchetes).
fn comandos(corpo: &str) -> Vec<&str> {
    let mut saida = Vec::new();
    let (mut prof, mut ini) = (0, 0);
    for (i, c) in corpo.char_indices() {
        match c {
            '{' | '[' => prof += 1,
            '}' | ']' => prof -= 1,
            ';' if prof == 0 => {
                saida.push(corpo[ini..i].trim());
                ini = i + 1;
            }
            _ => {}
        }
    }
    saida
}

/// O desenho do `tikzpicture` como SVG, com a largura e a altura em pontos, e se algum comando
/// não foi entendido.
pub(super) fn svg(corpo: &str, cor: &dyn Fn(&str) -> Option<u32>) -> Option<(String, f32, f32, bool)> {
    let mut d = Desenho {
        cor,
        distancia: (CM, CM),
        unidade: (CM, CM),
        todo_no: Estilo::default(),
        estilos: HashMap::new(),
        nos: HashMap::new(),
        lista: Vec::new(),
        svg: String::new(),
        pontos: Vec::new(),
        setas: Vec::new(),
    };
    let mut corpo = corpo.trim_start();
    if let Some((g, k)) = opcional(corpo) {
        for (chave, v) in opcoes(g) {
            match chave.as_str() {
                // `node distance=1cm and 2cm`: a vertical e a horizontal.
                "node distance" => {
                    let (a, b) = v.split_once(" and ").unwrap_or((&v, &v));
                    d.distancia = (medida(a).unwrap_or(CM), medida(b).unwrap_or(CM));
                }
                "x" => d.unidade.0 = medida(&v).unwrap_or(CM),
                "y" => d.unidade.1 = medida(&v).unwrap_or(CM),
                "every node/.style" => d.todo_no = d.estilo(&Estilo::default(), &opcoes(&v)),
                c => {
                    if let Some(nome) = c.strip_suffix("/.style") {
                        d.estilos.insert(nome.trim().to_string(), opcoes(&v));
                    }
                }
            }
        }
        corpo = &corpo[k..];
    }
    let mut faltou = false;
    for c in comandos(corpo).into_iter().filter(|c| !c.is_empty()) {
        let ok = if let Some(r) = c.strip_prefix("\\node") {
            d.no(r)
        } else if let Some(r) = c.strip_prefix("\\filldraw").or_else(|| c.strip_prefix("\\fill")) {
            d.caminho(r, true)
        } else if let Some(r) = c.strip_prefix("\\draw") {
            d.caminho(r, false)
        } else {
            None
        };
        faltou |= ok.is_none();
    }
    if d.lista.is_empty() && d.pontos.is_empty() {
        return None;
    }
    // Os nós por cima das linhas: as linhas já estão no `svg`, os nós entram depois.
    let mut nos = String::new();
    for n in &d.lista {
        let (x, y) = (n.x - n.w / 2.0, n.y - n.h / 2.0);
        d.pontos.extend([(x, y), (x + n.w, y + n.h)]);
        let fundo = n.estilo.fundo.map_or("none".into(), Desenho::cor_hex);
        let traco = if n.estilo.borda { Desenho::cor_hex(n.estilo.cor.unwrap_or(0x000000FF)) } else { "none".into() };
        let espessura = if n.estilo.espessura > 0.0 { n.estilo.espessura } else { FINA };
        if n.estilo.circulo {
            let _ = writeln!(nos, r#"<circle cx="{}" cy="{}" r="{}" fill="{fundo}" stroke="{traco}" stroke-width="{espessura}"/>"#, n.x, n.y, n.w / 2.0);
        } else {
            let raio = if n.estilo.arredondado { 4.0 } else { 0.0 };
            let _ = writeln!(nos, r#"<rect x="{x}" y="{y}" width="{}" height="{}" rx="{raio}" fill="{fundo}" stroke="{traco}" stroke-width="{espessura}"/>"#, n.w, n.h);
        }
        // O texto do nó tem a cor do `text=`, senão a do `color=`/nome de cor solto, senão preto.
        let cor = Desenho::cor_hex(n.estilo.texto.or(n.estilo.cor.filter(|_| !n.estilo.borda)).unwrap_or(0x000000FF));
        let corpo = if n.estilo.fonte > 0.0 { n.estilo.fonte } else { CORPO };
        let altura_linha = corpo * 1.2;
        let primeira = n.y - altura_linha * (n.linhas.len() as f32 - 1.0) / 2.0 + corpo * 0.35;
        for (i, linha) in n.linhas.iter().enumerate() {
            let texto = linha.replace('&', "&amp;").replace('<', "&lt;");
            let _ = writeln!(nos, r#"<text x="{}" y="{}" font-family="Liberation Serif" font-size="{corpo}" fill="{cor}" text-anchor="middle">{texto}</text>"#, n.x, primeira + altura_linha * i as f32);
        }
    }
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for (x, y) in &d.pontos {
        (x0, y0, x1, y1) = (x0.min(*x), y0.min(*y), x1.max(*x), y1.max(*y));
    }
    let margem = 2.0;
    let (w, h) = (x1 - x0 + 2.0 * margem, y1 - y0 + 2.0 * margem);
    // As pontas de seta medem 5 pt e têm a cor da linha (um marcador por cor).
    let mut marcadores = String::new();
    for cor in &d.setas {
        let id = &cor[1..];
        let _ = write!(marcadores, r#"<marker id="seta-{id}" viewBox="0 0 10 10" refX="9" refY="5" markerUnits="userSpaceOnUse" markerWidth="5" markerHeight="5" orient="auto"><path d="M0 0L10 5L0 10L3 5z" fill="{cor}"/></marker><marker id="volta-{id}" viewBox="0 0 10 10" refX="1" refY="5" markerUnits="userSpaceOnUse" markerWidth="5" markerHeight="5" orient="auto"><path d="M10 0L0 5L10 10L7 5z" fill="{cor}"/></marker>"#);
    }
    // Três vezes o tamanho em pixels: o desenho fica nítido com zoom.
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="{} {} {w} {h}"><defs>{marcadores}</defs>
{}{nos}</svg>"#,
        w * 3.0, h * 3.0, x0 - margem, y0 - margem, d.svg
    );
    Some((svg, w, h, faltou))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn pic_com_estilo_de_parametro_e_variaveis_do_tikzmath() {
        let preambulo = "\\tikzmath{ real \\d, \\col; \\d = 1; \\col = (2 * \\d); % comentário\n}
            \\tikzset{quadro/.style={rectangle, fill=#1, minimum size=\\d cm}}
            \\tikzset{pics/logo/.style n args={2}{code = { \\node[quadro=#1] at (\\col,0) {}; \\node[quadro=#2] at (\\col+1.2*1,0) {}; }}}";
        let d = definicoes(preambulo);
        assert!((calcular("(2.2+1.2*2) - 1").unwrap() - 3.6).abs() < 1e-5);
        let corpo = com_definicoes("\\pic{logo={red}{blue}};", &d);
        let (svg, w, h, faltou) = svg(&corpo, &|c: &str| match c { "red" => Some(0xFF0000FF), "blue" => Some(0x0000FFFF), _ => None }).unwrap();
        assert!(!faltou, "{corpo}");
        assert!(svg.contains("#ff0000") && svg.contains("#0000ff"), "{svg}");
        // Dois quadrados de 1 cm com 1,2 cm entre os centros (e a margem de 2 pt).
        assert!((w - (2.2 * CM + 4.0)).abs() < 0.5 && (h - (CM + 4.0)).abs() < 0.5, "{w} {h}");
    }

    #[test]
    fn nos_em_fila_e_setas() {
        let corpo = "[node distance=1.6cm, every node/.style={draw, rounded corners, fill=cinza, minimum width=2.6cm}]
            \\node (a) {Consumo};
            \\node (b) [right=of a] {Irradiação};
            \\node (d) [below=of b] {Fim};
            \\draw[->] (a) -- (b);
            \\draw[->] (b) -- (d);";
        let cor = |n: &str| (n == "cinza").then_some(0xF2F4F7FF);
        let (svg, w, h, faltou) = svg(corpo, &cor).unwrap();
        assert!(!faltou);
        assert_eq!(svg.matches("<rect").count(), 3);
        assert_eq!(svg.matches("marker-end").count(), 2);
        assert!(svg.contains("Irradiação") && svg.contains("#f2f4f7"));
        // Dois nós de 2,6 cm e 1,6 cm entre eles, mais a margem.
        assert!((w - (2.6 * 2.0 + 1.6) * CM - 4.0).abs() < 1.0, "{w}");
        assert!(h > 50.0);
    }

    #[test]
    fn estilos_com_nome_ancoras_e_unidades() {
        let corpo = "[node distance=1cm, bloco/.style={draw=verde, fill=claro, font=\\small}, seta/.style={-{Stealth[length=2.5mm]}, thick, verde}]
            \\node[bloco] (a) {Arranjo FV\\\\48 kWp};
            \\node[bloco, right=2cm of a] (b) {2 $\\times$ 10 kW};
            \\draw[seta] (a.east) -- (a.east -| b.west);
            \\draw[seta, <->, dashed] (a) |- (b);";
        let cor = |n: &str| match n { "verde" => Some(0x1B5E20FF), "claro" => Some(0xE8F5E9FF), _ => None };
        let (svg, _, _, faltou) = svg(corpo, &cor).unwrap();
        assert!(!faltou, "{svg}");
        assert!(svg.contains(">Arranjo FV<") && svg.contains(">48 kWp<") && svg.contains("2 × 10 kW"));
        assert!(svg.contains(r##"stroke="#1b5e20" stroke-width="0.8""##) && svg.contains("stroke-dasharray"));
        assert_eq!(svg.matches("marker-start").count(), 1);
        assert!(svg.contains(r#"font-size="9""#));
        // `x=0.5cm, y=1pt`: 10 unidades de x são 5 cm.
        let (_, w, h, faltou) = super::svg("[x=0.5cm, y=1pt] \\draw[->] (0,0) -- (10,0) node[right, font=\\tiny] {h}; \\draw (0,0) -- (0,20);", &cor).unwrap();
        assert!(!faltou);
        // O "h" (5 pt, uns 9 pt de largura com a folga) encosta à direita da ponta.
        assert!((w - 5.0 * CM - 4.0 - 9.2).abs() < 2.0, "{w}");
        assert!(h > 24.0 && h < 35.0, "{h}");
    }
}
