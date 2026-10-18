//! Lê um `.tex` pelo leitor e mostra o que ficou de fora e os blocos que saíram.
//! Rodar: `cargo run --example ler_tex -- arquivo.tex` (ou um `.zip` de projeto).

fn main() {
    let caminho = std::env::args().nth(1).expect("uso: ler_tex arquivo.tex");
    let l = if caminho.ends_with(".zip") {
        featherdocs_io::latex::projeto::ler_zip(&std::fs::read(&caminho).expect("não li o arquivo")).expect("zip").0
    } else {
        let fonte = std::fs::read_to_string(&caminho).expect("não li o arquivo");
        featherdocs_io::latex::ler_na_pasta(&fonte, std::path::Path::new(&caminho).parent())
    };
    println!("== avisos ({})", l.avisos.len());
    for a in &l.avisos {
        println!("{}", a.descricao());
    }
    let d = &l.documento;
    println!("== página {:?}\n== extras {:?}\n== meta {:?}\n== numeração {:?}", d.pagina, d.extras, d.meta, d.estilos.numeracao);
    println!("== fontes {:?}", d.bibliografia.fontes.iter().map(|f| (&f.chave, f.autores.len(), &f.ano)).collect::<Vec<_>>());
    println!("== blocos ({})", d.blocos.len());
    for b in &d.blocos {
        let t = featherdocs_io::exportar::bloco_markdown(&b.conteudo);
        let curto: String = t.chars().take(90).collect::<String>().replace(|c: char| c == '\u{2028}', "↵");
        println!("{:?} | {:?} | {}", std::mem::discriminant(&b.conteudo), (b.atrib.alinhamento, b.atrib.recuo_esq, b.atrib.primeira, b.atrib.antes, b.atrib.depois), curto.replace('\n', "⏎"));
    }
}
