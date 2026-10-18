//! Medidas e posições dos casos-chave, conferidas contra as constantes da tabela MATH da
//! Latin Modern Math (em em; corpo 10 pt aqui, então 1 em = 10 pt).

use feather_math::{compor, Formula, Modo, Numero};

const T: f32 = 10.0;
const EIXO: f32 = 0.25 * T;

/// O glifo do caractere `c` (pelo id que a fonte dá) — o `n`-ésimo, da esquerda para a direita.
fn achar(f: &Formula, latex_do_caractere: &str, n: usize) -> feather_math::Glifo {
    let alvo = compor(latex_do_caractere, Modo::Linha, T).glifos[0].id;
    let mut gs: Vec<_> = f.glifos.iter().filter(|g| g.id == alvo).copied().collect();
    gs.sort_by(|a, b| a.x.total_cmp(&b.x));
    gs[n]
}

fn perto(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}

#[test]
fn fracao_display_com_barra_no_eixo() {
    let f = compor(r"\frac{a}{b}", Modo::Exibida, T);
    assert_eq!(f.retangulos.len(), 1, "uma barra");
    let barra = f.retangulos[0];
    assert!(perto(barra.y + barra.altura / 2.0, -EIXO, 0.01), "barra no eixo: {barra:?}");
    assert!(perto(barra.altura, 0.04 * T, 0.001), "FractionRuleThickness");
    let a = achar(&f, "a", 0);
    let b = achar(&f, "b", 0);
    assert!(-a.y >= 0.677 * T - 0.001, "numerador sobe ao menos FractionNumeratorDisplayStyleShiftUp: {}", a.y);
    assert!(b.y >= 0.686 * T - 0.001, "denominador desce ao menos FractionDenominatorDisplayStyleShiftDown: {}", b.y);
    // Nulldelimiterspace de cada lado e as letras centradas sobre a barra.
    assert!(perto(barra.x, 0.12 * T, 0.001));
    assert!(perto(f.largura, barra.largura + 0.24 * T, 0.001));
    // Em linha (estilo de texto), a fração fica menor e mais baixa.
    let t = compor(r"\frac{a}{b}", Modo::Linha, T);
    assert!(t.altura < f.altura && t.largura < f.largura);
}

#[test]
fn expoente_e_indice_juntos() {
    let f = compor("x_i^2", Modo::Linha, T);
    let dois = achar(&f, "2", 0);
    let i = achar(&f, "i", 0);
    assert!(perto(dois.tamanho, 0.7 * T, 0.001), "ScriptPercentScaleDown = 70");
    let u = -dois.y;
    let v = i.y;
    assert!(u >= 0.363 * T - 0.001, "SuperscriptShiftUp: {u}");
    assert!(v >= 0.247 * T - 0.001, "SubscriptShiftDown: {v}");
    // O índice começa onde o x termina; o expoente, depois da correção itálica.
    assert!(dois.x >= i.x, "expoente ao menos tão à direita quanto o índice");
    assert!(f.largura > i.x);
}

#[test]
fn raiz_com_traco_por_cima() {
    let f = compor(r"\sqrt{x}", Modo::Linha, T);
    assert_eq!(f.retangulos.len(), 1);
    let r = f.retangulos[0];
    let x = achar(&f, "x", 0);
    assert!(r.x <= x.x + 0.01 && r.x + r.largura >= x.x, "traço sobre o radicando");
    assert!(-r.y > 0.43 * T, "o traço passa acima do x: {r:?}");
    assert!(perto(r.altura, 0.04 * T, 0.001), "RadicalRuleThickness");
    let n = compor(r"\sqrt[3]{x}", Modo::Linha, T);
    assert!(n.glifos.len() == f.glifos.len() + 1, "o índice da raiz");
}

#[test]
fn somatorio_com_limites_no_display() {
    let f = compor(r"\sum_{i=1}^{n} i", Modo::Exibida, T);
    let soma = f.glifos[0];
    let n = achar(&f, "n", 0);
    let um = achar(&f, "1", 0);
    assert!(n.y < -0.5 * T && um.y > 0.6 * T, "n em cima, i=1 embaixo: {n:?} {um:?}");
    assert!(n.x > soma.x && n.x < soma.x + 1.444 * T, "limite de cima sobre o ∑");
    assert!(soma.tamanho == T);
    // Em linha os limites vão para o lado.
    let t = compor(r"\sum_{i=1}^{n} i", Modo::Linha, T);
    assert!(t.altura < f.altura);
    let n2 = achar(&t, "n", 0);
    assert!(n2.x > t.glifos[0].x + 0.9 * T, "expoente ao lado");
}

#[test]
fn matriz_2x2_alinhada_em_colunas_e_linhas() {
    let f = compor(r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}", Modo::Linha, T);
    let (a, b, c, d) = (achar(&f, "a", 0), achar(&f, "b", 0), achar(&f, "c", 0), achar(&f, "d", 0));
    assert!(perto(a.y, b.y, 0.001) && perto(c.y, d.y, 0.001), "mesma linha, mesma base");
    assert!(perto(c.y - a.y, 1.2 * T, 0.01), "uma entrelinha de 12 pt entre as linhas: {}", c.y - a.y);
    assert!(b.x - a.x > 1.0 * T, "1 em entre as colunas");
    // Centrada no eixo: a altura acima do eixo = a profundidade abaixo dele.
    assert!(perto(f.altura - EIXO, f.profundidade + EIXO, 0.6), "{} {}", f.altura, f.profundidade);
    assert!(f.glifos.len() >= 6, "4 letras + 2 parênteses");
}

#[test]
fn align_de_tres_linhas_numeradas_e_alinhadas_no_igual() {
    let f = compor(r"\begin{align} a &= b + c \\ dd &= e \\ h &= i + j + k \end{align}", Modo::Exibida, T);
    assert_eq!(f.linhas.len(), 3);
    assert!(f.linhas.iter().all(|l| l.numero == Numero::Automatico));
    assert!(perto(f.linhas[0].base, 0.0, 0.001), "a fórmula se apoia na primeira linha");
    for k in 1..3 {
        assert!(f.linhas[k].base - f.linhas[k - 1].base >= 1.5 * T - 0.01, "entrelinha de 15 pt");
    }
    let iguais: Vec<f32> = (0..3).map(|k| achar(&f, "=", k).x).collect();
    let ys: Vec<f32> = (0..3).map(|k| achar(&f, "=", k).y).collect();
    assert!(ys[0] < ys[1] && ys[1] < ys[2]);
    let primeiro = iguais[0];
    assert!(iguais.iter().all(|x| perto(*x, primeiro, 0.001)), "os três = na mesma coluna: {iguais:?}");
    let sem = compor(r"\begin{align*} a &= b \\ c &= d \end{align*}", Modo::Exibida, T);
    assert!(sem.linhas.iter().all(|l| l.numero == Numero::Nenhum));
    assert_eq!(feather_math::quantos_numeros(r"\begin{align} a \\ b \nonumber \\ c \tag{x} \end{align}"), 1);
}

#[test]
fn delimitadores_crescem_com_o_conteudo() {
    let baixo = compor(r"\left( x \right)", Modo::Linha, T);
    let alto = compor(r"\left( \frac{\frac{a}{b}}{\frac{c}{d}} \right)", Modo::Exibida, T);
    let altura = |f: &Formula| f.altura + f.profundidade;
    assert!(altura(&alto) > 2.0 * altura(&baixo));
    // O parêntese grande é montado ou é variante: em qualquer caso, cobre o conteúdo.
    let par = compor(r"\left( \begin{matrix} 1 \\ 2 \\ 3 \\ 4 \\ 5 \\ 6 \end{matrix} \right)", Modo::Linha, T);
    assert!(altura(&par) > 6.0 * T, "seis linhas de 1,2 em: {}", altura(&par));
}

#[test]
fn espaco_entre_classes() {
    // a+b: espaço médio (4 mu) dos dois lados do binário; a=b: grosso (5 mu).
    let mais = compor("a+b", Modo::Linha, T);
    let igual = compor("a=b", Modo::Linha, T);
    let w_mais = compor("+", Modo::Linha, T).largura;
    let w_igual = compor("=", Modo::Linha, T).largura;
    let so_letras = compor("ab", Modo::Linha, T).largura;
    assert!(perto(mais.largura - so_letras - w_mais, 8.0 / 18.0 * T, 0.01));
    assert!(perto(igual.largura - so_letras - w_igual, 10.0 / 18.0 * T, 0.01));
    // No expoente (estilo script) o espaço do binário some.
    let s = compor("x^{a+b}", Modo::Linha, T);
    let s2 = compor("x^{ab+}", Modo::Linha, T);
    assert!(s.largura < s2.largura + 0.5);
    // -x: o binário sem operando vira ordinário (sem espaço).
    let menos = compor("-x", Modo::Linha, T).largura;
    assert!(menos < compor("-", Modo::Linha, T).largura + compor("x", Modo::Linha, T).largura + 0.01);
}
