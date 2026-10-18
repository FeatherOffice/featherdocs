//! Os comandos dos objetos presos ao texto: notas de rodapé e de fim, campos e (depois) citações
//! e entradas de índice.

use featherdocs_core::cursor::Pos;
use featherdocs_core::modelo::Marca;
use featherdocs_core::sessao::Sessao;
use serde::Deserialize;

use crate::{args, json, Ponte};

/// O que a interface diz uma vez (na largada, ao trocar de língua ou de arquivo) e vale para todo
/// documento aberto depois: a data de hoje, o nome do arquivo e os títulos das listas geradas.
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct Contexto(featherdocs_core::objetos::ContextoExterno);

impl Contexto {
    pub(crate) fn aplicar(&self, s: &mut Sessao) -> featherdocs_core::sessao::Mudanca {
        s.contexto_definir(&self.0)
    }

    fn rotulo_da_comparacao(&self) -> String {
        if self.0.comparacao.is_empty() {
            "Comparação".to_string()
        } else {
            self.0.comparacao.clone()
        }
    }
}

/// Um ponto do texto, como a interface manda (o mesmo que o cursor tem).
#[derive(Deserialize)]
struct Ponto {
    bloco: u64,
    #[serde(default)]
    celula: Option<(u32, u32)>,
    desl: usize,
}

impl Ponto {
    fn pos(&self) -> Pos {
        Pos { bloco: self.bloco, celula: self.celula, desl: self.desl }
    }
}

#[derive(Deserialize)]
struct NovaNota {
    #[serde(default)]
    fim: bool,
    #[serde(default)]
    fonte: String,
}

#[derive(Deserialize)]
struct NotaEditada {
    #[serde(flatten)]
    ponto: Ponto,
    fonte: String,
    #[serde(default)]
    fim: bool,
}

#[derive(Deserialize)]
struct NaPagina {
    pagina: usize,
    x: f32,
    y: f32,
}

#[derive(Deserialize)]
struct FormatoDasNotas {
    rodape: u8,
    fim: u8,
}

#[derive(Deserialize)]
struct Campo {
    campo: String,
}

#[derive(Deserialize)]
struct FonteSalva {
    fonte: featherdocs_core::modelo::Fonte,
    #[serde(default)]
    antiga: String,
}

#[derive(Deserialize)]
struct Chave {
    chave: String,
}

#[derive(Deserialize)]
struct TextoBib {
    texto: String,
}

#[derive(Deserialize)]
struct Nome {
    nome: String,
}

#[derive(Deserialize)]
struct Citacao {
    chaves: Vec<String>,
    #[serde(default)]
    pagina: String,
}

#[derive(Deserialize)]
struct CitacaoEditada {
    #[serde(flatten)]
    ponto: Ponto,
    chaves: Vec<String>,
    #[serde(default)]
    pagina: String,
}

#[derive(Deserialize)]
struct Todas {
    #[serde(default)]
    todas: bool,
}

#[derive(Deserialize)]
struct Entrada {
    termo: String,
    #[serde(default)]
    sub: String,
    #[serde(default)]
    todas: bool,
}

/// A resposta de quem muda e conta (importar fontes, marcar todas as entradas).
#[derive(serde::Serialize)]
struct Contado {
    mudanca: featherdocs_core::sessao::Mudanca,
    n: usize,
}

/// O objeto achado sob o ponto da página.
#[derive(serde::Serialize)]
struct Achado {
    ponto: Pos,
    marca: Marca,
}

#[derive(Deserialize)]
struct Sim {
    sim: bool,
}

#[derive(Deserialize)]
struct Resolver {
    aceitar: bool,
    #[serde(default)]
    tudo: bool,
}

#[derive(Deserialize)]
struct NovoComentario {
    texto: String,
}

#[derive(Deserialize)]
struct PorId {
    id: u64,
}

#[derive(Deserialize)]
struct ComentarioMudado {
    id: u64,
    #[serde(default)]
    texto: Option<String>,
    #[serde(default)]
    resposta: Option<String>,
    #[serde(default)]
    resolvido: Option<bool>,
}

/// Os comandos que passam com o documento travado (somente leitura ou protegido): olhar, andar,
/// copiar, procurar, comentar, trocar de modo, abrir outro documento.
const LIBERADOS: &[&str] = &[
    "documento_novo", "documento_abrir_fdoz", "documento_abrir_md", "documento_abrir_tex", "documento_abrir_docx", "documento_marcar_gravado", "documento_converter",
    "tecla", "clicar", "clicar_direito", "selecionar_tudo", "cursor_ir_para", "copiar", "pincel_copiar", "busca_definir", "busca_ir", "busca_limpar", "marcas", "grade",
    "modo_edicao", "proteger", "comentario_novo", "comentario_mudar", "comentario_excluir", "comentario_ir", "objetos_contexto", "sumario_rotulo_definir",
    "capitulo_rotulo_definir", "legendas_rotulos_definir",
];

/// O comando muda o documento e o documento está travado?
pub(crate) fn barrado(s: &Sessao, nome: &str) -> bool {
    s.bloqueado() && !LIBERADOS.contains(&nome)
}

/// Os campos que o Inserir → Campo oferece.
const CAMPOS: [&str; 6] = ["autor", "titulo", "data", "pagina", "paginas", "arquivo"];

impl Ponte {
    /// Os comandos que só leem.
    pub(crate) fn objetos_leitura(&self, nome: &str, corpo: &[u8]) -> Option<Result<Vec<u8>, String>> {
        Some(match nome {
            "nota_info" => args::<Ponto>(corpo).and_then(|a| self.sessao.nota_info(&a.pos())).and_then(|i| json(&i)),
            "nota_no_ponto" => args::<NaPagina>(corpo).and_then(|a| json(&self.sessao.nota_no_ponto(a.pagina, a.x, a.y))),
            "objeto_no_ponto" => args::<NaPagina>(corpo).and_then(|a| json(&self.sessao.objeto_no_ponto(a.pagina, a.x, a.y).map(|(ponto, marca)| Achado { ponto, marca }))),
            "bibliografia" => json(self.sessao.bibliografia()),
            "revisoes" => json(&self.sessao.revisoes()),
            "comentarios" => json(&self.sessao.comentarios()),
            "comentario_no_ponto" => args::<NaPagina>(corpo).and_then(|a| json(&self.sessao.comentario_no_ponto(a.pagina, a.x, a.y))),
            "fontes_exportar_bib" => Ok(featherdocs_io::bib::gravar(&self.sessao.bibliografia().fontes).into_bytes()),
            _ => return None,
        })
    }

    /// Os comandos que mudam o documento.
    pub(crate) fn objetos(&mut self, nome: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
        let s = &mut self.sessao;
        match nome {
            "nota_inserir" => {
                let a: NovaNota = args(corpo)?;
                json(&s.nota_inserir(a.fim, &a.fonte)?)
            }
            "nota_editar" => {
                let a: NotaEditada = args(corpo)?;
                json(&s.nota_editar(&a.ponto.pos(), &a.fonte, a.fim)?)
            }
            "notas_formato" => {
                let a: FormatoDasNotas = args(corpo)?;
                json(&s.notas_formato(a.rodape, a.fim)?)
            }
            "campo_inserir" => {
                let a: Campo = args(corpo)?;
                if !CAMPOS.contains(&a.campo.as_str()) {
                    return Err(format!("Não conheço o campo \"{}\".", a.campo));
                }
                json(&s.objeto_inserir(Marca::Campo { campo: a.campo })?)
            }
            "fonte_salvar" => {
                let a: FonteSalva = args(corpo)?;
                json(&s.fonte_salvar(a.fonte, &a.antiga)?)
            }
            "fonte_remover" => json(&s.fonte_remover(&args::<Chave>(corpo)?.chave)),
            "fontes_importar_bib" => {
                let fontes = featherdocs_io::bib::ler(&args::<TextoBib>(corpo)?.texto);
                if fontes.is_empty() {
                    return Err("Não achei nenhuma fonte (@book, @article…) neste arquivo .bib.".to_string());
                }
                let (mudanca, n) = s.fontes_importar(fontes);
                json(&Contado { mudanca, n })
            }
            "estilo_citacao" => json(&s.estilo_citacao_definir(&args::<Nome>(corpo)?.nome)?),
            "citacao_inserir" => {
                let a: Citacao = args(corpo)?;
                json(&s.citacao_inserir(&a.chaves, &a.pagina)?)
            }
            "citacao_editar" => {
                let a: CitacaoEditada = args(corpo)?;
                json(&s.citacao_editar(&a.ponto.pos(), &a.chaves, &a.pagina)?)
            }
            "bibliografia_inserir" => json(&s.bibliografia_inserir(args::<Todas>(corpo)?.todas)),
            "indice_marcar" => {
                let a: Entrada = args(corpo)?;
                let (mudanca, n) = s.indice_marcar(&a.termo, &a.sub, a.todas)?;
                json(&Contado { mudanca, n })
            }
            "indice_inserir" => json(&s.indice_inserir()),
            "modo_edicao" => json(&s.modo_edicao_definir(&args::<Nome>(corpo)?.nome)?),
            "proteger" => json(&s.proteger(args::<Sim>(corpo)?.sim)),
            "revisoes_resolver" => {
                let a: Resolver = args(corpo)?;
                json(&s.revisoes_resolver(a.aceitar, a.tudo))
            }
            "comentario_novo" => json(&s.comentario_novo(&args::<NovoComentario>(corpo)?.texto)?),
            "comentario_mudar" => {
                let a: ComentarioMudado = args(corpo)?;
                json(&s.comentario_mudar(a.id, a.texto.as_deref(), a.resposta.as_deref(), a.resolvido)?)
            }
            "comentario_excluir" => json(&s.comentario_excluir(args::<PorId>(corpo)?.id)),
            "comentario_ir" => json(&s.comentario_ir(args::<PorId>(corpo)?.id)),
            "documento_comparar_fdoz" | "documento_comparar_md" | "documento_comparar_tex" | "documento_comparar_docx" => {
                let formato = featherdocs_io::Formato::da_extensao(nome.rsplit('_').next().unwrap_or("")).ok_or("Formato desconhecido.")?;
                let antigo = featherdocs_io::abrir(formato, corpo)?;
                let rotulo = self.contexto.rotulo_da_comparacao();
                json(&self.sessao.comparar_com(&antigo, &rotulo))
            }
            "objetos_contexto" => {
                self.contexto = args(corpo)?;
                json(&self.contexto.aplicar(&mut self.sessao))
            }
            _ => Err(format!("Comando desconhecido: {nome}")),
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn chamar(p: &mut Ponte, nome: &str, corpo: &str) -> serde_json::Value {
        serde_json::from_slice(&p.chamar(nome, corpo.as_bytes()).unwrap()).unwrap()
    }

    #[test]
    fn nota_inserida_editada_e_achada_na_pagina() {
        let mut p = Ponte::default();
        chamar(&mut p, "texto_inserir", r#"{"texto":"Olá"}"#);
        chamar(&mut p, "nota_inserir", r#"{"fonte":"uma *nota*"}"#);
        let bloco = p.sessao.cursor().foco.bloco;
        let info = chamar(&mut p, "nota_info", &format!(r#"{{"bloco":{bloco},"desl":3}}"#));
        assert_eq!(info["fonte"], "uma *nota*");
        assert_eq!(info["numero"], "1");
        chamar(&mut p, "nota_editar", &format!(r#"{{"bloco":{bloco},"desl":3,"fonte":"outra","fim":true}}"#));
        assert_eq!(chamar(&mut p, "nota_info", &format!(r#"{{"bloco":{bloco},"desl":3}}"#))["numero"], "i");
        chamar(&mut p, "notas_formato", r#"{"rodape":0,"fim":0}"#);
        assert_eq!(chamar(&mut p, "nota_info", &format!(r#"{{"bloco":{bloco},"desl":3}}"#))["numero"], "1");
        assert!(p.chamar("campo_inserir", br#"{"campo":"xis"}"#).is_err());
        chamar(&mut p, "objetos_contexto", r#"{"hoje":"06/10/2026","notas":"Notas"}"#);
        chamar(&mut p, "campo_inserir", r#"{"campo":"data"}"#);
        let md = String::from_utf8(p.chamar("documento_exportar_md", b"{}").unwrap()).unwrap();
        assert!(md.contains("06/10/2026"), "{md}");
    }

    #[test]
    fn importar_bib_citar_e_achar_a_citacao_na_pagina() {
        let mut p = Ponte::default();
        chamar(&mut p, "texto_inserir", r#"{"texto":"Segundo "}"#);
        let r = chamar(&mut p, "fontes_importar_bib", r#"{"texto":"@book{silva2023, author={Silva, Ana}, title={Livro}, year={2023}}"}"#);
        assert_eq!(r["n"], 1);
        assert!(p.chamar("fontes_importar_bib", br#"{"texto":"nada"}"#).is_err());
        chamar(&mut p, "citacao_inserir", r#"{"chaves":["silva2023"],"pagina":"4"}"#);
        assert_eq!(chamar(&mut p, "bibliografia", "{}")["fontes"][0]["chave"], "silva2023");
        chamar(&mut p, "estilo_citacao", r#"{"nome":"apa"}"#);
        chamar(&mut p, "bibliografia_inserir", "{}");
        let bloco = p.sessao.documento().blocos[0].id;
        let g = p.sessao.geometria(&Pos { bloco, celula: None, desl: 8 }).unwrap();
        let achado = chamar(&mut p, "objeto_no_ponto", &format!(r#"{{"pagina":0,"x":{},"y":{}}}"#, g.x + 2.0, g.y + g.h / 2.0));
        assert_eq!(achado["marca"]["tipo"], "citacao");
        assert_eq!(achado["marca"]["pagina"], "4");
        let bib = String::from_utf8(p.chamar("fontes_exportar_bib", b"{}").unwrap()).unwrap();
        assert!(bib.starts_with("@book{silva2023,"));
    }

    #[test]
    fn protegido_barra_edicao_mas_deixa_comentar_e_comparar_marca_diferencas() {
        let mut p = Ponte::default();
        chamar(&mut p, "texto_inserir", r#"{"texto":"o gato preto"}"#);
        let antigo = p.chamar("documento_gravar_fdoz", b"").unwrap();
        chamar(&mut p, "proteger", r#"{"sim":true}"#);
        assert!(p.chamar("papel_definir", br#"{"nome":"carta"}"#).unwrap_err().contains("só para leitura"));
        chamar(&mut p, "selecionar_tudo", "{}");
        chamar(&mut p, "comentario_novo", r#"{"texto":"Bom"}"#);
        assert_eq!(chamar(&mut p, "comentarios", "{}")[0]["texto"], "Bom");
        chamar(&mut p, "proteger", r#"{"sim":false}"#);
        chamar(&mut p, "modo_edicao", r#"{"nome":"sugerindo"}"#);
        assert_eq!(chamar(&mut p, "estado", "{}")["modo_edicao"], "sugerindo");
        chamar(&mut p, "modo_edicao", r#"{"nome":"editando"}"#);
        chamar(&mut p, "tecla", r#"{"tecla":"End","ctrl":true}"#);
        chamar(&mut p, "texto_inserir", r#"{"texto":" e branco"}"#);
        p.chamar("documento_comparar_fdoz", &antigo).unwrap();
        let r = chamar(&mut p, "revisoes", "{}");
        assert_eq!(r[0]["tipo"], "insercao");
        assert_eq!(r[0]["autor"], "Comparação");
        assert_eq!(chamar(&mut p, "estado", "{}")["tem_revisoes"], true);
        chamar(&mut p, "revisoes_resolver", r#"{"aceitar":false,"tudo":true}"#);
        assert_eq!(chamar(&mut p, "estado", "{}")["tem_revisoes"], false);
    }
}
