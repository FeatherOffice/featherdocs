//! A árvore da fórmula: a lista de átomos (os "noads" do TeX), cada um com núcleo,
//! expoente e índice. O analisador (`analise`) a monta; a composição (`composicao`)
//! a transforma em caixas; o OMML (`omml`) a traduz para a equação do Word.

/// A classe do átomo, que decide o espaço entre vizinhos (TeXbook, cap. 18).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Classe {
    Ord,
    Op,
    Bin,
    Rel,
    Abre,
    Fecha,
    Pont,
    Interno,
}

/// O alfabeto de uma letra: o normal da matemática (latinas e gregas minúsculas em itálico) ou
/// os de `\mathrm`, `\mathbf`, `\mathbb`…
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Alfabeto {
    #[default]
    Normal,
    Romano,
    Negrito,
    Italico,
    NegritoItalico,
    Duplo,
    Caligrafico,
    Fraktur,
    SemSerifa,
    Mono,
}

/// Estilo do TeX: display, text, script, scriptscript.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nivel {
    D,
    T,
    S,
    SS,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alinh {
    Esq,
    Centro,
    Dir,
}

/// O tipo da matriz/ambiente, que decide espaçamento entre colunas, estilo das células e numeração.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipoMatriz {
    /// `matrix` e companhia, `array`, `cases`, `substack`: células em estilo de texto.
    Matriz,
    /// `aligned`, `align`, `split`, `alignat`, `flalign`: pares r/l em estilo display.
    Alinhado,
    /// `gathered`, `gather`, `equation`: uma coluna centrada, display.
    Reunido,
    /// `multline`: primeira à esquerda, última à direita.
    Multlinha,
    /// `eqnarray`: r c l com o espaço do `array`.
    Eqnarray,
    /// `\substack`: linhas centradas em estilo de índice.
    Pilha,
}

/// O número de uma linha de equação.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Numero {
    /// Sem número (`align*`, `\nonumber`, `\[ … \]`).
    #[default]
    Nenhum,
    /// O próximo número da sequência.
    Automatico,
    /// `\tag{…}`: o texto, sem os parênteses.
    Tag(String),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Matriz {
    pub tipo: Option<TipoMatriz>,
    pub linhas: Vec<Vec<Vec<No>>>,
    /// O alinhamento de cada coluna (`array{lcr}`); falta = centrado.
    pub colunas: Vec<Alinh>,
    /// Coluna com traço vertical à esquerda dela (`|` no `array`); índice = coluna.
    pub tracos: Vec<usize>,
    /// Linhas com `\hline` acima delas (o índice `linhas.len()` = embaixo da última).
    pub hlines: Vec<usize>,
    /// Espaço entre as colunas, em em (metade de cada lado).
    pub sep: f32,
    /// Meio `sep` também antes da primeira e depois da última coluna (o `array`).
    pub margem: bool,
    /// Espaço extra pedido por `\\[…]`, em em, depois de cada linha.
    pub extra: Vec<f32>,
    /// O número de cada linha (só nos ambientes de equação numerada).
    pub numeros: Vec<Numero>,
    /// Os `\label` de cada linha.
    pub rotulos: Vec<Option<String>>,
    /// `\intertext`: o texto que vem antes da linha de índice tal.
    pub intertextos: Vec<(usize, Vec<No>)>,
    pub esq: Option<char>,
    pub dir: Option<char>,
}

/// O núcleo de um átomo.
#[derive(Clone, Debug, PartialEq)]
pub enum Nucleo {
    /// Um caractere matemático (símbolo, letra, algarismo) num alfabeto.
    Car(char, Alfabeto),
    /// Texto em pé (`\text`, nome de função): sem espaço matemático entre as letras.
    Texto(String, Alfabeto),
    Grupo(Vec<No>),
    Fracao { num: Vec<No>, den: Vec<No>, barra: bool, nivel: Option<Nivel>, esq: Option<char>, dir: Option<char> },
    Raiz { indice: Option<Vec<No>>, corpo: Vec<No> },
    /// `\left … \middle … \right`: os pedaços entre os delimitadores.
    Delimitado { esq: Option<char>, dir: Option<char>, partes: Vec<Vec<No>>, meios: Vec<Option<char>> },
    /// Delimitador de tamanho fixo (`\big(`): altura total em em.
    Grande { c: Option<char>, altura: f32 },
    /// Acento: o caractere combinante; `largo` = cresce com a base (`\widehat`, setas).
    Acento { c: char, corpo: Vec<No>, largo: bool, baixo: bool },
    /// `\overline` (`cima`) e `\underline`.
    Barra { corpo: Vec<No>, cima: bool },
    /// Chave/parêntese horizontal que cresce (`\overbrace`, `\underbrace`, `\xrightarrow`).
    Chave { c: char, corpo: Vec<No>, cima: bool },
    /// Espaço em mu (1/18 em do estilo atual).
    Espaco(f32),
    /// Espaço em em (não muda com o estilo: `\hspace`, `\quad` medem em em).
    EspacoEm(f32),
    Matriz(Box<Matriz>),
    /// Troca de estilo dali em diante (`\displaystyle`).
    Estilo(Nivel),
    Cor(u32, Vec<No>),
    Fantasma { corpo: Vec<No>, largura: bool, altura: bool },
    /// `\smash`: a caixa sem altura nem profundidade.
    Achatado(Vec<No>),
    Caixa(Vec<No>),
    /// `\not`: uma barra sobre o átomo seguinte.
    Negado(Box<No>),
    /// `\mathclap`/`\mathllap`/`\mathrlap`: o conteúdo sem largura (centrado, para a esquerda ou para a direita).
    Lap(Vec<No>, Alinh),
    /// `\cancel` (1 = /), `\bcancel` (2 = \), `\xcancel` (3 = X), `\cancelto` (4 = / com seta e valor).
    Riscado { corpo: Vec<No>, tipo: u8, valor: Option<Vec<No>> },
    /// `\rule`: retângulo cheio (medidas em em; `sobe` desloca da linha de base).
    Regra { largura: f32, altura: f32, sobe: f32 },
    /// `\intertext{…}` entre as linhas de um `align` (texto à esquerda, numa linha própria).
    Intertexto(Vec<No>),
    /// Marcas de alinhamento do ambiente (`&`, `\\`), tiradas pelo analisador antes de compor.
    SepColuna,
    SepLinha(f32),
    Hline,
    Rotulo(String),
    Tag(String),
    SemNumero,
    Vazio,
}

/// Um átomo: classe, núcleo e os scripts. `limites`: `None` = o padrão do operador (em cima/embaixo só
/// no display); `Some(true)` = `\limits`; `Some(false)` = `\nolimits`.
#[derive(Clone, Debug, PartialEq)]
pub struct No {
    pub classe: Classe,
    pub nucleo: Nucleo,
    pub sup: Option<Vec<No>>,
    pub sub: Option<Vec<No>>,
    pub limites: Option<bool>,
}

impl No {
    pub fn novo(classe: Classe, nucleo: Nucleo) -> Self {
        Self { classe, nucleo, sup: None, sub: None, limites: None }
    }

    pub fn ord(nucleo: Nucleo) -> Self {
        Self::novo(Classe::Ord, nucleo)
    }

    /// Não é átomo: não entra na conta de vizinhos do espaçamento.
    pub fn eh_marca(&self) -> bool {
        matches!(
            self.nucleo,
            Nucleo::Espaco(_) | Nucleo::EspacoEm(_) | Nucleo::Estilo(_) | Nucleo::Rotulo(_) | Nucleo::Tag(_) | Nucleo::SemNumero | Nucleo::SepColuna | Nucleo::SepLinha(_) | Nucleo::Hline
        ) && self.sup.is_none()
            && self.sub.is_none()
    }
}
