//! A ponte: um despacho único (`Ponte::chamar`) casa o nome do comando com a
//! função do motor. Comando que muda algo devolve **a versão nova e onde mudou**
//! (os `id`s dos blocos tocados); a página em si só sai pelo `pagina_quadro`, em
//! binário, e só se a versão que a interface tem já não vale.
//!
//! Arquivo atravessa como bytes crus (`documento_abrir_*`, `documento_gravar_*`),
//! nunca em JSON: cada byte viraria um número decimal.

#[cfg(target_arch = "wasm32")]
mod wasm;
mod objetos;

use featherdocs_core::modelo;
use featherdocs_core::sessao::Sessao;
use featherdocs_io::conversao::{self, Vertente};
use featherdocs_io::{csv, dividido, docx, exportar, fdoz, Formato};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Os comandos que a ponte atende (o teste confere contra o TypeScript e contra o despacho).
pub const COMANDOS_DA_PONTE: &[&str] = &[
    "comandos_da_ponte",
    "estado",
    "documento_novo",
    "documento_abrir_fdoz",
    "documento_abrir_md",
    "documento_abrir_tex",
    "documento_abrir_docx",
    "documento_gravar_fdoz",
    "documento_gravar_docx",
    "documento_vertente",
    "documento_perdas",
    "documento_converter",
    "documento_exportar_md",
    "documento_exportar_tex",
    "documento_texto_simples",
    "documento_marcar_gravado",
    "documento_estrutura",
    "papel_definir",
    "margens_definir",
    "papel_personalizado",
    "orientacao_definir",
    "meta_definir",
    "sumario_rotulo_definir",
    "pagina_quadro",
    "cursor_visao",
    "texto_inserir",
    "tecla",
    "clicar",
    "clicar_direito",
    "selecionar_tudo",
    "cursor_ir_para",
    "copiar",
    "recortar",
    "colar",
    "formatar",
    "paragrafo_estilo",
    "inserir_bloco",
    "nivel",
    "alinhar",
    "recuar",
    "recuos_definir",
    "fonte",
    "tamanho",
    "tamanho_passo",
    "cor",
    "realce",
    "posicao",
    "limpar_formatacao",
    "maiusculas",
    "pincel_copiar",
    "pincel_aplicar",
    "entrelinha",
    "espaco",
    "fundo",
    "borda",
    "lista",
    "lista_reiniciar",
    "tarefa_alternar",
    "clicar_tarefa",
    "busca_definir",
    "busca_ir",
    "busca_limpar",
    "substituir",
    "substituir_tudo",
    "inserir_tabela",
    "inserir_imagem",
    "inserir_sumario",
    "remover_sumario",
    "link_definir",
    "link_remover",
    "chamada",
    "contagem",
    "tabela_linha",
    "tabela_coluna",
    "tabela_excluir",
    "tabela_cabecalho",
    "cabecalho_definir",
    "rodape_definir",
    "primeira_diferente",
    "numero_de_pagina",
    "numero_remover",
    "cor_pagina",
    "marca_dagua",
    "borda_pagina",
    "marcas",
    "desfazer",
    "refazer",
    "tabela_csv",
    "tabela_csv_substituir",
    "estilo_aplicar",
    "estilo_criar",
    "estilo_modificar",
    "estilo_excluir",
    "numeracao_definir",
    "titulo_sem_numero",
    "capitulo_rotulo_definir",
    "codigo_exportar",
    "codigo_aplicar",
    "codigo_cores",
    "sumario_configurar",
    "grade",
    "vertical_definir",
    "numerar_linhas_definir",
    "legenda_inserir",
    "lista_inserir",
    "legendas_rotulos_definir",
    "marcador_adicionar",
    "marcador_remover",
    "referencia_alvos",
    "referencia_inserir",
    "referencias_atualizar",
    "nota_inserir",
    "nota_info",
    "nota_editar",
    "nota_no_ponto",
    "notas_formato",
    "campo_inserir",
    "objetos_contexto",
    "objeto_no_ponto",
    "bibliografia",
    "fontes_exportar_bib",
    "fonte_salvar",
    "fonte_remover",
    "fontes_importar_bib",
    "estilo_citacao",
    "citacao_inserir",
    "citacao_editar",
    "bibliografia_inserir",
    "indice_marcar",
    "indice_inserir",
    "selecao_texto",
    "revisoes",
    "comentarios",
    "comentario_no_ponto",
    "modo_edicao",
    "proteger",
    "revisoes_resolver",
    "comentario_novo",
    "comentario_mudar",
    "comentario_excluir",
    "comentario_ir",
    "documento_comparar_fdoz",
    "documento_comparar_md",
    "documento_comparar_tex",
    "documento_comparar_docx",
];

/// O motor de um documento aberto.
#[derive(Default)]
pub struct Ponte {
    sessao: Sessao,
    fonte_tex: Option<String>,
    fonte_md: Option<String>,
    /// O que a interface diz uma vez e vale para todo documento aberto depois: a data de hoje
    /// (o campo `data`) e o título das notas de fim.
    contexto: objetos::Contexto,
}

#[derive(Deserialize)]
struct ParaFormato {
    formato: String,
}

#[derive(Deserialize)]
struct PorId {
    id: u64,
}

/// Um estilo com nome como a caixa "Novo estilo" manda.
#[derive(Deserialize)]
struct DadosDoEstilo {
    nome: String,
    #[serde(default)]
    base: u8,
    /// `sans`, `serifa` ou `mono`.
    #[serde(default)]
    fonte: String,
    #[serde(default)]
    negrito: bool,
    #[serde(default)]
    italico: bool,
    #[serde(default)]
    sublinhado: bool,
    #[serde(default)]
    tamanho: f32,
    /// `#rrggbb`; vazio = automática.
    #[serde(default)]
    cor: String,
    #[serde(default)]
    alinhamento: modelo::Alinhamento,
    #[serde(default)]
    antes: Option<f32>,
    #[serde(default)]
    depois: Option<f32>,
    #[serde(default)]
    entrelinha: f32,
}

impl DadosDoEstilo {
    fn estilo(self) -> Result<modelo::Estilo, String> {
        let familia = match self.fonte.as_str() {
            "serifa" => modelo::SERIFA,
            "mono" => modelo::CODIGO,
            _ => 0,
        };
        let bits = familia | if self.negrito { modelo::NEGRITO } else { 0 } | if self.italico { modelo::ITALICO } else { 0 } | if self.sublinhado { modelo::SUBLINHADO } else { 0 };
        let cor = featherdocs_core::fonte::ler_cor(&self.cor)?;
        Ok(modelo::Estilo { id: 0, nome: self.nome, base: self.base, bits, tamanho: self.tamanho, cor, alinhamento: self.alinhamento, antes: self.antes, depois: self.depois, entrelinha: self.entrelinha })
    }
}

#[derive(Deserialize)]
struct SumarioPersonalizado {
    niveis: u8,
    #[serde(default)]
    numeros: bool,
    #[serde(default)]
    pontilhado: bool,
}

#[derive(Deserialize)]
struct Referencia {
    id: u64,
    modo: String,
}

#[derive(Deserialize)]
struct RotulosDeLegenda {
    figura: String,
    tabela: String,
    equacao: String,
    lista_figuras: String,
    lista_tabelas: String,
}

/// O modo dividido: o formato do código (`md` ou `tex`) e, ao aplicar, o código editado.
#[derive(Deserialize)]
struct Codigo {
    formato: String,
    #[serde(default)]
    texto: String,
}

#[derive(Deserialize)]
struct NovoEstilo {
    estilo: DadosDoEstilo,
}

#[derive(Deserialize)]
struct EstiloMudado {
    id: u32,
    estilo: DadosDoEstilo,
}

#[derive(Deserialize)]
struct Quadro {
    pagina: usize,
    #[serde(default)]
    versao: u64,
}

#[derive(Deserialize)]
struct Paginas {
    de: usize,
    ate: usize,
}

#[derive(Deserialize)]
struct Texto {
    texto: String,
    #[serde(default)]
    sem_formatacao: bool,
}

#[derive(Deserialize)]
struct Tecla {
    tecla: String,
    #[serde(default)]
    ctrl: bool,
    #[serde(default)]
    shift: bool,
    #[serde(default)]
    alt: bool,
}

#[derive(Deserialize)]
struct Clique {
    pagina: usize,
    x: f32,
    y: f32,
    #[serde(default)]
    estender: bool,
    #[serde(default = "um")]
    contagem: u8,
}

fn um() -> u8 {
    1
}

#[derive(Deserialize)]
struct Estilo {
    estilo: String,
}

#[derive(Deserialize)]
struct Tipo {
    tipo: String,
}

#[derive(Deserialize)]
struct Alinhar {
    como: String,
}

#[derive(Deserialize)]
struct Recuos {
    esq: f32,
    dir: f32,
    primeira: f32,
}

#[derive(Deserialize)]
struct Medidas {
    largura: f32,
    altura: f32,
}

#[derive(Deserialize)]
struct Orientacao {
    paisagem: bool,
}

#[derive(Deserialize)]
struct Grade {
    linhas: usize,
    colunas: usize,
}

#[derive(Deserialize)]
struct Imagem {
    origem: String,
    #[serde(default)]
    largura: f32,
    #[serde(default)]
    altura: f32,
}

#[derive(Deserialize)]
struct Niveis {
    niveis: u8,
}

#[derive(Deserialize)]
struct Link {
    url: String,
    #[serde(default)]
    texto: String,
}

#[derive(Deserialize)]
struct Sim {
    sim: bool,
}

#[derive(Deserialize)]
struct Tres {
    #[serde(default)]
    esquerda: String,
    #[serde(default)]
    centro: String,
    #[serde(default)]
    direita: String,
}

#[derive(Deserialize)]
struct NumeroDePagina {
    modelo: String,
    onde: String,
}

#[derive(Deserialize)]
struct Marca {
    texto: String,
    #[serde(default)]
    cor: String,
}

#[derive(Deserialize)]
struct BordaPagina {
    cor: String,
    espessura: f32,
    distancia: f32,
}

#[derive(Deserialize)]
struct Nome {
    nome: String,
}

#[derive(Deserialize)]
struct Pt {
    pt: f32,
}

#[derive(Deserialize)]
struct Cor {
    cor: String,
}

#[derive(Deserialize)]
struct Posicao {
    p: i8,
}

#[derive(Deserialize)]
struct Fator {
    fator: f32,
}

#[derive(Deserialize)]
struct Espaco {
    antes: Option<f32>,
    depois: Option<f32>,
}

#[derive(Deserialize)]
struct Lados {
    lados: u8,
}

#[derive(Deserialize)]
struct Pagina {
    pagina: usize,
    x: f32,
    y: f32,
}

#[derive(Deserialize)]
struct Procurar {
    consulta: String,
    #[serde(flatten)]
    opcoes: featherdocs_core::busca::Opcoes,
}

#[derive(Deserialize)]
struct Por {
    por: String,
}

/// Uma substituição: o que mudou e como ficou a contagem.
#[derive(Serialize)]
struct Substituido {
    mudanca: featherdocs_core::sessao::Mudanca,
    total: usize,
    atual: usize,
    /// Quantos foram trocados (em "substituir tudo").
    trocados: usize,
}

#[derive(Deserialize)]
struct Delta {
    delta: i32,
}

#[derive(Deserialize)]
struct Papel {
    nome: String,
}

#[derive(Deserialize)]
struct Margens {
    esq: f32,
    dir: f32,
    sup: f32,
    inf: f32,
}

#[derive(Deserialize)]
struct Meta {
    titulo: String,
    autor: String,
}

#[derive(Deserialize)]
struct Rotulo {
    rotulo: String,
}

/// O texto copiado (e, no recorte, o que mudou).
#[derive(Serialize)]
struct Copiado {
    texto: String,
    mudanca: Option<featherdocs_core::sessao::Mudanca>,
}

fn args<T: DeserializeOwned>(corpo: &[u8]) -> Result<T, String> {
    serde_json::from_slice(corpo).map_err(|e| format!("Pedido ilegível: {e}"))
}

fn json<T: Serialize + ?Sized>(v: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(v).map_err(|e| format!("Não consegui montar a resposta: {e}"))
}

/// O bit de formatação de um nome.
fn bit_do_estilo(nome: &str) -> Result<u8, String> {
    Ok(match nome {
        "negrito" => modelo::NEGRITO,
        "italico" => modelo::ITALICO,
        "sublinhado" => modelo::SUBLINHADO,
        "tachado" => modelo::TACHADO,
        "codigo" => modelo::CODIGO,
        outro => return Err(format!("Não conheço a formatação \"{outro}\".")),
    })
}

impl Ponte {
    /// Atende um comando que **só lê** (roda com a trava de leitura, junto com outros). `None` se o comando muda algo.
    pub fn chamar_leitura(&self, nome: &str, corpo: &[u8]) -> Option<Result<Vec<u8>, String>> {
        Some(match nome {
            "comandos_da_ponte" => json(COMANDOS_DA_PONTE),
            "estado" => json(&self.sessao.estado()),
            "pagina_quadro" => args::<Quadro>(corpo).and_then(|a| self.sessao.quadro(a.pagina, a.versao)),
            "cursor_visao" => args::<Paginas>(corpo).and_then(|a| json(&self.sessao.visao(a.de, a.ate))),
            "documento_estrutura" => json(&self.sessao.estrutura()),
            "tabela_csv" => args::<PorId>(corpo).and_then(|a| self.sessao.tabela_celulas(a.id)).map(|c| csv::gravar(&c).into_bytes()),
            "documento_gravar_fdoz" => fdoz::gravar(self.sessao.documento()),
            "documento_gravar_docx" => docx::gravar(self.sessao.documento()),
            "documento_vertente" => {
                let v = self.vertente();
                json(&serde_json::json!({ "vertente": v, "formatos": v.formatos() }))
            }
            "documento_perdas" => args::<ParaFormato>(corpo).and_then(|a| json(&conversao::perdas(self.sessao.documento(), self.vertente(), &a.formato))),
            "documento_exportar_md" => {
                if let Some(ref md) = self.fonte_md {
                    Ok(md.clone().into_bytes())
                } else {
                    Ok(exportar::markdown(self.sessao.documento()).into_bytes())
                }
            }
            "documento_exportar_tex" => {
                if let Some(ref tex) = self.fonte_tex {
                    Ok(tex.clone().into_bytes())
                } else {
                    Ok(exportar::latex(self.sessao.documento()).into_bytes())
                }
            }
            "codigo_cores" => args::<Codigo>(corpo).and_then(|a| json(&featherdocs_io::realce::cores(dividido::Fonte::do_nome(&a.formato)?, &a.texto))),
            "documento_texto_simples" => Ok(exportar::texto_simples(self.sessao.documento()).into_bytes()),
            "referencia_alvos" => json(&self.sessao.alvos_de_referencia()),
            "nota_info" | "nota_no_ponto" | "objeto_no_ponto" | "bibliografia" | "fontes_exportar_bib" | "revisoes" | "comentarios" | "comentario_no_ponto" => return self.objetos_leitura(nome, corpo),
            "selecao_texto" => json(&self.sessao.texto_selecionado()),
            "codigo_exportar" => args::<Codigo>(corpo).and_then(|a| {
                let d = self.sessao.documento();
                match dividido::Fonte::do_nome(&a.formato)? {
                    dividido::Fonte::Markdown => {
                        if let Some(ref md) = self.fonte_md {
                            json(&exportar::mapa_de_texto(md, d, dividido::Fonte::Markdown))
                        } else {
                            json(&exportar::markdown_com_mapa(d))
                        }
                    }
                    dividido::Fonte::Latex => {
                        if let Some(ref tex) = self.fonte_tex {
                            json(&exportar::mapa_de_texto(tex, d, dividido::Fonte::Latex))
                        } else {
                            json(&exportar::latex_com_mapa(d))
                        }
                    }
                }
            }),
            _ => return None,
        })
    }

    /// Atende um comando. Erro é sempre uma mensagem em português para mostrar ao usuário.
    pub fn chamar(&mut self, nome: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
        if let Some(r) = self.chamar_leitura(nome, corpo) {
            return r;
        }
        if objetos::barrado(&self.sessao, nome) {
            return Err("O documento está só para leitura (Revisão → Modo de edição, ou Proteger).".to_string());
        }
        match nome {
            "documento_novo" => {
                let rotulo = self.sessao.rotulo_do_sumario();
                self.sessao = Sessao::default();
                self.sessao.rotulo_do_sumario_definir(&rotulo);
                self.contexto.aplicar(&mut self.sessao);
                self.fonte_tex = None;
                self.fonte_md = None;
                json(&self.sessao.estado())
            }
            "codigo_aplicar" => {
                let a: Codigo = args(corpo)?;
                let fonte = dividido::Fonte::do_nome(&a.formato)?;
                let r = dividido::reconciliar(self.sessao.documento(), fonte, &a.texto);
                let mudanca = self.sessao.trocar_blocos(r.blocos, r.meta, r.numeracao);
                match fonte {
                    dividido::Fonte::Latex => {
                        self.fonte_tex = Some(a.texto);
                    }
                    dividido::Fonte::Markdown => {
                        self.fonte_md = Some(a.texto);
                    }
                }
                json(&mudanca)
            }
            "documento_converter" => self.converter(corpo),
            "documento_abrir_fdoz" => self.abrir(Formato::Fdoz, corpo),
            "documento_abrir_md" => self.abrir(Formato::Markdown, corpo),
            "documento_abrir_tex" => self.abrir(Formato::Latex, corpo),
            "documento_abrir_docx" => self.abrir(Formato::Docx, corpo),
            "documento_marcar_gravado" => {
                self.sessao.marcar_gravado();
                json(&self.sessao.estado())
            }
            "papel_definir" => args::<Papel>(corpo).and_then(|a| self.sessao.papel_definir(&a.nome)).and_then(|m| json(&m)),
            "papel_personalizado" => args::<Medidas>(corpo).and_then(|a| self.sessao.papel_personalizado(a.largura, a.altura)).and_then(|m| json(&m)),
            "orientacao_definir" => args::<Orientacao>(corpo).and_then(|a| self.sessao.orientacao_definir(a.paisagem)).and_then(|m| json(&m)),
            "margens_definir" => args::<Margens>(corpo).and_then(|a| self.sessao.margens_definir(a.esq, a.dir, a.sup, a.inf)).and_then(|m| json(&m)),
            "meta_definir" => {
                let a: Meta = args(corpo)?;
                self.sessao.meta_definir(&a.titulo, &a.autor);
                json(&self.sessao.estado())
            }
            "sumario_rotulo_definir" => {
                let a: Rotulo = args(corpo)?;
                self.sessao.rotulo_do_sumario_definir(&a.rotulo);
                json(&self.sessao.estado())
            }
            "tabela_csv_substituir" => self.tabela_substituir(corpo),
            "legendas_rotulos_definir" => {
                let a: RotulosDeLegenda = args(corpo)?;
                self.sessao.rotulos_de_legenda_definir([a.figura, a.tabela, a.equacao], [a.lista_figuras, a.lista_tabelas]);
                json(&self.sessao.estado())
            }
            "capitulo_rotulo_definir" => {
                let a: Rotulo = args(corpo)?;
                self.sessao.rotulo_do_capitulo_definir(&a.rotulo);
                json(&self.sessao.estado())
            }
            _ => self.editar(nome, corpo),
        }
    }

    /// A vertente do documento: o código de onde ele veio (LaTeX, Markdown) ou nenhum (convencional).
    fn vertente(&self) -> Vertente {
        if self.fonte_tex.is_some() {
            Vertente::Latex
        } else if self.fonte_md.is_some() {
            Vertente::Markdown
        } else {
            Vertente::Convencional
        }
    }

    /// Grava o documento como o texto de outra vertente de código (`md` ou `tex`) e passa a ser dela:
    /// o LaTeX salvo como Markdown continua, dali em diante, um documento Markdown.
    fn converter(&mut self, corpo: &[u8]) -> Result<Vec<u8>, String> {
        let a: ParaFormato = args(corpo)?;
        let d = self.sessao.documento();
        match Vertente::do_formato(&a.formato) {
            Some(Vertente::Markdown) => {
                let md = exportar::markdown(d);
                self.fonte_md = Some(md.clone());
                self.fonte_tex = None;
                Ok(md.into_bytes())
            }
            Some(Vertente::Latex) => {
                let tex = exportar::latex(d);
                self.fonte_tex = Some(tex.clone());
                self.fonte_md = None;
                Ok(tex.into_bytes())
            }
            _ => Err(format!("Não sei converter para \"{}\".", a.formato)),
        }
    }

    fn abrir(&mut self, formato: Formato, bytes: &[u8]) -> Result<Vec<u8>, String> {
        let doc = featherdocs_io::abrir(formato, bytes)?;
        let rotulo = self.sessao.rotulo_do_sumario();
        self.sessao = Sessao::com_documento(doc);
        self.sessao.rotulo_do_sumario_definir(&rotulo);
        self.contexto.aplicar(&mut self.sessao);
        match formato {
            Formato::Latex => {
                self.fonte_tex = String::from_utf8(bytes.to_vec()).ok();
                self.fonte_md = None;
            }
            Formato::Markdown => {
                self.fonte_md = String::from_utf8(bytes.to_vec()).ok();
                self.fonte_tex = None;
            }
            _ => {
                self.fonte_tex = None;
                self.fonte_md = None;
            }
        }
        if formato == Formato::Fdoz {
            self.sessao.marcar_gravado();
        }
        json(&self.sessao.estado())
    }

    /// Os comandos que mexem no cursor e no texto.
    fn editar(&mut self, nome: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
        match nome {
            "texto_inserir" => {
                let a: Texto = args(corpo)?;
                json(&self.sessao.inserir_texto(&a.texto))
            }
            "tecla" => {
                let a: Tecla = args(corpo)?;
                json(&self.sessao.tecla(&a.tecla, a.ctrl, a.shift, a.alt))
            }
            "clicar" => {
                let a: Clique = args(corpo)?;
                self.sessao.clicar(a.pagina, a.x, a.y, a.estender, a.contagem);
                json(&())
            }
            "clicar_direito" => {
                let a: Pagina = args(corpo)?;
                self.sessao.clicar_direito(a.pagina, a.x, a.y);
                json(&())
            }
            "cursor_ir_para" => {
                self.sessao.ir_para_bloco(args::<PorId>(corpo)?.id);
                json(&())
            }
            "selecionar_tudo" => {
                self.sessao.selecionar_tudo();
                json(&())
            }
            "copiar" => json(&Copiado { texto: self.sessao.copiar(), mudanca: None }),
            "recortar" => {
                let (texto, m) = self.sessao.recortar();
                json(&Copiado { texto, mudanca: Some(m) })
            }
            "colar" => {
                let a: Texto = args(corpo)?;
                json(&if a.sem_formatacao { self.sessao.colar_sem_formatacao(&a.texto) } else { self.sessao.colar(&a.texto) })
            }
            "formatar" => {
                let a: Estilo = args(corpo)?;
                json(&self.sessao.formatar(bit_do_estilo(&a.estilo)?))
            }
            "paragrafo_estilo" => {
                let a: Tipo = args(corpo)?;
                json(&self.sessao.paragrafo_estilo(&a.tipo))
            }
            "inserir_bloco" => {
                let a: Tipo = args(corpo)?;
                json(&self.sessao.inserir_bloco(&a.tipo))
            }
            "nivel" => {
                let a: Delta = args(corpo)?;
                json(&self.sessao.mudar_nivel(a.delta))
            }
            "alinhar" => {
                let a: Alinhar = args(corpo)?;
                json(&self.sessao.alinhar(&a.como)?)
            }
            "recuar" => {
                let a: Delta = args(corpo)?;
                json(&self.sessao.recuar(a.delta))
            }
            "recuos_definir" => {
                let a: Recuos = args(corpo)?;
                json(&self.sessao.recuos_definir(a.esq, a.dir, a.primeira))
            }
            "desfazer" => json(&self.sessao.desfazer()),
            "refazer" => json(&self.sessao.refazer()),
            _ => self.formatar_rico(nome, corpo),
        }
    }

    /// Fonte, cor, parágrafo, listas, pincel e busca.
    fn formatar_rico(&mut self, nome: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
        match nome {
            "fonte" => json(&self.sessao.fonte_definir(&args::<Nome>(corpo)?.nome)?),
            "tamanho" => json(&self.sessao.tamanho_definir(args::<Pt>(corpo)?.pt)?),
            "tamanho_passo" => json(&self.sessao.tamanho_passo(args::<Delta>(corpo)?.delta)),
            "cor" => json(&self.sessao.cor_definir(&args::<Cor>(corpo)?.cor)?),
            "realce" => json(&self.sessao.realce_definir(&args::<Cor>(corpo)?.cor)?),
            "posicao" => json(&self.sessao.posicao_alternar(args::<Posicao>(corpo)?.p)),
            "limpar_formatacao" => json(&self.sessao.limpar_formatacao()),
            "maiusculas" => json(&self.sessao.maiusculas(&args::<Nome>(corpo)?.nome)?),
            "pincel_copiar" => {
                self.sessao.pincel_copiar();
                json(&())
            }
            "pincel_aplicar" => json(&self.sessao.pincel_aplicar()),
            "entrelinha" => json(&self.sessao.entrelinha_definir(args::<Fator>(corpo)?.fator)?),
            "espaco" => {
                let a: Espaco = args(corpo)?;
                json(&self.sessao.espaco_definir(a.antes, a.depois)?)
            }
            "fundo" => json(&self.sessao.fundo_definir(&args::<Cor>(corpo)?.cor)?),
            "borda" => json(&self.sessao.borda_definir(args::<Lados>(corpo)?.lados)),
            "lista" => json(&self.sessao.lista_estilo(&args::<Nome>(corpo)?.nome)?),
            "lista_reiniciar" => json(&self.sessao.lista_reiniciar()),
            "tarefa_alternar" => json(&self.sessao.tarefa_alternar(args::<PorId>(corpo)?.id)),
            "clicar_tarefa" => {
                let a: Pagina = args(corpo)?;
                json(&self.sessao.clicar_tarefa(a.pagina, a.x, a.y))
            }
            _ => self.buscar(nome, corpo),
        }
    }

    /// Inserir (tabela, imagem, sumário, link, chamada), tabela contextual, página e contagem.
    fn inserir_e_pagina(&mut self, nome: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
        let s = &mut self.sessao;
        match nome {
            "inserir_tabela" => args::<Grade>(corpo).and_then(|a| s.inserir_tabela(a.linhas, a.colunas)).and_then(|m| json(&m)),
            "inserir_imagem" => args::<Imagem>(corpo).and_then(|a| s.inserir_imagem(&a.origem, a.largura, a.altura)).and_then(|m| json(&m)),
            "inserir_sumario" => json(&s.inserir_sumario(args::<Niveis>(corpo)?.niveis)),
            "remover_sumario" => json(&s.remover_sumario()),
            "link_definir" => args::<Link>(corpo).and_then(|a| s.link_definir(&a.url, &a.texto)).and_then(|m| json(&m)),
            "link_remover" => json(&s.link_remover()),
            "chamada" => json(&s.chamada(&args::<Nome>(corpo)?.nome)?),
            "contagem" => json(&s.contagem()),
            "tabela_linha" => json(&s.tabela_linha(args::<Sim>(corpo)?.sim)),
            "tabela_coluna" => json(&s.tabela_coluna(args::<Sim>(corpo)?.sim)),
            "tabela_excluir" => json(&s.tabela_excluir(&args::<Nome>(corpo)?.nome)?),
            "tabela_cabecalho" => json(&s.tabela_cabecalho()),
            "cabecalho_definir" => args::<Tres>(corpo).and_then(|a| json(&s.cabecalho_definir(&a.esquerda, &a.centro, &a.direita))),
            "rodape_definir" => args::<Tres>(corpo).and_then(|a| json(&s.rodape_definir(&a.esquerda, &a.centro, &a.direita))),
            "primeira_diferente" => json(&s.primeira_diferente_definir(args::<Sim>(corpo)?.sim)),
            "numero_de_pagina" => args::<NumeroDePagina>(corpo).and_then(|a| s.numero_de_pagina(&a.modelo, &a.onde)).and_then(|m| json(&m)),
            "numero_remover" => json(&s.numero_remover()),
            "cor_pagina" => json(&s.cor_pagina_definir(&args::<Cor>(corpo)?.cor)?),
            "marca_dagua" => args::<Marca>(corpo).and_then(|a| s.marca_dagua_definir(&a.texto, &a.cor)).and_then(|m| json(&m)),
            "borda_pagina" => args::<BordaPagina>(corpo).and_then(|a| s.borda_pagina_definir(&a.cor, a.espessura, a.distancia)).and_then(|m| json(&m)),
            "marcas" => json(&s.marcas_definir(args::<Sim>(corpo)?.sim)),
            "estilo_aplicar" => json(&s.estilo_aplicar(u32::try_from(args::<PorId>(corpo)?.id).map_err(|_| "Estilo inválido.")?)?),
            "estilo_criar" => json(&s.estilo_criar(args::<NovoEstilo>(corpo)?.estilo.estilo()?)?),
            "estilo_modificar" => {
                let a: EstiloMudado = args(corpo)?;
                json(&s.estilo_modificar(a.id, a.estilo.estilo()?)?)
            }
            "estilo_excluir" => json(&s.estilo_excluir(u32::try_from(args::<PorId>(corpo)?.id).map_err(|_| "Estilo inválido.")?)?),
            "numeracao_definir" => json(&s.numeracao_definir(&args::<Nome>(corpo)?.nome)?),
            "titulo_sem_numero" => json(&s.titulo_sem_numero()),
            "sumario_configurar" => {
                let a: SumarioPersonalizado = args(corpo)?;
                json(&s.sumario_configurar(a.niveis, a.numeros, a.pontilhado))
            }
            "grade" => json(&s.grade_definir(args::<Sim>(corpo)?.sim)),
            "vertical_definir" => json(&s.vertical_definir(&args::<Nome>(corpo)?.nome)?),
            "numerar_linhas_definir" => json(&s.numerar_linhas_definir(&args::<Nome>(corpo)?.nome)?),
            "legenda_inserir" => json(&s.legenda_inserir(&args::<Nome>(corpo)?.nome)?),
            "lista_inserir" => json(&s.lista_inserir(args::<Niveis>(corpo)?.niveis)),
            "marcador_adicionar" => json(&s.marcador_adicionar(&args::<Nome>(corpo)?.nome)?),
            "marcador_remover" => json(&s.marcador_remover(&args::<Nome>(corpo)?.nome)),
            "referencia_inserir" => {
                let a: Referencia = args(corpo)?;
                json(&s.referencia_inserir(a.id, &a.modo)?)
            }
            "referencias_atualizar" => json(&s.referencias_atualizar()),
            _ => self.objetos(nome, corpo),
        }
    }

    /// Localizar e substituir.
    fn buscar(&mut self, nome: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
        match nome {
            "busca_definir" => {
                let a: Procurar = args(corpo)?;
                json(&self.sessao.busca_definir(&a.consulta, a.opcoes))
            }
            "busca_ir" => json(&self.sessao.busca_ir(args::<Delta>(corpo)?.delta)),
            "busca_limpar" => {
                self.sessao.busca_limpar();
                json(&())
            }
            "substituir" => {
                let (mudanca, r) = self.sessao.substituir(&args::<Por>(corpo)?.por);
                json(&Substituido { mudanca, total: r.total, atual: r.atual, trocados: 0 })
            }
            "substituir_tudo" => {
                let (mudanca, n) = self.sessao.substituir_tudo(&args::<Por>(corpo)?.por);
                json(&Substituido { mudanca, total: 0, atual: 0, trocados: n })
            }
            _ => self.inserir_e_pagina(nome, corpo),
        }
    }

    /// O corpo é `id` (8 bytes, little-endian) seguido do CSV: a tabela que voltou do FeatherSheet.
    fn tabela_substituir(&mut self, corpo: &[u8]) -> Result<Vec<u8>, String> {
        if corpo.len() < 8 {
            return Err("Pedido ilegível: faltou o id da tabela.".to_string());
        }
        let (id, csv_bytes) = corpo.split_at(8);
        let id = u64::from_le_bytes(id.try_into().map_err(|_| "Pedido ilegível.")?);
        let texto = String::from_utf8(csv_bytes.to_vec()).map_err(|_| "O CSV não está em UTF-8.".to_string())?;
        json(&self.sessao.tabela_substituir(id, &csv::ler(&texto))?)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn chamar_json(p: &mut Ponte, nome: &str, corpo: &str) -> serde_json::Value {
        serde_json::from_slice(&p.chamar(nome, corpo.as_bytes()).unwrap()).unwrap()
    }

    #[test]
    fn todo_comando_listado_e_atendido() {
        let mut p = Ponte::default();
        for nome in COMANDOS_DA_PONTE {
            if let Err(e) = p.chamar(nome, b"{}") {
                assert!(!e.starts_with("Comando desconhecido"), "{nome} não está no despacho");
            }
        }
        assert!(p.chamar("nao_existe", b"{}").unwrap_err().starts_with("Comando desconhecido"));
    }

    #[test]
    fn digitar_e_ler_o_quadro_e_o_cursor() {
        let mut p = Ponte::default();
        let m = chamar_json(&mut p, "texto_inserir", r#"{"texto":"Olá"}"#);
        assert_eq!(m["afetados"].as_array().unwrap().len(), 1);
        let q = p.chamar("pagina_quadro", br#"{"pagina":0,"versao":0}"#).unwrap();
        assert_eq!(q[0], 1);
        let v = m["versao"].as_u64().unwrap();
        assert_eq!(p.chamar("pagina_quadro", format!("{{\"pagina\":0,\"versao\":{v}}}").as_bytes()).unwrap(), vec![0]);
        let visao = chamar_json(&mut p, "cursor_visao", r#"{"de":0,"ate":0}"#);
        assert!(visao["cursor"]["x"].as_f64().unwrap() > 72.0);
    }

    #[test]
    fn tecla_devolve_se_foi_tratada() {
        let mut p = Ponte::default();
        assert_eq!(chamar_json(&mut p, "tecla", r#"{"tecla":"ArrowRight"}"#)["tratada"], true);
        assert_eq!(chamar_json(&mut p, "tecla", r#"{"tecla":"c","ctrl":true}"#)["tratada"], false);
    }

    #[test]
    fn vertentes_salvam_nos_seus_formatos_e_latex_vira_markdown_com_aviso() {
        let mut p = Ponte::default();
        assert_eq!(chamar_json(&mut p, "documento_vertente", "{}")["formatos"], serde_json::json!(["fdoz", "docx"]));
        let docx = p.chamar("documento_gravar_docx", b"").unwrap();
        assert!(Ponte::default().chamar("documento_abrir_docx", &docx).is_ok());
        p.chamar("documento_abrir_tex", b"\\documentclass{article}\\begin{document}\\section{A}texto\\end{document}").unwrap();
        assert_eq!(chamar_json(&mut p, "documento_vertente", "{}")["vertente"], "latex");
        assert!(!chamar_json(&mut p, "documento_perdas", r#"{"formato":"md"}"#).as_array().unwrap().is_empty());
        let md = String::from_utf8(p.chamar("documento_converter", br#"{"formato":"md"}"#).unwrap()).unwrap();
        assert!(md.contains("# A") || md.contains("## A"), "{md}");
        assert_eq!(chamar_json(&mut p, "documento_vertente", "{}")["vertente"], "markdown");
        assert!(chamar_json(&mut p, "documento_perdas", r#"{"formato":"tex"}"#).as_array().unwrap().is_empty());
    }

    #[test]
    fn abrir_markdown_e_gravar_fdoz_e_reabrir() {
        let mut p = Ponte::default();
        p.chamar("documento_abrir_md", b"# T\n\ntexto\n").unwrap();
        let fdoz = p.chamar("documento_gravar_fdoz", b"").unwrap();
        let mut q = Ponte::default();
        let est: serde_json::Value = serde_json::from_slice(&q.chamar("documento_abrir_fdoz", &fdoz).unwrap()).unwrap();
        assert_eq!(est["blocos"], 2);
        assert_eq!(est["sujo"], false);
    }

    #[test]
    fn erro_de_arquivo_e_em_portugues() {
        let e = Ponte::default().chamar("documento_abrir_docx", b"lixo").unwrap_err();
        assert!(e.contains("docx"));
    }

    #[test]
    fn copiar_e_colar() {
        let mut p = Ponte::default();
        chamar_json(&mut p, "texto_inserir", r#"{"texto":"abc"}"#);
        chamar_json(&mut p, "selecionar_tudo", "{}");
        let c = chamar_json(&mut p, "copiar", "{}");
        assert_eq!(c["texto"], "abc");
        chamar_json(&mut p, "tecla", r#"{"tecla":"End","ctrl":true}"#);
        chamar_json(&mut p, "colar", r#"{"texto":"abc"}"#);
        let est = chamar_json(&mut p, "estado", "");
        assert_eq!(est["caracteres"], 6);
    }

    #[test]
    fn tabela_vai_e_volta_por_csv() {
        let mut p = Ponte::default();
        chamar_json(&mut p, "inserir_bloco", r#"{"tipo":"tabela"}"#);
        let est = chamar_json(&mut p, "estado", "");
        assert_eq!(est["blocos"], 2);
        let id = 1u64;
        let csv = p.chamar("tabela_csv", format!("{{\"id\":{id}}}").as_bytes()).unwrap();
        assert_eq!(String::from_utf8(csv).unwrap().lines().count(), 3);
        let mut corpo = id.to_le_bytes().to_vec();
        corpo.extend_from_slice(b"x;y\n1;2\n");
        p.chamar("tabela_csv_substituir", &corpo).unwrap();
        let csv = String::from_utf8(p.chamar("tabela_csv", format!("{{\"id\":{id}}}").as_bytes()).unwrap()).unwrap();
        assert_eq!(csv, "x,y\n1,2\n");
    }

    #[test]
    fn estilo_com_nome_pela_ponte() {
        let mut p = Ponte::default();
        chamar_json(&mut p, "texto_inserir", r#"{"texto":"Capa"}"#);
        let m = chamar_json(&mut p, "estilo_criar", r##"{"estilo":{"nome":"Capa","base":1,"fonte":"serifa","negrito":true,"tamanho":30,"cor":"#336699","alinhamento":"centro"}}"##);
        assert_eq!(m["afetados"].as_array().unwrap().len(), 1);
        let est = chamar_json(&mut p, "estado", "");
        assert_eq!(est["estilos"]["lista"][0]["nome"], "Capa");
        let visao = chamar_json(&mut p, "cursor_visao", r#"{"de":0,"ate":0}"#);
        assert_eq!(visao["formato"]["estilo_proprio"], 1);
        assert_eq!(visao["formato"]["estilo"], "titulo1");
        chamar_json(&mut p, "numeracao_definir", r#"{"nome":"decimal"}"#);
        assert_eq!(chamar_json(&mut p, "estado", "")["estilos"]["numeracao"], "decimal");
        assert!(p.chamar("estilo_criar", br#"{"estilo":{"nome":"x","cor":"azul"}}"#).unwrap_err().contains("Cor"));
    }

    #[test]
    fn modo_dividido_pela_ponte() {
        let mut p = Ponte::default();
        p.chamar("documento_abrir_md", b"# T\n\num\n\ndois\n").unwrap();
        let mapa = chamar_json(&mut p, "codigo_exportar", r#"{"formato":"md"}"#);
        assert_eq!(mapa["blocos"].as_array().unwrap().len(), 3);
        assert_eq!(mapa["blocos"][1]["de"], 2, "o segundo bloco começa na linha 2 (depois da linha em branco)");
        let texto = mapa["texto"].as_str().unwrap().replace("dois", "dois e três");
        let corpo = serde_json::json!({ "formato": "md", "texto": texto }).to_string();
        let m = chamar_json(&mut p, "codigo_aplicar", &corpo);
        assert_eq!(m["afetados"].as_array().unwrap().len(), 1, "só o parágrafo editado");
        assert!(chamar_json(&mut p, "codigo_exportar", r#"{"formato":"md"}"#)["texto"].as_str().unwrap().contains("dois e três"));
        assert!(p.chamar("codigo_exportar", br#"{"formato":"docx"}"#).is_err());
    }

    #[test]
    fn latex_curriculo_preserva_tudo_pela_ponte() {
        let mut p = Ponte::default();
        let tex_original = "% Comentário inicial\n\\documentclass[10pt, a4paper]{article}\n\\usepackage{helvet}\n\\definecolor{darkblue}{HTML}{1A3A5C}\n\\newcommand{\\seccao}[1]{\\textbf{#1}}\n\\begin{document}\n\\colorbox{darkblue}{\\textbf{Marcelo A. Gonçalves}}\n\\seccao{Perfil}\nEngenheiro Eletricista.\n\\end{document}\n";
        p.chamar("documento_abrir_tex", tex_original.as_bytes()).unwrap();
        let mapa = chamar_json(&mut p, "codigo_exportar", r#"{"formato":"tex"}"#);
        assert_eq!(mapa["texto"].as_str().unwrap(), tex_original);
        let exportado = p.chamar("documento_exportar_tex", b"").unwrap();
        assert_eq!(String::from_utf8(exportado).unwrap(), tex_original);

        let tex_editado = tex_original.replace("Engenheiro Eletricista.", "Engenheiro Eletricista e Cientista de Dados.");
        let corpo = serde_json::json!({ "formato": "tex", "texto": tex_editado }).to_string();
        p.chamar("codigo_aplicar", corpo.as_bytes()).unwrap();
        let exportado2 = p.chamar("documento_exportar_tex", b"").unwrap();
        assert_eq!(String::from_utf8(exportado2).unwrap(), tex_editado);
    }
}

