//! Citações e referências: o que `(SILVA, 2023, p. 4)` mostra no texto e como cada fonte aparece
//! na lista de referências, em cinco estilos — ABNT (NBR 10520 autor-data e NBR 6023), APA 7,
//! Vancouver, IEEE e Chicago (autor-data). Tudo calculado aqui; a interface só cadastra as fontes.

use std::collections::HashMap;

use crate::modelo::{Autor, EstiloDeCitacao, Fonte, TipoDeFonte, Trecho, ITALICO, NEGRITO};

/// O estilo pelo nome (`abnt`, `apa`, `vancouver`, `ieee`, `chicago`).
pub fn estilo_do_nome(nome: &str) -> Option<EstiloDeCitacao> {
    Some(match nome {
        "abnt" => EstiloDeCitacao::Abnt,
        "apa" => EstiloDeCitacao::Apa,
        "vancouver" => EstiloDeCitacao::Vancouver,
        "ieee" => EstiloDeCitacao::Ieee,
        "chicago" => EstiloDeCitacao::Chicago,
        _ => return None,
    })
}

/// O estilo numera as fontes pela ordem da primeira citação (Vancouver, IEEE).
pub fn numerico(e: EstiloDeCitacao) -> bool {
    matches!(e, EstiloDeCitacao::Vancouver | EstiloDeCitacao::Ieee)
}

/// Letras sem acento e em minúsculas, para pôr em ordem alfabética.
pub fn dobrar(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            'ñ' | 'Ñ' => 'n',
            c => c.to_ascii_lowercase(),
        })
        .collect()
}

/// As iniciais do prenome: "João Pedro" → "J. P." (ou "JP", sem pontos nem espaço).
fn iniciais(nome: &str, pontos: bool) -> String {
    let letras: Vec<String> = nome.split([' ', '-']).filter(|p| !p.is_empty()).filter_map(|p| p.chars().next()).map(|c| c.to_uppercase().to_string()).collect();
    if pontos {
        letras.iter().map(|l| format!("{l}.")).collect::<Vec<_>>().join(" ")
    } else {
        letras.concat()
    }
}

/// O sobrenome como o estilo cita no texto. Na ABNT (NBR 10520:2023, e o `biblatex-abnt`) também
/// em maiúsculas e minúsculas: as maiúsculas ficam só na lista de referências.
fn sobrenome_na_citacao(a: &Autor, _e: EstiloDeCitacao) -> String {
    a.sobrenome.clone()
}

/// A primeira palavra do título, quando a fonte não tem autor (a entrada passa a ser pelo título).
fn primeira_palavra(f: &Fonte, e: EstiloDeCitacao) -> String {
    let p = f.titulo.split_whitespace().next().unwrap_or("?");
    if e == EstiloDeCitacao::Abnt {
        format!("{}…", p.to_uppercase())
    } else {
        format!("{p}…")
    }
}

/// Os autores de uma citação autor-data: "SILVA; SOUZA", "Silva & Souza", "Silva et al.".
fn autores_na_citacao(f: &Fonte, e: EstiloDeCitacao) -> String {
    let n: Vec<String> = f.autores.iter().map(|a| sobrenome_na_citacao(a, e)).collect();
    match (n.len(), e) {
        (0, _) => primeira_palavra(f, e),
        (1, _) => n[0].clone(),
        (2, EstiloDeCitacao::Apa) => format!("{} & {}", n[0], n[1]),
        (2, EstiloDeCitacao::Chicago) => format!("{} and {}", n[0], n[1]),
        (2 | 3, EstiloDeCitacao::Abnt) => n.join("; "),
        (3, EstiloDeCitacao::Chicago) => format!("{}, {}, and {}", n[0], n[1], n[2]),
        _ => format!("{} et al.", n[0]),
    }
}

/// A citação de várias fontes, no texto: `(SILVA, 2023; SOUZA, 2020, p. 4)`, `[1], [3]`, `(1,3)`.
/// `numeros` é o número de cada chave nos estilos numéricos; a chave sem fonte aparece entre `?`.
pub fn citacao(e: EstiloDeCitacao, chaves: &[String], pagina: &str, fontes: &[Fonte], numeros: &HashMap<String, usize>) -> String {
    let achar = |c: &String| fontes.iter().find(|f| &f.chave == c);
    if numerico(e) {
        let ns: Vec<String> = chaves.iter().map(|c| numeros.get(c).map_or(format!("?{c}?"), usize::to_string)).collect();
        return match (e, pagina.is_empty()) {
            (EstiloDeCitacao::Ieee, true) => ns.iter().map(|n| format!("[{n}]")).collect::<Vec<_>>().join(", "),
            (EstiloDeCitacao::Ieee, false) => format!("[{}, p. {pagina}]", ns.join(", ")),
            (_, true) => format!("({})", ns.join(",")),
            (_, false) => format!("({}, p. {pagina})", ns.join(",")),
        };
    }
    let partes: Vec<String> = chaves
        .iter()
        .map(|c| match achar(c) {
            Some(f) if e == EstiloDeCitacao::Chicago => format!("{} {}", autores_na_citacao(f, e), ano(f)),
            Some(f) => format!("{}, {}", autores_na_citacao(f, e), ano(f)),
            None => format!("?{c}?"),
        })
        .collect();
    let mut s = partes.join("; ");
    if !pagina.is_empty() {
        s.push_str(&if e == EstiloDeCitacao::Chicago { format!(", {pagina}") } else { format!(", p. {pagina}") });
    }
    format!("({s})")
}

fn ano(f: &Fonte) -> &str {
    if f.ano.is_empty() {
        "s.d."
    } else {
        &f.ano
    }
}

/// Monta uma referência em trechos (texto, negrito, itálico).
#[derive(Default)]
struct Ref {
    t: Vec<Trecho>,
}

impl Ref {
    fn com(&mut self, s: &str, estilo: u8) -> &mut Self {
        if s.is_empty() {
            return self;
        }
        match self.t.last_mut() {
            Some(u) if u.estilo == estilo => u.texto.push_str(s),
            _ => self.t.push(Trecho { texto: s.to_string(), estilo, ..Trecho::default() }),
        }
        self
    }
    fn txt(&mut self, s: &str) -> &mut Self {
        self.com(s, 0)
    }
    fn neg(&mut self, s: &str) -> &mut Self {
        self.com(s, NEGRITO)
    }
    fn ita(&mut self, s: &str) -> &mut Self {
        self.com(s, ITALICO)
    }
    /// `antes` + valor + `depois`, só se o valor existe.
    fn se(&mut self, antes: &str, v: &str, depois: &str) -> &mut Self {
        if !v.is_empty() {
            self.txt(antes).txt(v).txt(depois);
        }
        self
    }
}

/// O título com o subtítulo ("Título: subtítulo").
fn titulo_completo(f: &Fonte) -> String {
    if f.subtitulo.is_empty() {
        f.titulo.clone()
    } else {
        format!("{}: {}", f.titulo, f.subtitulo)
    }
}

/// Os autores na referência ABNT: "SILVA, João; SOUZA, Maria" (quatro ou mais: o primeiro et al.).
fn autores_abnt(f: &Fonte) -> String {
    let um = |a: &Autor| if a.nome.is_empty() { a.sobrenome.to_uppercase() } else { format!("{}, {}", a.sobrenome.to_uppercase(), a.nome) };
    match f.autores.len() {
        0 => String::new(),
        1..=3 => f.autores.iter().map(um).collect::<Vec<_>>().join("; "),
        _ => format!("{} et al", um(&f.autores[0])),
    }
}

/// ABNT NBR 6023. `sem_sl_sn`: o local e a editora que faltam não viram "[S. l.]: [s. n.]" (e o
/// relatório sem local não mostra a instituição), como o `noslsn` do biblatex-abnt.
fn referencia_abnt(f: &Fonte, sem_sl_sn: bool) -> Vec<Trecho> {
    let mut r = Ref::default();
    let autores = autores_abnt(f);
    if !autores.is_empty() {
        // "Reddy, Thomas B." já termina em ponto: não dobra.
        r.txt(&autores).txt(if autores.ends_with('.') { " " } else { ". " });
    }
    let local = if f.local.is_empty() { "[S. l.]" } else { &f.local };
    let editora = if f.editora.is_empty() { "[s. n.]" } else { &f.editora };
    match f.tipo {
        TipoDeFonte::Artigo => {
            r.txt(&titulo_completo(f)).txt(". ").neg(&f.veiculo);
            r.se(", ", &f.local, "").se(", v. ", &f.volume, "").se(", n. ", &f.numero, "").se(", p. ", &f.paginas, "").txt(", ").txt(ano(f)).txt(".");
        }
        TipoDeFonte::Capitulo => {
            r.txt(&titulo_completo(f)).txt(". In: ").neg(&f.veiculo).txt(". ");
            r.txt(local).txt(": ").txt(editora).txt(", ").txt(ano(f)).txt(".").se(" p. ", &f.paginas, ".");
        }
        TipoDeFonte::Tese => {
            r.neg(&f.titulo).se(": ", &f.subtitulo, "").txt(". ").txt(ano(f)).txt(".");
            r.se(" ", &f.veiculo, "").se(" – ", &f.editora, "").se(", ", &f.local, "").txt(", ").txt(ano(f)).txt(".");
        }
        _ => {
            r.neg(&f.titulo).se(": ", &f.subtitulo, "").txt(".").se(" ", &f.edicao, ". ed.");
            if sem_sl_sn && f.local.is_empty() {
                // "8. ed.: Pearson, 2010." e "Campus, 2002.", como o biblatex-abnt escreve.
                if !f.editora.is_empty() && f.tipo != TipoDeFonte::Relatorio {
                    r.txt(if f.edicao.is_empty() { " " } else { ": " }).txt(&f.editora).txt(",");
                }
                r.txt(" ").txt(ano(f)).txt(".");
            } else {
                r.txt(" ").txt(local).txt(": ").txt(editora).txt(", ").txt(ano(f)).txt(".");
            }
        }
    }
    r.se(" ", &f.nota, ".").se(" DOI: ", &f.doi, ".").se(" Disponível em: ", &f.url, ".").se(" Acesso em: ", &f.acesso, ".");
    r.t
}

/// Os autores na referência APA: "Silva, J. P., & Souza, M.".
fn autores_apa(f: &Fonte) -> String {
    let um = |a: &Autor| if a.nome.is_empty() { a.sobrenome.clone() } else { format!("{}, {}", a.sobrenome, iniciais(&a.nome, true)) };
    let v: Vec<String> = f.autores.iter().map(um).collect();
    match v.len() {
        0 => String::new(),
        1 => v[0].clone(),
        n => format!("{}, & {}", v[..n - 1].join(", "), v[n - 1]),
    }
}

/// APA 7.
fn referencia_apa(f: &Fonte) -> Vec<Trecho> {
    let mut r = Ref::default();
    let autores = autores_apa(f);
    if autores.is_empty() {
        r.ita(&titulo_completo(f)).txt(". (").txt(ano(f)).txt(").");
    } else {
        r.txt(&autores).txt(" (").txt(ano(f)).txt("). ");
        match f.tipo {
            TipoDeFonte::Artigo | TipoDeFonte::Capitulo => r.txt(&titulo_completo(f)).txt("."),
            _ => r.ita(&titulo_completo(f)),
        };
    }
    match f.tipo {
        TipoDeFonte::Artigo => {
            r.txt(" ").ita(&f.veiculo);
            if !f.volume.is_empty() {
                r.txt(", ").ita(&f.volume);
            }
            r.se("(", &f.numero, ")").se(", ", &f.paginas, "").txt(".");
        }
        TipoDeFonte::Capitulo => {
            r.txt(" In ").ita(&f.veiculo).se(" (pp. ", &f.paginas, ")").txt(".").se(" ", &f.editora, ".");
        }
        TipoDeFonte::Tese => {
            r.txt(" [").txt(if f.veiculo.is_empty() { "Tese" } else { &f.veiculo }).txt("].").se(" ", &f.editora, ".");
        }
        _ if !autores.is_empty() => {
            r.se(" (", &f.edicao, ". ed.)").txt(".").se(" ", &f.editora, ".");
        }
        _ => {}
    }
    if !f.doi.is_empty() {
        r.txt(" https://doi.org/").txt(&f.doi);
    } else {
        r.se(" ", &f.url, "");
    }
    r.t
}

/// Os autores na referência Vancouver: "Silva JP, Souza M" (mais de seis: os seis e "et al.").
fn autores_vancouver(f: &Fonte) -> String {
    let v: Vec<String> = f.autores.iter().take(6).map(|a| format!("{} {}", a.sobrenome, iniciais(&a.nome, false)).trim().to_string()).collect();
    let mut s = v.join(", ");
    if f.autores.len() > 6 {
        s.push_str(", et al");
    }
    s
}

/// Vancouver (ICMJE).
fn referencia_vancouver(f: &Fonte) -> Vec<Trecho> {
    let mut r = Ref::default();
    r.se("", &autores_vancouver(f), ". ").txt(&titulo_completo(f));
    match f.tipo {
        TipoDeFonte::Artigo => {
            r.txt(". ").txt(&f.veiculo).txt(". ").txt(ano(f)).se(";", &f.volume, "").se("(", &f.numero, ")").se(":", &f.paginas, "").txt(".");
        }
        TipoDeFonte::Site => {
            r.txt(" [Internet]. ").se("", &f.local, ": ").se("", &f.editora, "; ").txt(ano(f)).se(" [citado ", &f.acesso, "]").txt(".").se(" Disponível em: ", &f.url, "");
        }
        TipoDeFonte::Capitulo => {
            r.txt(". In: ").txt(&f.veiculo).txt(". ").se("", &f.local, ": ").se("", &f.editora, "; ").txt(ano(f)).se(". p. ", &f.paginas, "").txt(".");
        }
        _ => {
            r.txt(".").se(" ", &f.edicao, "ª ed.").txt(" ").se("", &f.local, ": ").se("", &f.editora, "; ").txt(ano(f)).txt(".");
        }
    }
    if f.tipo != TipoDeFonte::Site {
        r.se(" doi:", &f.doi, "");
    }
    r.t
}

/// Os autores na referência IEEE: "J. P. Silva, M. Souza, and A. Lima" (sete ou mais: o primeiro et al.).
fn autores_ieee(f: &Fonte) -> String {
    let um = |a: &Autor| format!("{} {}", iniciais(&a.nome, true), a.sobrenome).trim().to_string();
    let v: Vec<String> = f.autores.iter().map(um).collect();
    match v.len() {
        0 => String::new(),
        1 => v[0].clone(),
        2 => format!("{} and {}", v[0], v[1]),
        n if n >= 7 => format!("{} et al.", v[0]),
        n => format!("{}, and {}", v[..n - 1].join(", "), v[n - 1]),
    }
}

/// IEEE.
fn referencia_ieee(f: &Fonte) -> Vec<Trecho> {
    let mut r = Ref::default();
    r.se("", &autores_ieee(f), ", ");
    match f.tipo {
        TipoDeFonte::Artigo => {
            r.txt("“").txt(&titulo_completo(f)).txt(",” ").ita(&f.veiculo);
            r.se(", vol. ", &f.volume, "").se(", no. ", &f.numero, "").se(", pp. ", &f.paginas, "").txt(", ").txt(ano(f)).txt(".");
        }
        TipoDeFonte::Site => {
            r.txt("“").txt(&titulo_completo(f)).txt(".”").se(" ", &f.editora, ".").se(" Accessed: ", &f.acesso, ".").se(" [Online]. Available: ", &f.url, "");
        }
        TipoDeFonte::Capitulo => {
            r.txt("“").txt(&titulo_completo(f)).txt(",” in ").ita(&f.veiculo).txt(". ").se("", &f.local, ": ").se("", &f.editora, ", ").txt(ano(f)).se(", pp. ", &f.paginas, "").txt(".");
        }
        _ => {
            r.ita(&titulo_completo(f)).se(", ", &f.edicao, " ed").txt(". ").se("", &f.local, ": ").se("", &f.editora, ", ").txt(ano(f)).txt(".");
        }
    }
    if f.tipo != TipoDeFonte::Site {
        r.se(" doi: ", &f.doi, ".");
    }
    r.t
}

/// Os autores na referência Chicago: "Silva, João, and Maria Souza".
fn autores_chicago(f: &Fonte) -> String {
    let primeiro = |a: &Autor| if a.nome.is_empty() { a.sobrenome.clone() } else { format!("{}, {}", a.sobrenome, a.nome) };
    let outro = |a: &Autor| format!("{} {}", a.nome, a.sobrenome).trim().to_string();
    let v: Vec<String> = f.autores.iter().enumerate().map(|(k, a)| if k == 0 { primeiro(a) } else { outro(a) }).collect();
    match v.len() {
        0 => String::new(),
        1 => v[0].clone(),
        // O primeiro vem invertido ("Silva, João"), então a vírgula antes do "and" fica mesmo com dois.
        n => format!("{}, and {}", v[..n - 1].join(", "), v[n - 1]),
    }
}

/// Chicago, autor-data.
fn referencia_chicago(f: &Fonte) -> Vec<Trecho> {
    let mut r = Ref::default();
    let autores = autores_chicago(f);
    r.se("", &autores, if autores.ends_with('.') { " " } else { ". " }).txt(ano(f)).txt(". ");
    match f.tipo {
        TipoDeFonte::Artigo => {
            r.txt("“").txt(&titulo_completo(f)).txt(".” ").ita(&f.veiculo).se(" ", &f.volume, "").se(" (", &f.numero, ")").se(": ", &f.paginas, "").txt(".");
        }
        TipoDeFonte::Capitulo => {
            r.txt("“").txt(&titulo_completo(f)).txt(".” In ").ita(&f.veiculo).se(", ", &f.paginas, "").txt(". ").se("", &f.local, ": ").se("", &f.editora, ".");
        }
        _ => {
            r.ita(&titulo_completo(f)).txt(".").se(" ", &f.local, ":").se(" ", &f.editora, ".");
        }
    }
    r.se(" https://doi.org/", &f.doi, ".").se(" ", if f.doi.is_empty() { &f.url } else { "" }, ".");
    r.t
}

/// A referência de uma fonte (sem o número; quem numera é `lista`).
pub fn referencia(e: EstiloDeCitacao, f: &Fonte) -> Vec<Trecho> {
    match e {
        EstiloDeCitacao::Abnt => referencia_abnt(f, false),
        EstiloDeCitacao::Apa => referencia_apa(f),
        EstiloDeCitacao::Vancouver => referencia_vancouver(f),
        EstiloDeCitacao::Ieee => referencia_ieee(f),
        EstiloDeCitacao::Chicago => referencia_chicago(f),
    }
}

/// A chave de ordem alfabética: o primeiro sobrenome (ou o título), depois o ano.
fn ordem_alfabetica(f: &Fonte) -> (String, String, String) {
    let quem = f.autores.first().map_or_else(|| f.titulo.clone(), |a| format!("{} {}", a.sobrenome, a.nome));
    (dobrar(&quem), f.ano.clone(), dobrar(&f.titulo))
}

/// A lista de referências: nos estilos numéricos, na ordem da primeira citação e com o número
/// na frente; nos outros, em ordem alfabética. `citadas` são as chaves na ordem em que aparecem;
/// com `todas`, entram também as fontes não citadas (no fim, nos numéricos).
pub fn lista(e: EstiloDeCitacao, fontes: &[Fonte], citadas: &[String], todas: bool, sem_sl_sn: bool) -> Vec<Vec<Trecho>> {
    let referencia = |e, f| if e == EstiloDeCitacao::Abnt { referencia_abnt(f, sem_sl_sn) } else { referencia(e, f) };
    let mut escolhidas: Vec<&Fonte> = citadas.iter().filter_map(|c| fontes.iter().find(|f| &f.chave == c)).collect();
    if todas {
        escolhidas.extend(fontes.iter().filter(|f| !citadas.contains(&f.chave)));
    }
    if !numerico(e) {
        escolhidas.sort_by_key(|f| ordem_alfabetica(f));
        return escolhidas.into_iter().map(|f| referencia(e, f)).collect();
    }
    escolhidas
        .into_iter()
        .enumerate()
        .map(|(k, f)| {
            let numero = if e == EstiloDeCitacao::Ieee { format!("[{}] ", k + 1) } else { format!("{}. ", k + 1) };
            let mut t = vec![Trecho::simples(&numero)];
            t.extend(referencia(e, f));
            t
        })
        .collect()
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::inline::texto_puro;

    fn autor(s: &str, n: &str) -> Autor {
        Autor { sobrenome: s.into(), nome: n.into() }
    }

    fn livro() -> Fonte {
        Fonte { chave: "silva2023".into(), autores: vec![autor("Silva", "João Pedro"), autor("Souza", "Maria")], titulo: "Redes elétricas".into(), subtitulo: "teoria e prática".into(), ano: "2023".into(), edicao: "2".into(), local: "São Paulo".into(), editora: "Blucher".into(), ..Fonte::default() }
    }

    fn artigo() -> Fonte {
        Fonte { chave: "lima2020".into(), tipo: TipoDeFonte::Artigo, autores: vec![autor("Lima", "Ana")], titulo: "Proteção de transformadores".into(), ano: "2020".into(), veiculo: "Revista Eletricidade".into(), volume: "12".into(), numero: "3".into(), paginas: "10-20".into(), local: "Recife".into(), ..Fonte::default() }
    }

    #[test]
    fn sem_sl_sn_e_nota_como_o_biblatex_abnt() {
        let livro = Fonte { chave: "a".into(), autores: vec![autor("Horstmann", "Cay S.")], titulo: "Core Java".into(), ano: "2010".into(), edicao: "8".into(), editora: "Pearson".into(), ..Fonte::default() };
        let site = Fonte { chave: "b".into(), autores: vec![autor("Larman", "Craig")], titulo: "UML".into(), ano: "2007".into(), editora: "Bookman".into(), nota: "https://x.br/1/".into(), ..Fonte::default() };
        let relatorio = Fonte { chave: "c".into(), tipo: TipoDeFonte::Relatorio, autores: vec![autor("Caelum", "")], titulo: "FJ-11".into(), ano: "2008".into(), editora: "Caelum Ensino".into(), ..Fonte::default() };
        let l = lista(EstiloDeCitacao::Abnt, &[livro, site, relatorio], &[], true, true);
        let t: Vec<String> = l.iter().map(|r| texto_puro(r)).collect();
        assert_eq!(t, ["CAELUM. FJ-11. 2008.", "HORSTMANN, Cay S. Core Java. 8. ed.: Pearson, 2010.", "LARMAN, Craig. UML. Bookman, 2007. https://x.br/1/."]);
    }

    #[test]
    fn citacoes_autor_data_nos_tres_estilos() {
        let fontes = [livro(), artigo()];
        let n = HashMap::new();
        let ch = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(citacao(EstiloDeCitacao::Abnt, &ch(&["silva2023"]), "4", &fontes, &n), "(Silva; Souza, 2023, p. 4)");
        assert_eq!(citacao(EstiloDeCitacao::Abnt, &ch(&["lima2020", "silva2023"]), "", &fontes, &n), "(Lima, 2020; Silva; Souza, 2023)");
        assert_eq!(citacao(EstiloDeCitacao::Apa, &ch(&["silva2023"]), "", &fontes, &n), "(Silva & Souza, 2023)");
        assert_eq!(citacao(EstiloDeCitacao::Chicago, &ch(&["lima2020"]), "12", &fontes, &n), "(Lima 2020, 12)");
        assert_eq!(citacao(EstiloDeCitacao::Abnt, &ch(&["xis"]), "", &fontes, &n), "(?xis?)");
        let mut quatro = livro();
        quatro.autores.extend([autor("A", ""), autor("B", "")]);
        assert_eq!(citacao(EstiloDeCitacao::Abnt, &ch(&["silva2023"]), "", &[quatro], &n), "(Silva et al., 2023)");
    }

    #[test]
    fn citacoes_numericas() {
        let n = HashMap::from([("silva2023".to_string(), 1), ("lima2020".to_string(), 2)]);
        let ch = vec!["silva2023".to_string(), "lima2020".to_string()];
        assert_eq!(citacao(EstiloDeCitacao::Ieee, &ch, "", &[], &n), "[1], [2]");
        assert_eq!(citacao(EstiloDeCitacao::Vancouver, &ch, "", &[], &n), "(1,2)");
        assert_eq!(citacao(EstiloDeCitacao::Ieee, &ch[..1], "7", &[], &n), "[1, p. 7]");
    }

    #[test]
    fn referencias_abnt_com_negrito_no_titulo() {
        let r = referencia(EstiloDeCitacao::Abnt, &livro());
        assert_eq!(texto_puro(&r), "SILVA, João Pedro; SOUZA, Maria. Redes elétricas: teoria e prática. 2. ed. São Paulo: Blucher, 2023.");
        assert!(r.iter().any(|t| t.estilo == NEGRITO && t.texto == "Redes elétricas"));
        let a = referencia(EstiloDeCitacao::Abnt, &artigo());
        assert_eq!(texto_puro(&a), "LIMA, Ana. Proteção de transformadores. Revista Eletricidade, Recife, v. 12, n. 3, p. 10-20, 2020.");
        let sem = Fonte { chave: "x".into(), titulo: "Norma".into(), ..Fonte::default() };
        assert_eq!(texto_puro(&referencia(EstiloDeCitacao::Abnt, &sem)), "Norma. [S. l.]: [s. n.], s.d..");
    }

    #[test]
    fn referencias_apa_vancouver_ieee_chicago() {
        let l = livro();
        let a = artigo();
        assert_eq!(texto_puro(&referencia(EstiloDeCitacao::Apa, &l)), "Silva, J. P., & Souza, M. (2023). Redes elétricas: teoria e prática (2. ed.). Blucher.");
        assert_eq!(texto_puro(&referencia(EstiloDeCitacao::Apa, &a)), "Lima, A. (2020). Proteção de transformadores. Revista Eletricidade, 12(3), 10-20.");
        assert_eq!(texto_puro(&referencia(EstiloDeCitacao::Vancouver, &a)), "Lima A. Proteção de transformadores. Revista Eletricidade. 2020;12(3):10-20.");
        assert_eq!(texto_puro(&referencia(EstiloDeCitacao::Ieee, &l)), "J. P. Silva and M. Souza, Redes elétricas: teoria e prática, 2 ed. São Paulo: Blucher, 2023.");
        assert_eq!(texto_puro(&referencia(EstiloDeCitacao::Chicago, &l)), "Silva, João Pedro, and Maria Souza. 2023. Redes elétricas: teoria e prática. São Paulo: Blucher.");
    }

    #[test]
    fn lista_em_ordem_alfabetica_ou_de_citacao() {
        let fontes = [livro(), artigo()];
        let citadas = vec!["silva2023".to_string(), "lima2020".to_string()];
        let abnt = lista(EstiloDeCitacao::Abnt, &fontes, &citadas, false, false);
        assert!(texto_puro(&abnt[0]).starts_with("LIMA"));
        let ieee = lista(EstiloDeCitacao::Ieee, &fontes, &citadas, false, false);
        assert!(texto_puro(&ieee[0]).starts_with("[1] J. P. Silva"));
        assert_eq!(lista(EstiloDeCitacao::Abnt, &fontes, &citadas[..1], false, false).len(), 1);
        assert_eq!(lista(EstiloDeCitacao::Abnt, &fontes, &citadas[..1], true, false).len(), 2);
    }
}
