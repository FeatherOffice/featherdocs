//! Estado e eventos da aplicação. Regras de edição ficam no featherdocs-core.
use crate::{
    arquivos,
    ajuda,
    configuracao::Configuracao,
    cor_ui::{AlvoCor, EstadoCor},
    faixa::{self, Aba},
    localidade::tr,
    pagina::Pagina,
    painel_codigo::{self, PainelCodigo},
    quadro::{Operacao, Quadro},
};
use featherdocs_io::dividido::Fonte;
use feather_ui::{icones::icone, tema::*, janelas::Janelas};
use featherdocs_core::{busca::Opcoes, geometria::Visao, modelo::Documento, sessao::Sessao};
use iced::widget::{
    button, canvas, column, container, row, scrollable, text, text_input, toggler, Space,
};
use iced::{keyboard, window, Alignment, Element, Length, Subscription, Task};
use std::{collections::HashMap, path::PathBuf, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JanelaAux { Configuracoes, Ajuda, Monitor }


#[derive(Clone, Debug)]
pub enum Comando {
    Novo,
    Abrir,
    Salvar,
    SalvarComo,
    Exportar,
    Desfazer,
    Refazer,
    Copiar,
    Recortar,
    Colar,
    Pincel,
    ColarSemFormatacao,
    Formato(u8),
    /// Sobrescrito (1) ou subscrito (-1); repetir desliga.
    Posicao(i8),
    TamanhoPasso(i32),
    LimparFormatacao,
    Maiusculas(&'static str),
    Recuar(i32),
    Lista(&'static str),
    ListaReiniciar,
    Entrelinha(f32),
    /// Espaço antes (`true`) ou depois do parágrafo, em pontos; o outro lado fica como está.
    Espaco(bool, f32),
    Borda(u8),
    /// Número de página: modelo e lugar (`base_centro`, `topo_direita`…).
    NumeroPagina(&'static str, &'static str),
    NumeroRemover,
    Legenda(&'static str),
    IndiceInserir,
    /// Lista de figuras (1) ou de tabelas (2).
    ListaDe(u8),
    SumarioRemover,
    SumarioNiveis(u8),
    ReferenciasAtualizar,
    /// Margens em cm: esquerda, direita, superior, inferior.
    Margens([f32; 4]),
    NumerarLinhas(&'static str),
    PrimeiraDiferente,
    Vertical(&'static str),
    Proteger,
    Formulario(crate::dialogos::Tipo),
    ImportarBib,
    Bibliografia(bool),
    EstiloCitacao(&'static str),
    Estilo(&'static str),
    EstiloNome(u32),
    /// Atualiza o estilo com nome do cursor para a formatação atual dele.
    EstiloAtualizar,
    EstiloExcluir,
    Alinhar(&'static str),
    Tabela,
    TabelaTam(usize, usize),
    Grafico(featherdocs_io::grafico_docs::Tipo),
    AtualizarGrafico,
    TabelaEstrutura(&'static str),
    TabelaMesclar,
    TabelaDesmesclar,
    TabelaCabecalho,
    TabelaExcluir,
    Imagem,
    Quebra,
    Sumario,
    Papel(&'static str),
    Orientacao(bool),
    Modo(&'static str),
    /// Aceitar (`true`) ou rejeitar; o segundo diz se é o documento todo ou só a seleção/cursor.
    Revisoes(bool, bool),
    PainelRevisao,
    Comparar,
    Estrutura,
    Marcas,
    /// Liga o painel de código nesta língua; de novo na mesma, desliga.
    Codigo(Fonte),
    Zoom(f32),
    Configuracoes,
    Buscar,
    Menu,
    Ajuda,
    Monitor,
}
#[derive(Clone, Debug)]
pub enum Destino {
    Novo,
    Abrir,
    /// Um arquivo já escolhido: recentes ou arrastado para a janela.
    AbrirCaminho(PathBuf),
    Fechar(window::Id),
}
#[derive(Clone, Debug)]
pub enum Msg {
    Atalho(Comando),
    Comando(Comando),
    Aba(Aba),
    Fonte(String),
    Tamanho(u16),
    AbrirCor(AlvoCor),
    EventoCor(feather_ui::seletor_cor::Evento),
    AjudaEvento(feather_ui::ajuda::Evento),
    Tecla(keyboard::Key, keyboard::Modifiers, Option<String>),
    Clicar(usize, f32, f32, bool, u8),
    IrBloco(u64),
    Colado(Option<String>),
    ColadoPuro(Option<String>),
    FormAbrir(crate::dialogos::Tipo),
    AbrirRecente(PathBuf),
    BibLido(Result<Option<String>, String>),
    /// Botão direito na página: página, x, y no papel e o ponto do mouse (no conteúdo rolado).
    BotaoDireito(usize, f32, f32, iced::Point),
    CtrlClique(usize, f32, f32),
    AbrirLink,
    /// Ação escolhida no menu do botão direito (fecha o menu e executa).
    DoMenu(Box<Msg>),
    FecharContexto,
    Rolou(iced::Vector),
    Comparado(Result<Option<(PathBuf, Documento)>, String>),
    ComentarioIr(u64),
    ComentarioResolver(u64, bool),
    ComentarioExcluir(u64),
    FormCampo(usize, String),
    FormOk,
    FormFechar,
    FormRemoverLink,
    /// Referência cruzada: o alvo escolhido.
    FormEscolher(u64),
    Aberto(Result<Option<(PathBuf, Documento)>, String>),
    Gravado(Result<Option<(PathBuf, u64, bool)>, String>),
    Imagem(Result<Option<(String, f32, f32)>, String>),
    AbrirEfeitosImagem,
    AplicarEfeitoImagem(&'static str),
    FecharEfeitosImagem,
    Fechar(window::Id),
    FecharAuxiliar(JanelaAux),
    JanelaAberta(window::Id),
    JanelaFechada(window::Id),
    ArrastarJanela(window::Id),
    Minimizar(window::Id),
    AlternarMaximizar(window::Id),
    AtalhoJanela(window::Id, keyboard::Key, keyboard::Modifiers),
    BarraPropria(bool),
    Descartar,
    SalvarEContinuar,
    Cancelar,
    Escuro(bool),
    TemaAuto,
    Acento(usize),
    LogoPadrao,
    ConfigSecao(u8),
    AlternarFaixa,
    Consulta(String),
    Substituicao(String),
    /// Liga/desliga uma opção do Localizar: 0 maiúsculas, 1 palavra inteira, 2 expressão regular.
    BuscaOpcao(u8),
    BuscaIr(i32),
    Substituir(bool),
    /// Ação no editor do painel de código (digitar, clicar, rolar).
    CodigoEditar(iced::widget::text_editor::Action),
    /// Pulso enquanto há código digitado à espera da pausa.
    CodigoPulso,
    CodigoFechar,
    /// Modo de captura (`FEATHERDOCS_CAPTURAR=arquivo.png`): prepara e fotografa.
    CapturarTela,
    TelaCapturada(window::Screenshot),
}
pub struct App {
    pub sessao: Sessao,
    pub config: Configuracao,
    pub aba: Aba,
    pub visao: Visao,
    quadros: Vec<Quadro>,
    imagens: HashMap<String, iced::widget::image::Handle>,
    caminho: Option<PathBuf>,
    status: String,
    foco_documento: bool,
    ocupado: bool,
    pagina_ativa: usize,
    janelas: Janelas<JanelaAux>,
    ultimo_clique_barra: Option<Instant>,
    busca: bool,
    consulta: String,
    substituicao: String,
    pendente: Option<Destino>,
    continuar_apos_salvar: bool,
    menu_aberto: bool,
    cor_ui: EstadoCor,
    /// Bloco alvo, bytes originais e preset durante a edição de imagem.
    pub(crate) efeito_imagem: Option<(u64, String, &'static str)>,
    config_secao: u8,
    ajuda_estado: feather_ui::ajuda::Estado,
    /// O painel de código LaTeX/Markdown (modo dividido), quando aberto.
    codigo: Option<PainelCodigo>,
    /// O texto que a pessoa escreveu (o `.tex`/`.md` aberto ou o último código aplicado): o
    /// painel o mostra na forma dela enquanto ele ainda descrever o documento.
    fonte_original: Option<(Fonte, String)>,
    passo_captura: u8,
    /// O diálogo de formulário aberto (link, nota, cabeçalho, margens…).
    formulario: Option<crate::dialogos::Formulario>,
    /// Painel lateral de revisão (estatísticas, comentários, alterações).
    painel_revisao: bool,
    /// Menu do botão direito aberto, no ponto da janela.
    menu_contexto: Option<iced::Point>,
    /// Quanto a página está rolada: o canvas recebe o mouse em coordenadas do conteúdo.
    rolagem: iced::Vector,
    busca_opcoes: Opcoes,
    /// A nota sob o botão direito, para o item "Editar nota" do menu.
    nota_contexto: Option<crate::dialogos::Tipo>,
}
impl Default for App {
    fn default() -> Self {
        let sessao = Sessao::default();
        let visao = sessao.visao(0, 0);
        let mut app = Self {
            sessao,
            visao,
            config: Configuracao::default(),
            aba: Aba::Inicio,
            quadros: vec![],
            imagens: HashMap::new(),
            caminho: None,
            status: String::new(),
            foco_documento: true,
            ocupado: false,
            pagina_ativa: 0,
            janelas: Janelas::default(),
            ultimo_clique_barra: None,
            busca: false,
            consulta: String::new(),
            substituicao: String::new(),
            pendente: None,
            continuar_apos_salvar: false,
            menu_aberto: false,
            cor_ui: EstadoCor::default(),
            efeito_imagem: None,
            config_secao: 0,
            ajuda_estado: Default::default(),
            codigo: None,
            fonte_original: None,
            passo_captura: 0,
            formulario: None,
            painel_revisao: false,
            menu_contexto: None,
            rolagem: iced::Vector::ZERO,
            busca_opcoes: Opcoes::default(),
            nota_contexto: None,
        };
        app.atualizar();
        app
    }
}
impl App {
    pub fn novo() -> (Self, Task<Msg>) {
        let mut app = Self {
            config: Configuracao::carregar(),
            ..Self::default()
        };
        definir_tema(escuro_resolvido(app.config.tema_auto, app.config.escuro), app.config.acento);
        feather_ui::marca::definir_marca(feather_ui::marca::Produto::FeatherDocs,
            app.config.logo_usa_acento());
        // Nunca fazer parse DOCX/FDOZ antes de criar a janela Iced.
        let inicial = std::env::args_os().nth(1).map(PathBuf::from);
        if inicial.is_some() {
            app.ocupado = true;
            app.status = "Abrindo documento em segundo plano…".into();
        }
        let (id, abrir) = window::open(window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            decorations: !app.config.barra_propria,
            exit_on_close_request: false,
            ..Default::default()
        });
        app.janelas.principal = Some(id);
        let janela = abrir.map(Msg::JanelaAberta);
        let tarefas = if let Some(p) = inicial {
            Task::batch([
                janela,
                Task::perform(
                    async move { Self::carregar_em_background(p).await.map(Some) },
                    Msg::Aberto,
                ),
            ])
        } else {
            janela
        };
        (app, tarefas)
    }

    /// Carregamento fora da thread Iced, inclusive abertura via CLI.
    async fn carregar_em_background(p: PathBuf) -> Result<(PathBuf, Documento), String> {
        tokio::task::spawn_blocking(move || arquivos::ler(&p).map(|doc| (p, doc)))
            .await
            .map_err(|e| format!("Falha ao processar documento: {e}"))?
    }
    fn alternar_aux(&mut self, tipo: JanelaAux) -> Task<Msg> {
        let tamanho = match tipo {
            JanelaAux::Configuracoes => iced::Size::new(900.0, 640.0),
            JanelaAux::Ajuda => iced::Size::new(980.0, 700.0),
            JanelaAux::Monitor => iced::Size::new(980.0, 640.0),
        };
        self.janelas.alternar(tipo, tamanho, self.config.barra_propria).map(Msg::JanelaAberta)
    }
    fn acoes_janela(id: window::Id) -> feather_ui::cromo::AcoesJanela<Msg> {
        feather_ui::cromo::AcoesJanela {
            arrastar: Msg::ArrastarJanela(id),
            minimizar: Msg::Minimizar(id),
            maximizar: Msg::AlternarMaximizar(id),
            fechar: Msg::Fechar(id),
        }
    }
    pub fn painel_revisao(&self) -> bool {
        self.painel_revisao
    }
    pub fn titulo(&self, janela: window::Id) -> String {
        match self.janelas.tipo(janela) {
            Some(JanelaAux::Configuracoes) => return "Configurações do FeatherDocs".into(),
            Some(JanelaAux::Ajuda) => return "Ajuda do FeatherDocs".into(),
            Some(JanelaAux::Monitor) => return "Monitor do FeatherDocs".into(),
            None => {}
        }
        let nome = self
            .caminho
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| tr("sem_titulo").into());
        format!(
            "{}{} — FeatherDocs",
            if self.sessao.estado().sujo { "* " } else { "" },
            nome
        )
    }
    pub fn tema(&self, _janela: window::Id) -> iced::Theme {
        tema_iced()
    }
    fn atualizar(&mut self) {
        let total = self.sessao.total_de_paginas();
        self.visao = self.sessao.visao(0, total.saturating_sub(1));
        self.pagina_ativa = self
            .visao
            .cursor
            .map_or(self.pagina_ativa, |c| c.pagina)
            .min(total.saturating_sub(1));
        self.quadros.clear();
        for p in 0..total {
            match self
                .sessao
                .quadro(p, u64::MAX)
                .and_then(|b| Quadro::ler(&b))
            {
                Ok(q) => {
                    for o in &q.operacoes {
                        if let Operacao::Imagem { origem, .. } = o {
                            if !self.imagens.contains_key(origem) {
                                if let Some(h) = crate::pagina::carregar_imagem(origem) {
                                    self.imagens.insert(origem.clone(), h);
                                }
                            }
                        }
                    }
                    self.quadros.push(q);
                }
                Err(e) => self.status = e,
            }
        }
        self.sincronizar_codigo();
    }
    fn carregar(&mut self, p: PathBuf, d: Documento) {
        self.config.lembrar(&p);
        self.sessao = Sessao::com_documento(d);
        self.sessao.marcar_gravado();
        // O texto de um .tex/.md aberto aparece no painel de código como a pessoa o escreveu.
        let ext = p.extension().and_then(|s| s.to_str()).map(str::to_ascii_lowercase);
        let fonte = match ext.as_deref() {
            Some("tex") => Some(Fonte::Latex),
            Some("md") => Some(Fonte::Markdown),
            _ => None,
        };
        self.fonte_original = fonte
            .and_then(|f| std::fs::read_to_string(&p).ok().map(|t| (f, t)));
        if let (Some(f), Some(painel)) = (fonte, self.codigo.as_mut()) {
            painel.fonte = f;
        }
        if let Some(painel) = self.codigo.as_mut() {
            painel.ultima_edicao = None;
            painel.versao = u64::MAX;
        }
        // Um documento importado é salvo como .fdoz, preservando o arquivo de origem.
        self.caminho = (ext.as_deref() == Some("fdoz")).then_some(p);
        self.imagens.clear();
        self.efeito_imagem = None;
        self.foco_documento = true;
        self.busca = false;
        self.sessao.busca_limpar();
        self.status.clear();
        self.atualizar();
    }
    fn solicitar(&mut self, d: Destino) -> Task<Msg> {
        if self.ocupado {
            return Task::none();
        }
        if self.sessao.estado().sujo {
            self.pendente = Some(d);
            self.foco_documento = false;
            Task::none()
        } else {
            self.executar_destino(d)
        }
    }
    fn executar_destino(&mut self, d: Destino) -> Task<Msg> {
        self.pendente = None;
        self.continuar_apos_salvar = false;
        match d {
            Destino::Novo => {
                self.sessao = Sessao::default();
                self.caminho = None;
                self.fonte_original = None;
                if let Some(painel) = self.codigo.as_mut() {
                    painel.ultima_edicao = None;
                    painel.versao = u64::MAX;
                }
                self.imagens.clear();
                self.busca = false;
                self.foco_documento = true;
                self.status.clear();
                self.atualizar();
                Task::none()
            }
            Destino::Fechar(_id) => iced::exit(),
            Destino::AbrirCaminho(p) => {
                self.ocupado = true;
                Task::perform(async move { Self::carregar_em_background(p).await.map(Some) }, Msg::Aberto)
            }
            Destino::Abrir => {
                self.ocupado = true;
                Task::perform(
                    async {
                        let p = rfd::AsyncFileDialog::new()
                            .add_filter("Documentos", &["fdoz", "docx", "md", "tex"])
                            .pick_file()
                            .await;
                        match p {
                            Some(p) => Self::carregar_em_background(p.path().to_path_buf())
                                .await
                                .map(Some),
                            None => Ok(None),
                        }
                    },
                    Msg::Aberto,
                )
            }
        }
    }
    fn salvar(&mut self, como: bool, exportar: bool) -> Task<Msg> {
        if self.ocupado {
            return Task::none();
        }
        self.ocupado = true;
        let p = if como || exportar {
            None
        } else {
            self.caminho.clone()
        };
        let doc = self.sessao.documento().clone();
        let versao = self.sessao.versao();
        // O PDF sai das páginas já compostas (as mesmas da tela), não do documento.
        let quadros = if exportar { self.quadros.clone() } else { Vec::new() };
        Task::perform(
            async move {
                let p = if let Some(p) = p {
                    Some(p)
                } else {
                    let dialogo = rfd::AsyncFileDialog::new().set_file_name(if exportar {
                        "documento.docx"
                    } else {
                        "documento.fdoz"
                    });
                    let dialogo = if exportar {
                        dialogo
                            .add_filter("Word", &["docx"])
                            .add_filter("PDF", &["pdf"])
                            .add_filter("Markdown", &["md"])
                            .add_filter("LaTeX", &["tex"])
                    } else {
                        dialogo.add_filter("FeatherDocs", &["fdoz"])
                    };
                    dialogo.save_file().await.map(|f| f.path().to_path_buf())
                };
                let Some(mut p) = p else {
                    return Ok(None);
                };
                if p.extension().is_none() {
                    p.set_extension(if exportar { "docx" } else { "fdoz" });
                    if p.exists() {
                        return Err("O arquivo já existe. Selecione seu nome completo no diálogo para confirmar a substituição.".into());
                    }
                }
                if !exportar
                    && !p
                        .extension()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| s.eq_ignore_ascii_case("fdoz"))
                {
                    return Err("Para salvar o documento, escolha um arquivo .fdoz. Use Exportar para outros formatos.".into());
                }
                // Serialização e escrita podem durar segundos; executar em worker.
                tokio::task::spawn_blocking(move || {
                    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("docx");
                    let b = if ext.eq_ignore_ascii_case("pdf") {
                        crate::pdf::gerar(&quadros)?
                    } else {
                        arquivos::bytes(&doc, ext)?
                    };
                    arquivos::gravar(&p, &b)?;
                    Ok::<_, String>(Some((p, versao, exportar)))
                })
                .await
                .map_err(|e| format!("Falha ao salvar documento: {e}"))?
            },
            Msg::Gravado,
        )
    }
    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        if (self.ocupado || self.pendente.is_some())
            && matches!(
                msg,
                Msg::Clicar(..)
                    | Msg::Tecla(..)
                    | Msg::Colado(..)
                    | Msg::ColadoPuro(..)
                    | Msg::BotaoDireito(..)
                    | Msg::CtrlClique(..)
                    | Msg::Fonte(..)
                    | Msg::Tamanho(..)
                    | Msg::AbrirCor(..)
                    | Msg::EventoCor(..)
                    | Msg::AbrirEfeitosImagem
                    | Msg::AplicarEfeitoImagem(..)
                    | Msg::IrBloco(..)
                    | Msg::Consulta(..)
                    | Msg::Substituicao(..)
                    | Msg::BuscaIr(..)
                    | Msg::Substituir(..)
            )
        {
            return Task::none();
        }
        match msg {
            Msg::CodigoEditar(acao) => return self.codigo_editar(acao),
            Msg::CodigoPulso => {
                if !self.codigo.as_ref().is_some_and(|p| p.maduro(Instant::now())) {
                    return Task::none();
                }
                self.aplicar_codigo();
            }
            Msg::CapturarTela => return self.capturar_tela(),
            Msg::TelaCapturada(s) => {
                if let Some(caminho) = std::env::var_os("FEATHERDOCS_CAPTURAR") {
                    if let Err(e) = image::save_buffer(&caminho, &s.bytes, s.size.width,
                        s.size.height, image::ExtendedColorType::Rgba8) {
                        eprintln!("Captura não gravada: {e}");
                    }
                }
                return iced::exit();
            }
            // Qualquer outra ação aplica antes o código que ainda esperava a pausa: a página
            // nunca é editada por cima de um código que vai substituí-la depois.
            _ => self.aplicar_codigo(),
        }
        let mut tarefa = Task::none();
        match msg {
            Msg::Comando(c) => return self.comando(c),
            Msg::CodigoFechar => {
                self.codigo = None;
                self.foco_documento = true;
            }
            Msg::Atalho(c) => {
                if matches!(c, Comando::Copiar | Comando::Recortar | Comando::Colar | Comando::ColarSemFormatacao)
                    && !self.foco_documento
                {
                    return Task::none();
                }
                return self.comando(c);
            }
            Msg::Fechar(id) => {
                if Some(id) == self.janelas.principal {
                    return self.solicitar(Destino::Fechar(id));
                }
                self.janelas.retirar(id);
                return window::close(id);
            }
            Msg::FecharAuxiliar(tipo) => return self.janelas.fechar_tipo(tipo),
            Msg::JanelaAberta(_) => {}
            Msg::JanelaFechada(id) => {
                if Some(id) == self.janelas.principal { return iced::exit(); }
                self.janelas.retirar(id);
            }
            Msg::ArrastarJanela(id) => return feather_ui::janelas::arrastar_ou_maximizar(id, &mut self.ultimo_clique_barra),
            Msg::Minimizar(id) => return window::minimize(id, true),
            Msg::AlternarMaximizar(id) => return window::toggle_maximize(id),
            Msg::AtalhoJanela(id, key, mods) => {
                if let Some(tipo) = self.janelas.tipo(id) {
                    // A Ajuda TS usava Ctrl+F para focalizar a pesquisa.
                    if tipo == JanelaAux::Ajuda && mods == keyboard::Modifiers::CTRL
                        && matches!(&key, keyboard::Key::Character(c)
                            if c.as_str().eq_ignore_ascii_case("f")) {
                        return feather_ui::ajuda::focar_busca::<Msg>();
                    }
                    use feather_ui::atalhos::PainelGlobal;
                    let painel = match tipo {
                        JanelaAux::Ajuda => PainelGlobal::Ajuda,
                        JanelaAux::Monitor => PainelGlobal::Monitor,
                        JanelaAux::Configuracoes => PainelGlobal::Configuracoes,
                    };
                    if feather_ui::atalhos::fechar_painel(painel, &key, mods) {
                        return self.janelas.fechar_tipo(tipo);
                    }
                    return Task::none();
                }
                if let Some(acao) = feather_ui::atalhos::acao_global(&key, mods) {
                    use feather_ui::atalhos::AcaoGlobal;
                    return match acao {
                        AcaoGlobal::Ajuda => self.comando(Comando::Ajuda),
                        AcaoGlobal::Monitor => self.comando(Comando::Monitor),
                        AcaoGlobal::AlternarFaixa => self.update(Msg::AlternarFaixa),
                    };
                }
                return match feather_ui::atalhos::combo_de(&key, mods).as_deref() {
                    Some("Escape") => self.update(Msg::Cancelar),
                    Some("Ctrl+N") => self.comando(Comando::Novo),
                    Some("Ctrl+O") => self.comando(Comando::Abrir),
                    Some("Ctrl+Shift+S") => self.comando(Comando::SalvarComo),
                    Some("Ctrl+S") => self.comando(Comando::Salvar),
                    Some("Ctrl+C") => self.update(Msg::Atalho(Comando::Copiar)),
                    Some("Ctrl+X") => self.update(Msg::Atalho(Comando::Recortar)),
                    Some("Ctrl+V") => self.update(Msg::Atalho(Comando::Colar)),
                    Some("Ctrl+Shift+V") => self.update(Msg::Atalho(Comando::ColarSemFormatacao)),
                    Some("Ctrl+F") => self.comando(Comando::Buscar),
                    Some("Ctrl+K") => self.update(Msg::FormAbrir(crate::dialogos::Tipo::Link)),
                    Some("Ctrl+,") => self.comando(Comando::Menu),
                    _ => Task::none(),
                };
            }
            Msg::BarraPropria(valor) => {
                self.config.barra_propria = valor;
                self.config.gravar();
            }
            Msg::SalvarEContinuar => {
                self.continuar_apos_salvar = true;
                return self.salvar(false, false);
            }
            Msg::Descartar => {
                if let Some(d) = self.pendente.take() {
                    return self.executar_destino(d);
                }
            }
            Msg::Cancelar => {
                if self.sessao.pincel_cheio() {
                    self.sessao.pincel_cancelar();
                    self.status = "Pincel cancelado".into();
                }
                self.pendente = None;
                self.continuar_apos_salvar = false;
                self.menu_aberto = false;
                self.cor_ui.alvo = None;
                self.efeito_imagem = None;
                self.formulario = None;
                self.menu_contexto = None;
                self.busca = false;
                self.sessao.busca_limpar();
                self.foco_documento = true;
            }
            Msg::Aba(a) => self.aba = a,
            Msg::Escuro(b) => {
                self.config.tema_auto = false;
                self.config.escuro = b;
                definir_tema(b, self.config.acento);
                self.config.gravar();
            }
            Msg::TemaAuto => {
                self.config.tema_auto = true;
                definir_tema(escuro_resolvido(true, self.config.escuro), self.config.acento);
                self.config.gravar();
            }
            Msg::Acento(i) => {
                self.config.acento = i.min(ACENTOS.len() - 1);
                self.config.marca_personalizada = Some(true);
                definir_tema(escuro_resolvido(self.config.tema_auto, self.config.escuro), self.config.acento);
                feather_ui::marca::definir_marca(feather_ui::marca::Produto::FeatherDocs,true);
                self.config.gravar();
            }
            Msg::LogoPadrao => {
                self.config.marca_personalizada = Some(false);
                feather_ui::marca::definir_marca(feather_ui::marca::Produto::FeatherDocs,false);
                self.config.gravar();
            }
            Msg::ConfigSecao(s) => self.config_secao = s.min(5),
            Msg::AjudaEvento(e) => self.ajuda_estado.atualizar(e, ajuda::modelo()),
            Msg::AlternarFaixa => {
                self.config.faixa_recolhida = !self.config.faixa_recolhida;
                self.config.gravar();
            },
            Msg::Aberto(r) => {
                self.ocupado = false;
                match r {
                    Ok(Some((p, d))) => self.carregar(p, d),
                    Ok(None) => {
                        self.status = "Abertura cancelada; documento preservado".into();
                    }
                    Err(e) => self.status = e,
                }
            }
            Msg::Gravado(r) => {
                self.ocupado = false;
                match r {
                    Ok(Some((p, v, exportar))) => {
                        let pdf = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
                        if !exportar {
                            self.config.lembrar(&p);
                            self.caminho = Some(p);
                            if self.sessao.versao() == v {
                                self.sessao.marcar_gravado();
                            }
                        }
                        self.status = tr(if exportar { "exportado" } else { "salvo" }).into();
                        if exportar && pdf {
                            if let Some(a) = crate::pdf::aviso(&self.quadros) {
                                self.status = a;
                            }
                        }
                        if self.continuar_apos_salvar && !self.sessao.estado().sujo {
                            if let Some(d) = self.pendente.take() {
                                return self.executar_destino(d);
                            }
                        }
                    }
                    Ok(None) => self.continuar_apos_salvar = false,
                    Err(e) => {
                        self.status = e;
                        self.continuar_apos_salvar = false;
                    }
                }
            }
            Msg::AbrirEfeitosImagem => {
                if self.efeito_imagem.is_some() {
                    self.efeito_imagem = None;
                } else {
                    let id = self.sessao.cursor().foco.bloco;
                    let origem = self.sessao.documento().blocos.iter()
                        .find(|b| b.id == id)
                        .and_then(|b| match &b.conteudo {
                            featherdocs_core::modelo::Conteudo::Imagem { origem, .. }
                                if origem.starts_with("data:image/png;base64,")
                                || origem.starts_with("data:image/jpeg;base64,") => Some(origem.clone()),
                            _ => None,
                        });
                    match origem {
                        Some(origem) => self.efeito_imagem = Some((id, origem, "original")),
                        None => self.status = "Selecione uma imagem PNG/JPEG embutida para editar.".into(),
                    }
                }
            }
            Msg::AplicarEfeitoImagem(chave) => {
                if let Some((id, origem, _)) = self.efeito_imagem.clone() {
                    let result = feather_image::EfeitoRapido::da_chave(chave)
                        .ok_or_else(|| "Efeito de imagem desconhecido.".to_string())
                        .and_then(|preset|
                            crate::imagem_compartilhada::aplicar_efeito_embutido(&origem, preset)
                        )
                        .and_then(|nova| self.sessao.imagem_origem_atualizar(id, &nova).map(|_| ()));
                    match result {
                        Ok(()) => {
                            self.imagens.clear();
                            self.efeito_imagem = Some((id, origem, chave));
                            self.status = format!("Efeito de imagem aplicado: {chave}");
                        }
                        Err(erro) => self.status = erro,
                    }
                }
            }
            Msg::FecharEfeitosImagem => self.efeito_imagem = None,
            Msg::Imagem(r) => {
                self.ocupado = false;
                match r {
                    Ok(Some((origem, w, h))) => {
                        if let Err(e) = self.sessao.inserir_imagem(&origem, w, h) {
                            self.status = e;
                        }
                    }
                    Ok(None) => {}
                    Err(e) => self.status = e,
                }
            }
            Msg::Clicar(p, x, y, shift, n) if !self.ocupado && self.pendente.is_none() => {
                self.foco_documento = true;
                // A página manda: o cursor do código vai ao bloco clicado (em `atualizar`).
                if let Some(painel) = self.codigo.as_mut() {
                    painel.bloco_marcado = None;
                }
                // Duplo clique numa nota abre o texto dela para editar.
                if !shift && n == 2 {
                    if let Some(tipo) = self.nota_em(p, x, y) {
                        return self.update(Msg::FormAbrir(tipo));
                    }
                }
                // Clique simples na caixa de um item de tarefa marca/desmarca, sem mover o cursor.
                if shift || n != 1 || self.sessao.clicar_tarefa(p, x, y).is_none() {
                    self.sessao.clicar(p, x, y, shift, n);
                }
                self.pagina_ativa = p;
                if shift && self.sessao.pincel_cheio() && !self.sessao.cursor().vazio() {
                    self.sessao.pincel_aplicar();
                    self.sessao.pincel_cancelar();
                    self.status = "Pincel aplicado à seleção".into();
                }
            }
            Msg::Tecla(k, m, t)
                if self.foco_documento && !self.ocupado && self.pendente.is_none() =>
            {
                if self.sessao.pincel_cheio()
                    && k == keyboard::Key::Named(keyboard::key::Named::Escape)
                {
                    self.sessao.pincel_cancelar();
                    self.status = "Pincel cancelado".into();
                    return Task::none();
                }
                let nome = nome_tecla(&k);
                let r = self.sessao.tecla(&nome, m.control(), m.shift(), m.alt());
                if !r.tratada && (!m.control() || m.alt()) {
                    if let Some(t) = t.filter(|t| !t.chars().any(char::is_control)) {
                        self.sessao.inserir_texto(&t);
                    }
                }
                tarefa = self.rolar_ao_cursor();
            }
            Msg::Colado(Some(t)) if !self.ocupado && self.pendente.is_none() => {
                self.sessao.colar(&t);
            }
            Msg::Comparado(r) => {
                self.ocupado = false;
                match r {
                    Ok(Some((p, antigo))) => {
                        let rotulo = p.file_name().and_then(|n| n.to_str()).unwrap_or("outro arquivo").to_string();
                        self.sessao.comparar_com(&antigo, &rotulo);
                        self.painel_revisao = true;
                        self.status = format!("Diferenças em relação a {rotulo} marcadas como alterações");
                    }
                    Ok(None) => {}
                    Err(e) => self.status = e,
                }
            }
            Msg::ComentarioIr(id) => {
                self.sessao.comentario_ir(id);
                self.foco_documento = true;
                tarefa = self.rolar_ao_cursor();
            }
            Msg::ComentarioResolver(id, sim) => {
                if let Err(e) = self.sessao.comentario_mudar(id, None, None, Some(sim)) {
                    self.status = e;
                }
            }
            Msg::ComentarioExcluir(id) => {
                self.sessao.comentario_excluir(id);
            }
            Msg::Rolou(v) => self.rolagem = v,
            Msg::BotaoDireito(p, x, y, abs) => {
                self.foco_documento = true;
                self.pagina_ativa = p;
                self.nota_contexto = self.nota_em(p, x, y);
                self.sessao.clicar_direito(p, x, y);
                self.menu_contexto = Some(abs - self.rolagem);
            }
            Msg::FecharContexto => self.menu_contexto = None,
            Msg::AbrirLink => {
                if let Some(link) = self.sessao.link_no_cursor() {
                    self.seguir_link(&link);
                }
            }
            Msg::DoMenu(m) => {
                self.menu_contexto = None;
                return self.update(*m);
            }
            Msg::CtrlClique(p, x, y) => {
                self.foco_documento = true;
                self.pagina_ativa = p;
                self.sessao.clicar(p, x, y, false, 1);
                if let Some(link) = self.sessao.link_no_cursor() {
                    self.seguir_link(&link);
                }
            }
            Msg::BibLido(r) => {
                self.ocupado = false;
                match r {
                    Ok(Some(t)) => {
                        let fontes = featherdocs_io::bib::ler(&t);
                        if fontes.is_empty() {
                            self.status = "Nenhuma referência reconhecida no .bib".into();
                        } else {
                            let (_, n) = self.sessao.fontes_importar(fontes);
                            self.status = format!("{n} fonte(s) importada(s) do .bib");
                        }
                    }
                    Ok(None) => {}
                    Err(e) => self.status = e,
                }
            }
            Msg::AbrirRecente(p) => {
                self.menu_aberto = false;
                return self.solicitar(Destino::AbrirCaminho(p));
            }
            Msg::FormAbrir(tipo) => {
                self.formulario = Some(crate::dialogos::Formulario::novo(tipo, &self.sessao));
                self.foco_documento = false;
                return iced::widget::text_input::focus(iced::widget::text_input::Id::new("form0"));
            }
            Msg::FormCampo(i, v) => {
                if let Some(c) = self.formulario.as_mut().and_then(|f| f.campos.get_mut(i)) {
                    *c = v;
                }
            }
            Msg::FormOk => {
                if let Some(f) = &self.formulario {
                    match f.aplicar(&mut self.sessao) {
                        Ok(recado) => {
                            self.status = recado;
                            self.formulario = None;
                            self.foco_documento = true;
                        }
                        // O diálogo fica aberto para a pessoa corrigir o campo.
                        Err(e) => self.status = e,
                    }
                }
            }
            Msg::FormFechar => {
                self.formulario = None;
                self.foco_documento = true;
            }
            Msg::FormRemoverLink => {
                self.sessao.link_remover();
                self.formulario = None;
                self.foco_documento = true;
            }
            Msg::FormEscolher(id) => {
                let modo = self.formulario.as_ref().map_or("texto".to_string(), |f| f.campos[0].clone());
                match self.sessao.referencia_inserir(id, &modo) {
                    Ok(_) => {
                        self.formulario = None;
                        self.foco_documento = true;
                    }
                    Err(e) => self.status = e,
                }
            }
            Msg::ColadoPuro(Some(t)) if !self.ocupado && self.pendente.is_none() => {
                self.sessao.colar_sem_formatacao(&t);
            }
            Msg::AbrirCor(alvo) => {
                let pagina = match self.sessao.documento().extras.cor_pagina {
                    0 => String::new(),
                    c => format!("#{:06x}", c >> 8),
                };
                let cor = match alvo {
                    AlvoCor::Texto => &self.visao.formato.cor,
                    AlvoCor::Realce => &self.visao.formato.realce,
                    AlvoCor::Fundo => &self.visao.formato.fundo,
                    AlvoCor::Pagina => &pagina,
                };
                if self.cor_ui.alvo == Some(alvo) {
                    self.cor_ui.alvo = None;
                } else {
                    self.cor_ui.abrir(alvo, cor);
                }
            }
            Msg::EventoCor(evento) => {
                if let Some((alvo, cor)) = self.cor_ui.evento(evento) {
                    let resultado = match alvo {
                        AlvoCor::Texto => self.sessao.cor_definir(&cor),
                        AlvoCor::Realce => self.sessao.realce_definir(&cor),
                        AlvoCor::Fundo => self.sessao.fundo_definir(&cor),
                        AlvoCor::Pagina => self.sessao.cor_pagina_definir(&cor),
                    };
                    if let Err(erro) = resultado {
                        self.status = erro;
                    }
                }
            }
            Msg::Fonte(f) if !self.ocupado => {
                if let Err(e) = self.sessao.fonte_definir(&f) {
                    self.status = e;
                }
            }
            Msg::Tamanho(t) if !self.ocupado => {
                if let Err(e) = self.sessao.tamanho_definir(t as f32) {
                    self.status = e;
                }
            }
            Msg::IrBloco(id) => {
                self.sessao.ir_para_bloco(id);
                self.foco_documento = true;
                tarefa = self.rolar_ao_cursor();
            }
            Msg::BuscaOpcao(i) => {
                let o = &mut self.busca_opcoes;
                match i {
                    0 => o.maiusculas = !o.maiusculas,
                    1 => o.palavra = !o.palavra,
                    _ => o.regex = !o.regex,
                }
                return self.update(Msg::Consulta(self.consulta.clone()));
            }
            Msg::Consulta(s) => {
                self.consulta = s;
                self.foco_documento = false;
                let r = self.sessao.busca_definir(&self.consulta, self.busca_opcoes);
                self.status = r
                    .erro
                    .unwrap_or_else(|| format!("{} / {}", r.atual, r.total));
                tarefa = self.rolar_ao_cursor();
            }
            Msg::Substituicao(s) => {
                self.substituicao = s;
                self.foco_documento = false;
            }
            Msg::BuscaIr(d) => {
                let r = self.sessao.busca_ir(d);
                self.status = format!("{} / {}", r.atual, r.total);
                tarefa = self.rolar_ao_cursor();
            }
            Msg::Substituir(tudo) => {
                if tudo {
                    self.sessao.substituir_tudo(&self.substituicao);
                } else {
                    self.sessao.substituir(&self.substituicao);
                }
            }
            _ => {}
        }
        self.atualizar();
        tarefa
    }
    fn comando(&mut self, c: Comando) -> Task<Msg> {
        if self.ocupado || self.pendente.is_some() {
            return Task::none();
        }
        let mut tarefa = Task::none();
        match c {
            Comando::Novo => return self.solicitar(Destino::Novo),
            Comando::Abrir => return self.solicitar(Destino::Abrir),
            Comando::Salvar => return self.salvar(false, false),
            Comando::SalvarComo => return self.salvar(true, false),
            Comando::Exportar => return self.salvar(true, true),
            Comando::Desfazer => {
                self.sessao.desfazer();
            }
            Comando::Refazer => {
                self.sessao.refazer();
            }
            Comando::Copiar => tarefa = iced::clipboard::write(self.sessao.copiar()),
            Comando::Recortar => {
                let (t, _) = self.sessao.recortar();
                tarefa = iced::clipboard::write(t);
            }
            Comando::Colar => return iced::clipboard::read().map(Msg::Colado),
            Comando::ColarSemFormatacao => return iced::clipboard::read().map(Msg::ColadoPuro),
            Comando::NumeroPagina(modelo, onde) => {
                if let Err(e) = self.sessao.numero_de_pagina(modelo, onde) {
                    self.status = e;
                }
            }
            Comando::Formulario(tipo) => return self.update(Msg::FormAbrir(tipo)),
            Comando::ImportarBib => {
                self.ocupado = true;
                return Task::perform(
                    async {
                        let Some(p) = rfd::AsyncFileDialog::new().add_filter("BibTeX", &["bib"]).pick_file().await else {
                            return Ok(None);
                        };
                        std::fs::read_to_string(p.path()).map(Some).map_err(|e| format!("Não consegui ler o .bib: {e}"))
                    },
                    Msg::BibLido,
                );
            }
            Comando::Bibliografia(todas) => {
                self.sessao.bibliografia_inserir(todas);
            }
            Comando::EstiloCitacao(nome) => {
                if let Err(e) = self.sessao.estilo_citacao_definir(nome) {
                    self.status = e;
                }
            }
            Comando::NumeroRemover => {
                self.sessao.numero_remover();
            }
            Comando::Legenda(tipo) => {
                if let Err(e) = self.sessao.legenda_inserir(tipo) {
                    self.status = e;
                }
            }
            Comando::IndiceInserir => {
                self.sessao.indice_inserir();
            }
            Comando::ListaDe(n) => {
                self.sessao.lista_inserir(n);
            }
            Comando::SumarioRemover => {
                self.sessao.remover_sumario();
            }
            Comando::SumarioNiveis(n) => {
                self.sessao.sumario_configurar(n, true, true);
            }
            Comando::ReferenciasAtualizar => {
                self.sessao.referencias_atualizar();
            }
            Comando::Margens(cm) => {
                let [e, d, s, i] = cm.map(|v| v * 72.0 / 2.54);
                if let Err(erro) = self.sessao.margens_definir(e, d, s, i) {
                    self.status = erro;
                }
            }
            Comando::NumerarLinhas(nome) => {
                if let Err(e) = self.sessao.numerar_linhas_definir(nome) {
                    self.status = e;
                }
            }
            Comando::PrimeiraDiferente => {
                let sim = !self.sessao.documento().extras.primeira_diferente;
                self.sessao.primeira_diferente_definir(sim);
            }
            Comando::Vertical(nome) => {
                if let Err(e) = self.sessao.vertical_definir(nome) {
                    self.status = e;
                }
            }
            Comando::Proteger => {
                let sim = !self.sessao.documento().extras.protegido;
                self.sessao.proteger(sim);
                self.status = if sim { "Documento protegido: só leitura e comentários" } else { "Proteção retirada" }.into();
            }
            Comando::Posicao(p) => {
                self.sessao.posicao_alternar(p);
            }
            Comando::TamanhoPasso(d) => {
                self.sessao.tamanho_passo(d);
            }
            Comando::LimparFormatacao => {
                self.sessao.limpar_formatacao();
            }
            Comando::Recuar(d) => {
                self.sessao.recuar(d);
            }
            Comando::ListaReiniciar => {
                self.sessao.lista_reiniciar();
            }
            Comando::Borda(lados) => {
                self.sessao.borda_definir(lados);
            }
            Comando::Maiusculas(modo) => {
                if let Err(e) = self.sessao.maiusculas(modo) {
                    self.status = e;
                }
            }
            Comando::Lista(nome) => {
                if let Err(e) = self.sessao.lista_estilo(nome) {
                    self.status = e;
                }
            }
            Comando::Entrelinha(fator) => {
                if let Err(e) = self.sessao.entrelinha_definir(fator) {
                    self.status = e;
                }
            }
            Comando::Espaco(antes, pt) => {
                let f = &self.visao.formato;
                let (a, d) = if antes { (pt, f.depois) } else { (f.antes, pt) };
                if let Err(e) = self.sessao.espaco_definir(Some(a), Some(d)) {
                    self.status = e;
                }
            }
            Comando::Pincel => {
                if self.sessao.pincel_cheio() {
                    if !self.sessao.cursor().vazio() {
                        self.sessao.pincel_aplicar();
                        self.status = "Pincel aplicado à seleção".into();
                    } else {
                        self.status = "Pincel cancelado".into();
                    }
                    self.sessao.pincel_cancelar();
                } else {
                    self.sessao.pincel_copiar();
                    self.status = "Pincel ativo: selecione o destino com Shift+clique; Esc cancela.".into();
                }
            }
            Comando::Formato(bit) => {
                self.sessao.formatar(bit);
            }
            Comando::EstiloAtualizar | Comando::EstiloExcluir => {
                let id = self.sessao.estilo_com_nome_no_cursor();
                let nome = self.sessao.documento().estilos.pegar(id).map(|e| e.nome.clone());
                let r = match (nome, &c) {
                    (None, _) => Err("O cursor não está num parágrafo com estilo com nome.".to_string()),
                    (Some(nome), Comando::EstiloAtualizar) => self.sessao
                        .estilo_modificar(id, crate::dialogos::estilo_do_cursor(&self.sessao, &nome)).map(|_| ()),
                    _ => self.sessao.estilo_excluir(id).map(|_| ()),
                };
                if let Err(e) = r {
                    self.status = e;
                }
            }
            Comando::EstiloNome(id) => {
                if let Err(e) = self.sessao.estilo_aplicar(id) {
                    self.status = e;
                }
            }
            Comando::Estilo(s) => {
                self.sessao.paragrafo_estilo(s);
            }
            Comando::Alinhar(s) => {
                if let Err(e) = self.sessao.alinhar(s) {
                    self.status = e;
                }
            }
            Comando::TabelaTam(l, c) => {
                if let Err(e) = self.sessao.inserir_tabela(l, c) {
                    self.status = e;
                }
            }
            Comando::Tabela => {
                if let Err(e) = self.sessao.inserir_tabela(3, 3) {
                    self.status = e;
                }
            }
            Comando::Grafico(tipo) => {
                if let Err(e) = featherdocs_io::grafico_docs::inserir_da_tabela(
                    &mut self.sessao, tipo,
                ) {
                    self.status = e;
                }
            }
            Comando::AtualizarGrafico => {
                if let Err(e) = featherdocs_io::grafico_docs::atualizar_no_cursor(
                    &mut self.sessao,
                ) {
                    self.status = e;
                }
            }
            Comando::TabelaEstrutura(acao) => {
                if let Err(erro) = featherdocs_tabelas::docs_editar_estrutura(
                    &mut self.sessao, acao,
                ) {
                    self.status = erro;
                }
            }
            Comando::TabelaMesclar => {
                if let Err(erro) = featherdocs_tabelas::docs_mesclar_celulas(&mut self.sessao) {
                    self.status = erro;
                }
            }
            Comando::TabelaDesmesclar => {
                if let Err(erro) = featherdocs_tabelas::docs_desmesclar_celulas(&mut self.sessao) {
                    self.status = erro;
                }
            }
            // Cabeçalho e exclusão continuam no Docs: são atributos do documento
            // e não dimensões/estilos genéricos do motor Tables.
            Comando::TabelaCabecalho => {
                if self.visao.formato.tabela.is_some() {
                    self.sessao.tabela_cabecalho();
                }
            }
            Comando::TabelaExcluir => {
                if let Err(erro) = self.sessao.tabela_excluir("tabela") {
                    self.status = erro;
                }
            }
            Comando::Imagem => {
                self.ocupado = true;
                return Task::perform(
                    async {
                        let Some(p) = rfd::AsyncFileDialog::new()
                            .add_filter("Imagens", &["png", "jpg", "jpeg"])
                            .pick_file()
                            .await
                        else {
                            return Ok(None);
                        };
                        let bytes = std::fs::read(p.path())
                            .map_err(|e| format!("Não consegui ler a imagem: {e}"))?;
                        let imagem = crate::imagem_compartilhada::preparar_insercao(&bytes)?;
                        Ok(Some(imagem))
                    },
                    Msg::Imagem,
                );
            }
            Comando::Quebra => {
                self.sessao.inserir_bloco("quebra_de_pagina");
            }
            Comando::Sumario => {
                self.sessao.inserir_sumario(3);
            }
            Comando::Papel(p) => {
                if let Err(e) = self.sessao.papel_definir(p) {
                    self.status = e;
                }
            }
            Comando::Orientacao(b) => {
                if let Err(e) = self.sessao.orientacao_definir(b) {
                    self.status = e;
                }
            }
            Comando::Modo(m) => {
                if let Err(e) = self.sessao.modo_edicao_definir(m) {
                    self.status = e;
                }
            }
            Comando::PainelRevisao => self.painel_revisao = !self.painel_revisao,
            Comando::Comparar => {
                self.ocupado = true;
                return Task::perform(
                    async {
                        let Some(p) = rfd::AsyncFileDialog::new()
                            .add_filter("Documentos", &["fdoz", "docx", "md", "tex"])
                            .pick_file()
                            .await
                        else {
                            return Ok(None);
                        };
                        Self::carregar_em_background(p.path().to_path_buf()).await.map(Some)
                    },
                    Msg::Comparado,
                );
            }
            Comando::Revisoes(b, tudo) => {
                self.sessao.revisoes_resolver(b, tudo);
            }
            Comando::Estrutura => {
                self.config.estrutura = !self.config.estrutura;
                self.config.gravar();
            }
            Comando::Marcas => {
                self.sessao.marcas_definir(!self.sessao.estado().marcas);
            }
            Comando::Codigo(fonte) => self.alternar_codigo(fonte),
            Comando::Zoom(d) => {
                self.config.zoom = (self.config.zoom + d).clamp(0.5, 2.);
                self.config.gravar();
            }
            Comando::Configuracoes => {
                self.menu_aberto = false;
                return self.alternar_aux(JanelaAux::Configuracoes);
            }
            Comando::Menu => self.menu_aberto = !self.menu_aberto,
            Comando::Ajuda => {
                self.menu_aberto = false;
                return self.alternar_aux(JanelaAux::Ajuda);
            }
            Comando::Monitor => {
                self.menu_aberto = false;
                return self.alternar_aux(JanelaAux::Monitor);
            }
            Comando::Buscar => {
                self.busca = !self.busca;
                self.foco_documento = !self.busca;
                if !self.busca {
                    self.sessao.busca_limpar();
                }
            }
        }
        self.atualizar();
        tarefa
    }
    /// A língua do painel de código aberto (para a faixa marcar o botão).
    pub fn fonte_do_codigo(&self) -> Option<Fonte> {
        self.codigo.as_ref().map(|p| p.fonte)
    }

    /// Liga o painel nesta língua, troca de língua ou, na mesma, desliga.
    fn alternar_codigo(&mut self, fonte: Fonte) {
        self.aplicar_codigo();
        if self.fonte_do_codigo() == Some(fonte) {
            self.codigo = None;
            self.foco_documento = true;
            return;
        }
        let inicio = Instant::now();
        let mapa = painel_codigo::gerar(self.sessao.documento(), fonte, self.fonte_original.as_ref());
        let mut painel = PainelCodigo::novo(fonte, mapa, self.sessao.versao());
        painel.ir_ao_bloco(self.sessao.cursor().foco.bloco);
        self.codigo = Some(painel);
        self.status = format!(
            "Código {} gerado em {} ms",
            painel_codigo::nome_da_fonte(fonte),
            inicio.elapsed().as_millis()
        );
    }

    /// Manda ao motor o código digitado (se houver): relê, troca só os blocos que mudaram e
    /// guarda o texto como o original da pessoa. Cada aplicação é um passo de Desfazer.
    fn aplicar_codigo(&mut self) {
        let Some(painel) = self.codigo.as_mut() else { return };
        if painel.ultima_edicao.take().is_none() {
            return;
        }
        let fonte = painel.fonte;
        let texto = painel.conteudo.text();
        if self.sessao.modo_edicao() == "leitura" {
            self.status = "O documento está só para leitura: o código não foi aplicado.".into();
            painel.versao = u64::MAX;
            return;
        }
        let inicio = Instant::now();
        let versao_antes = self.sessao.versao();
        let r = featherdocs_io::dividido::reconciliar(self.sessao.documento(), fonte, &texto);
        let mudanca = self.sessao.trocar_blocos(r.blocos, r.meta, r.numeracao);
        painel.blocos = featherdocs_io::exportar::mapa_de_texto(&texto, self.sessao.documento(), fonte).blocos;
        painel.versao = self.sessao.versao();
        painel.bloco_marcado = painel.bloco_da_linha(painel.conteudo.cursor_position().0);
        self.status = if self.sessao.versao() == versao_antes {
            String::new()
        } else {
            format!(
                "Código {} aplicado: {} bloco(s) em {} ms",
                painel_codigo::nome_da_fonte(fonte),
                mudanca.afetados.len(),
                inicio.elapsed().as_millis()
            )
        };
        self.fonte_original = Some((fonte, texto));
    }

    /// Depois de mudar a sessão: se o documento mudou por fora do código (página, faixa,
    /// desfazer), o código é gerado de novo; se a pessoa está na página, o código acompanha
    /// o bloco do cursor. Nada disso enquanto há código digitado à espera da pausa.
    fn sincronizar_codigo(&mut self) {
        let versao = self.sessao.versao();
        let bloco = self.sessao.cursor().foco.bloco;
        let Some(painel) = self.codigo.as_mut() else { return };
        if painel.pendente() {
            return;
        }
        if painel.versao != versao {
            let mapa = painel_codigo::gerar(self.sessao.documento(), painel.fonte, self.fonte_original.as_ref());
            painel.recarregar(mapa, versao);
        }
        if self.foco_documento {
            painel.ir_ao_bloco(bloco);
        }
    }

    /// Ação no editor de código: o foco sai da página; edição marca o código como pendente
    /// (aplicado depois da pausa); clique ou seta leva a página ao bloco da linha.
    fn codigo_editar(&mut self, acao: iced::widget::text_editor::Action) -> Task<Msg> {
        if self.ocupado || self.pendente.is_some() {
            if acao.is_edit() {
                return Task::none();
            }
        }
        let Some(painel) = self.codigo.as_mut() else { return Task::none() };
        use iced::widget::text_editor::Action;
        let move_cursor = matches!(acao, Action::Click(_) | Action::Move(_) | Action::Select(_)
            | Action::Drag(_) | Action::SelectWord | Action::SelectLine);
        let editar = acao.is_edit();
        if !matches!(acao, Action::Scroll { .. }) {
            self.foco_documento = false;
        }
        painel.conteudo.perform(acao);
        if editar {
            painel.ultima_edicao = Some(Instant::now());
            return Task::none();
        }
        if !move_cursor || painel.pendente() {
            return Task::none();
        }
        let linha = painel.conteudo.cursor_position().0;
        let Some(id) = painel.bloco_da_linha(linha) else { return Task::none() };
        if painel.bloco_marcado == Some(id) {
            return Task::none();
        }
        painel.bloco_marcado = Some(id);
        if self.sessao.cursor().foco.bloco != id {
            self.sessao.ir_para_bloco(id);
            self.atualizar();
            return self.rolar_ao_cursor();
        }
        Task::none()
    }

    /// `FEATHERDOCS_CAPTURAR=arquivo.png`: espera o arquivo da linha de comando abrir, liga o
    /// painel de código (`FEATHERDOCS_CAPTURAR_CODIGO=tex|md`), fotografa a janela e fecha.
    /// `FEATHERDOCS_CAPTURAR_TEMA=escuro|claro` escolhe o tema só para a foto.
    fn capturar_tela(&mut self) -> Task<Msg> {
        if self.ocupado {
            return Task::none();
        }
        match self.passo_captura {
            0 => {
                if let Ok(tema) = std::env::var("FEATHERDOCS_CAPTURAR_TEMA") {
                    definir_tema(tema == "escuro", self.config.acento);
                }
                match std::env::var("FEATHERDOCS_CAPTURAR_CODIGO").as_deref() {
                    Ok("tex") => self.alternar_codigo(Fonte::Latex),
                    Ok("md") => self.alternar_codigo(Fonte::Markdown),
                    _ => {}
                }
                // `FEATHERDOCS_CAPTURAR_ZOOM=1.5`: o zoom só para a foto (não grava na configuração).
                if let Some(z) = std::env::var("FEATHERDOCS_CAPTURAR_ZOOM").ok().and_then(|z| z.parse::<f32>().ok()) {
                    self.config.zoom = z.clamp(0.5, 2.);
                }
                if let Ok(n) = std::env::var("FEATHERDOCS_CAPTURAR_BLOCO") {
                    // Leva a página (e o código) ao n-ésimo bloco do documento.
                    if let Some(id) = n.parse::<usize>().ok()
                        .and_then(|i| self.sessao.documento().blocos.get(i)).map(|b| b.id) {
                        self.sessao.ir_para_bloco(id);
                    }
                }
                self.aba = match std::env::var("FEATHERDOCS_CAPTURAR_ABA").as_deref() {
                    Ok("inicio") => Aba::Inicio,
                    Ok("inserir") => Aba::Inserir,
                    Ok("layout") => Aba::Layout,
                    Ok("revisao") => Aba::Revisao,
                    _ => Aba::Exibir,
                };
                self.painel_revisao |= self.aba == Aba::Revisao;
                self.atualizar();
                // `FEATHERDOCS_CAPTURAR_PDF=saida.pdf`: grava também o PDF das páginas.
                if let Some(saida) = std::env::var_os("FEATHERDOCS_CAPTURAR_PDF") {
                    match crate::pdf::gerar(&self.quadros) {
                        Ok(b) => { let _ = std::fs::write(&saida, b); }
                        Err(e) => eprintln!("PDF não gerado: {e}"),
                    }
                }
                self.passo_captura = 1;
                self.rolar_ao_cursor()
            }
            1 => {
                self.passo_captura = 2;
                // A foto sai depois de uma pausa sem mensagem nenhuma no meio: o Iced desenha a
                // janela nesse intervalo e fotografa o último quadro. Uma mensagem refaria o
                // layout do `text_editor` (que troca o buffer dele) e o código sairia em branco.
                match self.janelas.principal {
                    Some(id) => Task::perform(
                        async {
                            let _ = tokio::task::spawn_blocking(|| {
                                std::thread::sleep(std::time::Duration::from_millis(600))
                            })
                            .await;
                        },
                        |_| (),
                    )
                    .then(move |_| window::screenshot(id))
                    .map(Msg::TelaCapturada),
                    None => Task::none(),
                }
            }
            _ => Task::none(),
        }
    }

    fn view_codigo<'a>(&'a self, painel: &'a PainelCodigo) -> Element<'a, Msg> {
        use iced::widget::text_editor;
        let (linha, coluna) = painel.conteudo.cursor_position();
        let (titulo, outra, rotulo_outra) = match painel.fonte {
            Fonte::Latex => (tr("codigo_titulo_latex"), Fonte::Markdown, tr("codigo_markdown")),
            Fonte::Markdown => (tr("codigo_titulo_markdown"), Fonte::Latex, tr("codigo_latex")),
        };
        let topo = container(
            row![
                icone("code", 16.),
                text(titulo).size(13),
                Space::with_width(Length::Fill),
                text(format!("Lin {}, Col {}", linha + 1, coluna + 1))
                    .size(11)
                    .color(COR_TEXTO_MUTED()),
                button(text(rotulo_outra).size(12))
                    .padding([3, 9])
                    .style(estilo_botao_pequeno(false))
                    .on_press(Msg::Comando(Comando::Codigo(outra))),
                button(icone("x", 14.))
                    .padding(4)
                    .style(estilo_botao_pequeno(false))
                    .on_press(Msg::CodigoFechar),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(estilo_container_topo);
        let editor = text_editor(&painel.conteudo)
            .on_action(Msg::CodigoEditar)
            .font(painel_codigo::FONTE_MONO)
            .size(13.5)
            .padding(10)
            .height(Length::Fill)
            .wrapping(iced::widget::text::Wrapping::None)
            .key_binding(|tecla| {
                use iced::keyboard::Key;
                use text_editor::{Binding, Status};
                if tecla.status == Status::Focused && tecla.modifiers.command() {
                    if let Key::Character(c) = tecla.key.as_ref() {
                        match (c.to_lowercase().as_str(), tecla.modifiers.shift()) {
                            ("z", false) => return Some(Binding::Custom(Msg::Comando(Comando::Desfazer))),
                            ("z", true) | ("y", _) => {
                                return Some(Binding::Custom(Msg::Comando(Comando::Refazer)))
                            }
                            _ => {}
                        }
                    }
                }
                Binding::from_key_press(tecla)
            })
            .style(|_tema, status| text_editor::Style {
                background: COR_PAINEL().into(),
                border: iced::Border {
                    width: 0.,
                    radius: 0.0.into(),
                    color: if status == text_editor::Status::Focused {
                        COR_AZUL_ACENTO()
                    } else {
                        COR_BORDA()
                    },
                },
                icon: COR_TEXTO_MUTED(),
                placeholder: COR_TEXTO_MUTED(),
                value: COR_TEXTO(),
                selection: COR_HOVER_AZUL(),
            })
            .highlight_with::<painel_codigo::Realce>(painel.fonte, painel_codigo::formato);
        container(column![topo, editor])
            .width(Length::FillPortion(45))
            .height(Length::Fill)
            .style(|_t| container::Style {
                background: Some(COR_PAINEL().into()),
                border: iced::Border { width: 1., color: COR_BORDA(), radius: 0.0.into() },
                ..Default::default()
            })
            .into()
    }
    fn rolar_ao_cursor(&self) -> Task<Msg> {
        let Some(c) = self.sessao.geometria(&self.sessao.cursor().foco) else {
            return Task::none();
        };
        let y = self
            .quadros
            .iter()
            .take(c.pagina)
            .map(|q| q.altura * self.config.zoom + 24.)
            .sum::<f32>()
            + c.y * self.config.zoom;
        scrollable::scroll_to(
            scrollable::Id::new("documento"),
            scrollable::AbsoluteOffset {
                x: 0.,
                y: (y - 140.).max(0.),
            },
        )
    }
    pub fn subscription(&self) -> Subscription<Msg> {
        Subscription::batch([
            iced::event::listen_with(|evento, estado, id| match evento {
                iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                    if estado == iced::event::Status::Ignored =>
                    Some(Msg::AtalhoJanela(id, key, modifiers)),
                iced::Event::Window(window::Event::FileDropped(p)) => Some(Msg::AbrirRecente(p)),
                _ => None,
            }),
            window::close_requests().map(Msg::Fechar),
            window::close_events().map(Msg::JanelaFechada),
            if self.codigo.as_ref().is_some_and(PainelCodigo::pendente) {
                iced::time::every(std::time::Duration::from_millis(100)).map(|_| Msg::CodigoPulso)
            } else {
                Subscription::none()
            },
            if self.passo_captura < 2 && std::env::var_os("FEATHERDOCS_CAPTURAR").is_some() {
                iced::time::every(std::time::Duration::from_millis(1500)).map(|_| Msg::CapturarTela)
            } else {
                Subscription::none()
            },
        ])
    }
    pub fn view(&self, janela: window::Id) -> Element<'_, Msg> {
        feather_ui::cromo::moldura_janela(self.view_conteudo(janela))
    }

    fn view_conteudo(&self, janela: window::Id) -> Element<'_, Msg> {
        let auxiliar = self.janelas.tipo(janela);
        if let Some(tipo) = auxiliar {
            let corpo = match tipo {
                JanelaAux::Configuracoes => self.view_configuracoes(),
                JanelaAux::Ajuda => self.view_ajuda(),
                JanelaAux::Monitor => self.view_monitor(),
            };
            if self.config.barra_propria {
                return feather_ui::cromo::janela_com_barra(
                    &self.titulo(janela), corpo, Self::acoes_janela(janela)
                );
            }
            return corpo;
        }
        self.view_editor(janela)
    }
    fn view_editor(&self, janela: window::Id) -> Element<'_, Msg> {
        let titulo_documento = self.caminho.as_ref()
            .and_then(|p| p.file_name()).and_then(|n| n.to_str())
            .unwrap_or("Documento sem título");
        let topo = feather_ui::cromo::barra_editor(
            titulo_documento,
            self.sessao.estado().sujo,
            Some(Msg::Comando(Comando::Menu)),
            [
                feather_ui::cromo::Acao { icone: "file-plus", dica: "Novo (Ctrl+N)", mensagem: Some(Msg::Comando(Comando::Novo)) },
                feather_ui::cromo::Acao { icone: "folder-open", dica: "Abrir (Ctrl+O)", mensagem: Some(Msg::Comando(Comando::Abrir)) },
                feather_ui::cromo::Acao { icone: "device-floppy", dica: "Salvar (Ctrl+S)", mensagem: Some(Msg::Comando(Comando::Salvar)) },
                feather_ui::cromo::Acao { icone: "file-export", dica: "Exportar", mensagem: Some(Msg::Comando(Comando::Exportar)) },
                feather_ui::cromo::Acao { icone: "arrow-back-up", dica: "Desfazer (Ctrl+Z)", mensagem: Some(Msg::Comando(Comando::Desfazer)) },
                feather_ui::cromo::Acao { icone: "arrow-forward-up", dica: "Refazer", mensagem: Some(Msg::Comando(Comando::Refazer)) },
                feather_ui::cromo::Acao { icone: "search", dica: "Pesquisar (Ctrl+F)", mensagem: Some(Msg::Comando(Comando::Buscar)) },
                feather_ui::cromo::Acao { icone: "help", dica: "Ajuda (F1)", mensagem: Some(Msg::Comando(Comando::Ajuda)) },
                feather_ui::cromo::Acao { icone: "device-desktop", dica: "Monitor (F12)", mensagem: Some(Msg::Comando(Comando::Monitor)) },
            ],
            Msg::Comando(Comando::Configuracoes),
        );
        let topo = if self.config.barra_propria {
            feather_ui::cromo::barra_integrada(topo, Self::acoes_janela(janela))
        } else { topo };
        let mut folhas = column![].spacing(24).align_x(Alignment::Center);
        for (i, q) in self.quadros.iter().enumerate() {
            folhas = folhas.push(
                canvas(Pagina {
                    quadro: q,
                    visao: &self.visao,
                    numero: i,
                    escala: self.config.zoom,
                    ativa: self.foco_documento
                        && self.pagina_ativa == i
                        && !self.ocupado
                        && self.pendente.is_none(),
                    imagens: &self.imagens,
                })
                .width(q.largura * self.config.zoom)
                .height(q.altura * self.config.zoom),
            );
        }
        let paginas = scrollable(
            container(folhas)
                .padding(24)
                .width(Length::Shrink)
                .align_x(Alignment::Center),
        )
        .direction(scrollable::Direction::Both {
            vertical: scrollable::Scrollbar::default(),
            horizontal: scrollable::Scrollbar::default(),
        })
        .id(scrollable::Id::new("documento"))
        .on_scroll(|v| {
            let o = v.absolute_offset();
            Msg::Rolou(iced::Vector::new(o.x, o.y))
        })
        .height(Length::Fill)
        .width(if self.codigo.is_some() { Length::FillPortion(55) } else { Length::Fill });
        let mut centro = row![].height(Length::Fill);
        if self.config.estrutura {
            let mut titulos = column![text(tr("estrutura")).size(16)].spacing(7);
            let estrutura = self.sessao.estrutura();
            if estrutura.is_empty() {
                titulos = titulos.push(text(tr("sem_titulos")).size(12).color(COR_TEXTO_MUTED()));
            }
            for t in estrutura {
                titulos = titulos.push(
                    button(text(t.texto).size(12))
                        .padding([6, 10 + (t.nivel.saturating_sub(1) as u16 * 10)])
                        .width(Length::Fill)
                        .style(estilo_botao_pequeno(false))
                        .on_press(Msg::IrBloco(t.id)),
                );
            }
            centro = centro.push(
                container(scrollable(titulos))
                    .padding(14)
                    .width(220)
                    .height(Length::Fill)
                    .style(estilo_painel_miniaturas),
            );
        }
        if let Some(painel) = &self.codigo {
            centro = centro.push(self.view_codigo(painel));
        }
        centro = centro.push(paginas);
        if self.painel_revisao {
            centro = centro.push(self.view_revisao());
        }
        let e = self.sessao.estado();
        let rodape = container(
            row![
                text(format!(
                    "{} {} / {} · {} {}",
                    tr("pagina"),
                    self.pagina_ativa + 1,
                    e.paginas,
                    e.palavras,
                    tr("palavras")
                ))
                .size(12),
                text(if self.ocupado {
                    tr("ocupado")
                } else {
                    &self.status
                })
                .size(12),
                Space::with_width(Length::Fill),
                text(format!("{:.0}%", self.config.zoom * 100.)).size(12),
                text(tr("local")).size(11)
            ]
            .spacing(16),
        )
        .padding([7, 12])
        .style(estilo_container_topo);
        let mut tela = column![topo].spacing(0);
        if !self.config.faixa_recolhida {
            tela = tela.push(container(faixa::view(self)).style(estilo_painel_faixa));
        }
        if let Some(alvo) = self.cor_ui.alvo {
            let titulo = match alvo {
                AlvoCor::Texto => "Cor do texto",
                AlvoCor::Realce => "Marca-texto",
                AlvoCor::Fundo => "Sombreamento do parágrafo",
                AlvoCor::Pagina => "Cor da página",
            };
            tela = tela.push(
                container(row![
                    Space::with_width(Length::Fill),
                    feather_ui::seletor_cor::painel(
                        titulo,
                        &self.cor_ui.atual,
                        self.cor_ui.hsva,
                        &self.cor_ui.texto_hex,
                    )
                    .map(Msg::EventoCor)
                ])
                .width(Length::Fill)
                .padding(8)
            );
        }
        if let Some((_, _, chave)) = &self.efeito_imagem {
            tela = tela.push(
                container(row![
                    Space::with_width(Length::Fill),
                    feather_ui::efeitos_imagem::painel(
                        chave,
                        Msg::AplicarEfeitoImagem,
                        Msg::FecharEfeitosImagem,
                    ),
                ]).width(Length::Fill).padding(8)
            );
        }
        if self.busca {
            tela = tela.push(self.view_busca());
        }
        if self.pendente.is_some() {
            tela = tela.push(
                container(
                    column![
                        text(tr("salvar_pendente")).size(19),
                        text(&self.status).size(12),
                        row![
                            button(text(tr("salvar")))
                                .on_press(Msg::SalvarEContinuar)
                                .style(estilo_botao_primario),
                            button(text(tr("descartar")))
                                .on_press(Msg::Descartar)
                                .style(estilo_botao_secundario),
                            button(text(tr("cancelar")))
                                .on_press(Msg::Cancelar)
                                .style(estilo_botao_secundario)
                        ]
                        .spacing(10)
                    ]
                    .spacing(15),
                )
                .padding(30)
                .width(Length::Fill)
                .height(Length::Fill),
            );
        } else if self.menu_aberto {
            tela = tela.push(row![self.view_menu(), centro].height(Length::Fill));
        } else {
            tela = tela.push(centro);
        }
        let tela: Element<'_, Msg> = tela.push(rodape).height(Length::Fill).into();
        let tela = match self.menu_contexto {
            Some(ponto) => self.view_contexto(tela, ponto),
            None => tela,
        };
        match &self.formulario {
            Some(f) => f.view(&self.sessao, tela),
            None => tela,
        }
    }
    /// O diálogo de editar a nota desenhada em (x, y) da página, se houver uma ali.
    fn nota_em(&self, pagina: usize, x: f32, y: f32) -> Option<crate::dialogos::Tipo> {
        let pos = self.sessao.nota_no_ponto(pagina, x, y)?;
        let fim = self.sessao.nota_info(&pos).ok()?.fim;
        Some(crate::dialogos::Tipo::EditarNota(pos, fim))
    }
    /// Segue um link: interno vai ao bloco; web e e-mail abrem no programa do sistema
    /// (só por Ctrl+clique, pedido explícito da pessoa).
    fn seguir_link(&mut self, link: &str) {
        if let Some(id) = featherdocs_core::referencias::destino_do_link(link) {
            self.sessao.ir_para_bloco(id);
            return;
        }
        let externo = ["http://", "https://", "mailto:"].iter().any(|p| link.starts_with(p));
        if !externo {
            self.status = format!("Link sem destino que eu saiba abrir: {link}");
            return;
        }
        #[cfg(target_os = "windows")]
        let r = std::process::Command::new("explorer").arg(link).spawn();
        #[cfg(target_os = "macos")]
        let r = std::process::Command::new("open").arg(link).spawn();
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let r = std::process::Command::new("xdg-open").arg(link).spawn();
        self.status = match r {
            Ok(_) => format!("Abrindo {link}"),
            Err(e) => format!("Não consegui abrir o link: {e}"),
        };
    }
    fn view_contexto<'a>(&'a self, base: Element<'a, Msg>, ponto: iced::Point) -> Element<'a, Msg> {
        let item = |ic: &str, rotulo: &str, m: Msg| -> Element<'a, Msg> {
            button(row![icone(ic, 15.), text(rotulo.to_string()).size(12)].spacing(8).align_y(Alignment::Center))
                .width(Length::Fill).padding([5, 10])
                .style(estilo_botao_pequeno(false))
                .on_press(Msg::DoMenu(Box::new(m)))
                .into()
        };
        let mut col = column![
            item("cut", "Recortar", Msg::Comando(Comando::Recortar)),
            item("copy", "Copiar", Msg::Comando(Comando::Copiar)),
            item("clipboard", "Colar", Msg::Comando(Comando::Colar)),
            item("clipboard", "Colar sem formatação", Msg::Comando(Comando::ColarSemFormatacao)),
        ].spacing(1);
        if self.sessao.link_no_cursor().is_some() {
            col = col
                .push(item("link-open", "Abrir link", Msg::AbrirLink))
                .push(item("link", "Editar link", Msg::FormAbrir(crate::dialogos::Tipo::Link)))
                .push(item("link-remove", "Remover link", Msg::FormRemoverLink));
        } else {
            col = col.push(item("link", "Link…", Msg::FormAbrir(crate::dialogos::Tipo::Link)));
        }
        if let Some(tipo) = self.nota_contexto {
            col = col.push(item("text-box", "Editar nota…", Msg::FormAbrir(tipo)));
        }
        col = col
            .push(item("comentario", "Novo comentário…", Msg::FormAbrir(crate::dialogos::Tipo::Comentario)))
            .push(item("clear", "Limpar formatação", Msg::Comando(Comando::LimparFormatacao)));
        if self.visao.formato.tabela.is_some() {
            col = col
                .push(item("plus", "Inserir linha", Msg::Comando(Comando::TabelaEstrutura("linha+"))))
                .push(item("plus", "Inserir coluna", Msg::Comando(Comando::TabelaEstrutura("coluna+"))))
                .push(item("minus", "Excluir linha", Msg::Comando(Comando::TabelaEstrutura("linha-"))))
                .push(item("minus", "Excluir coluna", Msg::Comando(Comando::TabelaEstrutura("coluna-"))))
                .push(item("trash", "Excluir tabela", Msg::Comando(Comando::TabelaExcluir)));
        }
        let menu = container(col).width(220).padding(4).style(estilo_popup_menu);
        iced::widget::stack![
            base,
            iced::widget::mouse_area(Space::new(Length::Fill, Length::Fill))
                .on_press(Msg::FecharContexto)
                .on_right_press(Msg::FecharContexto),
            container(menu).padding(iced::Padding { top: ponto.y, left: ponto.x, ..iced::Padding::ZERO }),
        ]
        .into()
    }
    fn view_revisao(&self) -> Element<'_, Msg> {
        use feather_ui::dialogo as dlg;
        let c = self.sessao.contagem();
        let mut col = column![
            row![text("Revisão").size(16), Space::with_width(Length::Fill),
                feather_ui::componentes::botao_icone(icone("x", 14.), "Fechar painel", Some(Msg::Comando(Comando::PainelRevisao)), false)],
            dlg::secao(if c.selecao { "Estatísticas da seleção" } else { "Estatísticas" }),
            dlg::grade_rotulo_valor(vec![
                ("Páginas".into(), c.paginas.to_string()),
                ("Palavras".into(), c.palavras.to_string()),
                ("Caracteres".into(), c.caracteres.to_string()),
                ("Sem espaços".into(), c.caracteres_sem_espaco.to_string()),
                ("Parágrafos".into(), c.paragrafos.to_string()),
                ("Linhas".into(), c.linhas.to_string()),
            ], 2),
            dlg::secao("Comentários"),
        ].spacing(8);
        let comentarios = self.sessao.comentarios();
        if comentarios.is_empty() {
            col = col.push(text("Nenhum comentário.").size(12).color(COR_TEXTO_MUTED()));
        }
        for k in comentarios {
            let mut cartao = column![
                text(format!("{} · p. {}", if k.autor.is_empty() { "Comentário" } else { &k.autor }, k.pagina)).size(11).color(COR_TEXTO_MUTED()),
                text(format!("“{}”", k.trecho)).size(11).color(COR_TEXTO_MUTED()),
                text(k.texto.clone()).size(13),
            ].spacing(3);
            for r in &k.respostas {
                cartao = cartao.push(container(text(r.texto.clone()).size(12)).padding(iced::Padding { left: 14.0, ..iced::Padding::ZERO }));
            }
            cartao = cartao.push(row![
                feather_ui::componentes::botao_icone(icone("search", 14.), "Ir ao trecho", Some(Msg::ComentarioIr(k.id)), false),
                feather_ui::componentes::botao_icone(icone("message-circle", 14.), "Responder", Some(Msg::FormAbrir(crate::dialogos::Tipo::Resposta(k.id))), false),
                feather_ui::componentes::botao_icone(icone("check", 14.), if k.resolvido { "Reabrir" } else { "Resolver" }, Some(Msg::ComentarioResolver(k.id, !k.resolvido)), k.resolvido),
                feather_ui::componentes::botao_icone(icone("trash", 14.), "Excluir comentário", Some(Msg::ComentarioExcluir(k.id)), false),
            ].spacing(2));
            col = col.push(container(cartao).padding(8).width(Length::Fill).style(dlg::estilo_bloco));
        }
        col = col.push(dlg::secao("Alterações sugeridas"));
        let revisoes = self.sessao.revisoes();
        if revisoes.is_empty() {
            col = col.push(text("Nenhuma alteração sugerida.").size(12).color(COR_TEXTO_MUTED()));
        }
        for r in revisoes {
            let tipo = match r.tipo {
                featherdocs_core::modelo::TipoDeRevisao::Insercao => "Inserido",
                featherdocs_core::modelo::TipoDeRevisao::Remocao => "Removido",
            };
            col = col.push(
                button(column![
                    text(format!("{tipo}{}", if r.autor.is_empty() { String::new() } else { format!(" · {}", r.autor) })).size(11).color(COR_TEXTO_MUTED()),
                    text(r.texto).size(12),
                ].spacing(2))
                .width(Length::Fill).padding([6, 8])
                .style(estilo_botao_pequeno(false))
                .on_press(Msg::IrBloco(r.bloco)),
            );
        }
        container(scrollable(col.padding(14)))
            .width(300)
            .height(Length::Fill)
            .style(estilo_painel_miniaturas)
            .into()
    }
    fn view_configuracoes(&self) -> Element<'_, Msg> {
        let secoes = ["Aparência", "Exibição", "Documento", "Salvamento", "Atalhos", "Sobre"];
        let menu = feather_ui::configuracoes::menu(
            "Configurações",
            secoes.iter().enumerate().map(|(i, nome)| (
                (*nome).to_owned(),
                self.config_secao == i as u8,
                Msg::ConfigSecao(i as u8),
            )),
        );
        let opcao = |rotulo: &str, ativa: bool, mensagem: Msg| {
            button(text(rotulo.to_owned()).size(12.5)).padding([6, 13])
                .style(estilo_botao_pequeno(ativa)).on_press(mensagem)
        };
        let cores = ACENTOS.iter().enumerate().fold(row![].spacing(8), |itens, (i, (nome, _, _))| {
            itens.push(opcao(nome, self.config.acento == i, Msg::Acento(i)))
        });
        let corpo: Element<'_, Msg> = match self.config_secao {
            0 => column![
                feather_ui::configuracoes::linha("Tema", "Automático usa a aparência do sistema na abertura.",
                    row![
                        opcao("Automático", self.config.tema_auto, Msg::TemaAuto),
                        opcao("Claro", !self.config.tema_auto && !self.config.escuro, Msg::Escuro(false)),
                        opcao("Escuro", !self.config.tema_auto && self.config.escuro, Msg::Escuro(true))
                    ].spacing(4)),
                feather_ui::configuracoes::linha("Cor de destaque", "Acento para botões, abas e seleção.", cores),
                feather_ui::configuracoes::linha("Logotipo", "Azul de fábrica ou cor escolhida",
                    opcao("Restaurar azul padrão", false, Msg::LogoPadrao)),
                feather_ui::configuracoes::linha("Barra de título integrada",
                    "Minimizar, maximizar e fechar na barra própria. Aplica-se na próxima abertura.",
                    toggler(self.config.barra_propria).on_toggle(Msg::BarraPropria)),
            ].spacing(10).into(),
            1 => column![
                feather_ui::configuracoes::linha("Faixa de opções", "Ctrl+F1 mostra ou recolhe a faixa.",
                    toggler(!self.config.faixa_recolhida).on_toggle(|_| Msg::AlternarFaixa)),
                feather_ui::configuracoes::linha("Estrutura", "Painel com os títulos do documento.",
                    toggler(self.config.estrutura).on_toggle(|_| Msg::Comando(Comando::Estrutura))),
                feather_ui::configuracoes::linha("Marcas de formatação", "Mostrar os símbolos invisíveis de edição.",
                    toggler(self.sessao.estado().marcas).on_toggle(|_| Msg::Comando(Comando::Marcas))),
            ].spacing(10).into(),
            2 => column![
                feather_ui::configuracoes::linha("Documento", "Papel e paginação calculados pelo motor Rust.",
                    text(format!("{} · {} páginas", self.sessao.estado().papel, self.sessao.total_de_paginas())).size(12.5)),
                feather_ui::configuracoes::linha("Estrutura", "Exibir títulos para navegação.",
                    toggler(self.config.estrutura).on_toggle(|_| Msg::Comando(Comando::Estrutura))),
            ].spacing(10).into(),
            3 => column![
                feather_ui::configuracoes::linha("Arquivo nativo", "O FeatherDocs salva localmente em .fdoz.",
                    text(if self.sessao.estado().sujo { "Alterações pendentes" } else { "Salvo" }).size(12.5)),
                feather_ui::configuracoes::linha("Salvar", "Também é possível exportar para Word, Markdown e LaTeX.",
                    opcao("Salvar agora", false, Msg::Comando(Comando::Salvar))),
            ].spacing(10).into(),
            4 => column![
                text("Atalhos de teclado").size(17),
                text("Ctrl+N Novo · Ctrl+O Abrir · Ctrl+S Salvar · Ctrl+Shift+S Salvar como"),
                text("Ctrl+F Buscar · Ctrl+Z Desfazer · F1 Ajuda · F12 Monitor · Ctrl+F1 Recolher faixa"),
                text("A personalização das combinações ainda será integrada ao registro global da suíte.")
                    .size(12).color(COR_TEXTO_MUTED()),
            ].spacing(12).into(),
            _ => column![
                text("FeatherDocs").size(22),
                text("Editor de documentos da suíte FeatherOffice, nativo em Rust/Iced."),
                text("Arquivos e operações são locais. Não há conta, telemetria ou envio automático de conteúdo."),
                text("Tema, botões, menus e configurações vêm da biblioteca feather-ui."),
            ].spacing(12).into(),
        };
        feather_ui::configuracoes::tela(
            menu,
            secoes[self.config_secao.min(5) as usize],
            corpo,
            None,
        )
    }

    fn view_menu(&self) -> Element<'_, Msg> {
        let menu = feather_ui::cromo::menu_arquivos("FeatherDocs", [
            ("file-plus", "Novo documento", Msg::Comando(Comando::Novo)),
            ("folder-open", "Abrir", Msg::Comando(Comando::Abrir)),
            ("device-floppy", "Salvar", Msg::Comando(Comando::Salvar)),
            ("device-floppy", "Salvar como", Msg::Comando(Comando::SalvarComo)),
            ("file-export", "Exportar", Msg::Comando(Comando::Exportar)),
            ("info-circle", "Propriedades", Msg::FormAbrir(crate::dialogos::Tipo::Propriedades)),
            ("settings", "Configurações", Msg::Comando(Comando::Configuracoes)),
            ("help", "Ajuda (F1)", Msg::Comando(Comando::Ajuda)),
            ("device-desktop", "Monitor (F12)", Msg::Comando(Comando::Monitor)),
            ("x", "Fechar menu", Msg::Cancelar),
        ]);
        if self.config.recentes.is_empty() {
            return menu;
        }
        let mut recentes = column![text("Recentes").size(12).color(COR_TEXTO_MUTED())].spacing(4);
        for p in &self.config.recentes {
            let nome = p.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string();
            let pasta = p.parent().map(|d| d.display().to_string()).unwrap_or_default();
            recentes = recentes.push(
                button(column![text(nome).size(13), text(pasta).size(10).color(COR_TEXTO_MUTED())])
                    .width(Length::Fill).padding([6, 12])
                    .style(estilo_botao_pequeno(false))
                    .on_press(Msg::AbrirRecente(p.clone())),
            );
        }
        column![
            menu,
            container(scrollable(recentes.padding(14))).width(280).height(Length::Fill)
                .style(estilo_painel_miniaturas),
        ].into()
    }

    fn view_ajuda(&self) -> Element<'_, Msg> {
        feather_ui::cromo::painel_utilitario(
            "Ajuda · FeatherDocs",
            ajuda::modelo().view(&self.ajuda_estado).map(Msg::AjudaEvento),
            Msg::FecharAuxiliar(JanelaAux::Ajuda),
        )
    }

    fn view_monitor(&self) -> Element<'_, Msg> {
        let e = self.sessao.estado();
        let memoria = std::fs::read_to_string("/proc/self/status").ok()
            .and_then(|s| s.lines().find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|n| n.parse::<f64>().ok()))
            .map(|kb| format!("{:.1} MiB de RAM no processo", kb / 1024.0))
            .unwrap_or_else(|| "Memória do processo: indisponível neste sistema".to_owned());
        feather_ui::cromo::painel_utilitario(
            "Monitor · FeatherDocs (F12)",
            column![
                text(memoria).size(14),
                text(format!("Versão do documento: {} · Páginas: {} · Blocos: {} · Palavras: {} · Caracteres: {}",
                    e.versao, e.paginas, e.blocos, e.palavras, e.caracteres)).size(13),
                text(format!("Modo: {} · Revisões: {} · Arquivo: {}",
                    e.modo_edicao, e.tem_revisoes, if e.sujo { "não salvo" } else { "salvo" })).size(13),
                text("F12 alterna o monitor. Não interrompe o conteúdo aberto.")
                    .size(12).color(COR_TEXTO_MUTED()),
            ].spacing(12).into(),
            Msg::FecharAuxiliar(JanelaAux::Monitor),
        )
    }

    fn view_busca(&self) -> Element<'_, Msg> {
        container(
            column![
                row![
                    text_input(tr("consulta"), &self.consulta)
                        .on_input(Msg::Consulta)
                        .on_submit(Msg::BuscaIr(1))
                        .padding(7),
                    button(text(tr("anterior"))).on_press(Msg::BuscaIr(-1)),
                    button(text(tr("proximo"))).on_press(Msg::BuscaIr(1)),
                    button(icone("x", 16.)).on_press(Msg::Cancelar)
                ]
                .spacing(7),
                row![
                    text_input(tr("substituicao"), &self.substituicao)
                        .on_input(Msg::Substituicao)
                        .padding(7),
                    button(text(tr("substituir"))).on_press(Msg::Substituir(false)),
                    button(text(tr("substituir_tudo"))).on_press(Msg::Substituir(true))
                ]
                .spacing(7),
                row![
                    iced::widget::checkbox("Diferenciar maiúsculas", self.busca_opcoes.maiusculas)
                        .on_toggle(|_| Msg::BuscaOpcao(0)).size(14).text_size(12),
                    iced::widget::checkbox("Palavra inteira", self.busca_opcoes.palavra)
                        .on_toggle(|_| Msg::BuscaOpcao(1)).size(14).text_size(12),
                    iced::widget::checkbox("Expressão regular", self.busca_opcoes.regex)
                        .on_toggle(|_| Msg::BuscaOpcao(2)).size(14).text_size(12),
                ]
                .spacing(16)
            ]
            .spacing(5),
        )
        .padding([8, 14])
        .style(estilo_painel_faixa)
        .into()
    }
}
fn nome_tecla(k: &keyboard::Key) -> String {
    use keyboard::{key::Named, Key};
    match k {
        Key::Character(c) => c.to_string(),
        Key::Named(n) => match n {
            Named::ArrowLeft => "ArrowLeft",
            Named::ArrowRight => "ArrowRight",
            Named::ArrowUp => "ArrowUp",
            Named::ArrowDown => "ArrowDown",
            Named::Enter => "Enter",
            Named::Backspace => "Backspace",
            Named::Delete => "Delete",
            Named::Home => "Home",
            Named::End => "End",
            Named::PageUp => "PageUp",
            Named::PageDown => "PageDown",
            Named::Tab => "Tab",
            Named::Escape => "Escape",
            _ => "",
        }
        .into(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ui013_ajuda_nativa_pesquisa_e_navega_sem_editar_documento() {
        let mut app=App::default();
        let antes=app.sessao.documento().clone();
        let _=app.update(Msg::AjudaEvento(feather_ui::ajuda::Evento::IrSecao(1)));
        assert_eq!(app.ajuda_estado.secao,1);
        let _=app.update(Msg::AjudaEvento(
            feather_ui::ajuda::Evento::Buscar("Ctrl+Shift+V".into())
        ));
        assert_eq!(app.ajuda_estado.busca,"Ctrl+Shift+V");
        assert!(ajuda::modelo().quantidade_encontrada(&app.ajuda_estado.busca)>0);
        let _=app.update(Msg::AjudaEvento(feather_ui::ajuda::Evento::Anterior));
        assert!(app.ajuda_estado.busca.is_empty());
        assert_eq!(*app.sessao.documento(),antes);
    }
    #[test]
    fn ui010_foco_por_clique_impede_digitacao_fora_do_documento() {
        let mut app = App::default();
        let documento = app.sessao.documento().clone();
        app.foco_documento = false;
        let _ = app.update(Msg::Tecla(
            keyboard::Key::Character("z".into()),
            keyboard::Modifiers::default(), Some("z".into()),
        ));
        assert_eq!(app.sessao.documento(), &documento);

        // Mouse ativa foco do editor, mas não insere nem modifica texto sozinho.
        let _ = app.update(Msg::Clicar(0, 20.0, 20.0, false, 1));
        assert!(app.foco_documento);
        assert_eq!(app.sessao.documento(), &documento);
    }

    #[test]
    fn ui010_atalho_de_janela_auxiliar_nao_modifica_o_documento() {
        use keyboard::{key::Named, Key, Modifiers};
        let mut app = App::default();
        let principal = window::Id::unique();
        app.janelas.principal = Some(principal);
        let original = app.sessao.documento().clone();
        let _ = app.update(Msg::AtalhoJanela(principal, Key::Named(Named::F1), Modifiers::default()));
        let ajuda = app.janelas.id(JanelaAux::Ajuda).expect("Ajuda deve abrir");
        let _ = app.update(Msg::AtalhoJanela(ajuda, Key::Named(Named::F12), Modifiers::default()));
        assert!(app.janelas.aberta(JanelaAux::Ajuda), "F12 não fecha a Ajuda");
        assert!(!app.janelas.aberta(JanelaAux::Monitor), "F12 não vaza da janela auxiliar");
        let _ = app.update(Msg::AtalhoJanela(ajuda, Key::Named(Named::Escape), Modifiers::default()));
        assert!(!app.janelas.aberta(JanelaAux::Ajuda));
        assert_eq!(app.sessao.documento(), &original);
    }
    #[test]
    fn pincel_na_ribbon_ativa_e_esc_cancela_sem_alterar_documento() {
        let mut app = App::default();
        let antes = app.sessao.documento().clone();
        let _ = app.update(Msg::Comando(Comando::Pincel));
        assert!(app.sessao.pincel_cheio());
        assert_eq!(app.sessao.documento(), &antes);
        let _ = app.update(Msg::Cancelar);
        assert!(!app.sessao.pincel_cheio());
        assert_eq!(app.sessao.documento(), &antes);
    }

    #[test]
    fn pincel_sem_selecao_cancela_no_segundo_clique() {
        let mut app = App::default();
        let _ = app.update(Msg::Comando(Comando::Pincel));
        assert!(app.sessao.pincel_cheio());
        let _ = app.update(Msg::Comando(Comando::Pincel));
        assert!(!app.sessao.pincel_cheio());
        assert!(app.status.contains("cancelado"));
    }

    #[test]
    fn atalhos_globais_preservam_conteudo_e_auxiliares_independentes() {
        use iced::keyboard::{key::Named, Key, Modifiers};
        let mut app = App::default();
        let principal = window::Id::unique();
        app.janelas.principal = Some(principal);
        let original = app.sessao.documento().clone();
        let _ = app.update(Msg::AtalhoJanela(principal, Key::Named(Named::F1), Modifiers::default()));
        assert!(app.janelas.aberta(JanelaAux::Ajuda));
        let id = app.janelas.id(JanelaAux::Ajuda).unwrap();
        let _ = app.update(Msg::AtalhoJanela(id, Key::Named(Named::F1), Modifiers::default()));
        assert!(!app.janelas.aberta(JanelaAux::Ajuda));
        let _ = app.update(Msg::AtalhoJanela(principal, Key::Named(Named::F12), Modifiers::default()));
        assert!(app.janelas.aberta(JanelaAux::Monitor));
        let id = app.janelas.id(JanelaAux::Monitor).unwrap();
        let _ = app.update(Msg::AtalhoJanela(id, Key::Named(Named::Escape), Modifiers::default()));
        assert!(!app.janelas.aberta(JanelaAux::Monitor));
        let antes = app.config.faixa_recolhida;
        let _ = app.update(Msg::AtalhoJanela(principal, Key::Named(Named::F1), Modifiers::CTRL));
        assert_ne!(app.config.faixa_recolhida, antes);
        assert_eq!(app.sessao.documento(), &original);
    }
    #[test]
    fn editor_e_janelas_auxiliares_rendem_independentes() {
        let mut app = App::default();
        let principal = window::Id::unique();
        app.janelas.principal = Some(principal);
        for aba in Aba::TODAS {
            app.aba = aba;
            let _ = app.view(principal);
        }
        for tipo in [JanelaAux::Configuracoes, JanelaAux::Ajuda, JanelaAux::Monitor] {
            let id = window::Id::unique();
            app.janelas.registrar(id, tipo);
            assert!(app.janelas.aberta(tipo));
            let _ = app.view(id);
            app.janelas.retirar(id);
        }
        app.busca = true;
        app.config.zoom = 2.;
        let _ = app.view(principal);
        app.pendente = Some(Destino::Novo);
        let _ = app.view(principal);
    }
    fn digitar(app: &mut App, s: &str) {
        let _ = app.update(Msg::Tecla(
            keyboard::Key::Character(s.into()),
            keyboard::Modifiers::default(),
            Some(s.into()),
        ));
    }
    #[test]
    fn cromo_global_conserva_documento_e_fecha_apenas_auxiliares() {
        let mut app = App::default();
        let principal = window::Id::unique();
        app.janelas.principal = Some(principal);
        digitar(&mut app, "Preservar documento");
        let original = app.sessao.documento().clone();
        let _ = app.update(Msg::Comando(Comando::Menu));
        assert!(app.menu_aberto);
        let _ = app.update(Msg::Comando(Comando::Configuracoes));
        assert!(app.janelas.aberta(JanelaAux::Configuracoes));
        assert!(!app.menu_aberto);
        let config_id = app.janelas.id(JanelaAux::Configuracoes).unwrap();
        let _ = app.view(config_id);
        let _ = app.update(Msg::ConfigSecao(4));
        let _ = app.view(config_id);
        let _ = app.update(Msg::AlternarFaixa);
        assert!(app.config.faixa_recolhida);
        let _ = app.update(Msg::Comando(Comando::Ajuda));
        assert!(app.janelas.aberta(JanelaAux::Ajuda));
        assert!(app.janelas.aberta(JanelaAux::Configuracoes));
        let ajuda_id = app.janelas.id(JanelaAux::Ajuda).unwrap();
        let _ = app.view(ajuda_id);
        let _ = app.update(Msg::AtalhoJanela(ajuda_id,
            keyboard::Key::Named(keyboard::key::Named::Escape),
            keyboard::Modifiers::default()));
        assert!(!app.janelas.aberta(JanelaAux::Ajuda));
        let _ = app.update(Msg::Comando(Comando::Monitor));
        assert!(app.janelas.aberta(JanelaAux::Monitor));
        let monitor_id = app.janelas.id(JanelaAux::Monitor).unwrap();
        let _ = app.view(monitor_id);
        let _ = app.update(Msg::Fechar(monitor_id));
        assert!(!app.janelas.aberta(JanelaAux::Monitor));
        assert!(app.janelas.aberta(JanelaAux::Configuracoes));
        assert_eq!(app.janelas.principal, Some(principal));
        assert_eq!(app.sessao.documento(), &original);
    }
    #[test]
    fn edicao_nativa_preserva_unicode_formato_e_historico() {
        let mut app = App::default();
        let _ = app.update(Msg::Comando(Comando::Formato(
            featherdocs_core::modelo::NEGRITO,
        )));
        digitar(&mut app, "Olá Rust");
        assert_eq!(
            featherdocs_io::exportar::texto_simples(app.sessao.documento()),
            "Olá Rust"
        );
        assert!(app.visao.formato.negrito);
        assert!(app.quadros[0].operacoes.iter().any(|o| matches!(o,Operacao::Texto {estilo,..} if estilo & featherdocs_core::modelo::NEGRITO != 0)));
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_eq!(
            featherdocs_io::exportar::texto_simples(app.sessao.documento()),
            ""
        );
        let _ = app.update(Msg::Comando(Comando::Refazer));
        assert_eq!(
            featherdocs_io::exportar::texto_simples(app.sessao.documento()),
            "Olá Rust"
        );
    }
    #[test]
    fn novo_e_fechamento_pedem_salvamento_e_cancelar_preserva_edicoes() {
        let mut app = App::default();
        digitar(&mut app, "Não perder");
        let antes = app.sessao.documento().clone();
        let _ = app.update(Msg::Comando(Comando::Novo));
        assert!(matches!(app.pendente, Some(Destino::Novo)));
        let _ = app.update(Msg::Fonte("serifa".into()));
        let _ = app.update(Msg::Cancelar);
        assert_eq!(app.sessao.documento(), &antes);
        let principal = window::Id::unique();
        app.janelas.principal = Some(principal);
        let _ = app.update(Msg::Fechar(principal));
        assert!(matches!(app.pendente, Some(Destino::Fechar(id)) if id == principal));
        let _ = app.update(Msg::Cancelar);
        assert_eq!(app.sessao.documento(), &antes);
    }
    #[test]
    fn qa009_cancelar_abertura_e_erro_preservam_documento_sujo() {
        let mut app = App::default();
        digitar(&mut app, "Texto inalterado no cancelamento");
        let antes = app.sessao.documento().clone();
        app.ocupado = true;
        let _ = app.update(Msg::Aberto(Ok(None)));
        assert!(!app.ocupado);
        assert_eq!(app.sessao.documento(), &antes);
        assert!(app.status.contains("cancelada"));
        app.ocupado = true;
        let _ = app.update(Msg::Aberto(Err("Documento corrompido".into())));
        assert!(!app.ocupado);
        assert_eq!(app.sessao.documento(), &antes);
        assert!(app.status.contains("corrompido"));
    }

    #[test]
    fn abrir_falha_e_exportar_nao_apagam_documento_sujo() {
        let mut app = App::default();
        digitar(&mut app, "Texto original");
        let antes = app.sessao.documento().clone();
        let _ = app.update(Msg::Aberto(Err("Arquivo inválido".into())));
        assert_eq!(app.sessao.documento(), &antes);
        let v = app.sessao.versao();
        let _ = app.update(Msg::Gravado(Ok(Some((
            PathBuf::from("copia.docx"),
            v,
            true,
        )))));
        assert!(app.sessao.estado().sujo);
        assert!(app.caminho.is_none());
        let _ = app.update(Msg::Gravado(Ok(Some((
            PathBuf::from("documento.fdoz"),
            v,
            false,
        )))));
        assert!(!app.sessao.estado().sujo);
    }
    #[test]
    fn busca_substituicao_e_tabela_usam_motor() {
        let mut app = App::default();
        digitar(&mut app, "ação ação");
        let _ = app.update(Msg::Consulta("ação".into()));
        let _ = app.update(Msg::Substituicao("edição".into()));
        let _ = app.update(Msg::Substituir(true));
        assert_eq!(
            featherdocs_io::exportar::texto_simples(app.sessao.documento()),
            "edição edição"
        );
        let _ = app.update(Msg::Cancelar);
        let _ = app.update(Msg::Comando(Comando::Tabela));
        assert!(app.sessao.documento().blocos.iter().any(|b| matches!(
            b.conteudo,
            featherdocs_core::modelo::Conteudo::Tabela { .. }
        )));
        let bytes = featherdocs_io::fdoz::gravar(app.sessao.documento()).unwrap();
        assert_eq!(
            featherdocs_io::fdoz::ler(&bytes).unwrap(),
            *app.sessao.documento()
        );
    }
    #[test]
    fn tabela_no_ribbon_usa_tables_e_preserva_undo_e_fdoz() {
        let mut app = App::default();
        let _ = app.update(Msg::Comando(Comando::Tabela));
        let original = app.sessao.documento().clone();
        let paginas = app.sessao.total_de_paginas();
        let _ = app.update(Msg::Comando(Comando::TabelaEstrutura("linha+")));
        let apos_linha = app.sessao.documento().clone();
        assert_ne!(apos_linha, original);
        let _ = app.update(Msg::Comando(Comando::TabelaEstrutura("coluna+")));
        let atualizado = app.sessao.documento().clone();
        assert_ne!(atualizado, apos_linha);
        assert!(app.sessao.total_de_paginas() >= paginas);
        assert_eq!(
            featherdocs_io::fdoz::ler(&featherdocs_io::fdoz::gravar(&atualizado).unwrap()).unwrap(),
            atualizado
        );
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_eq!(app.sessao.documento(), &apos_linha);
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_eq!(app.sessao.documento(), &original);
        let _ = app.update(Msg::Comando(Comando::Refazer));
        assert_eq!(app.sessao.documento(), &apos_linha);
        let _ = app.update(Msg::Comando(Comando::Refazer));
        assert_eq!(app.sessao.documento(), &atualizado);
    }

    #[test]
    fn mesclar_e_desmesclar_na_gui_preservam_undo_e_fdoz() {
        let mut app = App::default();
        let _ = app.update(Msg::Comando(Comando::Tabela));
        let original = app.sessao.documento().clone();
        let _ = app.update(Msg::Comando(Comando::TabelaMesclar));
        let mesclado = app.sessao.documento().clone();
        assert_ne!(mesclado, original);
        assert_eq!(mesclado.blocos.iter()
            .find(|b| matches!(b.conteudo, featherdocs_core::modelo::Conteudo::Tabela { .. }))
            .unwrap().mesclagens.len(), 1);
        let bytes = featherdocs_io::fdoz::gravar(&mesclado).unwrap();
        assert_eq!(featherdocs_io::fdoz::ler(&bytes).unwrap(), mesclado);
        let _ = app.update(Msg::Comando(Comando::TabelaDesmesclar));
        assert_eq!(app.sessao.documento(), &original);
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_eq!(app.sessao.documento(), &mesclado);
        let _ = app.update(Msg::Comando(Comando::Refazer));
        assert_eq!(app.sessao.documento(), &original);
    }

    #[test]
    fn app006_cabecalho_exclusao_undo_e_round_trip_fdoz() {
        let mut app = App::default();
        let _ = app.update(Msg::Comando(Comando::Tabela));
        let original = app.sessao.documento().clone();
        assert!(app.sessao.tabela_tem_cabecalho());
        assert!(app.visao.formato.tabela.is_some());

        let _ = app.update(Msg::Comando(Comando::TabelaCabecalho));
        assert!(!app.sessao.tabela_tem_cabecalho());
        let sem_cabecalho = app.sessao.documento().clone();
        assert_ne!(original, sem_cabecalho);
        let fdoz = featherdocs_io::fdoz::gravar(&sem_cabecalho).unwrap();
        assert_eq!(featherdocs_io::fdoz::ler(&fdoz).unwrap(), sem_cabecalho);

        let _ = app.update(Msg::Comando(Comando::TabelaExcluir));
        assert!(app.sessao.documento().blocos.iter().all(|b|
            !matches!(b.conteudo, featherdocs_core::modelo::Conteudo::Tabela { .. })
        ));
        // O contador de ids não volta no desfazer (evita reaproveitar id); o conteúdo volta exato.
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_eq!(app.sessao.documento().blocos, sem_cabecalho.blocos);
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_eq!(app.sessao.documento().blocos, original.blocos);
        let _ = app.update(Msg::Comando(Comando::Refazer));
        assert_eq!(app.sessao.documento().blocos, sem_cabecalho.blocos);
    }

    #[test]
    fn app006_edicao_estrutural_preserva_trechos_ricos_e_paginacao() {
        use featherdocs_core::modelo::{Conteudo, Documento, Trecho, NEGRITO};
        let mut negrito = Trecho::simples("Cabeçalho");
        negrito.estilo = NEGRITO;
        negrito.link = Some("https://example.org".into());
        let mut documento = Documento::vazio();
        let id = documento.acrescentar(Conteudo::Tabela {
            cabecalho: true,
            linhas: vec![
                vec![vec![negrito.clone()], vec![Trecho::simples("Dado")]],
                vec![vec![Trecho::simples("Linha 2")], vec![]],
            ],
        });
        let mut app = App::default();
        app.sessao = Sessao::com_documento(documento);
        app.atualizar();
        let antes = app.sessao.documento().clone();
        let paginas_antes = app.sessao.total_de_paginas();
        assert_eq!(app.sessao.cursor().foco.bloco, id);
        assert_eq!(app.sessao.cursor().foco.celula, Some((0, 0)));

        let _ = app.update(Msg::Comando(Comando::TabelaEstrutura("coluna+")));
        let _ = app.update(Msg::Comando(Comando::TabelaEstrutura("linha+")));
        let depois = app.sessao.documento().clone();
        let bloco = depois.blocos.iter().find(|b| b.id == id).unwrap();
        let Conteudo::Tabela { linhas, cabecalho } = &bloco.conteudo else {
            panic!("O bloco deixou de ser tabela.");
        };
        assert!(*cabecalho);
        assert_eq!((linhas.len(), linhas[0].len()), (3, 3));
        assert_eq!(linhas[0][0], vec![negrito]);
        assert_eq!(linhas[0][2][0].texto, "Dado");
        assert_eq!(linhas[2][0][0].texto, "Linha 2");
        assert!(app.sessao.total_de_paginas() >= paginas_antes);
        assert!(app.visao.formato.tabela.is_some());

        let fdoz = featherdocs_io::fdoz::gravar(&depois).unwrap();
        assert_eq!(featherdocs_io::fdoz::ler(&fdoz).unwrap(), depois);
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_eq!(app.sessao.documento(), &antes);
        let _ = app.update(Msg::Comando(Comando::Refazer));
        let _ = app.update(Msg::Comando(Comando::Refazer));
        assert_eq!(app.sessao.documento(), &depois);
    }

    #[test]
    fn app006_mescla_rejeita_celula_ocupada_sem_apagar_dados() {
        use featherdocs_core::modelo::{Conteudo, Documento, Trecho};
        let mut documento = Documento::vazio();
        documento.acrescentar(Conteudo::Tabela {
            cabecalho: true,
            linhas: vec![vec![
                vec![Trecho::simples("A")],
                vec![Trecho::simples("B")],
            ]],
        });
        let mut app = App::default();
        app.sessao = Sessao::com_documento(documento);
        app.atualizar();
        let antes = app.sessao.documento().clone();
        let _ = app.update(Msg::Comando(Comando::TabelaMesclar));
        assert_eq!(app.sessao.documento(), &antes);
        assert!(!app.status.is_empty(), "a GUI deve explicar por que a mesclagem falhou");
        let _ = app.update(Msg::Comando(Comando::TabelaCabecalho));
        assert_ne!(app.sessao.documento(), &antes);
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_eq!(app.sessao.documento(), &antes);
    }

    #[test]
    fn img014_preserva_id_legenda_fdoz_e_undo_da_imagem() {
        use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
        use std::io::Cursor;
        let mut png = Vec::new();
        let pixels = RgbaImage::from_fn(2, 2, |x, y| Rgba([
            if x == 0 { 255 } else { 0 }, if y == 0 { 0 } else { 255 }, 0, 128
        ]));
        DynamicImage::ImageRgba8(pixels)
            .write_to(&mut Cursor::new(&mut png), ImageFormat::Png).unwrap();
        let (origem, w, h) = crate::imagem_compartilhada::preparar_insercao(&png).unwrap();
        let mut app = App::default();
        app.sessao.inserir_imagem(&origem, w, h).unwrap();
        let id = app.sessao.documento().blocos.iter()
            .find(|b| matches!(b.conteudo, featherdocs_core::modelo::Conteudo::Imagem { .. }))
            .unwrap().id;
        app.sessao.ir_para_bloco(id);
        app.atualizar();
        let original = app.sessao.documento().clone();
        let _ = app.update(Msg::AbrirEfeitosImagem);
        assert!(app.efeito_imagem.is_some());
        let _ = app.update(Msg::AplicarEfeitoImagem("cinza"));
        let alterado = app.sessao.documento().clone();
        assert_ne!(alterado, original);
        assert!(alterado.blocos.iter().any(|b| b.id == id));
        let bytes = featherdocs_io::fdoz::gravar(&alterado).unwrap();
        assert_eq!(featherdocs_io::fdoz::ler(&bytes).unwrap(), alterado);
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_eq!(app.sessao.documento(), &original);
        let _ = app.update(Msg::Comando(Comando::Refazer));
        assert_eq!(app.sessao.documento(), &alterado);
        let _ = app.update(Msg::AplicarEfeitoImagem("original"));
        assert_eq!(app.sessao.documento(), &original);
        let _ = app.update(Msg::FecharEfeitosImagem);
        assert!(app.efeito_imagem.is_none());
    }

    #[test]
    fn img014_sem_imagem_nao_modifica_documento() {
        let mut app = App::default();
        let original = app.sessao.documento().clone();
        let _ = app.update(Msg::AbrirEfeitosImagem);
        assert!(app.efeito_imagem.is_none());
        let _ = app.update(Msg::AplicarEfeitoImagem("cinza"));
        assert_eq!(app.sessao.documento(), &original);
    }

    #[test]
    fn app008_seletor_compartilhado_edita_texto_realce_e_preserva_fdoz_undo() {
        use feather_ui::seletor_cor::Evento;
        let mut app = App::default();
        digitar(&mut app, "Texto colorido");
        app.sessao.selecionar_tudo();
        app.atualizar();
        let antes = app.sessao.documento().clone();

        let _ = app.update(Msg::AbrirCor(AlvoCor::Texto));
        assert_eq!(app.cor_ui.alvo, Some(AlvoCor::Texto));
        let _ = app.update(Msg::EventoCor(Evento::Escolhida("#112233".into())));
        assert_eq!(app.visao.formato.cor, "#112233");

        let _ = app.update(Msg::AbrirCor(AlvoCor::Realce));
        let _ = app.update(Msg::EventoCor(Evento::Escolhida("#ffee00".into())));
        assert_eq!(app.visao.formato.realce, "#ffee00");
        let colorido = app.sessao.documento().clone();
        assert_ne!(colorido, antes);
        let bytes = featherdocs_io::fdoz::gravar(&colorido).unwrap();
        assert_eq!(featherdocs_io::fdoz::ler(&bytes).unwrap(), colorido);
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_ne!(app.sessao.documento(), &colorido);
        let _ = app.update(Msg::Comando(Comando::Refazer));
        assert_eq!(app.sessao.documento(), &colorido);
    }

    #[test]
    fn app008_hex_invalido_e_fechar_nao_modificam_documento() {
        use feather_ui::seletor_cor::Evento;
        let mut app = App::default();
        digitar(&mut app, "Teste");
        app.sessao.selecionar_tudo();
        app.atualizar();
        let antes = app.sessao.documento().clone();
        let _ = app.update(Msg::AbrirCor(AlvoCor::Texto));
        let _ = app.update(Msg::EventoCor(Evento::TextoHex("#zz".into())));
        assert_eq!(app.sessao.documento(), &antes);
        let _ = app.update(Msg::EventoCor(Evento::Fechar));
        assert_eq!(app.cor_ui.alvo, None);
        assert_eq!(app.sessao.documento(), &antes);
        let _ = app.update(Msg::AbrirCor(AlvoCor::Realce));
        let _ = app.update(Msg::Cancelar);
        assert_eq!(app.cor_ui.alvo, None);
        assert_eq!(app.sessao.documento(), &antes);
    }

    #[test]
    fn cht016_ribbon_insere_grafico_real_e_reabre_fdoz() {
        use featherdocs_core::modelo::{Conteudo, Documento, Trecho};
        use featherdocs_io::grafico_docs::{eh_grafico, Tipo};
        let celula = |s: &str| vec![Trecho::simples(s)];
        let mut doc = Documento::vazio();
        doc.acrescentar(Conteudo::Tabela {
            cabecalho: true,
            linhas: vec![
                vec![celula("Mês"), celula("Consumo")],
                vec![celula("Jan"), celula("12,5")],
                vec![celula("Fev"), celula("13")],
            ],
        });
        let mut app = App::default();
        app.sessao = Sessao::com_documento(doc);
        app.atualizar();
        let _ = app.update(Msg::Comando(Comando::Grafico(Tipo::Colunas)));
        let grafico = app.sessao.documento().blocos.iter()
            .find(|b| matches!(b.conteudo, Conteudo::Imagem { .. }))
            .expect("a Ribbon não inseriu um bloco de gráfico");
        let id = grafico.id;
        let Conteudo::Imagem { origem, .. } = &grafico.conteudo else { unreachable!() };
        assert!(eh_grafico(origem), "a Ribbon não utilizou o adaptador Charts");
        let salvo = featherdocs_io::fdoz::gravar(app.sessao.documento()).unwrap();
        assert_eq!(featherdocs_io::fdoz::ler(&salvo).unwrap(), *app.sessao.documento());

        app.sessao.ir_para_bloco(id);
        let versao_antes = app.sessao.versao();
        let _ = app.update(Msg::Comando(Comando::AtualizarGrafico));
        assert_eq!(app.sessao.versao(), versao_antes, "gráfico igual não deve sujar o documento");
        assert_eq!(app.sessao.documento().blocos.iter().filter(|b| b.id == id).count(), 1);
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        // Atualização idempotente não adiciona um passo; Undo desfaz a inserção.
        assert!(!app.sessao.documento().blocos.iter().any(|b| b.id == id));
        let _ = app.update(Msg::Comando(Comando::Refazer));
        assert!(app.sessao.documento().blocos.iter().any(|b| b.id == id));
    }

    /// Digita `texto` no fim do editor de código e faz a pausa passar.
    fn digitar_no_codigo(app: &mut App, texto: &str) {
        use iced::widget::text_editor::{Action, Edit, Motion};
        let _ = app.update(Msg::CodigoEditar(Action::Click(iced::Point::ORIGIN)));
        let _ = app.update(Msg::CodigoEditar(Action::Move(Motion::DocumentEnd)));
        let _ = app.update(Msg::CodigoEditar(Action::Edit(Edit::Paste(std::sync::Arc::new(texto.into())))));
    }
    fn vencer_pausa(app: &mut App) {
        if let Some(p) = app.codigo.as_mut() {
            p.ultima_edicao = p.ultima_edicao.map(|t| t - painel_codigo::PAUSA);
        }
        let _ = app.update(Msg::CodigoPulso);
    }

    #[test]
    fn codigo_abrir_gera_latex_e_alterna_markdown() {
        let mut app = App::default();
        digitar(&mut app, "Olá mundo");
        let _ = app.update(Msg::Comando(Comando::Codigo(Fonte::Latex)));
        let texto = app.codigo.as_ref().unwrap().conteudo.text();
        assert!(texto.contains("\\begin{document}"), "{texto}");
        assert!(texto.contains("Olá mundo"));
        assert!(!app.codigo.as_ref().unwrap().blocos.is_empty());
        let _ = app.update(Msg::Comando(Comando::Codigo(Fonte::Markdown)));
        assert_eq!(app.fonte_do_codigo(), Some(Fonte::Markdown));
        assert_eq!(app.codigo.as_ref().unwrap().conteudo.text().trim(), "Olá mundo");
        let principal = window::Id::unique();
        app.janelas.principal = Some(principal);
        let _ = app.view(principal);
        let _ = app.update(Msg::Comando(Comando::Codigo(Fonte::Markdown)));
        assert!(app.codigo.is_none(), "o mesmo botão desliga o painel");
    }

    #[test]
    fn codigo_editado_vai_ao_documento_depois_da_pausa_e_desfaz_num_passo() {
        let mut app = App::default();
        digitar(&mut app, "Primeiro");
        let _ = app.update(Msg::Comando(Comando::Codigo(Fonte::Markdown)));
        let antes = app.sessao.documento().clone();
        digitar_no_codigo(&mut app, "\n\n## Segundo título\n");
        assert!(!app.foco_documento, "editar o código tira o foco da página");
        assert!(app.codigo.as_ref().unwrap().pendente());
        // O pulso antes da pausa não aplica nada.
        let _ = app.update(Msg::CodigoPulso);
        assert_eq!(app.sessao.documento(), &antes);
        vencer_pausa(&mut app);
        assert!(!app.codigo.as_ref().unwrap().pendente());
        assert!(app.sessao.documento().blocos.iter().any(|b| matches!(
            &b.conteudo, featherdocs_core::modelo::Conteudo::Titulo { nivel: 2, .. })));
        assert!(app.status.contains("aplicado"), "{}", app.status);
        // O texto da pessoa continua como ela escreveu.
        assert!(app.codigo.as_ref().unwrap().conteudo.text().contains("## Segundo título"));
        // Ctrl+Z (pelo editor) desfaz a aplicação inteira e o código volta.
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_eq!(app.sessao.documento().blocos, antes.blocos);
        assert!(!app.codigo.as_ref().unwrap().conteudo.text().contains("Segundo"));
        let _ = app.update(Msg::Comando(Comando::Refazer));
        assert!(app.codigo.as_ref().unwrap().conteudo.text().contains("## Segundo título"));
    }

    #[test]
    fn codigo_pendente_e_aplicado_antes_de_editar_a_pagina() {
        let mut app = App::default();
        digitar(&mut app, "Base");
        let _ = app.update(Msg::Comando(Comando::Codigo(Fonte::Markdown)));
        digitar_no_codigo(&mut app, "\n\nDo código\n");
        // Clicar na página antes da pausa aplica o código primeiro.
        let _ = app.update(Msg::Clicar(0, 20.0, 20.0, false, 1));
        let texto = featherdocs_io::exportar::texto_simples(app.sessao.documento());
        assert!(texto.contains("Do código"), "{texto}");
        assert!(app.foco_documento);
    }

    #[test]
    fn codigo_regenera_quando_a_pagina_muda_e_segue_o_bloco_clicado() {
        let mut app = App::default();
        digitar(&mut app, "Um");
        let _ = app.update(Msg::Tecla(keyboard::Key::Named(keyboard::key::Named::Enter),
            keyboard::Modifiers::default(), None));
        digitar(&mut app, "Dois");
        let _ = app.update(Msg::Comando(Comando::Codigo(Fonte::Latex)));
        app.foco_documento = true;
        digitar(&mut app, " três");
        let painel = app.codigo.as_ref().unwrap();
        assert!(painel.conteudo.text().contains("Dois três"));
        assert_eq!(painel.versao, app.sessao.versao());
        // O cursor do código está na linha do bloco do cursor da página.
        let id = app.sessao.cursor().foco.bloco;
        let de = painel.blocos.iter().find(|b| b.id == id).unwrap().de;
        assert_eq!(painel.conteudo.cursor_position().0, de);
        assert_eq!(painel.conteudo.line(de).unwrap().trim(), "Dois três");
        // Ir ao primeiro bloco pela estrutura leva o código junto.
        let primeiro = app.sessao.documento().blocos[0].id;
        let _ = app.update(Msg::IrBloco(primeiro));
        let painel = app.codigo.as_ref().unwrap();
        let de = painel.blocos.iter().find(|b| b.id == primeiro).unwrap().de;
        assert_eq!(painel.conteudo.cursor_position().0, de);
    }

    #[test]
    fn codigo_clicado_leva_a_pagina_ao_bloco() {
        use iced::widget::text_editor::{Action, Motion};
        let mut app = App::default();
        digitar(&mut app, "Um");
        let _ = app.update(Msg::Tecla(keyboard::Key::Named(keyboard::key::Named::Enter),
            keyboard::Modifiers::default(), None));
        digitar(&mut app, "Dois");
        let primeiro = app.sessao.documento().blocos[0].id;
        let _ = app.update(Msg::Comando(Comando::Codigo(Fonte::Markdown)));
        assert_ne!(app.sessao.cursor().foco.bloco, primeiro);
        let _ = app.update(Msg::CodigoEditar(Action::Move(Motion::DocumentStart)));
        assert_eq!(app.sessao.cursor().foco.bloco, primeiro);
        assert!(!app.foco_documento);
    }

    #[test]
    fn colar_tex_no_painel_de_documento_vazio_da_o_mesmo_que_abrir_o_arquivo() {
        use iced::widget::text_editor::{Action, Edit};
        // O cabeçalho do currículo de teste do usuário: faixa azul com nome e subtítulo.
        let tex = "\\documentclass{article}\n\\usepackage{xcolor}\n\\definecolor{darkblue}{HTML}{1A3A5C}\n\\definecolor{lightblue}{HTML}{D6E8F7}\n\\begin{document}\n\\noindent\n\\colorbox{darkblue}{%\n  \\begin{minipage}{\\dimexpr\\textwidth-2\\fboxsep\\relax}\n    \\vspace{8pt}\n    \\centering\n    \\textbf{\\LARGE\\color{white}Marcelo A. Gonçalves}\\\\[3pt]\n    {\\color{lightblue}Engenheiro Eletricista}\n    \\vspace{8pt}\n  \\end{minipage}%\n}\\par\n\\begin{center}\n  {\\small Vitória -- ES}\n\\end{center}\nTexto do perfil.\n\\end{document}\n";
        let lido = featherdocs_io::latex::ler(tex);
        let mut app = App::default();
        let _ = app.update(Msg::Comando(Comando::Codigo(Fonte::Latex)));
        let _ = app.update(Msg::CodigoEditar(Action::SelectAll));
        let _ = app.update(Msg::CodigoEditar(Action::Edit(Edit::Paste(std::sync::Arc::new(tex.into())))));
        vencer_pausa(&mut app);
        let d = app.sessao.documento();
        assert_eq!(d.blocos.len(), lido.blocos.len());
        for (a, b) in d.blocos.iter().zip(&lido.blocos) {
            assert_eq!((&a.conteudo, a.atrib), (&b.conteudo, b.atrib));
        }
        assert_ne!(d.blocos[0].atrib.fundo, 0, "o nome fica na faixa (não some branco no branco)");
        assert_eq!(d.blocos[0].atrib.fundo, d.blocos[1].atrib.fundo, "uma faixa só para nome e subtítulo");
    }

    #[test]
    fn abrir_tex_mostra_o_texto_original_no_painel() {
        let tex = "\\documentclass{article}\n% meu comentário\n\\begin{document}\nTexto do arquivo.\n\\end{document}\n";
        let p = std::env::temp_dir().join(format!("featherdocs-codigo-{}.tex", std::process::id()));
        std::fs::write(&p, tex).unwrap();
        let doc = crate::arquivos::ler(&p).unwrap();
        let mut app = App::default();
        let _ = app.update(Msg::Comando(Comando::Codigo(Fonte::Markdown)));
        let _ = app.update(Msg::Aberto(Ok(Some((p.clone(), doc)))));
        std::fs::remove_file(&p).unwrap();
        let painel = app.codigo.as_ref().unwrap();
        assert_eq!(painel.fonte, Fonte::Latex, "o painel passa à língua do arquivo");
        assert_eq!(painel.conteudo.text(), tex.trim_end_matches('\n').to_string() + "\n");
        assert!(app.caminho.is_none(), "salvar continua pedindo um .fdoz");
    }

    #[test]
    fn formularios_aplicam_no_motor_e_erro_mantem_aberto() {
        use crate::dialogos::Tipo;
        let mut app = App::default();
        let _ = app.update(Msg::FormAbrir(Tipo::Margens));
        assert!(!app.foco_documento);
        let _ = app.update(Msg::FormCampo(0, "3,0".into()));
        let _ = app.update(Msg::FormOk);
        assert!(app.formulario.is_none() && app.foco_documento);
        assert!((app.sessao.documento().pagina.margem_esq - 3.0 * 72.0 / 2.54).abs() < 0.01);

        let _ = app.update(Msg::FormAbrir(Tipo::Margens));
        let _ = app.update(Msg::FormCampo(1, "abc".into()));
        let _ = app.update(Msg::FormOk);
        assert!(app.formulario.is_some(), "campo inválido não fecha o diálogo");
        let _ = app.update(Msg::Cancelar);
        assert!(app.formulario.is_none());

        let _ = app.update(Msg::FormAbrir(Tipo::Cabecalho));
        let _ = app.update(Msg::FormCampo(2, "{titulo}".into()));
        let _ = app.update(Msg::FormOk);
        assert_eq!(app.sessao.documento().extras.cabecalho.direita, "{titulo}");

        digitar(&mut app, "x");
        let _ = app.update(Msg::FormAbrir(Tipo::Link));
        let _ = app.update(Msg::FormCampo(0, "https://exemplo.org".into()));
        let _ = app.update(Msg::FormCampo(1, "site".into()));
        let _ = app.update(Msg::FormOk);
        assert!(app.formulario.is_none());
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        let _ = app.update(Msg::Comando(Comando::Desfazer));
        assert_eq!(app.sessao.documento().extras.cabecalho.direita, "{titulo}");
    }

    #[test]
    fn menu_do_botao_direito_executa_e_fecha() {
        let mut app = App::default();
        digitar(&mut app, "abc");
        let _ = app.update(Msg::Rolou(iced::Vector::new(0.0, 100.0)));
        let _ = app.update(Msg::BotaoDireito(0, 20.0, 20.0, iced::Point::new(50.0, 300.0)));
        assert_eq!(app.menu_contexto, Some(iced::Point::new(50.0, 200.0)));
        let _ = app.update(Msg::DoMenu(Box::new(Msg::FormAbrir(crate::dialogos::Tipo::Comentario))));
        assert!(app.menu_contexto.is_none() && app.formulario.is_some());
        let _ = app.update(Msg::Cancelar);
        let _ = app.update(Msg::BotaoDireito(0, 20.0, 20.0, iced::Point::new(50.0, 300.0)));
        let _ = app.update(Msg::Cancelar);
        assert!(app.menu_contexto.is_none());
    }

    #[test]
    fn nova_fonte_e_citacao_pelos_dialogos() {
        use crate::dialogos::Tipo;
        let mut app = App::default();
        let _ = app.update(Msg::FormAbrir(Tipo::NovaFonte));
        for (i, v) in [(0, "silva2023"), (1, "artigo"), (2, "Silva, Ana; Souza, Bia"), (3, "Um título"), (4, "2023")] {
            let _ = app.update(Msg::FormCampo(i, v.into()));
        }
        let _ = app.update(Msg::FormOk);
        assert!(app.formulario.is_none(), "{}", app.status);
        let f = &app.sessao.documento().bibliografia.fontes[0];
        assert_eq!((f.chave.as_str(), f.autores.len(), f.autores[1].nome.as_str()), ("silva2023", 2, "Bia"));

        let _ = app.update(Msg::FormAbrir(Tipo::Citacao));
        let _ = app.update(Msg::FormOk);
        assert!(app.formulario.is_some(), "sem fonte marcada não insere");
        let _ = app.update(Msg::FormCampo(0, "silva2023".into()));
        let _ = app.update(Msg::FormOk);
        assert!(app.formulario.is_none(), "{}", app.status);
        assert!(format!("{:?}", app.sessao.documento().blocos).contains("(SILVA; SOUZA, 2023)"));
    }

    #[test]
    fn nota_inserida_e_editada_pelo_dialogo() {
        use crate::dialogos::Tipo;
        let mut app = App::default();
        let _ = app.update(Msg::FormAbrir(Tipo::NotaRodape));
        let _ = app.update(Msg::FormCampo(0, "primeira".into()));
        let _ = app.update(Msg::FormOk);
        let bloco = app.sessao.documento().blocos[0].id;
        let pos = featherdocs_core::cursor::Pos { bloco, celula: None, desl: 0 };
        assert_eq!(app.sessao.nota_info(&pos).unwrap().fonte, "primeira");
        let _ = app.update(Msg::FormAbrir(Tipo::EditarNota(pos, false)));
        assert_eq!(app.formulario.as_ref().unwrap().campos[0], "primeira");
        let _ = app.update(Msg::FormCampo(0, "segunda".into()));
        let _ = app.update(Msg::FormOk);
        assert_eq!(app.sessao.nota_info(&pos).unwrap().fonte, "segunda");
    }

    #[test]
    fn novo_estilo_guarda_a_formatacao_e_exclui() {
        use crate::dialogos::Tipo;
        let mut app = App::default();
        digitar(&mut app, "abc");
        let _ = app.update(Msg::Comando(Comando::Alinhar("centro")));
        let _ = app.update(Msg::FormAbrir(Tipo::NovoEstilo));
        let _ = app.update(Msg::FormCampo(0, "Destaque".into()));
        let _ = app.update(Msg::FormOk);
        assert!(app.formulario.is_none(), "{}", app.status);
        let e = &app.sessao.documento().estilos.lista[0];
        assert_eq!((e.nome.as_str(), e.alinhamento), ("Destaque", featherdocs_core::modelo::Alinhamento::Centro));
        assert_eq!(app.sessao.estilo_com_nome_no_cursor(), e.id);
        let _ = app.update(Msg::Comando(Comando::EstiloExcluir));
        assert!(app.sessao.documento().estilos.lista.is_empty(), "{}", app.status);
    }

    #[test]
    fn recentes_sem_repetir_e_no_maximo_dez() {
        let mut c = crate::configuracao::Configuracao::default();
        for i in 0..12 {
            c.lembrar(std::path::Path::new(&format!("/tmp/{i}.fdoz")));
        }
        c.lembrar(std::path::Path::new("/tmp/5.fdoz"));
        assert_eq!(c.recentes.len(), 10);
        assert_eq!(c.recentes[0], PathBuf::from("/tmp/5.fdoz"));
        assert_eq!(c.recentes.iter().filter(|p| p.ends_with("5.fdoz")).count(), 1);
    }

    #[test]
    fn recursos_embutidos_e_localidade_cobrem_a_faixa() {
        let dados: serde_json::Value =
            serde_json::from_str(include_str!("../dados/pt-BR.json")).unwrap();
        for chave in [
            "inicio",
            "salvar",
            "negrito",
            "tabela",
            "configuracoes",
            "salvar_pendente",
        ] {
            assert!(dados.get(chave).is_some());
        }
        assert!(crate::pagina::carregar_imagem("https://exemplo.com/imagem.png").is_none());
        assert!(crate::pagina::carregar_imagem("/tmp/imagem.png").is_none());
    }
}
