//! A composição de **um bloco**: quebra de linha e alturas, sem saber em que
//! página ele cai (isso é da `paginacao`). O resultado de um bloco só depende
//! do conteúdo dele e da largura útil, por isso pode ser guardado e reaproveitado
//! enquanto o bloco não muda — é o que torna a edição barata em documento grande.

use crate::inline::texto_puro;
use crate::medida::{largura, largura_do_caractere};
use crate::modelo::{Alinhamento, Atrib, Celula, Conteudo, Estilo, Lista, Marca as Objeto, MesclagemTabela, Trecho, CODIGO, EXIBIDA, ITALICO, MATEMATICA, NEGRITO, SUBLINHADO};

pub const COR_TEXTO: u32 = 0x1a1a_1aff;
pub const COR_LINK: u32 = 0x1a56_c4ff;
pub const COR_CITACAO: u32 = 0x5555_55ff;
pub const COR_CINZA: u32 = 0x8086_8bff;
pub const COR_FUNDO_CODIGO: u32 = 0xf1f3_f4ff;
pub const COR_BORDA: u32 = 0xbdc1_c6ff;

/// O fundo do trecho comentado.
pub const COR_COMENTADO: u32 = 0xfff0_b3ff;

/// A cor das sugestões de cada autor (o mesmo nome dá sempre a mesma cor).
pub fn cor_do_autor(autor: &str) -> u32 {
    const CORES: [u32; 6] = [0xc522_1fff, 0x1a73_e8ff, 0x1880_38ff, 0x9334_e6ff, 0xe374_00ff, 0x007b_83ff];
    let h = autor.bytes().fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(b)));
    CORES[(h % 6) as usize]
}

/// Corpo do texto, em pontos, e a entrelinha (múltiplo do corpo).
pub const CORPO: f32 = 11.0;
const ENTRELINHA: f32 = 1.4;
const TAMANHOS_TITULO: [f32; 6] = [24.0, 18.0, 15.0, 13.0, 12.0, 11.0];

/// Um pedaço de texto já posicionado: `x` na linha, `y` é a linha de base abaixo do topo da linha.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pedaco {
    pub x: f32,
    pub y: f32,
    pub texto: String,
    pub estilo: u8,
    pub tamanho: f32,
    pub cor: u32,
    /// Posição (em caracteres) do primeiro caractere deste pedaço no texto do parágrafo ou da célula.
    pub de: usize,
    /// Numa tabela, a coluna da célula a que o pedaço pertence.
    pub celula: Option<u16>,
    /// Cor do marca-texto atrás do pedaço (`0xRRGGBBAA`); 0 = sem.
    pub realce: u32,
    /// Quanto o texto sobe (+) ou desce (−) em relação à linha de base (sobrescrito/subscrito).
    pub desloc: f32,
    /// Um objeto preso ao texto (nota, citação, campo): o `texto` é o rótulo dele, mas para o cursor é
    /// **um caractere só**.
    pub atomo: bool,
}

impl Pedaco {
    /// Quantos caracteres do texto do parágrafo o pedaço ocupa (o objeto conta um).
    pub fn caracteres(&self) -> usize {
        if self.atomo {
            1
        } else {
            self.texto.chars().count()
        }
    }
}

/// Uma nota (de rodapé ou de fim) já composta, presa à linha onde está a chamada dela.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NotaComposta {
    pub fim: bool,
    pub linhas: Vec<Linha>,
    /// Onde está a chamada: o caractere no texto do parágrafo (ou da célula) e, em tabela, a coluna.
    pub desl: usize,
    pub coluna: Option<u16>,
}

impl NotaComposta {
    /// A altura que a nota ocupa no pé da página (com um respiro embaixo).
    pub fn altura(&self) -> f32 {
        self.linhas.iter().map(|l| l.altura).sum::<f32>() + 2.0
    }
}

/// Onde o texto de uma célula de tabela começa na linha: para o cursor numa célula vazia.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InfoCelula {
    pub x0: f32,
    pub base: f32,
    pub tam: f32,
}

/// Algo que não é texto: fundo, traço, caixa de imagem (`y` relativo ao topo da linha).
#[derive(Clone, Debug, PartialEq)]
pub enum Deco {
    Retangulo { x: f32, y: f32, w: f32, h: f32, cor: u32 },
    Traco { x1: f32, y1: f32, x2: f32, y2: f32, espessura: f32, cor: u32 },
    Imagem { x: f32, y: f32, w: f32, h: f32, origem: String },
    /// Texto que não é do documento (o número de uma equação): o cursor não passa por ele. `y` = linha de base.
    Texto { x: f32, y: f32, texto: String, tamanho: f32, estilo: u8, cor: u32 },
}

/// Uma linha composta; é a menor unidade que a paginação separa.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Linha {
    pub altura: f32,
    pub pedacos: Vec<Pedaco>,
    pub deco: Vec<Deco>,
    /// Entrada de sumário: o título para onde ela aponta (o número da página sai no quadro).
    pub alvo: Option<u64>,
    /// Na entrada de sumário: 1 = com o número da página, 2 = com o pontilhado até ele.
    pub sumario: u8,
    /// Posição do primeiro caractere desta linha no texto do parágrafo (para achar o cursor).
    pub de: usize,
    /// Onde o texto começa (recuo) e a linha de base e o corpo, para o cursor em linha vazia.
    pub x0: f32,
    pub base: f32,
    pub tam: f32,
    /// Só em linha de tabela: uma entrada por coluna.
    pub celulas: Vec<InfoCelula>,
    /// As notas cujas chamadas estão nesta linha, na ordem (a paginação as põe no pé da mesma página).
    pub notas: Vec<NotaComposta>,
    /// Os comentários que começam nesta linha (o quadro põe um balão na margem direita).
    pub comentarios: Vec<u64>,
}

/// O bloco composto, com as regras de quebra de página dele.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutBloco {
    pub linhas: Vec<Linha>,
    pub antes: f32,
    pub depois: f32,
    /// Quantas linhas, no mínimo, ficam no fim de uma página antes de quebrar (órfã).
    pub minimo_antes: usize,
    /// E quantas vão para a página seguinte (viúva).
    pub minimo_depois: usize,
    pub manter_com_proximo: bool,
    pub quebra_antes: bool,
    /// Posição `x` do marcador de lista (•, 1.), se for item.
    pub marcador_x: Option<f32>,
    /// Depois deste bloco, o que vem começa uma página nova (quebra de página).
    pub quebra_depois: bool,
    /// O número automático do título, desenhado antes da primeira linha (fora do texto: o cursor não passa por ele).
    pub numero: Option<NumeroDoTitulo>,
    /// A cor do marcador de lista; 0 = a do texto.
    pub cor_marcador: u32,
}

/// O número de um título ("1.2"), com a letra do próprio título.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NumeroDoTitulo {
    pub texto: String,
    pub x: f32,
    pub tamanho: f32,
    pub estilo: u8,
    pub cor: u32,
}

/// O que vem de fora do bloco e muda a composição dele: o número do título e o estilo com nome.
#[derive(Clone, Copy, Debug, Default)]
pub struct Entorno<'a> {
    pub numero: Option<&'a str>,
    pub estilo: Option<&'a Estilo>,
}

/// O que decide como um pedaço de texto se desenha: igual em fichas vizinhas = mesmo pedaço.
#[derive(Clone, Copy, PartialEq)]
struct Marca {
    estilo: u8,
    cor: u32,
    tamanho: f32,
    realce: u32,
    /// Quanto a linha de base sobe (+) ou desce (−) por causa de sobrescrito/subscrito, em pontos.
    desloc: f32,
}

/// Uma palavra ou um espaço, com a largura já medida.
struct Ficha {
    texto: String,
    marca: Marca,
    largura: f32,
    espaco: bool,
    /// Tabulação (`\t`): o que vem depois dela na linha encosta na margem direita (o `\hfill` do LaTeX).
    tab: bool,
    /// Posição do primeiro caractere no texto do parágrafo.
    de: usize,
    /// Objeto preso ao texto: o `texto` é o rótulo, e ele ocupa um caractere.
    atomo: bool,
    /// Fórmula em linha (`$…$`): uma ficha só, que não se parte nem se junta com a vizinha.
    formula: bool,
}

/// Onde o texto de um bloco começa, o quanto cabe, e como medir o tipo de bloco.
#[derive(Clone, Copy)]
struct Base {
    tamanho: f32,
    extra: u8,
    cor: u32,
    x0: f32,
    limite: f32,
    /// `false` em célula de tabela: o tamanho e a posição do trecho não valem (a altura da linha é fixa).
    usar_tamanho: bool,
    /// A largura da área de texto da página (onde as notas de rodapé se compõem); 0 = `limite`.
    pagina: f32,
}

impl Base {
    fn novo(tamanho: f32, extra: u8, cor: u32, x0: f32, limite: f32) -> Self {
        Self { tamanho, extra, cor, x0, limite, usar_tamanho: true, pagina: 0.0 }
    }

    fn largura_da_pagina(&self) -> f32 {
        if self.pagina > 0.0 {
            self.pagina
        } else {
            self.x0 + self.limite
        }
    }
}

fn estilo_efetivo(t: &Trecho, extra: u8) -> u8 {
    let mut e = t.estilo | extra;
    if t.link.is_some() {
        e |= SUBLINHADO;
    }
    e
}

/// Como o trecho se desenha neste bloco: tamanho, cor, realce e deslocamento de sobrescrito/subscrito.
fn marca_do_trecho(t: &Trecho, b: &Base) -> Marca {
    let mut tamanho = b.tamanho;
    let mut desloc = 0.0;
    let posicao = if matches!(t.marca.as_deref(), Some(Objeto::Nota { .. })) { 1 } else { t.fmt.posicao };
    if b.usar_tamanho || posicao > 0 && t.marca.is_some() {
        if t.fmt.tamanho > 0.0 && b.usar_tamanho {
            tamanho = t.fmt.tamanho;
        }
        if posicao != 0 {
            desloc = tamanho * if posicao > 0 { 0.35 } else { -0.14 };
            tamanho *= 0.65;
        }
    }
    let cor = if let Some(r) = t.rev.as_deref() {
        cor_do_autor(&r.autor)
    } else if t.link.is_some() {
        COR_LINK
    } else if t.fmt.cor != 0 {
        t.fmt.cor
    } else {
        b.cor
    };
    let mut estilo = estilo_efetivo(t, b.extra);
    match t.rev.as_deref().map(|r| r.tipo) {
        Some(crate::modelo::TipoDeRevisao::Insercao) => estilo |= SUBLINHADO,
        Some(crate::modelo::TipoDeRevisao::Remocao) => estilo |= crate::modelo::TACHADO,
        None => {}
    }
    Marca { estilo, cor, tamanho, realce: t.fmt.realce, desloc }
}

/// Parte os trechos em palavras e espaços.
fn fichas(trechos: &[Trecho], b: &Base) -> Vec<Ficha> {
    let mut saida = Vec::new();
    let mut de = 0;
    // Os comentários abertos neste ponto do texto: o trecho deles ganha fundo.
    let mut comentados = 0usize;
    for t in trechos {
        let mut marca = marca_do_trecho(t, b);
        if let Some(Objeto::Comentario { fim, .. }) = t.marca.as_deref() {
            comentados = if *fim { comentados.saturating_sub(1) } else { comentados + 1 };
        } else if comentados > 0 && marca.realce == 0 {
            marca.realce = COR_COMENTADO;
        }
        if t.marca.is_some() {
            saida.push(Ficha { atomo: true, ..ficha(&texto_do_objeto(t), marca, false, de) });
            de += 1;
            continue;
        }
        if marca.estilo & MATEMATICA != 0 && !t.texto.is_empty() {
            saida.push(Ficha { formula: true, ..ficha(&t.texto, marca, false, de) });
            de += t.texto.chars().count();
            continue;
        }
        let mut atual = String::new();
        let mut inicio = de;
        let mut espaco = false;
        for c in t.texto.chars() {
            if c == '\t' {
                if !atual.is_empty() {
                    saida.push(ficha(&atual, marca, espaco, inicio));
                    atual.clear();
                }
                saida.push(Ficha { texto: " ".into(), marca, largura: 0.0, espaco: false, tab: true, de, atomo: false, formula: false });
                de += 1;
                inicio = de;
                espaco = false;
                continue;
            }
            let c = if c == '\n' { ' ' } else { c };
            if !atual.is_empty() && (c == ' ') != espaco {
                saida.push(ficha(&atual, marca, espaco, inicio));
                atual.clear();
                inicio = de;
            }
            espaco = c == ' ';
            atual.push(c);
            de += 1;
        }
        if !atual.is_empty() {
            saida.push(ficha(&atual, marca, espaco, inicio));
        }
    }
    saida
}

fn ficha(texto: &str, marca: Marca, espaco: bool, de: usize) -> Ficha {
    Ficha { texto: texto.to_string(), marca, largura: largura(texto, marca.estilo, marca.tamanho), espaco, tab: false, de, atomo: false, formula: false }
}

/// O que um objeto preso ao texto mostra: o rótulo que o motor calculou. O campo de página, que só se
/// sabe no quadro, reserva o espaço de dois algarismos.
fn texto_do_objeto(t: &Trecho) -> String {
    t.rotulo.clone()
}

/// Uma palavra maior que a linha inteira é partida por caracteres.
fn partir_palavra(f: &Ficha, limite: f32) -> Vec<Ficha> {
    let mut partes = Vec::new();
    let mut atual = String::new();
    let mut inicio = f.de;
    let mut w = 0.0;
    for (passou, c) in f.texto.chars().enumerate() {
        let cw = f32::from(largura_do_caractere(c, f.marca.estilo)) * f.marca.tamanho / 1000.0;
        if w + cw > limite && !atual.is_empty() {
            partes.push(ficha(&atual, f.marca, false, inicio));
            atual.clear();
            inicio = f.de + passou;
            w = 0.0;
        }
        atual.push(c);
        w += cw;
    }
    if !atual.is_empty() {
        partes.push(ficha(&atual, f.marca, false, inicio));
    }
    partes
}

/// Quebra gulosa: enche a linha até não caber. Devolve as linhas como listas de fichas.
/// `limite_geral` vale para as linhas de continuação; a primeira tem `limite_geral - primeira` (recuo de primeira linha).
fn quebrar(fichas: Vec<Ficha>, limite_geral: f32, primeira: f32) -> Vec<Vec<Ficha>> {
    let mut linhas: Vec<Vec<Ficha>> = Vec::new();
    let mut atual: Vec<Ficha> = Vec::new();
    let mut w = 0.0;
    // O que vem depois de cada ficha (para a tabulação saber se o resto cabe na linha).
    let mut depois: Vec<f32> = fichas.iter().rev().scan(0.0, |soma, f| {
        let antes = *soma;
        *soma += f.largura;
        Some(antes)
    }).collect();
    depois.reverse();
    for (k, f) in fichas.into_iter().enumerate() {
        let limite = if linhas.is_empty() { (limite_geral - primeira).max(1.0) } else { limite_geral };
        // Tabulação: o que vem depois dela vai inteiro para a direita; se não cabe, desce junto para a linha seguinte.
        if f.tab && w + depois[k] > limite && !atual.is_empty() {
            linhas.push(std::mem::take(&mut atual));
            w = 0.0;
        }
        if f.espaco {
            // Espaço no começo de uma linha de continuação some; no começo do parágrafo fica.
            if !atual.is_empty() || linhas.is_empty() {
                w += f.largura;
                atual.push(f);
            }
            continue;
        }
        if w + f.largura > limite && !atual.is_empty() {
            linhas.push(std::mem::take(&mut atual));
            w = 0.0;
        }
        let pecas = if f.largura > limite && !f.atomo && !f.formula { partir_palavra(&f, limite) } else { vec![f] };
        for p in pecas {
            if w + p.largura > limite && !atual.is_empty() {
                linhas.push(std::mem::take(&mut atual));
                w = 0.0;
            }
            w += p.largura;
            atual.push(p);
        }
    }
    if !atual.is_empty() {
        linhas.push(atual);
    }
    linhas
}

/// Junta fichas vizinhas de mesma marca num pedaço só, sem o espaço do fim. Depois de uma tabulação,
/// o resto da linha encosta em `direita`.
fn pedacos_da_linha(mut fichas: Vec<Ficha>, x0: f32, y: f32, ultima: bool, justificar: Option<f32>, direita: f32) -> Vec<Pedaco> {
    // O espaço do fim some quando a linha quebra; na última linha fica (o cursor anda ao digitar espaço).
    while !ultima && fichas.last().is_some_and(|f| f.espaco) {
        fichas.pop();
    }
    let mut saida: Vec<Pedaco> = Vec::new();
    let mut x = x0;
    // Justificado: o que sobra na linha é repartido entre os espaços; cada palavra vira um pedaço.
    let extra = justificar.map_or(0.0, |sobra| {
        let espacos = fichas.iter().rposition(|f| !f.espaco).map_or(0, |u| fichas[..u].iter().filter(|f| f.espaco).count());
        if espacos == 0 {
            0.0
        } else {
            sobra / espacos as f32
        }
    });
    let partir = justificar.is_some() && extra > 0.0;
    let larguras: Vec<f32> = fichas.iter().map(|f| f.largura).collect();
    for (k, f) in fichas.into_iter().enumerate() {
        if f.tab {
            let resto: f32 = larguras[k + 1..].iter().sum();
            x = x.max(direita - resto);
            // O pedaço começa no caractere depois da tabulação: o texto que vier junta nele, já encostado à direita.
            saida.push(Pedaco { x, y, texto: String::new(), estilo: f.marca.estilo, tamanho: f.marca.tamanho, cor: f.marca.cor, de: f.de + 1, realce: f.marca.realce, desloc: f.marca.desloc, celula: None, atomo: false });
            continue;
        }
        let w = f.largura;
        let (espaco, m, atomo) = (f.espaco, f.marca, f.atomo);
        let formula = f.formula || saida.last().is_some_and(|p| p.estilo & MATEMATICA != 0);
        match saida.last_mut() {
            Some(p) if !partir && !atomo && !p.atomo && !formula && p.estilo == m.estilo && p.cor == m.cor && p.tamanho == m.tamanho && p.realce == m.realce && p.desloc == m.desloc => p.texto.push_str(&f.texto),
            _ => saida.push(Pedaco { x, y, texto: f.texto, estilo: m.estilo, tamanho: m.tamanho, cor: m.cor, de: f.de, celula: None, realce: m.realce, desloc: m.desloc, atomo }),
        }
        x += w;
        if partir && espaco {
            x += extra;
        }
    }
    saida
}

/// A linha de base dentro de uma linha de altura `altura` para texto de `tamanho`.
fn linha_de_base(altura: f32, tamanho: f32) -> f32 {
    (altura - tamanho) / 2.0 + tamanho * 0.8
}

/// Compõe um texto corrido em linhas.
fn texto_corrido(trechos: &[Trecho], tamanho: f32, extra: u8, cor: u32, x0: f32, limite: f32) -> Vec<Linha> {
    texto_formatado(trechos, &Base::novo(tamanho, extra, cor, x0, limite), &Atrib::default())
}

/// O texto de uma célula de tabela: sem tamanho próprio, porque a altura da linha da tabela é fixa.
fn texto_celula(trechos: &[Trecho], tamanho: f32, extra: u8, limite: f32, pagina: f32) -> Vec<Linha> {
    let b = Base { usar_tamanho: false, pagina, ..Base::novo(tamanho, extra, COR_TEXTO, 0.0, limite) };
    texto_formatado(trechos, &b, &Atrib::default())
}

/// A largura do que a linha mostra, sem os espaços do fim (que não entram no alinhamento).
fn largura_visivel(fichas: &[Ficha]) -> f32 {
    let fim = fichas.iter().rposition(|f| !f.espaco).map_or(0, |u| u + 1);
    fichas[..fim].iter().map(|f| f.largura).sum()
}

/// A entrelinha de um parágrafo como múltiplo do padrão (1,15).
fn fator_entrelinha(a: &Atrib) -> f32 {
    if a.entrelinha > 0.0 {
        a.entrelinha / 1.15
    } else {
        1.0
    }
}

/// O maior corpo entre o que a linha mostra (o do bloco, se só há espaço).
fn corpo_da_linha(f: &[Ficha], padrao: f32) -> f32 {
    f.iter().filter(|x| !x.espaco).map(|x| x.marca.tamanho).fold(0.0, f32::max).max(if f.iter().any(|x| !x.espaco) { 0.0 } else { padrao })
}

/// Como `texto_corrido`, com o alinhamento, os recuos e a entrelinha do parágrafo. `b.x0`/`b.limite` já
/// descontam o que o próprio tipo de bloco reserva (marcador de lista, barra da citação); `a.recuo_esq` e
/// `a.recuo_dir` entram por cima.
fn texto_formatado(trechos: &[Trecho], b: &Base, a: &Atrib) -> Vec<Linha> {
    let esq = b.x0 + a.recuo_esq.max(0.0);
    let util = (b.limite - a.recuo_esq.max(0.0) - a.recuo_dir.max(0.0)).max(1.0);
    // A primeira linha pode começar antes (negativo) da margem do parágrafo, até a borda esquerda.
    let primeira = a.primeira.max(-esq);
    let quebradas = quebrar(fichas(trechos, b), util, primeira);
    let total = quebradas.len();
    let fator = fator_entrelinha(a);
    let mut linhas: Vec<Linha> = quebradas
        .into_iter()
        .enumerate()
        .map(|(k, f)| {
            let ultima = k + 1 == total;
            let disponivel = if k == 0 { util - primeira } else { util };
            let inicio = if k == 0 { esq + primeira } else { esq };
            let visivel = largura_visivel(&f);
            let (x_ini, justificar) = match a.alinhamento {
                Alinhamento::Esquerda => (inicio, None),
                Alinhamento::Centro => (inicio + ((disponivel - visivel) / 2.0).max(0.0), None),
                Alinhamento::Direita => (inicio + (disponivel - visivel).max(0.0), None),
                Alinhamento::Justificado => (inicio, (!ultima).then_some((disponivel - visivel).max(0.0))),
            };
            let tam = corpo_da_linha(&f, b.tamanho);
            let mut altura = (tam * ENTRELINHA * fator).round();
            let mut base = linha_de_base(altura, tam);
            let mut pedacos = pedacos_da_linha(f, x_ini, base, ultima, justificar, inicio + disponivel);
            if b.usar_tamanho {
                acomodar_formulas(&mut altura, &mut base, &mut pedacos);
            }
            Linha { altura, de: pedacos.first().map_or(0, |p| p.de), x0: x_ini, base, tam, pedacos, ..Linha::default() }
        })
        .collect();
    if linhas.is_empty() {
        linhas.push(linha_vazia(b, a, esq + primeira, util - primeira));
    }
    decorar(&mut linhas, esq, util, a);
    if a.fundo != 0 {
        acolchoar(&mut linhas, FOLGA_DO_FUNDO);
    }
    prender_notas(&mut linhas, trechos, b.largura_da_pagina());
    linhas
}

/// A fórmula em linha mais alta (ou mais funda) que a linha a faz crescer, como o `\lineskip` do
/// TeX: a linha de base desce o que falta em cima e a altura cresce o que falta embaixo.
fn acomodar_formulas(altura: &mut f32, base: &mut f32, pedacos: &mut [Pedaco]) {
    const FOLGA: f32 = 1.0;
    let (mut sobe, mut desce) = (0.0f32, 0.0f32);
    for p in pedacos.iter().filter(|p| p.estilo & MATEMATICA != 0) {
        let f = feather_math::compor_guardado(&p.texto, feather_math::Modo::Linha, p.tamanho);
        sobe = sobe.max(f.altura + p.desloc + FOLGA);
        desce = desce.max(f.profundidade - p.desloc + FOLGA);
    }
    let extra_cima = (sobe - *base).max(0.0);
    if extra_cima > 0.0 {
        *base += extra_cima;
        *altura += extra_cima;
        for p in pedacos.iter_mut() {
            p.y += extra_cima;
        }
    }
    let extra_baixo = (desce - (*altura - *base)).max(0.0);
    *altura += extra_baixo;
}

/// O corpo do texto das notas de rodapé e de fim.
pub const CORPO_NOTA: f32 = 9.0;

/// Compõe o texto de cada nota do parágrafo e o prende à linha onde está a chamada dela.
fn prender_notas(linhas: &mut [Linha], trechos: &[Trecho], largura_pagina: f32) {
    let mut pos = 0;
    for t in trechos {
        let k = linhas.iter().rposition(|l| l.de <= pos).unwrap_or(0);
        match (t.marca.as_deref(), linhas.get_mut(k)) {
            (Some(Objeto::Nota { fim, texto }), Some(l)) => l.notas.push(NotaComposta { fim: *fim, linhas: compor_nota(&t.rotulo, texto, largura_pagina), desl: pos, coluna: None }),
            (Some(Objeto::Comentario { id, fim: false }), Some(l)) => l.comentarios.push(*id),
            _ => {}
        }
        pos += t.texto.chars().count();
    }
}

/// O texto de uma nota: corpo menor, o número sobrescrito na frente (fora do texto).
pub fn compor_nota(numero: &str, texto: &[Trecho], largura_pagina: f32) -> Vec<Linha> {
    // Uma nota dentro de nota não existe: o texto dela não traz objetos.
    let limpo: Vec<Trecho> = texto.iter().filter(|t| t.marca.is_none()).cloned().collect();
    let b = Base::novo(CORPO_NOTA, 0, COR_TEXTO, 0.0, largura_pagina);
    let tam_num = CORPO_NOTA * 0.65;
    let recuo = largura(numero, 0, tam_num) + 3.0;
    let mut linhas = texto_formatado(&limpo, &b, &Atrib { entrelinha: 1.0, ..Atrib::default() });
    for l in &mut linhas {
        for p in &mut l.pedacos {
            p.x += recuo;
        }
        l.x0 += recuo;
    }
    if let Some(l) = linhas.first_mut() {
        let y = l.base;
        l.pedacos.insert(0, Pedaco { x: 0.0, y, texto: numero.to_string(), estilo: 0, tamanho: tam_num, cor: COR_TEXTO, desloc: CORPO_NOTA * 0.35, ..Pedaco::default() });
    }
    linhas
}

/// A folga, em cima e embaixo, do parágrafo com fundo (o `\fboxsep` do LaTeX é 3 pt).
const FOLGA_DO_FUNDO: f32 = 4.0;

/// A linha de um parágrafo sem texto: o cursor fica onde o alinhamento manda.
fn linha_vazia(b: &Base, a: &Atrib, inicio: f32, disponivel: f32) -> Linha {
    let altura = (b.tamanho * ENTRELINHA * fator_entrelinha(a)).round();
    let x0 = match a.alinhamento {
        Alinhamento::Centro => inicio + disponivel / 2.0,
        Alinhamento::Direita => inicio + disponivel,
        _ => inicio,
    };
    Linha { altura, x0, base: linha_de_base(altura, b.tamanho), tam: b.tamanho, ..Linha::default() }
}

/// Fundo e bordas do parágrafo: um retângulo por linha e traços nas bordas pedidas.
fn decorar(linhas: &mut [Linha], esq: f32, util: f32, a: &Atrib) {
    if a.fundo == 0 && a.borda == 0 {
        return;
    }
    let (x1, x2) = (esq - 4.0, esq + util + 4.0);
    let n = linhas.len();
    for (k, l) in linhas.iter_mut().enumerate() {
        let h = l.altura;
        let mut deco = Vec::new();
        if a.fundo != 0 {
            // Um tico a mais para baixo: dois blocos de fundo seguidos (a faixa do cabeçalho) não mostram a emenda.
            deco.push(Deco::Retangulo { x: x1, y: 0.0, w: x2 - x1, h: h + 0.75, cor: a.fundo });
        }
        // A chamada (cor própria) tem a barra mais grossa.
        let (espessura, cor) = if a.cor_borda != 0 { (3.0, a.cor_borda) } else { (0.75, COR_TEXTO) };
        let traco = |xa, ya, xb, yb| Deco::Traco { x1: xa, y1: ya, x2: xb, y2: yb, espessura, cor };
        if a.borda & 1 != 0 {
            deco.push(traco(x1, 0.0, x1, h));
        }
        if a.borda & 2 != 0 {
            deco.push(traco(x2, 0.0, x2, h));
        }
        if a.borda & 4 != 0 && k == 0 {
            deco.push(traco(x1, 0.0, x2, 0.0));
        }
        if a.borda & 8 != 0 && k + 1 == n {
            deco.push(traco(x1, h, x2, h));
        }
        l.deco.splice(0..0, deco);
    }
}

fn bloco_de_texto(linhas: Vec<Linha>, antes: f32, depois: f32) -> LayoutBloco {
    LayoutBloco { linhas, antes, depois, minimo_antes: 2, minimo_depois: 2, ..LayoutBloco::default() }
}

/// A letra do bloco: a do tipo, ou a do estilo com nome quando o bloco tem um.
fn base_do_estilo(tamanho: f32, extra: u8, limite: f32, e: Option<&Estilo>) -> Base {
    match e {
        Some(e) => Base::novo(if e.tamanho > 0.0 { e.tamanho } else { tamanho }, e.bits, if e.cor != 0 { e.cor } else { COR_TEXTO }, 0.0, limite),
        None => Base::novo(tamanho, extra, COR_TEXTO, 0.0, limite),
    }
}

fn titulo(nivel: u8, texto: &[Trecho], limite: f32, a: &Atrib, en: &Entorno) -> LayoutBloco {
    let i = usize::from(nivel.clamp(1, 6)) - 1;
    let b = base_do_estilo(TAMANHOS_TITULO[i], NEGRITO, limite, en.estilo);
    let antes = if i == 0 { 18.0 } else { 12.0 };
    let (linhas, numero) = com_numero(texto, &b, a, en.numero);
    LayoutBloco { manter_com_proximo: true, numero, ..bloco_de_texto(linhas, antes, 6.0) }
}

/// O texto com o número automático na frente (título numerado, legenda): o número fica fora do
/// texto, com um vão de meio quadratim, e a primeira linha recua o tanto dele.
fn com_numero(texto: &[Trecho], b: &Base, a: &Atrib, numero: Option<&str>) -> (Vec<Linha>, Option<NumeroDoTitulo>) {
    let Some(n) = numero else { return (texto_formatado(texto, b, a), None) };
    let recuo = largura(n, b.extra, b.tamanho) + b.tamanho * 0.6;
    let linhas = texto_formatado(texto, b, &Atrib { primeira: a.primeira + recuo, ..*a });
    let x = linhas.first().map_or(0.0, |l| l.x0) - recuo;
    (linhas, Some(NumeroDoTitulo { texto: n.to_string(), x, tamanho: b.tamanho, estilo: b.extra, cor: b.cor }))
}

/// Um parágrafo; a legenda (figura, tabela) leva o número e fica junto do que vem depois dela.
fn paragrafo(texto: &[Trecho], limite: f32, a: &Atrib, en: &Entorno) -> LayoutBloco {
    let (linhas, numero) = com_numero(texto, &base_do_estilo(CORPO, 0, limite, en.estilo), a, en.numero);
    LayoutBloco { numero, manter_com_proximo: a.legenda == 2, ..bloco_de_texto(linhas, 0.0, 6.0) }
}

fn item(nivel: u8, texto: &[Trecho], limite: f32, a: &Atrib) -> LayoutBloco {
    let recuo = 18.0 + 18.0 * f32::from(nivel);
    let extra = if a.lista == Lista::Tarefa && a.marcado { COR_CINZA } else { COR_TEXTO };
    let mut linhas = texto_formatado(texto, &Base { pagina: limite, ..Base::novo(CORPO, 0, extra, recuo, limite - recuo) }, a);
    let x = recuo + a.recuo_esq.max(0.0) - 14.0;
    if a.lista == Lista::Tarefa {
        if let Some(l) = linhas.first_mut() {
            caixa_de_tarefa(l, x - 2.0, a.marcado);
        }
        return bloco_de_texto(linhas, 0.0, 3.0);
    }
    LayoutBloco { marcador_x: Some(x), cor_marcador: a.cor_marcador, ..bloco_de_texto(linhas, 0.0, 3.0) }
}

/// A caixa de seleção de um item de tarefa (e o visto, se marcada), na primeira linha.
fn caixa_de_tarefa(l: &mut Linha, x: f32, marcado: bool) {
    let lado = 9.0;
    let y = (l.base - lado * 0.95).max(0.0);
    if marcado {
        l.deco.push(Deco::Retangulo { x, y, w: lado, h: lado, cor: COR_FUNDO_CODIGO });
    }
    let borda = |x1, y1, x2, y2| Deco::Traco { x1, y1, x2, y2, espessura: 0.9, cor: COR_CINZA };
    l.deco.push(borda(x, y, x + lado, y));
    l.deco.push(borda(x + lado, y, x + lado, y + lado));
    l.deco.push(borda(x + lado, y + lado, x, y + lado));
    l.deco.push(borda(x, y + lado, x, y));
    if marcado {
        l.deco.push(Deco::Traco { x1: x + 1.8, y1: y + lado * 0.55, x2: x + lado * 0.42, y2: y + lado - 2.0, espessura: 1.4, cor: COR_TEXTO });
        l.deco.push(Deco::Traco { x1: x + lado * 0.42, y1: y + lado - 2.0, x2: x + lado - 1.6, y2: y + 2.0, espessura: 1.4, cor: COR_TEXTO });
    }
}

fn citacao(texto: &[Trecho], limite: f32, a: &Atrib) -> LayoutBloco {
    let mut linhas = texto_formatado(texto, &Base { pagina: limite, ..Base::novo(CORPO, ITALICO, COR_CITACAO, 22.0, limite - 22.0) }, a);
    for l in &mut linhas {
        l.deco.push(Deco::Traco { x1: 6.0, y1: 0.0, x2: 6.0, y2: l.altura, espessura: 2.5, cor: COR_BORDA });
    }
    bloco_de_texto(linhas, 2.0, 8.0)
}

/// Código: caractere monoespaçado, quebra por colunas, fundo cinza em cada linha.
fn codigo(fonte: &str, limite: f32) -> LayoutBloco {
    let tamanho = 9.5;
    let altura = 13.0;
    let base = linha_de_base(altura, tamanho);
    let colunas = (((limite - 12.0) / (tamanho * 0.6)).floor() as usize).max(1);
    let mut linhas = Vec::new();
    let mut de = 0;
    for crua in fonte.split('\n') {
        let cs: Vec<char> = crua.chars().collect();
        let partes: Vec<String> = cs.chunks(colunas).map(|p| p.iter().collect::<String>()).chain(cs.is_empty().then(String::new)).collect();
        for parte in partes {
            let n = parte.chars().count();
            let pedacos = vec![Pedaco { x: 6.0, y: base, texto: parte, estilo: CODIGO, tamanho, cor: COR_TEXTO, de, ..Pedaco::default() }];
            let deco = vec![Deco::Retangulo { x: 0.0, y: 0.0, w: limite, h: altura, cor: COR_FUNDO_CODIGO }];
            linhas.push(Linha { altura, pedacos, deco, de, x0: 6.0, base, tam: tamanho, ..Linha::default() });
            de += n;
        }
        de += 1; // a quebra de linha do código
    }
    acolchoar(&mut linhas, 4.0);
    LayoutBloco { linhas, antes: 4.0, depois: 10.0, minimo_antes: 2, minimo_depois: 2, ..LayoutBloco::default() }
}

/// Aumenta a primeira e a última linha (o "padding" do bloco de código).
fn acolchoar(linhas: &mut [Linha], extra: f32) {
    let n = linhas.len();
    for (i, l) in linhas.iter_mut().enumerate() {
        let em_cima = if i == 0 { extra } else { 0.0 };
        let em_baixo = if i + 1 == n { extra } else { 0.0 };
        l.altura += em_cima + em_baixo;
        for p in &mut l.pedacos {
            p.y += em_cima;
        }
        l.base += em_cima;
        for d in &mut l.deco {
            if let Deco::Retangulo { h, .. } = d {
                *h += em_cima + em_baixo;
            }
        }
    }
}

fn celulas_da_linha(linha: &[Vec<Trecho>], colunas: usize, largura_col: f32, cabecalho: bool, mesclagens: &[MesclagemTabela], indice_linha: usize) -> Linha {
    let tam = 10.0;
    let extra = if cabecalho { NEGRITO } else { 0 };
    let pagina = largura_col * colunas as f32;
    let compostas: Vec<Vec<Linha>> = (0..colunas).map(|c| {
        let regiao = mesclagens.iter().find(|m| m.contem(indice_linha, c));
        if regiao.is_some_and(|m| m.coluna != c) {
            return Vec::new(); // interior da mesclagem: sem texto ou borda interna
        }
        let largura = regiao.map_or(largura_col, |m| largura_col * m.colunas as f32);
        texto_celula(linha.get(c).map_or(&[][..], Vec::as_slice), tam, extra, largura - 10.0, pagina)
    }).collect();
    let linhas_max = compostas.iter().map(Vec::len).max().unwrap_or(1);
    let altura = (tam * ENTRELINHA).round();
    let total = altura * linhas_max as f32 + 8.0;
    let mut saida = Linha { altura: total, ..Linha::default() };
    let base = 4.0 + linha_de_base(altura, tam);
    if cabecalho {
        saida.deco.push(Deco::Retangulo { x: 0.0, y: 0.0, w: largura_col * colunas as f32, h: total, cor: COR_FUNDO_CODIGO });
    }
    for (c, celula) in compostas.into_iter().enumerate() {
        let x0 = largura_col * c as f32;
        saida.celulas.push(InfoCelula { x0: x0 + 5.0, base, tam });
        for (k, l) in celula.into_iter().enumerate() {
            saida.notas.extend(l.notas.into_iter().map(|n| NotaComposta { coluna: Some(c as u16), ..n }));
            saida.comentarios.extend(l.comentarios);
            for mut p in l.pedacos {
                p.x += x0 + 5.0;
                p.y += 4.0 + altura * k as f32;
                p.celula = Some(c as u16);
                saida.pedacos.push(p);
            }
        }
        if !mesclagens.iter().any(|m| m.contem(indice_linha, c) && m.coluna != c) {
            saida.deco.push(Deco::Traco { x1: x0, y1: 0.0, x2: x0, y2: total, espessura: 0.5, cor: COR_BORDA });
        }
    }
    let fim = largura_col * colunas as f32;
    saida.deco.push(Deco::Traco { x1: fim, y1: 0.0, x2: fim, y2: total, espessura: 0.5, cor: COR_BORDA });
    saida.deco.push(Deco::Traco { x1: 0.0, y1: total, x2: fim, y2: total, espessura: 0.5, cor: COR_BORDA });
    saida
}

/// Tabela: colunas iguais, uma `Linha` por linha da tabela (a paginação nunca parte uma linha).
fn tabela(cabecalho: bool, linhas: &[Vec<Celula>], limite: f32, mesclagens: &[MesclagemTabela]) -> LayoutBloco {
    let colunas = linhas.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let largura_col = limite / colunas as f32;
    let mut compostas: Vec<Linha> = linhas.iter().enumerate().map(|(i, l)| celulas_da_linha(l, colunas, largura_col, cabecalho && i == 0, mesclagens, i)).collect();
    if let Some(primeira) = compostas.first_mut() {
        let fim = largura_col * colunas as f32;
        primeira.deco.push(Deco::Traco { x1: 0.0, y1: 0.0, x2: fim, y2: 0.0, espessura: 0.5, cor: COR_BORDA });
    }
    LayoutBloco { linhas: compostas, antes: 4.0, depois: 10.0, minimo_antes: 1, minimo_depois: 1, ..LayoutBloco::default() }
}

/// Uma linha única que não se parte (imagem, equação, régua).
fn indivisivel(linha: Linha, antes: f32, depois: f32) -> LayoutBloco {
    LayoutBloco { linhas: vec![linha], antes, depois, minimo_antes: 1, minimo_depois: 0, ..LayoutBloco::default() }
}

fn imagem(origem: &str, legenda: &[Trecho], w: f32, h: f32, limite: f32) -> LayoutBloco {
    let (mut w, mut h) = if w > 0.0 && h > 0.0 { (w, h) } else { (limite * 0.7, limite * 0.5) };
    let escala = (limite / w).min(520.0 / h).min(1.0);
    w *= escala;
    h *= escala;
    let deco = vec![Deco::Imagem { x: (limite - w) / 2.0, y: 0.0, w, h, origem: origem.to_string() }];
    let mut bloco = indivisivel(Linha { altura: h, deco, ..Linha::default() }, 6.0, 4.0);
    if !legenda.is_empty() {
        let mut texto = texto_corrido(legenda, 9.5, ITALICO, COR_CITACAO, 0.0, limite);
        for l in &mut texto {
            for p in &mut l.pedacos {
                p.x += ((limite - largura(&p.texto, p.estilo, p.tamanho)) / 2.0).max(0.0);
            }
        }
        bloco.linhas.extend(texto);
        bloco.minimo_antes = 2;
    }
    bloco.depois = 10.0;
    bloco
}

/// O espaço acima e abaixo da equação em destaque (o `\abovedisplayskip` de 10 pt da classe `article`).
const ESPACO_DA_EQUACAO: f32 = 10.0;

/// Equação em destaque: a fórmula composta pelo `feather-math` (estilo display), centrada, e o número
/// de cada linha numerada encostado à direita. `numero` = o primeiro número automático dela.
fn equacao(latex: &str, limite: f32, numero: Option<&str>) -> LayoutBloco {
    let tamanho = CORPO;
    let estilo = MATEMATICA | EXIBIDA;
    let f = feather_math::compor_guardado(latex, feather_math::Modo::Exibida, tamanho);
    let folga = 2.0;
    let base = folga + f.altura;
    let altura = (f.altura + f.profundidade + 2.0 * folga).max(tamanho * ENTRELINHA);
    // O número sai na fonte da fórmula, em pé (o `\tag` do LaTeX usa a romana do texto, a mesma Latin Modern).
    let rotulos: Vec<(f32, String)> = rotulos_da_equacao(&f, numero).into_iter().map(|(y, r)| (y, format!("\\text{{{r}}}"))).collect();
    // O número não pode encostar na fórmula: sem espaço, ela vai para a esquerda (como o TeX faz).
    let maior = rotulos.iter().map(|(_, r)| largura(r, MATEMATICA, tamanho)).fold(0.0, f32::max);
    let mut x = ((limite - f.largura) / 2.0).max(0.0);
    if maior > 0.0 && x + f.largura > limite - maior - tamanho {
        x = (limite - maior - tamanho - f.largura).max(0.0);
    }
    let p = Pedaco { x, y: base, texto: latex.to_string(), estilo, tamanho, cor: COR_TEXTO, ..Pedaco::default() };
    let deco = rotulos
        .into_iter()
        .map(|(y, texto)| Deco::Texto { x: limite - largura(&texto, MATEMATICA, tamanho), y: base + y, tamanho, estilo: MATEMATICA, cor: COR_TEXTO, texto })
        .collect();
    let linha = Linha { altura, pedacos: vec![p], deco, x0: x, base, tam: tamanho, ..Linha::default() };
    indivisivel(linha, ESPACO_DA_EQUACAO - folga, ESPACO_DA_EQUACAO - folga)
}

/// "(1)", "(2)"… (ou o `\tag`) de cada linha numerada, com a posição da linha de base dela.
fn rotulos_da_equacao(f: &feather_math::Formula, numero: Option<&str>) -> Vec<(f32, String)> {
    let mut n: u32 = numero.and_then(|s| s.parse().ok()).unwrap_or(1);
    f.linhas
        .iter()
        .filter_map(|l| match &l.numero {
            feather_math::Numero::Automatico => {
                let r = format!("({n})");
                n += 1;
                Some((l.base, r))
            }
            feather_math::Numero::Tag(t) => Some((l.base, format!("({t})"))),
            feather_math::Numero::Nenhum => None,
        })
        .collect()
}

/// A quebra de página aparece como uma linha discreta no fim da página; o que vem depois começa a seguinte.
fn quebra_de_pagina(limite: f32) -> LayoutBloco {
    let texto = "quebra de página";
    let tamanho = 8.5;
    let altura = 16.0;
    let w = largura(texto, ITALICO, tamanho);
    let x = ((limite - w) / 2.0).max(0.0);
    let base = linha_de_base(altura, tamanho);
    let p = Pedaco { x, y: base, texto: texto.to_string(), estilo: ITALICO, tamanho, cor: COR_CINZA, ..Pedaco::default() };
    let deco = vec![
        Deco::Traco { x1: 0.0, y1: altura / 2.0, x2: (x - 6.0).max(0.0), y2: altura / 2.0, espessura: 0.5, cor: COR_BORDA },
        Deco::Traco { x1: x + w + 6.0, y1: altura / 2.0, x2: limite, y2: altura / 2.0, espessura: 0.5, cor: COR_BORDA },
    ];
    let linha = Linha { altura, pedacos: vec![p], deco, x0: x, base, tam: tamanho, ..Linha::default() };
    LayoutBloco { quebra_depois: true, ..indivisivel(linha, 2.0, 2.0) }
}

/// A régua: na cor da borda do bloco (o `{\color{x}\hrule}` do LaTeX), ou cinza.
fn regua(limite: f32, a: &Atrib) -> LayoutBloco {
    let (cor, espessura) = if a.cor_borda != 0 { (a.cor_borda, 1.0) } else { (COR_BORDA, 0.75) };
    let deco = vec![Deco::Traco { x1: 0.0, y1: 5.0, x2: limite, y2: 5.0, espessura, cor }];
    indivisivel(Linha { altura: 10.0, deco, base: 8.0, tam: CORPO, ..Linha::default() }, 4.0, 4.0)
}

/// Uma entrada do sumário: o título e para onde ela aponta.
#[derive(Clone, Debug, PartialEq)]
pub struct EntradaSumario {
    pub id: u64,
    pub nivel: u8,
    pub texto: String,
}

/// O sumário: uma linha por título até `niveis`. A altura não depende do número da
/// página (que sai só no quadro), então a paginação não precisa rodar duas vezes.
pub fn sumario(entradas: &[EntradaSumario], niveis: u8, (numeros, pontilhado): (bool, bool), limite: f32, rotulo: &str) -> LayoutBloco {
    let marcas = u8::from(numeros) | if numeros && pontilhado { 2 } else { 0 };
    let mut linhas = texto_corrido(&[Trecho::simples(rotulo)], 16.0, NEGRITO, COR_TEXTO, 0.0, limite);
    let altura = 18.0;
    for e in entradas.iter().filter(|e| e.nivel <= niveis) {
        let recuo = 14.0 * f32::from(e.nivel.saturating_sub(1));
        let estilo = if e.nivel == 1 { NEGRITO } else { 0 };
        let texto = cortar(&e.texto, estilo, CORPO, limite - recuo - if numeros { 40.0 } else { 0.0 });
        let p = Pedaco { x: recuo, y: linha_de_base(altura, CORPO), texto, estilo, tamanho: CORPO, cor: COR_TEXTO, ..Pedaco::default() };
        linhas.push(Linha { altura, pedacos: vec![p], alvo: Some(e.id), sumario: marcas, ..Linha::default() });
    }
    if linhas.len() == 1 {
        let p = Pedaco { x: 0.0, y: linha_de_base(altura, CORPO), texto: "—".to_string(), estilo: ITALICO, tamanho: CORPO, cor: COR_CINZA, ..Pedaco::default() };
        linhas.push(Linha { altura, pedacos: vec![p], ..Linha::default() });
    }
    LayoutBloco { linhas, antes: 6.0, depois: 14.0, minimo_antes: 2, minimo_depois: 2, ..LayoutBloco::default() }
}

/// Um parágrafo de uma lista gerada (referências, índice): o texto, os recuos e o espaço antes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParagrafoGerado {
    pub texto: Vec<Trecho>,
    pub atrib: Atrib,
    pub antes: f32,
}

/// Uma lista gerada pelo motor (a bibliografia, o índice remissivo): o título e os parágrafos. Como o
/// sumário, não se edita à mão — o cursor passa por cima dela inteira.
pub fn lista_gerada(titulo: &str, paragrafos: &[ParagrafoGerado], limite: f32) -> LayoutBloco {
    let mut linhas = texto_corrido(&[Trecho::simples(titulo)], 16.0, NEGRITO, COR_TEXTO, 0.0, limite);
    if let Some(l) = linhas.last_mut() {
        l.altura += 6.0;
    }
    for p in paragrafos {
        let mut ls = texto_formatado(&p.texto, &Base::novo(CORPO, 0, COR_TEXTO, 0.0, limite), &p.atrib);
        if let Some(l) = ls.first_mut() {
            l.altura += p.antes;
            l.base += p.antes;
            for x in &mut l.pedacos {
                x.y += p.antes;
            }
        }
        linhas.extend(ls);
    }
    if paragrafos.is_empty() {
        let p = Pedaco { x: 0.0, y: linha_de_base(18.0, CORPO), texto: "—".to_string(), estilo: ITALICO, tamanho: CORPO, cor: COR_CINZA, ..Pedaco::default() };
        linhas.push(Linha { altura: 18.0, pedacos: vec![p], ..Linha::default() });
    }
    LayoutBloco { linhas, antes: 6.0, depois: 14.0, minimo_antes: 2, minimo_depois: 2, ..LayoutBloco::default() }
}

/// Corta o texto com reticências para caber em `limite`.
fn cortar(texto: &str, estilo: u8, tamanho: f32, limite: f32) -> String {
    if largura(texto, estilo, tamanho) <= limite {
        return texto.to_string();
    }
    let mut saida = String::new();
    for c in texto.chars() {
        if largura(&format!("{saida}{c}…"), estilo, tamanho) > limite {
            break;
        }
        saida.push(c);
    }
    saida.push('…');
    saida
}

/// O texto do título para o sumário.
pub fn texto_do_titulo(c: &Conteudo) -> Option<(u8, String)> {
    match c {
        Conteudo::Titulo { nivel, texto } => Some((*nivel, texto_puro(texto))),
        _ => None,
    }
}

/// O corpo padrão do tipo de bloco, em pontos.
pub fn tamanho_base(c: &Conteudo) -> f32 {
    match c {
        Conteudo::Titulo { nivel, .. } => TAMANHOS_TITULO[usize::from((*nivel).clamp(1, 6)) - 1],
        Conteudo::Codigo { .. } => 9.5,
        Conteudo::Tabela { .. } => 10.0,
        _ => CORPO,
    }
}

/// Compõe um bloco (menos o sumário, que depende dos outros: ver `sumario`).
pub fn compor(c: &Conteudo, a: &Atrib, limite: f32) -> LayoutBloco {
    compor_em(c, a, limite, &Entorno::default())
}

/// Como `compor`, com o número do título e o estilo com nome do bloco.
pub fn compor_em(c: &Conteudo, a: &Atrib, limite: f32, en: &Entorno) -> LayoutBloco {
    compor_em_mesclada(c, a, limite, en, &[])
}

/// Compõe tabelas mescladas sem suprimir divisórias em outras linhas.
pub fn compor_em_mesclada(
    c: &Conteudo, a: &Atrib, limite: f32, en: &Entorno,
    mesclagens: &[MesclagemTabela],
) -> LayoutBloco {
    let mut lay = compor_bloco(c, a, limite, en, mesclagens);
    if matches!(c, Conteudo::Titulo { .. } | Conteudo::Paragrafo { .. } | Conteudo::Item { .. } | Conteudo::Citacao { .. } | Conteudo::Regua) {
        lay.antes = a.antes.unwrap_or(lay.antes);
        lay.depois = a.depois.unwrap_or(lay.depois);
    }
    lay
}

fn compor_bloco(c: &Conteudo, a: &Atrib, limite: f32, en: &Entorno, mesclagens: &[MesclagemTabela]) -> LayoutBloco {
    match c {
        Conteudo::Titulo { nivel, texto } => titulo(*nivel, texto, limite, a, en),
        Conteudo::Paragrafo { texto } => paragrafo(texto, limite, a, en),
        Conteudo::Item { nivel, texto, .. } => item(*nivel, texto, limite, a),
        Conteudo::Citacao { texto } => citacao(texto, limite, a),
        Conteudo::Codigo { codigo: fonte, .. } => codigo(fonte, limite),
        Conteudo::Tabela { cabecalho, linhas } => tabela(*cabecalho, linhas, limite, mesclagens),
        Conteudo::Imagem { origem, legenda, largura, altura } => imagem(origem, legenda, *largura, *altura, limite),
        Conteudo::Equacao { latex } => equacao(latex, limite, en.numero),
        Conteudo::Regua => regua(limite, a),
        Conteudo::QuebraDePagina => quebra_de_pagina(limite),
        Conteudo::Sumario { .. } | Conteudo::Bibliografia { .. } | Conteudo::IndiceRemissivo => LayoutBloco::default(),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn par(texto: &str) -> LayoutBloco {
        compor(&Conteudo::Paragrafo { texto: vec![Trecho::simples(texto)] }, &Atrib::default(), 200.0)
    }

    #[test]
    fn tabulacao_encosta_o_resto_na_direita_e_desce_inteira_se_nao_cabe() {
        let l = par("Curso\tJan 2020 – Dez 2024");
        assert_eq!(l.linhas.len(), 1);
        let ultimo = l.linhas[0].pedacos.last().unwrap();
        assert!((ultimo.x + largura(&ultimo.texto, ultimo.estilo, ultimo.tamanho) - 200.0).abs() < 0.01, "{ultimo:?}");
        let l = par(&format!("{}\tJan 2020 – Dez 2024", "palavra ".repeat(3)));
        assert_eq!(l.linhas.len(), 2, "a data desce inteira");
        assert_eq!(l.linhas[1].pedacos.iter().map(|p| p.texto.as_str()).collect::<String>(), "Jan 2020 – Dez 2024");
    }

    #[test]
    fn quebra_em_varias_linhas_sem_estourar() {
        let l = par(&"palavra ".repeat(40));
        assert!(l.linhas.len() > 3);
        for linha in &l.linhas {
            for p in &linha.pedacos {
                assert!(p.x + largura(&p.texto, p.estilo, p.tamanho) <= 200.01);
            }
        }
    }

    #[test]
    fn palavra_gigante_e_partida() {
        let l = par(&"x".repeat(300));
        assert!(l.linhas.len() > 1);
    }

    #[test]
    fn paragrafo_vazio_ocupa_uma_linha() {
        assert_eq!(par("").linhas.len(), 1);
    }

    #[test]
    fn tabela_tem_uma_linha_por_linha() {
        let cel = |t: &str| vec![Trecho::simples(t)];
        let l = compor(&Conteudo::Tabela { cabecalho: true, linhas: vec![vec![cel("a"), cel("b")], vec![cel("c"), cel("d")]] }, &Atrib::default(), 200.0);
        assert_eq!(l.linhas.len(), 2);
    }

    fn numeros_da_equacao(l: &LayoutBloco) -> Vec<String> {
        l.linhas[0].deco.iter().filter_map(|d| match d {
            Deco::Texto { texto, .. } => Some(texto.clone()),
            _ => None,
        }).collect()
    }

    #[test]
    fn equacao_em_destaque_e_composta_e_numerada() {
        let c = Conteudo::Equacao { latex: "\\begin{align} a &= b \\\\ c &= d \\nonumber \\\\ e &= \\frac{1}{2} \\end{align}".into() };
        let l = compor_em(&c, &Atrib::default(), 400.0, &Entorno { numero: Some("4"), estilo: None });
        assert_eq!(numeros_da_equacao(&l), vec!["\\text{(4)}", "\\text{(5)}"]);
        let p = &l.linhas[0].pedacos[0];
        assert_eq!(p.estilo, MATEMATICA | EXIBIDA);
        // Três linhas de fórmula: a altura é a da fórmula composta, não um número fixo.
        let f = feather_math::compor(&p.texto, feather_math::Modo::Exibida, CORPO);
        assert!(l.linhas[0].altura >= f.altura + f.profundidade);
        assert!(l.linhas[0].altura > 3.0 * CORPO);
        // Centrada e com o número na margem direita.
        assert!((p.x - (400.0 - f.largura) / 2.0).abs() < 0.01);
        let sem = compor(&Conteudo::Equacao { latex: "x^2".into() }, &Atrib::default(), 400.0);
        assert!(numeros_da_equacao(&sem).is_empty(), "\\[ … \\] não leva número");
    }

    #[test]
    fn formula_em_linha_e_uma_ficha_so_e_alarga_a_linha() {
        let t = vec![Trecho::simples("antes "), Trecho { texto: "\\frac{\\frac{a}{b}}{\\frac{c}{d}}".into(), estilo: MATEMATICA, ..Trecho::default() }, Trecho::simples(" depois")];
        let l = compor(&Conteudo::Paragrafo { texto: t }, &Atrib::default(), 400.0);
        let f = l.linhas[0].pedacos.iter().find(|p| p.estilo & MATEMATICA != 0).expect("a fórmula");
        assert!(f.texto.starts_with("\\frac"), "não se parte nas chaves nem nos espaços");
        let normal = par("antes depois");
        assert!(l.linhas[0].altura > normal.linhas[0].altura, "a fração empilhada aumenta a linha");
        assert!((largura(&f.texto, f.estilo, f.tamanho) - feather_math::largura(&f.texto, feather_math::Modo::Linha, f.tamanho)).abs() < 0.001);
    }

    #[test]
    fn sumario_tem_uma_linha_por_titulo() {
        let e = vec![EntradaSumario { id: 1, nivel: 1, texto: "A".into() }, EntradaSumario { id: 2, nivel: 3, texto: "B".into() }];
        assert_eq!(sumario(&e, 2, (true, true), 300.0, "Sumário").linhas.len(), 2);
    }
}
