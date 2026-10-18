//! Diálogos internos de formulário curto (link, nota, comentário, cabeçalho, margens…).
//! Cada tipo diz os campos, de onde vem o valor inicial e qual chamada do motor aplica.
use crate::app::Msg;
use featherdocs_core::sessao::Sessao;
use feather_ui::dialogo::{self as dlg, Dialogo};
use feather_ui::tema::{estilo_botao_pequeno, COR_TEXTO_MUTED};
use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Length};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tipo {
    Link,
    NotaRodape,
    NotaFim,
    Comentario,
    Cabecalho,
    Rodape,
    MarcaDagua,
    Marcador,
    IndiceEntrada,
    Referencia,
    Margens,
    Papel,
    BordaPagina,
    Propriedades,
    Tabela,
    NovaFonte,
    /// Citação: campo 0 = chaves marcadas (separadas por vírgula), campo 1 = página.
    Citacao,
    /// Editar a nota que está em `Pos` (o `bool` diz se é nota de fim).
    EditarNota(featherdocs_core::cursor::Pos, bool),
    /// Novo estilo com nome, com a formatação do parágrafo do cursor.
    NovoEstilo,
    /// Resposta ao comentário `id`.
    Resposta(u64),
}

pub struct Formulario {
    pub tipo: Tipo,
    pub campos: Vec<String>,
}

const CM: f32 = 72.0 / 2.54;

const ROTULOS_FONTE: [&str; 14] = [
    "Chave (como silva2023)", "Tipo", "Autores", "Título", "Ano", "Edição", "Local", "Editora",
    "Periódico, livro ou programa", "Volume", "Número", "Páginas", "Endereço (URL)", "Acesso em",
];

/// Um estilo com nome que guarda a formatação do parágrafo onde está o cursor.
pub fn estilo_do_cursor(s: &Sessao, nome: &str) -> featherdocs_core::modelo::Estilo {
    use featherdocs_core::modelo::{Alinhamento, Estilo, CODIGO, ITALICO, NEGRITO, SERIFA, SUBLINHADO};
    let f = s.formato_do_cursor();
    let base = f.estilo.strip_prefix("titulo").and_then(|n| n.parse().ok()).unwrap_or(0);
    let bits = [(f.negrito, NEGRITO), (f.italico, ITALICO), (f.sublinhado, SUBLINHADO), (f.codigo, CODIGO), (f.fonte == "serifa", SERIFA)]
        .iter()
        .filter(|(sim, _)| *sim)
        .fold(0, |b, (_, bit)| b | bit);
    Estilo {
        id: 0,
        nome: nome.trim().to_string(),
        base,
        bits,
        tamanho: f.tamanho,
        cor: featherdocs_core::fonte::ler_cor(&f.cor).unwrap_or(0),
        alinhamento: match f.alinhamento.as_str() {
            "centro" => Alinhamento::Centro,
            "direita" => Alinhamento::Direita,
            "justificado" => Alinhamento::Justificado,
            _ => Alinhamento::Esquerda,
        },
        antes: Some(f.antes),
        depois: Some(f.depois),
        entrelinha: f.entrelinha,
    }
}

/// Monta a fonte a partir dos campos do diálogo (a ordem de `ROTULOS_FONTE`).
fn fonte_dos_campos(c: &[String]) -> Result<featherdocs_core::modelo::Fonte, String> {
    use featherdocs_core::modelo::{Autor, Fonte, TipoDeFonte};
    let tipo = match c[1].trim().to_lowercase().as_str() {
        "" | "livro" => TipoDeFonte::Livro,
        "capítulo" | "capitulo" => TipoDeFonte::Capitulo,
        "artigo" => TipoDeFonte::Artigo,
        "site" => TipoDeFonte::Site,
        "tese" => TipoDeFonte::Tese,
        "relatório" | "relatorio" => TipoDeFonte::Relatorio,
        "outro" => TipoDeFonte::Outro,
        t => return Err(format!("Tipo de fonte desconhecido: \"{t}\".")),
    };
    let autores = c[2].split(';').map(str::trim).filter(|a| !a.is_empty()).map(|a| {
        let (sobrenome, nome) = a.split_once(',').unwrap_or((a, ""));
        Autor { sobrenome: sobrenome.trim().into(), nome: nome.trim().into() }
    }).collect();
    let t = |i: usize| c[i].trim().to_string();
    Ok(Fonte {
        chave: t(0), tipo, autores, titulo: t(3), ano: t(4), edicao: t(5), local: t(6), editora: t(7),
        veiculo: t(8), volume: t(9), numero: t(10), paginas: t(11), url: t(12), acesso: t(13),
        ..Fonte::default()
    })
}

fn cm(pt: f32) -> String {
    format!("{:.2}", pt / CM).replace('.', ",")
}
fn numero(s: &str, rotulo: &str) -> Result<f32, String> {
    s.trim().replace(',', ".").parse::<f32>().map_err(|_| format!("{rotulo}: número inválido."))
}

impl Tipo {
    fn titulo(self) -> &'static str {
        match self {
            Self::Link => "Link",
            Self::NotaRodape => "Nota de rodapé",
            Self::NotaFim => "Nota de fim",
            Self::Comentario => "Novo comentário",
            Self::Cabecalho => "Cabeçalho",
            Self::Rodape => "Rodapé",
            Self::MarcaDagua => "Marca d'água",
            Self::Marcador => "Marcador",
            Self::IndiceEntrada => "Marcar entrada do índice",
            Self::Referencia => "Referência cruzada",
            Self::Margens => "Margens personalizadas",
            Self::Papel => "Tamanho do papel",
            Self::BordaPagina => "Borda da página",
            Self::Propriedades => "Propriedades do documento",
            Self::Tabela => "Inserir tabela",
            Self::NovaFonte => "Nova fonte bibliográfica",
            Self::Citacao => "Inserir citação",
            Self::NovoEstilo => "Novo estilo",
            Self::EditarNota(_, fim) => if fim { "Editar nota de fim" } else { "Editar nota de rodapé" },
            Self::Resposta(_) => "Responder comentário",
        }
    }
    fn icone(self) -> &'static str {
        match self {
            Self::Link => "link",
            Self::Comentario | Self::Resposta(_) => "comentario",
            Self::Cabecalho | Self::Rodape => "header-footer",
            Self::Referencia | Self::Marcador | Self::IndiceEntrada => "tag",
            Self::Margens | Self::Papel | Self::BordaPagina => "square",
            Self::Propriedades => "info-circle",
            _ => "text-box",
        }
    }
    fn rotulos(self) -> &'static [&'static str] {
        match self {
            Self::Link => &["Endereço (https://… ou e-mail)", "Texto a mostrar"],
            Self::NotaRodape | Self::NotaFim | Self::EditarNota(..) => &["Texto da nota"],
            Self::Comentario => &["Comentário"],
            Self::Resposta(_) => &["Resposta"],
            Self::Cabecalho | Self::Rodape => &["Esquerda", "Centro", "Direita"],
            Self::MarcaDagua => &["Texto (vazio tira a marca)", "Cor (#rrggbb; vazio = cinza claro)"],
            Self::Marcador => &["Nome (letras, números e _)"],
            Self::IndiceEntrada => &["Termo", "Subtermo (opcional)"],
            Self::Referencia => &[],
            Self::Margens => &["Esquerda (cm)", "Direita (cm)", "Superior (cm)", "Inferior (cm)"],
            Self::Papel => &["Largura (cm)", "Altura (cm)"],
            Self::BordaPagina => &["Cor (#rrggbb; vazio tira a borda)", "Espessura (pt)", "Distância do papel (pt)"],
            Self::Propriedades => &["Título", "Autor"],
            Self::Tabela => &["Linhas", "Colunas"],
            Self::NovaFonte => &ROTULOS_FONTE,
            Self::Citacao => &[],
            Self::NovoEstilo => &["Nome do estilo"],
        }
    }
    fn explicacao(self) -> Option<&'static str> {
        match self {
            Self::Cabecalho | Self::Rodape => Some("Campos: {pagina}, {total}, {pagina_romano}, {titulo}, {autor}"),
            Self::Link => Some("Sem seleção, o texto entra no cursor já com o link."),
            Self::Referencia => Some("Escolha o que mostrar e clique no alvo."),
            Self::NovaFonte => Some("Autores: \"Sobrenome, Nome; Sobrenome, Nome\". Tipo: livro, capítulo, artigo, site, tese, relatório ou outro."),
            Self::Citacao => Some("Marque uma ou mais fontes; a página é opcional."),
            Self::NovoEstilo => Some("Guarda a formatação do parágrafo do cursor e a aplica à seleção."),
            _ => None,
        }
    }
}

impl Formulario {
    pub fn novo(tipo: Tipo, s: &Sessao) -> Self {
        let d = s.documento();
        let p = &d.pagina;
        let x = &d.extras;
        let campos = match tipo {
            Tipo::Link => vec![s.link_no_cursor().unwrap_or_default(), s.texto_selecionado()],
            Tipo::Cabecalho => vec![x.cabecalho.esquerda.clone(), x.cabecalho.centro.clone(), x.cabecalho.direita.clone()],
            Tipo::Rodape => vec![x.rodape.esquerda.clone(), x.rodape.centro.clone(), x.rodape.direita.clone()],
            Tipo::MarcaDagua => vec![x.marca.clone(), String::new()],
            Tipo::Referencia => vec!["texto".into()],
            Tipo::Margens => vec![cm(p.margem_esq), cm(p.margem_dir), cm(p.margem_sup), cm(p.margem_inf)],
            Tipo::Papel => vec![cm(p.largura), cm(p.altura)],
            Tipo::BordaPagina => vec![String::new(), "1".into(), "24".into()],
            Tipo::Propriedades => vec![d.meta.titulo.clone(), d.meta.autor.clone()],
            Tipo::Tabela => vec!["3".into(), "3".into()],
            Tipo::NovaFonte => {
                let mut v = vec![String::new(); ROTULOS_FONTE.len()];
                v[1] = "livro".into();
                v
            }
            Tipo::Citacao => vec![String::new(), String::new()],
            Tipo::EditarNota(p, _) => vec![s.nota_info(&p).map(|i| i.fonte).unwrap_or_default()],
            _ => vec![String::new(); tipo.rotulos().len()],
        };
        Self { tipo, campos }
    }

    /// Aplica no motor; `Ok` com o recado para a barra de status.
    pub fn aplicar(&self, s: &mut Sessao) -> Result<String, String> {
        let c = &self.campos;
        match self.tipo {
            Tipo::Link => s.link_definir(&c[0], &c[1]).map(|_| ()),
            Tipo::NotaRodape => s.nota_inserir(false, &c[0]).map(|_| ()),
            Tipo::NotaFim => s.nota_inserir(true, &c[0]).map(|_| ()),
            Tipo::Comentario => s.comentario_novo(&c[0]).map(|_| ()),
            Tipo::Cabecalho => { s.cabecalho_definir(&c[0], &c[1], &c[2]); Ok(()) }
            Tipo::Rodape => { s.rodape_definir(&c[0], &c[1], &c[2]); Ok(()) }
            Tipo::MarcaDagua => s.marca_dagua_definir(&c[0], &c[1]).map(|_| ()),
            Tipo::Marcador => s.marcador_adicionar(c[0].trim()).map(|_| ()),
            Tipo::IndiceEntrada => {
                let (_, n) = s.indice_marcar(c[0].trim(), c[1].trim(), false)?;
                return Ok(format!("{n} entrada(s) do índice marcada(s)"));
            }
            Tipo::Referencia => Ok(()),
            Tipo::Margens => {
                let r = self.tipo.rotulos();
                let v: Result<Vec<f32>, String> = c.iter().zip(r).map(|(x, r)| numero(x, r).map(|n| n * CM)).collect();
                let v = v?;
                s.margens_definir(v[0], v[1], v[2], v[3]).map(|_| ())
            }
            Tipo::Papel => s.papel_personalizado(numero(&c[0], "Largura")? * CM, numero(&c[1], "Altura")? * CM).map(|_| ()),
            Tipo::BordaPagina => s.borda_pagina_definir(&c[0], numero(&c[1], "Espessura")?, numero(&c[2], "Distância")?).map(|_| ()),
            Tipo::Propriedades => { s.meta_definir(c[0].trim(), c[1].trim()); Ok(()) }
            Tipo::NovoEstilo => s.estilo_criar(estilo_do_cursor(s, &c[0])).map(|_| ()),
            Tipo::EditarNota(p, fim) => s.nota_editar(&p, &c[0], fim).map(|_| ()),
            Tipo::NovaFonte => s.fonte_salvar(fonte_dos_campos(c)?, "").map(|_| ()),
            Tipo::Citacao => {
                let chaves: Vec<String> = c[0].split(',').filter(|k| !k.is_empty()).map(String::from).collect();
                s.citacao_inserir(&chaves, c[1].trim()).map(|_| ())
            }
            Tipo::Tabela => {
                let n = |x: &str, r: &str| x.trim().parse::<usize>().map_err(|_| format!("{r}: use um número inteiro."));
                s.inserir_tabela(n(&c[0], "Linhas")?, n(&c[1], "Colunas")?).map(|_| ())
            }
            Tipo::Resposta(id) => s.comentario_mudar(id, None, Some(c[0].trim()), None).map(|_| ()),
        }?;
        Ok(format!("{} aplicado", self.tipo.titulo()))
    }

    pub fn view<'a>(&'a self, s: &'a Sessao, base: Element<'a, Msg>) -> Element<'a, Msg> {
        let mut corpo = column![].spacing(10);
        if let Some(e) = self.tipo.explicacao() {
            corpo = corpo.push(text(e).size(12).color(COR_TEXTO_MUTED()));
        }
        for (i, rotulo) in self.tipo.rotulos().iter().enumerate() {
            let mut campo = text_input("", &self.campos[i])
                .on_input(move |v| Msg::FormCampo(i, v))
                .on_submit(Msg::FormOk)
                .padding([6, 8])
                .size(13);
            if i == 0 {
                campo = campo.id(text_input::Id::new("form0"));
            }
            corpo = corpo.push(column![text(*rotulo).size(12), campo].spacing(4));
        }
        if self.tipo == Tipo::Referencia {
            corpo = corpo.push(self.view_referencia(s));
        }
        if self.tipo == Tipo::Citacao {
            corpo = corpo.push(self.view_citacao(s));
        }
        let mut d = Dialogo::new(self.tipo.titulo(), corpo)
            .icone(self.tipo.icone())
            .ao_fechar(Msg::FormFechar)
            .topo(170.0)
            .secundario("Cancelar", Some(Msg::FormFechar));
        if self.tipo != Tipo::Referencia {
            d = d.primario("OK", Some(Msg::FormOk));
        }
        if self.tipo == Tipo::Link && s.link_no_cursor().is_some() {
            d = d.rodape_esquerda(dlg::botao_secundario("Remover link", Some(Msg::FormRemoverLink)));
        }
        d.sobre(base)
    }

    fn view_citacao<'a>(&'a self, s: &'a Sessao) -> Element<'a, Msg> {
        let fontes = &s.documento().bibliografia.fontes;
        if fontes.is_empty() {
            return dlg::aviso_vazio("Nenhuma fonte no documento: use Nova fonte ou Importar .bib.");
        }
        let marcadas: Vec<&str> = self.campos[0].split(',').filter(|k| !k.is_empty()).collect();
        let lista = column(fontes.iter().map(|f| {
            let marcada = marcadas.contains(&f.chave.as_str());
            let novo: Vec<&str> = if marcada {
                marcadas.iter().copied().filter(|k| *k != f.chave).collect()
            } else {
                marcadas.iter().copied().chain([f.chave.as_str()]).collect()
            };
            let autor = f.autores.first().map_or(String::new(), |a| a.sobrenome.clone());
            button(text(format!("{}  ·  {} {}  ·  {}", f.chave, autor, f.ano, f.titulo)).size(12))
                .width(Length::Fill).padding([5, 8])
                .style(estilo_botao_pequeno(marcada))
                .on_press(Msg::FormCampo(0, novo.join(",")))
                .into()
        })).spacing(2);
        column![
            dlg::rolagem(lista),
            column![text("Página citada (opcional)").size(12),
                text_input("", &self.campos[1]).on_input(|v| Msg::FormCampo(1, v)).on_submit(Msg::FormOk).padding([6, 8]).size(13)]
                .spacing(4),
        ]
        .spacing(10)
        .into()
    }

    fn view_referencia<'a>(&'a self, s: &'a Sessao) -> Element<'a, Msg> {
        let modo = self.campos[0].as_str();
        let modos = row(["texto", "numero", "pagina"].into_iter().zip(["Texto", "Número", "Página"]).map(|(m, r)| {
            button(text(r).size(12)).padding([4, 10]).style(estilo_botao_pequeno(modo == m))
                .on_press(Msg::FormCampo(0, m.into())).into()
        })).spacing(4);
        let alvos = s.alvos_de_referencia();
        if alvos.is_empty() {
            return column![modos, dlg::aviso_vazio("Nenhum título, legenda ou marcador no documento.")].spacing(10).into();
        }
        let lista = column(alvos.into_iter().map(|a| {
            let rotulo = format!("{}  {} {}  · p. {}", a.tipo, a.numero, a.texto, a.pagina);
            button(text(rotulo).size(12)).width(Length::Fill).padding([5, 8])
                .style(estilo_botao_pequeno(false)).on_press(Msg::FormEscolher(a.id)).into()
        })).spacing(2);
        column![modos, dlg::rolagem(lista)].spacing(10).into()
    }
}
