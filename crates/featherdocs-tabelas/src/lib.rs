//! Tabelas do FeatherDocs pelo motor comum `feather-tables` (antes a parte "docs" do
//! `featheroffice-table-adapters` do monorepo, trazida para cá em 18/10/2026).
//!
//! Não muda a serialização de FSDZ/FDOZ, nem faz os motores dos aplicativos
//! dependerem do crate compartilhado. Os atributos fora do subconjunto comum
//! permanecem no modelo original (parâmetro `original` da volta).
//! Ao tentar modificar conteúdo que não cabe no modelo nativo, retorna erro.

pub mod edicao;
pub use edicao::{docs_desmesclar_celulas, docs_editar_celula, docs_editar_estrutura, docs_mesclar_celulas};

use featherdocs_core::modelo::{Conteudo, Trecho};
use feather_tables::{
    Celula, ErroTabela, PoliticaBordas, Tabela, Valor,
};

/// Limite de trabalho explícito para não materializar tabelas hostis.
const MAX_CELULAS: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroAdaptacao {
    TipoInvalido,
    GradeIrregular,
    DimensoesExcessivas,
    DadosInvalidos,
    FormatoIncompativel,
    MesclagensNaoSuportadas,
    TextoRicoAlterado,
}

impl From<ErroTabela> for ErroAdaptacao {
    fn from(_: ErroTabela) -> Self {
        Self::DadosInvalidos
    }
}

fn validar_dimensoes(linhas: usize, colunas: usize) -> Result<(), ErroAdaptacao> {
    if (linhas == 0 && colunas != 0)
        || linhas > MAX_CELULAS
        || colunas > MAX_CELULAS
        || linhas.checked_mul(colunas).map_or(true, |n| n > MAX_CELULAS)
    {
        return Err(ErroAdaptacao::DimensoesExcessivas);
    }
    Ok(())
}

/// Por ora os formatos nativos continuam donos da formatação. O retorno não
/// deve descartar estilos/margens/mesclagens editados no modelo compartilhado:
/// essas operações pertencem à futura migração TBL-010.
fn validar_sem_atributos_novos(tabela: &Tabela) -> Result<(), ErroAdaptacao> {
    let (linhas, colunas) = tabela.dimensoes();
    validar_dimensoes(linhas, colunas)?;
    if !tabela.mesclagens().is_empty() {
        return Err(ErroAdaptacao::MesclagensNaoSuportadas);
    }
    if tabela.iter().any(|(_, c)| c.estilo != Default::default())
        || tabela.iter_layout().next().is_some()
        || tabela.politica_bordas() != PoliticaBordas::Separadas
    {
        return Err(ErroAdaptacao::FormatoIncompativel);
    }
    if (0..linhas).any(|linha| tabela.altura(linha).is_some())
        || (0..colunas).any(|coluna| tabela.largura(coluna).is_some())
    {
        return Err(ErroAdaptacao::FormatoIncompativel);
    }
    Ok(())
}

fn texto_da_celula(tabela: &Tabela, linha: usize, coluna: usize) -> Result<String, ErroAdaptacao> {
    match tabela.obter(linha, coluna).map(|c| &c.valor) {
        None | Some(Valor::Vazio) => Ok(String::new()),
        Some(Valor::Texto(t)) => Ok(t.clone()),
        // Os modelos nativos são textuais: não converter número ou booleano
        // implicitamente, pois uma volta alteraria o tipo semântico do valor.
        Some(Valor::Numero(_)) | Some(Valor::Logico(_)) => {
            Err(ErroAdaptacao::FormatoIncompativel)
        }
    }
}

fn armazenar_texto(tabela: &mut Tabela, r: usize, c: usize, texto: String) -> Result<(), ErroAdaptacao> {
    if !texto.is_empty() {
        tabela.definir(r, c, Celula {
            valor: Valor::Texto(texto),
            ..Celula::default()
        })?;
    }
    Ok(())
}

fn texto_docs(celula: &[Trecho]) -> String {
    celula.iter().map(|trecho| trecho.texto.as_str()).collect()
}

/// Projeta os trechos formatados do Docs em texto editável compartilhado.
/// Estilos complexos são mantidos em `original`, não achatados na saída:
/// `comum_para_docs` os reconstrói sem perdas para células inalteradas.
pub fn docs_para_comum(tabela_docs: &Conteudo) -> Result<Tabela, ErroAdaptacao> {
    let Conteudo::Tabela { linhas, .. } = tabela_docs else {
        return Err(ErroAdaptacao::TipoInvalido);
    };
    let colunas = linhas.first().map_or(0, Vec::len);
    if linhas.iter().any(|linha| linha.len() != colunas) {
        return Err(ErroAdaptacao::GradeIrregular);
    }
    validar_dimensoes(linhas.len(), colunas)?;
    let mut comum = Tabela::nova(linhas.len(), colunas);
    for (r, linha) in linhas.iter().enumerate() {
        for (c, celula) in linha.iter().enumerate() {
            armazenar_texto(&mut comum, r, c, texto_docs(celula))?;
        }
    }
    Ok(comum)
}

/// Preserva a segmentação, hyperlinks, marcações, revisões e atributos
/// `Fmt` das células cujo texto não mudou. Uma célula com vários trechos
/// e conteúdo editado não pode preservar posições de estilos: recusa a
/// conversão; a GUI deve então usar seu editor nativo de trechos.
fn repor_celula(novo: &str, original: Option<&Vec<Trecho>>) -> Result<Vec<Trecho>, ErroAdaptacao> {
    let Some(originais) = original else {
        return Ok(if novo.is_empty() { Vec::new() } else { vec![Trecho::simples(novo)] });
    };
    if texto_docs(originais) == novo {
        return Ok(originais.clone());
    }
    if originais.len() > 1
        || originais.first().is_some_and(|t| t.marca.is_some() || t.rev.is_some())
    {
        return Err(ErroAdaptacao::TextoRicoAlterado);
    }
    if let Some(primeiro) = originais.first() {
        let mut trecho = primeiro.clone();
        trecho.texto = novo.to_string();
        trecho.rotulo.clear();
        Ok(vec![trecho])
    } else if novo.is_empty() {
        Ok(Vec::new())
    } else {
        Ok(vec![Trecho::simples(novo)])
    }
}

/// Converte de volta para `Conteudo::Tabela`, preservando cabeçalho e
/// trechos formatados originais. Aceita mudanças textuais simples e expansões
/// retangulares; rejeita atributos do motor comum sem correspondente nativo.
pub fn comum_para_docs(tabela: &Tabela, original: &Conteudo) -> Result<Conteudo, ErroAdaptacao> {
    let Conteudo::Tabela { cabecalho, linhas: originais } = original else {
        return Err(ErroAdaptacao::TipoInvalido);
    };
    docs_para_comum(original)?;
    validar_sem_atributos_novos(tabela)?;
    let (altura, largura) = tabela.dimensoes();
    let mut linhas = Vec::with_capacity(altura);
    for r in 0..altura {
        let mut linha = Vec::with_capacity(largura);
        for c in 0..largura {
            let novo = texto_da_celula(tabela, r, c)?;
            linha.push(repor_celula(
                &novo,
                originais.get(r).and_then(|v| v.get(c)),
            )?);
        }
        linhas.push(linha);
    }
    Ok(Conteudo::Tabela { cabecalho: *cabecalho, linhas })
}
