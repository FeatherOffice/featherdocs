//! Modo dividido: o código LaTeX ou Markdown à esquerda das páginas, os dois editáveis.
//!
//! A interface só guarda o texto e repassa: o código digitado vai ao motor depois de uma pausa
//! (`dividido::reconciliar` relê e `Sessao::trocar_blocos` troca só os blocos que mudaram);
//! quando o documento muda pela página, o código é gerado de novo pelo motor, com as linhas de
//! cada bloco. Enquanto a pessoa digita no código ele não é reescrito.
use featherdocs_core::modelo::Documento;
use featherdocs_io::dividido::{self, Fonte};
use featherdocs_io::exportar::{self, LinhasDoBloco, Mapa};
use featherdocs_io::realce::{self, Tipo};
use iced::advanced::text::highlighter::{Format, Highlighter};
use iced::widget::text_editor::{Action, Content, Motion};
use iced::{font, Color, Font};
use std::ops::Range;
use std::time::{Duration, Instant};

/// Linhas de contexto abaixo da linha do bloco quando o código rola até ela.
const MARGEM: usize = 12;

/// A pausa na digitação antes de mandar o código ao motor.
pub const PAUSA: Duration = Duration::from_millis(300);

/// A fonte monoespaçada do sistema (DejaVu Sans Mono, Consolas…). A Liberation Mono embutida
/// tem maiúsculas baixas demais nesse corpo: na tela o código parecia todo em minúsculas.
pub const FONTE_MONO: Font = Font::MONOSPACE;
pub const TAMANHO: f32 = 13.5;
pub const PADDING: f32 = 10.;
/// A altura de uma linha do editor: o `LineHeight` padrão do Iced (1,3 × o tamanho), o mesmo do
/// `text` dos números de linha, que assim ficam alinhados.
pub const ALTURA_DA_LINHA: f32 = TAMANHO * 1.3;

pub struct PainelCodigo {
    pub fonte: Fonte,
    pub conteudo: Content,
    /// As linhas `[de, ate)` de cada bloco no texto do editor.
    pub blocos: Vec<LinhasDoBloco>,
    /// A versão da sessão que o texto do editor representa.
    pub versao: u64,
    /// Instante da última edição ainda não aplicada (pendente).
    pub ultima_edicao: Option<Instant>,
    /// O bloco cuja linha o cursor do código mostra agora.
    pub bloco_marcado: Option<u64>,
}

impl PainelCodigo {
    pub fn novo(fonte: Fonte, mapa: Mapa, versao: u64) -> Self {
        Self {
            fonte,
            conteudo: Content::with_text(&mapa.texto),
            blocos: mapa.blocos,
            versao,
            ultima_edicao: None,
            bloco_marcado: None,
        }
    }

    pub fn pendente(&self) -> bool {
        self.ultima_edicao.is_some()
    }

    /// A pausa terminou: hora de aplicar o código.
    pub fn maduro(&self, agora: Instant) -> bool {
        self.ultima_edicao.is_some_and(|t| agora.duration_since(t) >= PAUSA)
    }

    /// Troca o texto inteiro do editor (o documento mudou pela página).
    pub fn recarregar(&mut self, mapa: Mapa, versao: u64) {
        if self.conteudo.text() != mapa.texto {
            self.conteudo = Content::with_text(&mapa.texto);
            self.bloco_marcado = None;
        }
        self.blocos = mapa.blocos;
        self.versao = versao;
    }

    /// O bloco da linha `linha` do código (o último que começa nela ou antes).
    pub fn bloco_da_linha(&self, linha: usize) -> Option<u64> {
        self.blocos
            .iter()
            .filter(|b| b.de <= linha)
            .max_by_key(|b| b.de)
            .map(|b| b.id)
    }

    /// Leva o cursor do código ao começo do bloco `id`; o editor rola até ele.
    pub fn ir_ao_bloco(&mut self, id: u64) -> bool {
        if self.bloco_marcado == Some(id) {
            return false;
        }
        let Some(linha) = self.blocos.iter().find(|b| b.id == id).map(|b| b.de) else {
            return false;
        };
        self.ir_a_linha(linha);
        self.bloco_marcado = Some(id);
        true
    }

    /// O `text_editor` do Iced 0.13 não tem "ir para a linha": desce linha a linha a partir do
    /// começo. Sem quebra automática de linha (o painel usa `Wrapping::None`), cada `Down` é uma
    /// linha do texto. Para não andar a partir do zero, começa da linha onde o cursor já está.
    pub fn ir_a_linha(&mut self, linha: usize) {
        let atual = self.conteudo.cursor_position().0;
        let ultima = self.conteudo.line_count().saturating_sub(1);
        let alvo = linha.min(ultima);
        if atual > alvo && atual - alvo < alvo {
            for _ in alvo..atual {
                self.conteudo.perform(Action::Move(Motion::Up));
            }
        } else if atual <= alvo {
            for _ in atual..alvo {
                self.conteudo.perform(Action::Move(Motion::Down));
            }
        } else {
            self.conteudo.perform(Action::Move(Motion::DocumentStart));
            for _ in 0..alvo {
                self.conteudo.perform(Action::Move(Motion::Down));
            }
        }
        // O editor só rola o bastante para o cursor aparecer (na beirada de baixo): desce mais
        // algumas linhas e volta, para a linha do bloco ficar com contexto embaixo.
        let folga = MARGEM.min(ultima - alvo);
        for _ in 0..folga {
            self.conteudo.perform(Action::Move(Motion::Down));
        }
        for _ in 0..folga {
            self.conteudo.perform(Action::Move(Motion::Up));
        }
        self.conteudo.perform(Action::Move(Motion::Home));
    }
}

/// O código do documento na `fonte` pedida. Se `original` é um texto da pessoa (o `.tex` aberto
/// ou o último código aplicado) e ainda descreve este documento, ele é mantido na forma dela;
/// senão o motor escreve o código na forma dele.
pub fn gerar(doc: &Documento, fonte: Fonte, original: Option<&(Fonte, String)>) -> Mapa {
    if let Some((f, texto)) = original {
        if *f == fonte && descreve(doc, fonte, texto) {
            return exportar::mapa_de_texto(texto, doc, fonte);
        }
    }
    match fonte {
        Fonte::Latex => exportar::latex_com_mapa(doc),
        Fonte::Markdown => exportar::markdown_com_mapa(doc),
    }
}

/// O texto, relido, daria exatamente este documento (nada a trocar)?
fn descreve(doc: &Documento, fonte: Fonte, texto: &str) -> bool {
    let r = dividido::reconciliar(doc, fonte, texto);
    r.meta.is_none()
        && r.pagina.is_none_or(|p| dividido::mesma_pagina(&p, &doc.pagina))
        // Sem geometry no texto, ele só descreve a página que a leitura dele daria (A4, 2,54 cm).
        && (r.pagina.is_some() || dividido::mesma_pagina(&featherdocs_core::modelo::ConfigPagina::a4(), &doc.pagina))
        && r.extras.is_none()
        && r.numeracao.is_none_or(|n| n == doc.estilos.numeracao)
        && r.blocos == doc.blocos
}

pub fn nome_da_fonte(fonte: Fonte) -> &'static str {
    match fonte {
        Fonte::Latex => "LaTeX",
        Fonte::Markdown => "Markdown",
    }
}

// ---------------------------------------------------------------- realce de sintaxe

/// Realce do código pelo motor (`featherdocs_io::realce`), linha a linha. No Markdown guarda,
/// por linha, se ela está dentro de um bloco de código cercado (```), que atravessa linhas.
pub struct Realce {
    fonte: Fonte,
    linha: usize,
    /// `dentro_de_cerca[k]`: a linha `k` começa dentro de um bloco ```.
    dentro_de_cerca: Vec<bool>,
}

impl Highlighter for Realce {
    type Settings = Fonte;
    type Highlight = Tipo;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Tipo)>;

    fn new(fonte: &Fonte) -> Self {
        Self { fonte: *fonte, linha: 0, dentro_de_cerca: vec![false] }
    }

    fn update(&mut self, fonte: &Fonte) {
        *self = Self::new(fonte);
    }

    fn change_line(&mut self, linha: usize) {
        self.linha = self.linha.min(linha);
        self.dentro_de_cerca.truncate(self.linha + 1);
    }

    fn highlight_line(&mut self, texto: &str) -> Self::Iterator<'_> {
        let k = self.linha;
        let dentro = self.dentro_de_cerca.get(k).copied().unwrap_or(false);
        let (trechos, depois) = trechos_da_linha(self.fonte, texto, dentro);
        self.dentro_de_cerca.truncate(k + 1);
        self.dentro_de_cerca.push(depois);
        self.linha += 1;
        trechos.into_iter()
    }

    fn current_line(&self) -> usize {
        self.linha
    }
}

/// Os trechos coloridos de uma linha e se a linha seguinte começa dentro de um bloco ```.
pub fn trechos_da_linha(fonte: Fonte, texto: &str, dentro: bool) -> (Vec<(Range<usize>, Tipo)>, bool) {
    match fonte {
        Fonte::Latex => (realce::latex(texto).into_iter().map(|(a, b, t)| (a..b, t)).collect(), false),
        Fonte::Markdown => {
            let aparada = texto.trim_start();
            let cerca = aparada.starts_with("```") || aparada.starts_with("~~~");
            if dentro || cerca {
                let tudo = if texto.is_empty() { vec![] } else { vec![(0..texto.len(), Tipo::Codigo)] };
                return (tudo, dentro != cerca);
            }
            (realce::markdown(texto).into_iter().map(|(a, b, t)| (a..b, t)).collect(), false)
        }
    }
}

/// Os temas do realce (Configurações → Exibição). Cada um: comando, chave, matemática, ambiente,
/// ênfase e link, no claro e no escuro. O `feather_ui::tema` ainda não tem tokens de realce de
/// código; o "Feather" usa a cor de destaque da suíte no comando.
pub const TEMAS: [&str; 4] = ["Feather", "Overleaf", "Monokai", "Solarizado"];
const PALETAS: [([u32; 6], [u32; 6]); 4] = [
    ([0, 0xb05a00, 0x8a2be2, 0x0b7f7f, 0xa0306a, 0x0b7f7f], [0, 0xe5a35a, 0xc39bff, 0x5fd0c8, 0xe88abf, 0x5fd0c8]),
    ([0x1f4fa8, 0x8a5300, 0x0b7a3b, 0x8b1c62, 0x8b1c62, 0x1f4fa8], [0x7aa7ff, 0xe0b36a, 0x7fd49a, 0xe08ac0, 0xe08ac0, 0x7aa7ff]),
    ([0xc4245c, 0x8f7f00, 0x7a4fd6, 0x4f8a10, 0xc56a00, 0x1f8aa3], [0xf92672, 0xe6db74, 0xae81ff, 0xa6e22e, 0xfd971f, 0x66d9ef]),
    ([0x268bd2, 0xb58900, 0x6c71c4, 0x2aa198, 0xd33682, 0xcb4b16], [0x268bd2, 0xb58900, 0x6c71c4, 0x2aa198, 0xd33682, 0xcb4b16]),
];
static TEMA: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn definir_tema(i: usize) {
    TEMA.store(i.min(TEMAS.len() - 1), std::sync::atomic::Ordering::Relaxed);
}

/// As cores do tema `i` no tema claro/escuro atual, na ordem de [`PALETAS`] (para a amostra).
pub fn cores_do_tema(i: usize) -> [Color; 6] {
    let (claro, noturno) = PALETAS[i.min(TEMAS.len() - 1)];
    let p = if feather_ui::tema::escuro() { noturno } else { claro };
    p.map(|v| if v == 0 { feather_ui::tema::COR_AZUL_ACENTO() } else { Color::from_rgb8((v >> 16) as u8, (v >> 8) as u8, v as u8) })
}

/// A cor e o peso de cada tipo; o comentário usa sempre a cor apagada do tema da suíte.
pub fn formato(tipo: &Tipo, _tema: &iced::Theme) -> Format<Font> {
    let [comando, chave, matematica, ambiente, enfase, link] = cores_do_tema(TEMA.load(std::sync::atomic::Ordering::Relaxed));
    let negrito = Font { weight: font::Weight::Bold, ..FONTE_MONO };
    let italico = Font { style: font::Style::Italic, ..FONTE_MONO };
    let (cor, fonte) = match tipo {
        Tipo::Comando => (comando, None),
        Tipo::Comentario => (feather_ui::tema::COR_TEXTO_MUTED(), Some(italico)),
        Tipo::Chave | Tipo::Codigo => (chave, None),
        Tipo::Matematica => (matematica, None),
        Tipo::Ambiente => (ambiente, Some(negrito)),
        Tipo::Titulo => (comando, Some(negrito)),
        Tipo::Enfase => (enfase, None),
        Tipo::Link => (link, None),
        Tipo::Marcador | Tipo::Citacao => (chave, Some(negrito)),
    };
    Format { color: Some(cor), font: fonte }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn realce_markdown_atravessa_bloco_cercado() {
        let (_, dentro) = trechos_da_linha(Fonte::Markdown, "```rust", false);
        assert!(dentro);
        let (t, dentro) = trechos_da_linha(Fonte::Markdown, "# não é título", true);
        assert!(dentro);
        assert_eq!(t, vec![(0.."# não é título".len(), Tipo::Codigo)]);
        let (_, dentro) = trechos_da_linha(Fonte::Markdown, "```", true);
        assert!(!dentro);
        let (t, _) = trechos_da_linha(Fonte::Markdown, "# Título", false);
        assert!(t.iter().any(|(_, k)| *k == Tipo::Titulo));
    }

    #[test]
    fn realce_latex_marca_comando_e_comentario() {
        let (t, _) = trechos_da_linha(Fonte::Latex, "\\section{Um} % nota", false);
        assert!(t.iter().any(|(r, k)| *k == Tipo::Comando && *r == (0..8)));
        assert!(t.iter().any(|(_, k)| *k == Tipo::Comentario));
    }

    #[test]
    fn realce_reinicia_da_linha_alterada() {
        let mut r = Realce::new(&Fonte::Markdown);
        for l in ["a", "```", "x", "```", "b"] {
            let _ = r.highlight_line(l).count();
        }
        assert_eq!(r.current_line(), 5);
        r.change_line(2);
        assert_eq!(r.current_line(), 2);
        let t: Vec<_> = r.highlight_line("x").collect();
        assert_eq!(t, vec![(0..1, Tipo::Codigo)], "a linha 2 continua dentro da cerca");
    }

    #[test]
    fn ir_a_linha_posiciona_o_cursor() {
        let texto: String = (0..400).map(|i| format!("linha {i}\n")).collect();
        let mut p = PainelCodigo::novo(Fonte::Markdown, Mapa { texto, blocos: vec![] }, 0);
        p.ir_a_linha(250);
        assert_eq!(p.conteudo.cursor_position().0, 250);
        p.ir_a_linha(10);
        assert_eq!(p.conteudo.cursor_position().0, 10);
        // Além do fim: para na última linha com texto (a vazia depois do último \n não conta).
        p.ir_a_linha(5000);
        assert!(p.conteudo.cursor_position().0 >= 399);
    }
}
