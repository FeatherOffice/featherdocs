//! Faixa com componentes e ícones da suíte; cada comando é executado pelo motor.
use crate::{
    app::{App, Comando, Msg},
    cor_ui::AlvoCor,
    localidade::tr,
};
use feather_ui::{
    componentes::{botao_grande, botao_icone, botao_pequeno, grupo_faixa, lista_suspensa},
    icones::icone,
    tema::*,
};
use crate::dialogos::Tipo as Form;
use featherdocs_io::dividido::Fonte;
use iced::widget::{button, column, row, scrollable, text};
use iced::{Alignment, Element, Length};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aba {
    Inicio,
    Inserir,
    Layout,
    Revisao,
    Exibir,
}
impl Aba {
    pub const TODAS: [Self; 5] = [
        Self::Inicio,
        Self::Inserir,
        Self::Layout,
        Self::Revisao,
        Self::Exibir,
    ];
    pub fn nome(self) -> &'static str {
        tr(match self {
            Self::Inicio => "inicio",
            Self::Inserir => "inserir",
            Self::Layout => "layout",
            Self::Revisao => "revisao",
            Self::Exibir => "exibir",
        })
    }
}
fn grande(nome: &str, chave: &str, cmd: Comando, ativo: bool) -> Element<'static, Msg> {
    botao_grande(icone(nome, 24.), tr(chave), Some(Msg::Comando(cmd)), ativo)
}
fn pequeno(nome: &str, chave: &str, cmd: Comando, ativo: bool) -> Element<'static, Msg> {
    botao_pequeno(icone(nome, 16.), tr(chave), Some(Msg::Comando(cmd)), ativo)
}
fn so_icone(nome: &str, dica: &str, cmd: Comando, ativo: bool) -> Element<'static, Msg> {
    botao_icone(icone(nome, 16.), dica, Some(Msg::Comando(cmd)), ativo)
}
/// Menu da faixa: o rótulo escolhido vira o comando na mesma posição de `comandos`.
fn menu(rotulo: &'static str, opcoes: &'static [&'static str], comandos: fn(usize) -> Comando, largura: f32) -> Element<'static, Msg> {
    lista_suspensa(rotulo, opcoes, None, move |o| {
        Msg::Comando(comandos(opcoes.iter().position(|x| *x == o).unwrap_or(0)))
    }, largura)
}

/// Um comando contextual não deve ser acionado fora de uma célula de tabela.
fn tabela_botao(rotulo: &'static str, comando: Comando, disponivel: bool) -> Element<'static, Msg> {
    let botao = button(text(rotulo).size(12)).padding([4, 8])
        .style(estilo_botao_pequeno(false));
    let botao = if disponivel { botao.on_press(Msg::Comando(comando)) } else { botao };
    botao.into()
}
/// Os estilos com nome guardados no documento (vindos do .docx ou criados nele).
fn estilos_do_documento(app: &App) -> Element<'_, Msg> {
    let lista = &app.sessao.documento().estilos.lista;
    if lista.is_empty() {
        return column![].into();
    }
    let nomes: Vec<String> = lista.iter().map(|e| e.nome.clone()).collect();
    let atual = app.sessao.estilo_com_nome_no_cursor();
    let selecionado = lista.iter().find(|e| e.id == atual).map(|e| e.nome.clone());
    iced::widget::pick_list(nomes, selecionado, move |nome: String| {
        let id = lista.iter().find(|e| e.nome == nome).map_or(0, |e| e.id);
        Msg::Comando(Comando::EstiloNome(id))
    })
    .placeholder("Estilos do documento")
    .text_size(11)
    .padding([4, 8])
    .width(Length::Fixed(140.))
    .style(estilo_menu_da_faixa)
    .menu_style(estilo_menu_popup)
    .into()
}
pub fn view(app: &App) -> Element<'_, Msg> {
    let formato = &app.visao.formato;
    let em_tabela = formato.tabela.is_some();
    let bloco_selecionado = app.sessao.cursor().foco.bloco;
    let em_imagem = app.sessao.documento().blocos.iter().any(|b| {
        b.id == bloco_selecionado
            && matches!(&b.conteudo,
                featherdocs_core::modelo::Conteudo::Imagem { origem, .. }
                    if origem.starts_with("data:image/png;base64,")
                    || origem.starts_with("data:image/jpeg;base64,")
            )
    });
    let em_grafico = app.sessao.documento().blocos.iter().any(|b| {
        b.id == bloco_selecionado && matches!(
            &b.conteudo,
            featherdocs_core::modelo::Conteudo::Imagem { origem, .. }
                if featherdocs_io::grafico_docs::eh_grafico(origem)
        )
    });
    // Modelo de abas da suíte; enum Aba e Msg continuam do FeatherDocs.
    let descritores: Vec<_> = Aba::TODAS.iter().copied()
        .map(|aba| feather_ui::ribbon::AbaRibbon::nova(aba, aba.nome()))
        .collect();
    let abas = feather_ui::ribbon::abas(&descritores, &app.aba, Msg::Aba);
    let conteudo: Element<'_, Msg> = match app.aba {
        Aba::Inicio => row![
            grupo_faixa(
                row![
                    grande("clipboard", "colar", Comando::Colar, false),
                    column![
                        pequeno("cut", "recortar", Comando::Recortar, false),
                        pequeno("copy", "copiar", Comando::Copiar, false),
                        botao_pequeno(
                            icone("brush", 16.0),
                            "Pincel de formatação",
                            Some(Msg::Comando(Comando::Pincel)),
                            formato.pincel,
                        )
                    ]
                    .spacing(5),
                    column![so_icone("clipboard", "Colar sem formatação (Ctrl+Shift+V)", Comando::ColarSemFormatacao, false)]
                ]
                .spacing(5)
                .into(),
                tr("area_transferencia")
            ),
            grupo_faixa(
                column![
                    row![
                        feather_ui::formatacao::seletor_fonte(
                            &["sans", "serifa", "mono"],
                            Some(formato.fonte.as_str()), |s| Msg::Fonte(s.into()), 110.0
                        ),
                        feather_ui::formatacao::seletor_tamanho(
                            &[8u16, 10, 11, 12, 14, 16, 18, 24, 32, 48],
                            Some(formato.tamanho.round() as u16), Msg::Tamanho, 65.0
                        ),
                        menu("Aa", &["Frase", "minúsculas", "MAIÚSCULAS", "Cada Palavra", "aLTERNAR"],
                            |i| Comando::Maiusculas(["frase", "minusculas", "maiusculas", "palavras", "alternar"][i]), 105.0),
                    ]
                    .spacing(5),
                    row![
                        so_icone("bold", "Negrito (Ctrl+B)", Comando::Formato(featherdocs_core::modelo::NEGRITO), formato.negrito),
                        so_icone("italic", "Itálico (Ctrl+I)", Comando::Formato(featherdocs_core::modelo::ITALICO), formato.italico),
                        so_icone("underline", "Sublinhado (Ctrl+U)", Comando::Formato(featherdocs_core::modelo::SUBLINHADO), formato.sublinhado),
                        so_icone("strikethrough", "Tachado (Ctrl+Shift+X)", Comando::Formato(featherdocs_core::modelo::TACHADO), formato.tachado),
                        so_icone("superscript", "Sobrescrito (Ctrl+Shift+=)", Comando::Posicao(1), formato.sobrescrito),
                        so_icone("subscript", "Subscrito (Ctrl+=)", Comando::Posicao(-1), formato.subscrito),
                        so_icone("code", "Código (Ctrl+Shift+E)", Comando::Formato(featherdocs_core::modelo::CODIGO), formato.codigo),
                        so_icone("font-increase", "Aumentar fonte (Ctrl+Shift+>)", Comando::TamanhoPasso(1), false),
                        so_icone("font-decrease", "Diminuir fonte (Ctrl+Shift+<)", Comando::TamanhoPasso(-1), false),
                        so_icone("clear", "Limpar formatação (Ctrl+Espaço)", Comando::LimparFormatacao, false),
                    ]
                    .spacing(2)
                    .align_y(Alignment::Center)
                ]
                .spacing(7)
                .into(),
                tr("fonte")
            ),
            grupo_faixa(
                column![
                    button(text("Cor do texto").size(12))
                        .on_press(Msg::AbrirCor(AlvoCor::Texto))
                        .style(estilo_botao_pequeno(false)),
                    button(text("Marca-texto").size(12))
                        .on_press(Msg::AbrirCor(AlvoCor::Realce))
                        .style(estilo_botao_pequeno(false)),
                    button(text("Sombreamento").size(12))
                        .on_press(Msg::AbrirCor(AlvoCor::Fundo))
                        .style(estilo_botao_pequeno(false)),
                ]
                .spacing(3)
                .into(),
                "Cores"
            ),
            grupo_faixa(
                column![
                    feather_ui::alinhamento::seletor_horizontal(
                        feather_ui::alinhamento::Horizontal::de_identificador(&formato.alinhamento),
                        true,
                        false,
                        |escolha| Msg::Comando(Comando::Alinhar(match escolha {
                            feather_ui::alinhamento::Horizontal::Esquerda => "esquerda",
                            feather_ui::alinhamento::Horizontal::Centro => "centro",
                            feather_ui::alinhamento::Horizontal::Direita => "direita",
                            feather_ui::alinhamento::Horizontal::Justificado => "justificado",
                        })),
                    ),
                    row![
                        menu("Entrelinha", &["1,0", "1,15", "1,5", "2,0", "2,5", "3,0", "Padrão"],
                            |i| Comando::Entrelinha([1.0, 1.15, 1.5, 2.0, 2.5, 3.0, 0.0][i]), 92.0),
                        menu("Espaço", &["Antes: 0 pt", "Antes: 6 pt", "Antes: 12 pt", "Depois: 0 pt", "Depois: 6 pt", "Depois: 12 pt"],
                            |i| Comando::Espaco(i < 3, [0.0, 6.0, 12.0][i % 3]), 92.0),
                        menu("Bordas", &["Sem borda", "Embaixo", "Em cima", "Esquerda", "Direita", "Contorno"],
                            |i| Comando::Borda([0, 8, 4, 1, 2, 15][i]), 92.0),
                    ]
                    .spacing(4),
                    row![
                        pequeno(
                            "list",
                            "lista",
                            Comando::Estilo("item"),
                            formato.estilo == "item"
                        ),
                        pequeno(
                            "list-numbers",
                            "numeracao",
                            Comando::Estilo("item_ordenado"),
                            formato.estilo == "item_ordenado"
                        ),
                        menu("Tipo de lista", &["Marcador", "Círculo", "Quadrado", "Seta", "1. 2. 3.", "a. b. c.", "i. ii. iii.", "Tarefas", "Reiniciar numeração"],
                            |i| match i {
                                8 => Comando::ListaReiniciar,
                                _ => Comando::Lista(["padrao", "circulo", "quadrado", "seta", "numerica", "alfabetica", "romana", "tarefa"][i]),
                            }, 110.0),
                        so_icone("indent-decrease", "Diminuir recuo (Shift+Tab)", Comando::Recuar(-1), false),
                        so_icone("indent-increase", "Aumentar recuo (Tab)", Comando::Recuar(1), false),
                    ]
                    .spacing(2)
                    .align_y(Alignment::Center)
                ]
                .spacing(7)
                .into(),
                tr("paragrafo")
            ),
            grupo_faixa(
                row![
                    grande(
                        "file-plus",
                        "normal",
                        Comando::Estilo("paragrafo"),
                        formato.estilo == "paragrafo"
                    ),
                    grande(
                        "list-details",
                        "titulo1",
                        Comando::Estilo("titulo1"),
                        formato.estilo == "titulo1"
                    ),
                    grande(
                        "list-details",
                        "titulo2",
                        Comando::Estilo("titulo2"),
                        formato.estilo == "titulo2"
                    ),
                    column![
                        row![
                            botao_pequeno(icone("list-details", 16.), "Título 3", Some(Msg::Comando(Comando::Estilo("titulo3"))), formato.estilo == "titulo3"),
                            botao_pequeno(icone("code", 16.), "Código", Some(Msg::Comando(Comando::Estilo("codigo"))), formato.estilo == "codigo"),
                        ]
                        .spacing(3),
                        botao_pequeno(icone("message-circle", 16.), "Citação", Some(Msg::Comando(Comando::Estilo("citacao"))), formato.estilo == "citacao"),
                        row![
                            estilos_do_documento(app),
                            so_icone("plus", "Novo estilo com a formatação do cursor", Comando::Formulario(Form::NovoEstilo), false),
                            so_icone("refresh", "Atualizar o estilo do cursor para a formatação atual", Comando::EstiloAtualizar, false),
                            so_icone("trash", "Excluir o estilo do cursor", Comando::EstiloExcluir, false),
                        ]
                        .spacing(2)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(3)
                ]
                .into(),
                tr("estilos")
            ),
        ]
        .into(),
        Aba::Inserir => row![
            grupo_faixa(
                row![
                    column![
                        grande("table", "tabela", Comando::Tabela, false),
                        menu("Tamanho", &["2 × 2", "4 × 4", "5 × 5", "6 × 6", "4 × 2", "8 × 4", "10 × 5", "Personalizada…"],
                            |i| match i {
                                7 => Comando::Formulario(Form::Tabela),
                                _ => {
                                    let (l, c) = [(2, 2), (4, 4), (5, 5), (6, 6), (4, 2), (8, 4), (10, 5)][i];
                                    Comando::TabelaTam(l, c)
                                }
                            }, 84.0),
                    ]
                    .spacing(2)
                    .align_x(Alignment::Center),
                    grande("photo", "imagem", Comando::Imagem, false),
                    button(text("Efeitos imagem").size(12))
                        .on_press_maybe(em_imagem.then_some(Msg::AbrirEfeitosImagem))
                        .style(estilo_botao_pequeno(app.efeito_imagem.is_some()))
                ]
                .spacing(6)
                .into(),
                tr("objetos")
            ),
            grupo_faixa(
                column![
                    row![
                        tabela_botao("Colunas", Comando::Grafico(featherdocs_io::grafico_docs::Tipo::Colunas), em_tabela),
                        tabela_botao("Barras", Comando::Grafico(featherdocs_io::grafico_docs::Tipo::Barras), em_tabela),
                        tabela_botao("Linhas", Comando::Grafico(featherdocs_io::grafico_docs::Tipo::Linhas), em_tabela),
                    ].spacing(4),
                    row![
                        tabela_botao("Área", Comando::Grafico(featherdocs_io::grafico_docs::Tipo::Area), em_tabela),
                        tabela_botao("Pizza", Comando::Grafico(featherdocs_io::grafico_docs::Tipo::Pizza), em_tabela),
                        tabela_botao("Dispersão", Comando::Grafico(featherdocs_io::grafico_docs::Tipo::Dispersao), em_tabela),
                    ].spacing(4),
                    row![
                        tabela_botao("Degraus", Comando::Grafico(featherdocs_io::grafico_docs::Tipo::Degraus), em_tabela),
                        tabela_botao("Atualizar selecionado", Comando::AtualizarGrafico, em_grafico),
                    ].spacing(4),
                ].spacing(5).into(),
                "Gráficos (Charts)"
            ),
            grupo_faixa(
                row![
                    grande("page-break", "quebra", Comando::Quebra, false),
                    column![
                        botao_pequeno(icone("equacao", 16.), "Equação", Some(Msg::Comando(Comando::Estilo("equacao"))), false),
                        botao_pequeno(icone("minus", 16.), "Linha horizontal", Some(Msg::Comando(Comando::Estilo("regua"))), false),
                    ]
                    .spacing(3),
                    grande("list-details", "sumario", Comando::Sumario, false)
                ]
                .spacing(6)
                .into(),
                tr("paginas")
            ),
            grupo_faixa(
                row![
                    grande("link", "link", Comando::Formulario(Form::Link), false),
                    column![
                        botao_pequeno(icone("text-box", 16.), "Nota de rodapé", Some(Msg::FormAbrir(Form::NotaRodape)), false),
                        botao_pequeno(icone("text-box", 16.), "Nota de fim", Some(Msg::FormAbrir(Form::NotaFim)), false),
                        botao_pequeno(icone("comentario", 16.), "Comentário", Some(Msg::FormAbrir(Form::Comentario)), false),
                    ]
                    .spacing(3)
                ]
                .spacing(5)
                .into(),
                "Links e notas"
            ),
            grupo_faixa(
                column![
                    row![
                        botao_pequeno(icone("header-footer", 16.), "Cabeçalho", Some(Msg::FormAbrir(Form::Cabecalho)), false),
                        botao_pequeno(icone("header-footer", 16.), "Rodapé", Some(Msg::FormAbrir(Form::Rodape)), false),
                    ]
                    .spacing(3),
                    menu("Número de página", &["Embaixo, centro", "Embaixo, direita", "Embaixo, esquerda", "Em cima, centro", "Em cima, direita", "Em cima, esquerda", "Página X de Y", "Remover número"],
                        |i| match i {
                            6 => Comando::NumeroPagina("Página {pagina} de {total}", "base_centro"),
                            7 => Comando::NumeroRemover,
                            _ => Comando::NumeroPagina("{pagina}", ["base_centro", "base_direita", "base_esquerda", "topo_centro", "topo_direita", "topo_esquerda"][i]),
                        }, 150.0),
                    botao_pequeno(icone("text-box", 16.), "Marca d'água", Some(Msg::FormAbrir(Form::MarcaDagua)), false),
                ]
                .spacing(4)
                .into(),
                "Cabeçalho e rodapé"
            ),
            grupo_faixa(
                column![
                    row![
                        menu("Legenda", &["Figura", "Tabela", "Equação"],
                            |i| Comando::Legenda(["figura", "tabela", "equacao"][i]), 92.0),
                        menu("Sumário", &["Até o nível 2", "Até o nível 3", "Até o nível 4", "Lista de figuras", "Lista de tabelas", "Remover sumários"],
                            |i| match i {
                                3 => Comando::ListaDe(1),
                                4 => Comando::ListaDe(2),
                                5 => Comando::SumarioRemover,
                                _ => Comando::SumarioNiveis(i as u8 + 2),
                            }, 92.0),
                    ]
                    .spacing(4),
                    row![
                        botao_pequeno(icone("tag", 16.), "Referência cruzada", Some(Msg::FormAbrir(Form::Referencia)), false),
                        botao_pequeno(icone("tag", 16.), "Marcador", Some(Msg::FormAbrir(Form::Marcador)), false),
                    ]
                    .spacing(3),
                    row![
                        botao_pequeno(icone("list-details", 16.), "Marcar entrada", Some(Msg::FormAbrir(Form::IndiceEntrada)), false),
                        botao_pequeno(icone("list-details", 16.), "Índice", Some(Msg::Comando(Comando::IndiceInserir)), false),
                        so_icone("refresh", "Atualizar referências", Comando::ReferenciasAtualizar, false),
                    ]
                    .spacing(3),
                ]
                .spacing(4)
                .into(),
                "Referências"
            ),
            grupo_faixa(
                column![
                    row![
                        botao_pequeno(icone("plus", 16.), "Nova fonte", Some(Msg::FormAbrir(Form::NovaFonte)), false),
                        botao_pequeno(icone("file-import", 16.), "Importar .bib", Some(Msg::Comando(Comando::ImportarBib)), false),
                    ]
                    .spacing(3),
                    botao_pequeno(icone("message-circle", 16.), "Inserir citação", Some(Msg::FormAbrir(Form::Citacao)), false),
                    row![
                        menu("Bibliografia", &["Só as citadas", "Todas as fontes"], |i| Comando::Bibliografia(i == 1), 100.0),
                        menu("Estilo", &["ABNT", "APA", "Vancouver", "IEEE", "Chicago"],
                            |i| Comando::EstiloCitacao(["abnt", "apa", "vancouver", "ieee", "chicago"][i]), 92.0),
                    ]
                    .spacing(4),
                ]
                .spacing(4)
                .into(),
                "Citações"
            ),
            grupo_faixa(
                column![
                    row![
                        tabela_botao("Linha +", Comando::TabelaEstrutura("linha+"), em_tabela),
                        tabela_botao("Linha −", Comando::TabelaEstrutura("linha-"), em_tabela),
                    ].spacing(5),
                    row![
                        tabela_botao("Coluna +", Comando::TabelaEstrutura("coluna+"), em_tabela),
                        tabela_botao("Coluna −", Comando::TabelaEstrutura("coluna-"), em_tabela),
                    ].spacing(5),
                    row![
                        tabela_botao("Mesclar →", Comando::TabelaMesclar, em_tabela),
                        tabela_botao("Desmesclar", Comando::TabelaDesmesclar, em_tabela),
                    ].spacing(5),
                ]
                .spacing(5)
                .into(),
                "Editar tabela"
            ),
            grupo_faixa(
                column![
                    tabela_botao(
                        if app.sessao.tabela_tem_cabecalho() { "Cabeçalho ✓" } else { "Cabeçalho" },
                        Comando::TabelaCabecalho,
                        em_tabela,
                    ),
                    tabela_botao("Excluir tabela", Comando::TabelaExcluir, em_tabela),
                ]
                .spacing(5)
                .into(),
                "Tabela"
            )
        ]
        .into(),
        Aba::Layout => row![
            grupo_faixa(
                row![
                    grande(
                        "file-plus",
                        "a4",
                        Comando::Papel("a4"),
                        app.sessao.estado().papel == "a4"
                    ),
                    grande(
                        "file-plus",
                        "carta",
                        Comando::Papel("carta"),
                        app.sessao.estado().papel == "carta"
                    )
                ]
                .into(),
                tr("papel")
            ),
            grupo_faixa(
                row![
                    grande(
                        "file-plus",
                        "retrato",
                        Comando::Orientacao(false),
                        !app.sessao.estado().paisagem
                    ),
                    grande(
                        "file-plus",
                        "paisagem",
                        Comando::Orientacao(true),
                        app.sessao.estado().paisagem
                    )
                ]
                .into(),
                tr("orientacao")
            ),
            grupo_faixa(
                column![
                    menu("Margens", &["Normal (2,54 cm)", "Estreita (1,27 cm)", "Moderada", "Larga", "ABNT (3 e 2 cm)", "Personalizadas…"],
                        |i| match i {
                            0 => Comando::Margens([2.54; 4]),
                            1 => Comando::Margens([1.27; 4]),
                            2 => Comando::Margens([1.91, 1.91, 2.54, 2.54]),
                            3 => Comando::Margens([5.08, 5.08, 2.54, 2.54]),
                            4 => Comando::Margens([3.0, 2.0, 3.0, 2.0]),
                            _ => Comando::Formulario(Form::Margens),
                        }, 130.0),
                    botao_pequeno(icone("square", 16.), "Papel personalizado", Some(Msg::FormAbrir(Form::Papel)), false),
                    menu("Texto na página", &["No topo", "No centro", "Na base"],
                        |i| Comando::Vertical(["topo", "centro", "base"][i]), 130.0),
                ]
                .spacing(4)
                .into(),
                "Configurar página"
            ),
            grupo_faixa(
                column![
                    button(text("Cor da página").size(12))
                        .on_press(Msg::AbrirCor(AlvoCor::Pagina))
                        .style(estilo_botao_pequeno(false)),
                    botao_pequeno(icone("square", 16.), "Borda da página", Some(Msg::FormAbrir(Form::BordaPagina)), false),
                    botao_pequeno(icone("text-box", 16.), "Marca d'água", Some(Msg::FormAbrir(Form::MarcaDagua)), false),
                ]
                .spacing(3)
                .into(),
                "Plano de fundo"
            ),
            grupo_faixa(
                column![
                    menu("Números de linha", &["Nenhum", "Contínuos", "Reiniciar a cada página"],
                        |i| Comando::NumerarLinhas(["nenhuma", "continua", "por_pagina"][i]), 150.0),
                    botao_pequeno(icone("header-footer", 16.), "Primeira página diferente",
                        Some(Msg::Comando(Comando::PrimeiraDiferente)), app.sessao.documento().extras.primeira_diferente),
                ]
                .spacing(4)
                .into(),
                "Linhas e capa"
            )
        ]
        .into(),
        Aba::Revisao => row![
            grupo_faixa(
                row![grande("search", "localizar", Comando::Buscar, false)].into(),
                tr("busca")
            ),
            grupo_faixa(
                row![
                    grande(
                        "file-plus",
                        "editar",
                        Comando::Modo("editando"),
                        app.sessao.modo_edicao() == "editando"
                    ),
                    grande(
                        "file-plus",
                        "sugerir",
                        Comando::Modo("sugerindo"),
                        app.sessao.modo_edicao() == "sugerindo"
                    ),
                    grande(
                        "file-plus",
                        "leitura",
                        Comando::Modo("leitura"),
                        app.sessao.modo_edicao() == "leitura"
                    )
                ]
                .into(),
                tr("modo")
            ),
            grupo_faixa(
                row![
                    botao_grande(icone("arrow-forward-up", 24.), "Aceitar", Some(Msg::Comando(Comando::Revisoes(true, false))), false),
                    botao_grande(icone("arrow-back-up", 24.), "Rejeitar", Some(Msg::Comando(Comando::Revisoes(false, false))), false),
                    column![
                        botao_pequeno(icone("checks", 16.), "Aceitar todas", Some(Msg::Comando(Comando::Revisoes(true, true))), false),
                        botao_pequeno(icone("x", 16.), "Rejeitar todas", Some(Msg::Comando(Comando::Revisoes(false, true))), false),
                        botao_pequeno(icone("columns", 16.), "Comparar com arquivo", Some(Msg::Comando(Comando::Comparar)), false),
                    ]
                    .spacing(3)
                ]
                .into(),
                tr("alteracoes")
            ),
            grupo_faixa(
                row![
                    grande("comentario", "novo_comentario", Comando::Formulario(Form::Comentario), false),
                    grande("layout-sidebar", "painel_revisao", Comando::PainelRevisao, app.painel_revisao()),
                ]
                .into(),
                "Comentários"
            ),
            grupo_faixa(
                row![
                    grande("lock", "proteger", Comando::Proteger, app.sessao.documento().extras.protegido),
                ]
                .into(),
                "Proteger"
            )
        ]
        .into(),
        Aba::Exibir => row![
            grupo_faixa(
                row![
                    grande(
                        "layout-sidebar",
                        "estrutura",
                        Comando::Estrutura,
                        app.config.estrutura
                    ),
                    grande(
                        "list-details",
                        "marcas",
                        Comando::Marcas,
                        app.sessao.estado().marcas
                    ),
                    grande(
                        "code",
                        "codigo_latex",
                        Comando::Codigo(Fonte::Latex),
                        app.fonte_do_codigo() == Some(Fonte::Latex)
                    ),
                    grande(
                        "file-code",
                        "codigo_markdown",
                        Comando::Codigo(Fonte::Markdown),
                        app.fonte_do_codigo() == Some(Fonte::Markdown)
                    )
                ]
                .into(),
                tr("paineis")
            ),
            grupo_faixa(
                row![
                    grande("zoom-out", "reduzir", Comando::Zoom(-0.1), false),
                    grande("zoom-in", "ampliar", Comando::Zoom(0.1), false)
                ]
                .into(),
                tr("zoom")
            ),
            grupo_faixa(
                row![grande(
                    "settings",
                    "configuracoes",
                    Comando::Configuracoes,
                    false
                )]
                .into(),
                tr("preferencias")
            )
        ]
        .into(),
    };
    column![
        abas,
        // A barra de rolagem ocupa a própria faixa de 6 px: não cobre as legendas dos grupos.
        scrollable(iced::widget::container(conteudo).height(106))
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::new().width(6).scroller_width(6).spacing(2)
            ))
    ]
    .spacing(0)
    .width(Length::Fill)
    .align_x(Alignment::Start)
    .into()
}
