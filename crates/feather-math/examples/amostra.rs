//! Gera um SVG com fórmulas, uma embaixo da outra, para conferir o desenho:
//! `cargo run -p feather-math --example amostra -- saida.svg [arquivo-com-uma-formula-por-linha]`.
//! Linha começando com `$` = fórmula em linha; as outras, em destaque.

use feather_math::{compor, svg, Modo};

const PADRAO: &str = r"$E = mc^2 \quad f'(x) \quad f''(x) \quad x_i^2 + y_{j}^{3} = \frac{a+b}{c-d}
\frac{1}{2} + \frac{\alpha}{\beta^2}
\sqrt{x^2+1} + \sqrt[3]{\frac{a}{b}}
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
\int_0^\infty e^{-x^2}\,dx = \frac{\sqrt{\pi}}{2}
\lim_{x \to 0} \frac{\sin x}{x} = 1
\left( \frac{a}{b} \right)^2 \left[ \sum_k x_k \right] \left\{ \int f \right\}
\begin{pmatrix} a & b \\ c & d \end{pmatrix} \begin{bmatrix} 1 & 0 & 0 \\ 0 & 1 & 0 \\ 0 & 0 & 1 \end{bmatrix}
f(x) = \begin{cases} x^2 & \text{se } x \ge 0 \\ -x & \text{caso contrário} \end{cases}
\hat{x} \bar{y} \vec{v} \dot{a} \ddot{b} \tilde{n} \widehat{xyz} \overline{AB}
\overbrace{a+b+c}^{3} \cdot \underbrace{x+y}_{2}
\mathbb{R} \mathcal{L} \mathfrak{g} \mathbf{v} \mathrm{d}x \mathsf{A} \mathtt{x}
\binom{n}{k} = \frac{n!}{k!(n-k)!}
\begin{align} a &= b + c \\ d &= e + f + g \\ h &= i \end{align}
\nabla \times \mathbf{E} = -\frac{\partial \mathbf{B}}{\partial t}
\Biggl( \biggl( \Bigl( \bigl( x \bigr) \Bigr) \biggr) \Biggr)
a \leq b \neq c \approx d \Rightarrow e \to f \subseteq g \in h
\oint_{\partial S} \mathbf{F} \cdot d\mathbf{r} = \iint_S (\nabla \times \mathbf{F}) \cdot d\mathbf{S}
X \xrightarrow{f} Y \xrightarrow[\text{iso}]{g} Z, \qquad a \stackrel{\text{def}}{=} b
\left\langle \psi \middle| \hat{H} \middle| \psi \right\rangle \quad \boxed{e^{i\pi}+1=0}
\left( \begin{matrix} 1 \\ 2 \\ 3 \\ 4 \\ 5 \\ 6 \\ 7 \end{matrix} \right) \left\{ \begin{matrix} 1 \\ 2 \\ 3 \\ 4 \\ 5 \\ 6 \\ 7 \end{matrix} \right. \left\| \begin{matrix} 1 \\ 2 \\ 3 \\ 4 \end{matrix} \right\|
\sqrt{\begin{matrix} 1 \\ 2 \\ 3 \\ 4 \\ 5 \end{matrix}}";

fn main() {
    let mut args = std::env::args().skip(1);
    let saida = args.next().unwrap_or_else(|| "amostra.svg".into());
    let texto = args.next().map(|a| std::fs::read_to_string(a).expect("ler")).unwrap_or_else(|| PADRAO.to_string());
    let mut y = 10.0f32;
    let mut corpo = String::new();
    for linha in texto.lines().filter(|l| !l.trim().is_empty()) {
        let (latex, modo) = match linha.strip_prefix('$') {
            Some(l) => (l, Modo::Linha),
            None => (linha, Modo::Exibida),
        };
        let f = compor(latex, modo, 20.0);
        y += f.altura + 8.0;
        let s = svg(&f, "#000", 0.0);
        let interno = s.split_once('>').map(|x| x.1).unwrap_or("").trim_end_matches("</svg>");
        corpo.push_str(&format!("<g transform=\"translate(20 {y})\">{interno}</g>"));
        corpo.push_str(&format!("<line x1=\"10\" x2=\"15\" y1=\"{y}\" y2=\"{y}\" stroke=\"red\" stroke-width=\"0.5\"/>"));
        y += f.profundidade + 8.0;
    }
    let doc = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"700\" height=\"{y}\" viewBox=\"0 0 700 {y}\"><rect width=\"700\" height=\"{y}\" fill=\"white\"/>{corpo}</svg>");
    std::fs::write(&saida, doc).expect("gravar");
}
