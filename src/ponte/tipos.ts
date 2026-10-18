/** Os tipos das respostas da ponte (espelham as structs de `crates/featherdocs-ponte`). */

export interface Estado {
  versao: number;
  paginas: number;
  blocos: number;
  palavras: number;
  caracteres: number;
  pode_desfazer: boolean;
  pode_refazer: boolean;
  sujo: boolean;
  titulo: string;
  largura: number;
  altura: number;
  margem_esq: number;
  margem_dir: number;
  margem_sup: number;
  margem_inf: number;
  /** `a4`, `carta`, `oficio`, `a3`, `a5` ou `personalizado`. */
  papel: Papel | 'personalizado';
  paisagem: boolean;
  /** Marcas de formatação (¶ ·) à vista. */
  marcas: boolean;
  /** Linhas de grade à vista. */
  grade: boolean;
  extras: Extras;
  /** Os estilos com nome e a numeração dos títulos. */
  estilos: Estilos;
  /** O estilo das citações e das referências. */
  estilo_citacao: EstiloDeCitacao;
  modo_edicao: ModoDeEdicao;
  /** Há sugestões à espera de aceitar ou rejeitar. */
  tem_revisoes: boolean;
}

export type ModoDeEdicao = 'editando' | 'sugerindo' | 'leitura';

/** Uma resposta num comentário. */
export interface Resposta {
  autor: string;
  quando: string;
  texto: string;
}

/** Um comentário, como o painel de revisão mostra. */
export interface InfoComentario {
  id: number;
  autor: string;
  quando: string;
  texto: string;
  resolvido: boolean;
  respostas: Resposta[];
  trecho: string;
  bloco: number;
  pagina: number;
}

/** Uma alteração sugerida. */
export interface InfoRevisao {
  tipo: 'insercao' | 'remocao';
  autor: string;
  quando: string;
  texto: string;
  bloco: number;
}

/** A numeração automática dos títulos. */
export type Numeracao = 'nenhuma' | 'decimal' | 'capitulo';

/** Um estilo com nome, como o motor guarda (bits de letra e cor `0xRRGGBBAA`). */
export interface EstiloComNome {
  id: number;
  nome: string;
  /** 0 = Normal; 1 a 6 = Título 1 a 6. */
  base: number;
  bits: number;
  /** 0 = o da base. */
  tamanho: number;
  /** 0 = automática. */
  cor: number;
  alinhamento: Alinhamento;
  antes: number | null;
  depois: number | null;
  /** 0 = a padrão. */
  entrelinha: number;
}

export interface Estilos {
  numeracao: Numeracao;
  lista: EstiloComNome[];
}

/** O que a caixa "Novo estilo" manda ao motor. */
export interface DadosDoEstilo {
  nome: string;
  base: number;
  fonte: 'sans' | 'serifa' | 'mono';
  negrito: boolean;
  italico: boolean;
  sublinhado: boolean;
  tamanho: number;
  /** `#rrggbb`; vazio = automática. */
  cor: string;
  alinhamento: Alinhamento;
  antes: number | null;
  depois: number | null;
  entrelinha: number;
}

/** O que uma edição mudou: a versão nova e onde. `afetados` vazio = o documento não mudou. */
export interface Mudanca {
  versao: number;
  paginas: number;
  afetados: number[];
  primeira_pagina: number;
}

/** A posição do cursor na página (pontos): o topo da barrinha e a altura. */
export interface Geo {
  pagina: number;
  x: number;
  y: number;
  h: number;
}

/** Um retângulo de seleção numa página (pontos). */
export interface Ret {
  pagina: number;
  x: number;
  y: number;
  w: number;
  h: number;
}

/** A formatação no cursor, para pintar os botões. */
/** Os estilos de lista do motor. */
export type EstiloDeLista = 'padrao' | 'circulo' | 'quadrado' | 'seta' | 'tarefa' | 'numerica' | 'alfabetica' | 'romana';

/** `esquerda`/`centro`/`direita`/`justificado`. */
export type Alinhamento = 'esquerda' | 'centro' | 'direita' | 'justificado';

export interface Formato {
  negrito: boolean;
  italico: boolean;
  sublinhado: boolean;
  tachado: boolean;
  codigo: boolean;
  alinhamento: Alinhamento;
  /** `sans`, `serifa` ou `mono`. */
  fonte: 'sans' | 'serifa' | 'mono';
  /** O corpo em pontos no cursor. */
  tamanho: number;
  /** `#rrggbb`, ou vazio (automática). */
  cor: string;
  realce: string;
  sobrescrito: boolean;
  subscrito: boolean;
  entrelinha: number;
  /** Espaço antes e depois do parágrafo, em pontos. */
  antes: number;
  depois: number;
  fundo: string;
  /** Bordas: 1 esquerda, 2 direita, 4 em cima, 8 embaixo. */
  borda: number;
  lista: EstiloDeLista;
  marcado: boolean;
  pincel: boolean;
  /** O endereço do link no cursor. */
  link: string | null;
  tabela_cabecalho: boolean;
  /** `info`, `dica`, `aviso`, `atencao` ou vazio. */
  chamada: string;
  /** Recuo esquerdo do parágrafo no cursor, em pontos. */
  recuo: number;
  recuo_dir: number;
  primeira: number;
  estilo: string;
  /** O estilo com nome do parágrafo no cursor (0 = só o de fábrica). */
  estilo_proprio: number;
  /** O `id` do bloco do cursor. */
  bloco: number;
  /** O título no cursor está fora da numeração automática. */
  sem_numero: boolean;
  tem_selecao: boolean;
  /** O `id` da tabela, se o cursor está numa célula. */
  tabela: number | null;
}

/** Tudo para desenhar o cursor depois de uma ação. */
export interface Visao {
  cursor: Geo | null;
  retangulos: Ret[];
  /** Os achados da busca à vista (o realce). */
  achados: Ret[];
  formato: Formato;
}

export interface ResultadoDeTecla {
  tratada: boolean;
  mudanca: Mudanca;
}

export interface Copiado {
  texto: string;
  mudanca: Mudanca | null;
}

export interface Titulo {
  id: number;
  nivel: number;
  texto: string;
  pagina: number;
}

export type Papel = 'a4' | 'carta' | 'oficio' | 'a3' | 'a5';

export type FormatoDeArquivo = 'fdoz' | 'md' | 'tex' | 'docx';

/** Convencional (salva em `.fdoz` ou `.docx`) ou código (salva em `.tex` ou `.md`). */
export type Vertente = 'convencional' | 'latex' | 'markdown';

export interface InfoDeVertente {
  vertente: Vertente;
  /** As extensões em que o documento salva; a primeira é a padrão. */
  formatos: FormatoDeArquivo[];
}

/** O resultado de salvar. Com `perdas`, nada foi gravado: falta o usuário aceitar a conversão. */
export interface Salvo {
  nome: string;
  formato: FormatoDeArquivo;
  perdas?: string[];
  caminho?: string;
}

/** Uma operação de desenho do quadro (coordenadas em pontos, do canto superior esquerdo da página). */
export type Operacao =
  | { op: 'texto'; x: number; y: number; tamanho: number; estilo: number; cor: number; texto: string }
  | { op: 'retangulo'; x: number; y: number; w: number; h: number; cor: number }
  | { op: 'traco'; x1: number; y1: number; x2: number; y2: number; espessura: number; cor: number }
  | { op: 'imagem'; x: number; y: number; w: number; h: number; origem: string }
  /** Texto girado em volta do centro `(x, y)` (a marca d'água); `angulo` em graus, anti-horário. */
  | { op: 'girado'; x: number; y: number; tamanho: number; angulo: number; estilo: number; cor: number; texto: string };

export interface Quadro {
  versao: number;
  largura: number;
  altura: number;
  operacoes: Operacao[];
}

/** Bits de `estilo` (iguais aos de `featherdocs_core::modelo`). */
export const SERIFA = 64;
export const NEGRITO = 1;
export const ITALICO = 2;
export const CODIGO = 4;
export const SUBLINHADO = 8;
export const TACHADO = 16;
export const MATEMATICA = 32;

/** As opções do painel de busca. */
export interface OpcoesDeBusca {
  maiusculas: boolean;
  palavra: boolean;
  regex: boolean;
}

/** O "3 de 12" do painel (ou o erro da expressão regular). */
export interface ResultadoDeBusca {
  total: number;
  atual: number;
  erro: string | null;
}

export interface Substituido {
  mudanca: Mudanca;
  total: number;
  atual: number;
  trocados: number;
}

/** Cabeçalho ou rodapé: os três lugares de texto (com `{pagina}`, `{total}`, `{titulo}`…). */
export interface Faixas {
  esquerda: string;
  centro: string;
  direita: string;
}

/** O que vale para a página toda: cabeçalho, rodapé e plano de fundo. */
export interface Extras {
  cabecalho: Faixas;
  rodape: Faixas;
  primeira_diferente: boolean;
  cor_pagina: number;
  marca: string;
  marca_cor: number;
  borda_cor: number;
  borda_espessura: number;
  borda_distancia: number;
  vertical: 'topo' | 'centro' | 'base';
  numerar_linhas: 'nenhuma' | 'continua' | 'por_pagina';
  /** O número das notas: 0 = 1, 2, 3 · 1 = a, b, c · 2 = i, ii, iii · 3 = *, †, ‡. */
  formato_notas: number;
  formato_notas_fim: number;
  controlar_alteracoes: boolean;
  protegido: boolean;
}

/** Um ponto do texto (o mesmo que o cursor tem). */
export interface Ponto {
  bloco: number;
  celula: [number, number] | null;
  desl: number;
}

/** Um autor de uma fonte. */
export interface Autor {
  sobrenome: string;
  nome: string;
}

export type TipoDeFonte = 'livro' | 'capitulo' | 'artigo' | 'site' | 'tese' | 'relatorio' | 'outro';
export type EstiloDeCitacao = 'abnt' | 'apa' | 'vancouver' | 'ieee' | 'chicago';

/** Uma fonte citada (o `veiculo` é o periódico, o livro do capítulo ou o curso da tese). */
export interface Fonte {
  chave: string;
  tipo: TipoDeFonte;
  autores: Autor[];
  titulo: string;
  subtitulo: string;
  ano: string;
  edicao: string;
  local: string;
  editora: string;
  veiculo: string;
  volume: string;
  numero: string;
  paginas: string;
  url: string;
  acesso: string;
  doi: string;
}

export interface Bibliografia {
  estilo: EstiloDeCitacao;
  fontes: Fonte[];
}

/** Um objeto preso ao texto. */
export type Marca =
  | { tipo: 'nota'; fim?: boolean; texto: unknown[] }
  | { tipo: 'citacao'; chaves: string[]; pagina?: string }
  | { tipo: 'indice'; termo: string; sub?: string }
  | { tipo: 'campo'; campo: string }
  | { tipo: 'comentario'; id: number; fim?: boolean };

/** O objeto achado sob um ponto da página e onde ele está. */
export interface ObjetoAchado {
  ponto: Ponto;
  marca: Marca;
}

/** A nota cuja chamada está num ponto. */
export interface InfoNota {
  fonte: string;
  fim: boolean;
  numero: string;
}

/** Revisão → Contagem de palavras. */
export interface Contagem {
  palavras: number;
  caracteres: number;
  caracteres_sem_espaco: number;
  paragrafos: number;
  linhas: number;
  paginas: number;
  selecao: boolean;
}

/** O modo dividido: o código exportado e as linhas `[de, ate)` (de 0) de cada bloco nele. */
export interface MapaDoCodigo {
  texto: string;
  blocos: Array<{ id: number; de: number; ate: number }>;
}

/** A língua do código no modo dividido. */
export type FormatoDoCodigo = 'md' | 'tex';

/** Um título, uma legenda ou um marcador que a referência cruzada pode apontar. */
export interface AlvoDeReferencia {
  tipo: 'titulo' | 'figura' | 'tabela' | 'equacao' | 'marcador';
  id: number;
  texto: string;
  numero: string;
  pagina: number;
}
