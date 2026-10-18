//! O corpus de `.tex` reais (`tests/corpus_latex/`, origem e licença em `ORIGEM.tsv`): o leitor
//! passa por todos sem quebrar, e o ranking do que ele não entende (em quantos arquivos aparece)
//! decide a ordem do trabalho. Para ver o ranking:
//! `cargo test -p featherdocs-io --test corpus_latex -- --nocapture`

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use featherdocs_io::latex::{ler_na_pasta, TipoDeAviso};

#[test]
fn corpus_le_tudo_e_imprime_o_ranking() {
    let pasta = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus_latex");
    let mut arquivos: Vec<_> = std::fs::read_dir(&pasta).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|e| e == "tex")).collect();
    arquivos.sort();
    assert!(arquivos.len() >= 20, "o corpus tem {} arquivos", arquivos.len());
    // (tipo, nome) → arquivos que usam e vezes no total.
    let mut ranking: BTreeMap<(TipoDeAviso, String), (BTreeSet<String>, usize)> = BTreeMap::new();
    let mut total_avisos = 0;
    let mut uso: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for p in &arquivos {
        let fonte = std::fs::read_to_string(p).unwrap();
        // Arquivo só de pedaço (incluído por outro) não tem documento próprio.
        if !fonte.contains("\\begin{document}") {
            continue;
        }
        let lido = ler_na_pasta(&fonte, Some(&pasta));
        let nome = p.file_name().unwrap().to_string_lossy().into_owned();
        for pacote in pacotes(&fonte) {
            uso.entry(pacote).or_default().insert(nome.clone());
        }
        assert!(!lido.documento.blocos.is_empty(), "{nome}: nenhum bloco");
        println!("{nome}: {} blocos, {} avisos", lido.documento.blocos.len(), lido.avisos.len());
        total_avisos += lido.avisos.len();
        for a in lido.avisos {
            let e = ranking.entry((a.tipo, a.nome)).or_default();
            e.0.insert(nome.clone());
            e.1 += a.vezes;
        }
    }
    let mut linhas: Vec<_> = ranking.into_iter().collect();
    linhas.sort_by(|a, b| b.1 .0.len().cmp(&a.1 .0.len()).then(b.1 .1.cmp(&a.1 .1)).then(a.0.cmp(&b.0)));
    println!("\nRANKING (tipo · nome · arquivos · vezes) — {} itens, {} avisos somados", linhas.len(), total_avisos);
    for ((tipo, nome), (arqs, vezes)) in &linhas {
        println!("{:<9} {:<28} {:>3} {:>5}", tipo.rotulo(), nome, arqs.len(), vezes);
    }
    let mut uso: Vec<_> = uso.into_iter().map(|(p, a)| (a.len(), p)).collect();
    uso.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    println!("\nPACOTES USADOS (arquivos)");
    for (n, p) in uso {
        println!("{p:<28} {n:>3}");
    }
}

/// Os pacotes de `\usepackage[…]{a,b}` fora de comentário.
fn pacotes(fonte: &str) -> Vec<String> {
    let mut saida = Vec::new();
    for linha in fonte.lines() {
        let linha = linha.split('%').next().unwrap_or("");
        let mut resto = linha;
        while let Some(i) = resto.find("\\usepackage") {
            resto = &resto[i + 11..];
            let r = resto.trim_start();
            let r = if r.starts_with('[') { r.find(']').map_or(r, |f| &r[f + 1..]) } else { r };
            if let Some(g) = r.trim_start().strip_prefix('{').and_then(|g| g.split_once('}')).map(|g| g.0) {
                saida.extend(g.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()));
            }
        }
    }
    saida.sort();
    saida.dedup();
    saida
}
