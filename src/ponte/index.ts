/**
 * O motor visto do TypeScript: cada método é um comando da ponte, tipado. Aqui
 * não há regra de documento — só montar o pedido e entregar a resposta. O cursor, a
 * seleção e toda decisão de edição são do Rust.
 */

import { chamarJson, type Canal } from './canal.ts';
import { canalMedido } from './metricas.ts';
import { decodificarQuadro } from './quadro.ts';
import type { AlvoDeReferencia, Bibliografia, EstiloDeCitacao, Fonte, InfoComentario, InfoNota, InfoRevisao, ModoDeEdicao, ObjetoAchado, Ponto, Alinhamento, Contagem, Copiado, DadosDoEstilo, EstiloDeLista, FormatoDoCodigo, MapaDoCodigo, Numeracao, OpcoesDeBusca, ResultadoDeBusca, Substituido, Estado, FormatoDeArquivo, InfoDeVertente, Mudanca, Papel, Quadro, ResultadoDeTecla, Titulo, Visao } from './tipos.ts';

/**
 * Os comandos que a interface chama. Um teste trava esta lista contra
 * `COMANDOS_DA_PONTE` do Rust; na largada, se o motor não atende algum, a janela
 * mostra "motor desatualizado".
 */
export const COMANDOS_DA_PONTE_ESPERADOS = [
  'comandos_da_ponte',
  'estado',
  'documento_novo',
  'documento_abrir_fdoz',
  'documento_abrir_md',
  'documento_abrir_tex',
  'documento_abrir_docx',
  'documento_gravar_fdoz',
  'documento_gravar_docx',
  'documento_vertente',
  'documento_perdas',
  'documento_converter',
  'codigo_cores',
  'documento_exportar_md',
  'documento_exportar_tex',
  'documento_texto_simples',
  'documento_marcar_gravado',
  'documento_estrutura',
  'papel_definir',
  'papel_personalizado',
  'orientacao_definir',
  'margens_definir',
  'meta_definir',
  'sumario_rotulo_definir',
  'pagina_quadro',
  'cursor_visao',
  'texto_inserir',
  'tecla',
  'clicar',
  'clicar_direito',
  'selecionar_tudo',
  'cursor_ir_para',
  'copiar',
  'recortar',
  'colar',
  'formatar',
  'paragrafo_estilo',
  'inserir_bloco',
  'nivel',
  'alinhar',
  'recuar',
  'recuos_definir',
  'fonte',
  'tamanho',
  'tamanho_passo',
  'cor',
  'realce',
  'posicao',
  'limpar_formatacao',
  'maiusculas',
  'pincel_copiar',
  'pincel_aplicar',
  'entrelinha',
  'espaco',
  'fundo',
  'borda',
  'lista',
  'lista_reiniciar',
  'tarefa_alternar',
  'clicar_tarefa',
  'busca_definir',
  'busca_ir',
  'busca_limpar',
  'substituir',
  'substituir_tudo',
  'inserir_tabela',
  'inserir_imagem',
  'inserir_sumario',
  'remover_sumario',
  'link_definir',
  'link_remover',
  'chamada',
  'contagem',
  'tabela_linha',
  'tabela_coluna',
  'tabela_excluir',
  'tabela_cabecalho',
  'cabecalho_definir',
  'rodape_definir',
  'primeira_diferente',
  'numero_de_pagina',
  'numero_remover',
  'cor_pagina',
  'marca_dagua',
  'borda_pagina',
  'marcas',
  'desfazer',
  'refazer',
  'tabela_csv',
  'tabela_csv_substituir',
  'estilo_aplicar',
  'estilo_criar',
  'estilo_modificar',
  'estilo_excluir',
  'numeracao_definir',
  'titulo_sem_numero',
  'capitulo_rotulo_definir',
  'codigo_exportar',
  'codigo_aplicar',
  'sumario_configurar',
  'grade',
  'vertical_definir',
  'numerar_linhas_definir',
  'legenda_inserir',
  'lista_inserir',
  'legendas_rotulos_definir',
  'marcador_adicionar',
  'marcador_remover',
  'referencia_alvos',
  'referencia_inserir',
  'referencias_atualizar',
  'nota_inserir',
  'nota_info',
  'nota_editar',
  'nota_no_ponto',
  'notas_formato',
  'campo_inserir',
  'objetos_contexto',
  'objeto_no_ponto',
  'bibliografia',
  'fontes_exportar_bib',
  'fonte_salvar',
  'fonte_remover',
  'fontes_importar_bib',
  'estilo_citacao',
  'citacao_inserir',
  'citacao_editar',
  'bibliografia_inserir',
  'indice_marcar',
  'indice_inserir',
  'selecao_texto',
  'revisoes',
  'comentarios',
  'comentario_no_ponto',
  'modo_edicao',
  'proteger',
  'revisoes_resolver',
  'comentario_novo',
  'comentario_mudar',
  'comentario_excluir',
  'comentario_ir',
  'documento_comparar_fdoz',
  'documento_comparar_md',
  'documento_comparar_tex',
  'documento_comparar_docx',
] as const;

/** As formatações que o motor conhece (`Ctrl+B` e companhia, e os botões). */
export type Estilo = 'negrito' | 'italico' | 'sublinhado' | 'tachado' | 'codigo';

const utf8 = new TextDecoder();

export class Motor {
  private readonly canal: Canal;

  constructor(canal: Canal) {
    this.canal = canalMedido(canal);
  }

  private json<T>(comando: string, args?: unknown): T {
    return chamarJson<T>(this.canal, comando, args);
  }

  /** Quais comandos o motor atende (para detectar motor desatualizado). */
  comandosDaPonte(): string[] {
    return this.json('comandos_da_ponte');
  }

  /** Falta algum comando que a interface usa? */
  comandosFaltando(): string[] {
    const tem = new Set(this.comandosDaPonte());
    return COMANDOS_DA_PONTE_ESPERADOS.filter((c) => !tem.has(c));
  }

  // --- documento ---------------------------------------------------------------

  estado(): Estado {
    return this.json('estado');
  }

  novo(): Estado {
    return this.json('documento_novo');
  }

  /** Abre os bytes de um arquivo no formato dado. */
  abrir(formato: FormatoDeArquivo, bytes: Uint8Array): Estado {
    return this.json(`documento_abrir_${formato}`, bytes);
  }

  gravarFdoz(): Uint8Array {
    return this.canal.chamar('documento_gravar_fdoz');
  }

  /** As cores do código (realce de sintaxe): trios planos `[início, fim, tipo, …]` em UTF-16. */
  codigoCores(formato: FormatoDoCodigo, texto: string): number[] {
    return this.json('codigo_cores', { formato, texto });
  }

  gravarDocx(): Uint8Array {
    return this.canal.chamar('documento_gravar_docx');
  }

  /** A vertente do documento (convencional, LaTeX ou Markdown) e os formatos em que ele salva. */
  vertente(): InfoDeVertente {
    return this.json('documento_vertente');
  }

  /** O que se perde ao gravar o documento no `formato` (chaves de `perda.*`); vazio = nada. */
  perdas(formato: string): string[] {
    return this.json('documento_perdas', { formato });
  }

  /** Grava como o texto de outra vertente de código, que passa a ser a do documento. */
  converter(formato: 'md' | 'tex'): string {
    return utf8.decode(this.canal.chamar('documento_converter', { formato }));
  }

  exportarTexto(formato: 'md' | 'tex'): string {
    return utf8.decode(this.canal.chamar(`documento_exportar_${formato}`));
  }

  marcarGravado(): Estado {
    return this.json('documento_marcar_gravado');
  }

  papel(nome: Papel): Mudanca {
    return this.json('papel_definir', { nome });
  }

  /** As margens do documento, em pontos. O motor recusa as que não deixam espaço para o texto. */
  /** Papel de medida própria, em pontos. */
  papelPersonalizado(largura: number, altura: number): Mudanca {
    return this.json('papel_personalizado', { largura, altura });
  }

  orientacao(paisagem: boolean): Mudanca {
    return this.json('orientacao_definir', { paisagem });
  }

  margens(esq: number, dir: number, sup: number, inf: number): Mudanca {
    return this.json('margens_definir', { esq, dir, sup, inf });
  }

  meta(titulo: string, autor: string): Estado {
    return this.json('meta_definir', { titulo, autor });
  }

  /** O título do sumário, na língua da interface. */
  rotuloDoSumario(rotulo: string): Estado {
    return this.json('sumario_rotulo_definir', { rotulo });
  }

  estrutura(): Titulo[] {
    return this.json('documento_estrutura');
  }

  /** O quadro da página; `null` se a versão que a interface tem ainda vale. */
  quadro(pagina: number, versao: number): Quadro | null {
    return decodificarQuadro(this.canal.chamar('pagina_quadro', { pagina, versao }));
  }

  // --- cursor e seleção ----------------------------------------------------------

  /** O cursor, a seleção (nas páginas `de..ate`, as à vista) e a formatação no cursor. */
  visao(de: number, ate: number): Visao {
    return this.json('cursor_visao', { de, ate });
  }

  /** Clique (ou arrasto, com `estender`) na página, em pontos. `contagem`: 1, 2 (palavra), 3 (parágrafo). */
  clicar(pagina: number, x: number, y: number, estender = false, contagem = 1): void {
    this.json('clicar', { pagina, x, y, estender, contagem });
  }

  /** O botão direito: guarda a seleção se o clique cai nela; senão, leva o cursor para lá. */
  clicarDireito(pagina: number, x: number, y: number): void {
    this.json('clicar_direito', { pagina, x, y });
  }

  selecionarTudo(): void {
    this.json('selecionar_tudo');
  }

  /** Põe o cursor no começo do bloco `id` (um título da estrutura). */
  irParaBloco(id: number): void {
    this.json('cursor_ir_para', { id });
  }

  /** Uma tecla de comando (setas, Enter, Backspace, Ctrl+B…). `tratada = false`: o navegador faz o que quiser. */
  tecla(tecla: string, ctrl: boolean, shift: boolean, alt: boolean): ResultadoDeTecla {
    return this.json('tecla', { tecla, ctrl, shift, alt });
  }

  // --- edição -----------------------------------------------------------------------

  /** Escreve no cursor (no lugar da seleção); aceita o texto de uma colagem, com quebras de linha. */
  inserirTexto(texto: string): Mudanca {
    return this.json('texto_inserir', { texto });
  }

  copiar(): string {
    return this.json<Copiado>('copiar').texto;
  }

  recortar(): Copiado {
    return this.json('recortar');
  }

  colar(texto: string, semFormatacao = false): Mudanca {
    return this.json('colar', { texto, sem_formatacao: semFormatacao });
  }

  formatar(estilo: Estilo): Mudanca {
    return this.json('formatar', { estilo });
  }

  /** `paragrafo`, `titulo1`…`titulo6`, `item`, `item_ordenado`, `citacao`, `codigo`, `equacao`. */
  paragrafoEstilo(tipo: string): Mudanca {
    return this.json('paragrafo_estilo', { tipo });
  }

  /** `tabela`, `sumario`, `regua`, `quebra_de_pagina`, `codigo`, `equacao`. */
  inserirBloco(tipo: string): Mudanca {
    return this.json('inserir_bloco', { tipo });
  }

  nivel(delta: number): Mudanca {
    return this.json('nivel', { delta });
  }

  /** `esquerda`, `centro`, `direita`, `justificado`. */
  alinhar(como: Alinhamento): Mudanca {
    return this.json('alinhar', { como });
  }

  /** Mais (`1`) ou menos (`-1`) recuo: nível nos itens de lista, 1,25 cm nos demais parágrafos. */
  recuar(delta: number): Mudanca {
    return this.json('recuar', { delta });
  }

  /** Recuos esquerdo, direito e da primeira linha, em pontos (a régua). */
  recuosDefinir(esq: number, dir: number, primeira: number): Mudanca {
    return this.json('recuos_definir', { esq, dir, primeira });
  }

  /** `sans`, `serifa` ou `mono`. */
  fonte(nome: 'sans' | 'serifa' | 'mono'): Mudanca {
    return this.json('fonte', { nome });
  }

  /** Corpo em pontos (1 a 400); 0 volta ao do tipo de bloco. */
  tamanho(pt: number): Mudanca {
    return this.json('tamanho', { pt });
  }

  /** Aumentar (`1`) ou diminuir (`-1`) a fonte. */
  tamanhoPasso(delta: number): Mudanca {
    return this.json('tamanho_passo', { delta });
  }

  /** `#rrggbb`; vazio = automática. */
  cor(cor: string): Mudanca {
    return this.json('cor', { cor });
  }

  /** `#rrggbb`; vazio = sem realce. */
  realce(cor: string): Mudanca {
    return this.json('realce', { cor });
  }

  /** Sobrescrito (`1`) ou subscrito (`-1`). */
  posicao(p: 1 | -1): Mudanca {
    return this.json('posicao', { p });
  }

  limparFormatacao(): Mudanca {
    return this.json('limpar_formatacao');
  }

  /** `frase`, `minusculas`, `maiusculas`, `palavras`, `alternar`. */
  maiusculas(nome: string): Mudanca {
    return this.json('maiusculas', { nome });
  }

  pincelCopiar(): void {
    this.json('pincel_copiar');
  }

  pincelAplicar(): Mudanca {
    return this.json('pincel_aplicar');
  }

  /** Entrelinha como múltiplo (1,0 · 1,15 · 1,5 · 2,0); 0 = padrão. */
  entrelinha(fator: number): Mudanca {
    return this.json('entrelinha', { fator });
  }

  /** Espaço antes e depois do parágrafo em pontos; `null` = o do tipo de bloco. */
  espaco(antes: number | null, depois: number | null): Mudanca {
    return this.json('espaco', { antes, depois });
  }

  /** Cor de fundo do parágrafo (`#rrggbb`; vazio = sem). */
  fundo(cor: string): Mudanca {
    return this.json('fundo', { cor });
  }

  /** Bordas do parágrafo: 1 esquerda, 2 direita, 4 em cima, 8 embaixo. */
  borda(lados: number): Mudanca {
    return this.json('borda', { lados });
  }

  lista(nome: EstiloDeLista): Mudanca {
    return this.json('lista', { nome });
  }

  listaReiniciar(): Mudanca {
    return this.json('lista_reiniciar');
  }

  tarefaAlternar(id: number): Mudanca {
    return this.json('tarefa_alternar', { id });
  }

  /** Clique na caixa de uma tarefa: marca/desmarca e devolve a mudança; `null` se não foi numa caixa. */
  cliqueNaTarefa(pagina: number, x: number, y: number): Mudanca | null {
    return this.json('clicar_tarefa', { pagina, x, y });
  }

  buscaDefinir(consulta: string, opcoes: OpcoesDeBusca): ResultadoDeBusca {
    return this.json('busca_definir', { consulta, ...opcoes });
  }

  buscaIr(delta: number): ResultadoDeBusca {
    return this.json('busca_ir', { delta });
  }

  buscaLimpar(): void {
    this.json('busca_limpar');
  }

  substituir(por: string): Substituido {
    return this.json('substituir', { por });
  }

  substituirTudo(por: string): Substituido {
    return this.json('substituir_tudo', { por });
  }

  inserirTabela(linhas: number, colunas: number): Mudanca {
    return this.json('inserir_tabela', { linhas, colunas });
  }

  /** `origem` = data URL; largura e altura naturais em pontos (0 = automático). */
  inserirImagem(origem: string, largura = 0, altura = 0): Mudanca {
    return this.json('inserir_imagem', { origem, largura, altura });
  }

  inserirSumario(niveis: number): Mudanca {
    return this.json('inserir_sumario', { niveis });
  }

  removerSumario(): Mudanca {
    return this.json('remover_sumario');
  }

  /** Liga a seleção ao endereço; sem seleção, insere `texto` já com o link. */
  linkDefinir(url: string, texto = ''): Mudanca {
    return this.json('link_definir', { url, texto });
  }

  linkRemover(): Mudanca {
    return this.json('link_remover');
  }

  /** `info`, `dica`, `aviso`, `atencao` ou `nenhuma`. */
  chamada(nome: string): Mudanca {
    return this.json('chamada', { nome });
  }

  contagem(): Contagem {
    return this.json('contagem');
  }

  tabelaLinha(abaixo: boolean): Mudanca {
    return this.json('tabela_linha', { sim: abaixo });
  }

  tabelaColuna(direita: boolean): Mudanca {
    return this.json('tabela_coluna', { sim: direita });
  }

  /** `linha`, `coluna` ou `tabela`. */
  tabelaExcluir(nome: 'linha' | 'coluna' | 'tabela'): Mudanca {
    return this.json('tabela_excluir', { nome });
  }

  tabelaCabecalho(): Mudanca {
    return this.json('tabela_cabecalho');
  }

  /** Aplica o estilo com nome `id` aos parágrafos da seleção. */
  estiloAplicar(id: number): Mudanca {
    return this.json('estilo_aplicar', { id });
  }

  /** Cria um estilo com nome e já o aplica aos parágrafos da seleção. */
  estiloCriar(estilo: DadosDoEstilo): Mudanca {
    return this.json('estilo_criar', { estilo });
  }

  /** Muda o estilo `id`: todo parágrafo que o usa muda junto. */
  estiloModificar(id: number, estilo: DadosDoEstilo): Mudanca {
    return this.json('estilo_modificar', { id, estilo });
  }

  estiloExcluir(id: number): Mudanca {
    return this.json('estilo_excluir', { id });
  }

  numeracao(nome: Numeracao): Mudanca {
    return this.json('numeracao_definir', { nome });
  }

  /** Tira (ou devolve) o título do cursor da numeração automática. */
  tituloSemNumero(): Mudanca {
    return this.json('titulo_sem_numero');
  }

  /** O modo dividido: o documento como código, com as linhas de cada bloco. */
  codigoExportar(formato: FormatoDoCodigo): MapaDoCodigo {
    return this.json('codigo_exportar', { formato });
  }

  /** O código editado de volta para o documento: só os blocos que mudaram são trocados (um passo de desfazer). */
  codigoAplicar(formato: FormatoDoCodigo, texto: string): Mudanca {
    return this.json('codigo_aplicar', { formato, texto });
  }

  /** Sumário personalizado: muda os sumários que existem (ou insere um). */
  sumarioConfigurar(niveis: number, numeros: boolean, pontilhado: boolean): Mudanca {
    return this.json('sumario_configurar', { niveis, numeros, pontilhado });
  }

  /** Linhas de grade sobre o texto (só na tela). */
  grade(sim: boolean): Mudanca {
    return this.json('grade', { sim });
  }

  vertical(nome: 'topo' | 'centro' | 'base'): Mudanca {
    return this.json('vertical_definir', { nome });
  }

  numerarLinhas(nome: 'nenhuma' | 'continua' | 'por_pagina'): Mudanca {
    return this.json('numerar_linhas_definir', { nome });
  }

  legenda(nome: 'figura' | 'tabela' | 'equacao'): Mudanca {
    return this.json('legenda_inserir', { nome });
  }

  /** Lista de figuras (1) ou de tabelas (2). */
  listaDeLegendas(niveis: 1 | 2): Mudanca {
    return this.json('lista_inserir', { niveis });
  }

  rotulosDeLegenda(r: { figura: string; tabela: string; equacao: string; lista_figuras: string; lista_tabelas: string }): Estado {
    return this.json('legendas_rotulos_definir', r);
  }

  marcadorAdicionar(nome: string): Mudanca {
    return this.json('marcador_adicionar', { nome });
  }

  marcadorRemover(nome: string): Mudanca {
    return this.json('marcador_remover', { nome });
  }

  /** Títulos, legendas e marcadores que uma referência cruzada pode apontar. */
  alvosDeReferencia(): AlvoDeReferencia[] {
    return this.json('referencia_alvos');
  }

  referenciaInserir(id: number, modo: 'texto' | 'numero' | 'pagina'): Mudanca {
    return this.json('referencia_inserir', { id, modo });
  }

  referenciasAtualizar(): Mudanca {
    return this.json('referencias_atualizar');
  }

  /** Uma nota no cursor: de rodapé, ou de fim (`fim`), com o texto em fonte em linha. */
  notaInserir(fonte: string, fim: boolean): Mudanca {
    return this.json('nota_inserir', { fonte, fim });
  }

  /** A nota cuja chamada está em `p`. */
  notaInfo(p: Ponto): InfoNota {
    return this.json('nota_info', p);
  }

  notaEditar(p: Ponto, fonte: string, fim: boolean): Mudanca {
    return this.json('nota_editar', { ...p, fonte, fim });
  }

  /** A chamada da nota sob o ponto da página (no texto, no pé ou na lista do fim), se houver. */
  notaNoPonto(pagina: number, x: number, y: number): Ponto | null {
    return this.json('nota_no_ponto', { pagina, x, y });
  }

  /** O formato do número: 0 = 1, 2, 3 · 1 = a, b, c · 2 = i, ii, iii · 3 = *, †, ‡. */
  notasFormato(rodape: number, fim: number): Mudanca {
    return this.json('notas_formato', { rodape, fim });
  }

  campoInserir(campo: string): Mudanca {
    return this.json('campo_inserir', { campo });
  }

  /** A data de hoje (campo `data`), o nome do arquivo e os títulos das listas geradas. */
  objetosContexto(c: { hoje: string; arquivo: string; notas: string; referencias: string; indice: string; autor: string; comparacao: string }): Mudanca {
    return this.json('objetos_contexto', c);
  }

  /** O objeto (nota, citação, campo, entrada de índice) sob o ponto da página, se houver. */
  objetoNoPonto(pagina: number, x: number, y: number): ObjetoAchado | null {
    return this.json('objeto_no_ponto', { pagina, x, y });
  }

  /** As fontes do documento e o estilo de citação. */
  bibliografia(): Bibliografia {
    return this.json('bibliografia');
  }

  /** As fontes em BibTeX. */
  fontesExportarBib(): string {
    return utf8.decode(this.canal.chamar('fontes_exportar_bib'));
  }

  /** Grava uma fonte nova (`antiga` vazia) ou troca a de chave `antiga`. */
  fonteSalvar(fonte: Fonte, antiga: string): Mudanca {
    return this.json('fonte_salvar', { fonte, antiga });
  }

  fonteRemover(chave: string): Mudanca {
    return this.json('fonte_remover', { chave });
  }

  /** Junta as fontes de um `.bib` às do documento; `n` = quantas. */
  fontesImportarBib(texto: string): { mudanca: Mudanca; n: number } {
    return this.json('fontes_importar_bib', { texto });
  }

  estiloCitacao(nome: EstiloDeCitacao): Mudanca {
    return this.json('estilo_citacao', { nome });
  }

  citacaoInserir(chaves: string[], pagina: string): Mudanca {
    return this.json('citacao_inserir', { chaves, pagina });
  }

  citacaoEditar(p: Ponto, chaves: string[], pagina: string): Mudanca {
    return this.json('citacao_editar', { ...p, chaves, pagina });
  }

  bibliografiaInserir(todas: boolean): Mudanca {
    return this.json('bibliografia_inserir', { todas });
  }

  /** Marca o termo do índice no fim da seleção (ou depois de toda ocorrência, com `todas`). */
  indiceMarcar(termo: string, sub: string, todas: boolean): { mudanca: Mudanca; n: number } {
    return this.json('indice_marcar', { termo, sub, todas });
  }

  indiceInserir(): Mudanca {
    return this.json('indice_inserir');
  }

  /** Revisão → Modo: editando, sugerindo (controle de alterações) ou somente leitura. */
  modoEdicao(nome: ModoDeEdicao): Mudanca {
    return this.json('modo_edicao', { nome });
  }

  proteger(sim: boolean): Mudanca {
    return this.json('proteger', { sim });
  }

  /** Aceita (ou rejeita) as sugestões da seleção, a do cursor, ou (`tudo`) todas. */
  revisoesResolver(aceitar: boolean, tudo: boolean): Mudanca {
    return this.json('revisoes_resolver', { aceitar, tudo });
  }

  revisoes(): InfoRevisao[] {
    return this.json('revisoes');
  }

  comentarios(): InfoComentario[] {
    return this.json('comentarios');
  }

  /** O comentário cujo balão (na margem direita) está sob o ponto. */
  comentarioNoPonto(pagina: number, x: number, y: number): number | null {
    return this.json('comentario_no_ponto', { pagina, x, y });
  }

  comentarioNovo(texto: string): Mudanca {
    return this.json('comentario_novo', { texto });
  }

  comentarioMudar(id: number, m: { texto?: string; resposta?: string; resolvido?: boolean }): Mudanca {
    return this.json('comentario_mudar', { id, ...m });
  }

  comentarioExcluir(id: number): Mudanca {
    return this.json('comentario_excluir', { id });
  }

  /** Seleciona o trecho comentado; `false` se o comentário não está mais no texto. */
  comentarioIr(id: number): boolean {
    return this.json('comentario_ir', { id });
  }

  /** Marca no documento o que mudou desde o arquivo `bytes` (Revisão → Comparar). */
  comparar(formato: FormatoDeArquivo, bytes: Uint8Array): Mudanca {
    return this.json(`documento_comparar_${formato}`, bytes);
  }

  /** O texto selecionado (para preencher caixas), sem mexer no que foi copiado. */
  selecaoTexto(): string {
    return this.json('selecao_texto');
  }

  /** A palavra antes do número do capítulo ("Capítulo", "Chapter"). */
  rotuloDoCapitulo(rotulo: string): Estado {
    return this.json('capitulo_rotulo_definir', { rotulo });
  }

  cabecalho(esquerda: string, centro: string, direita: string): Mudanca {
    return this.json('cabecalho_definir', { esquerda, centro, direita });
  }

  rodape(esquerda: string, centro: string, direita: string): Mudanca {
    return this.json('rodape_definir', { esquerda, centro, direita });
  }

  primeiraDiferente(sim: boolean): Mudanca {
    return this.json('primeira_diferente', { sim });
  }

  /** `modelo` com `{pagina}` (ou `{pagina_romano}`); `onde` = `topo_esquerda` … `base_direita`. */
  numeroDePagina(modelo: string, onde: string): Mudanca {
    return this.json('numero_de_pagina', { modelo, onde });
  }

  numeroRemover(): Mudanca {
    return this.json('numero_remover');
  }

  corPagina(cor: string): Mudanca {
    return this.json('cor_pagina', { cor });
  }

  marcaDagua(texto: string, cor = ''): Mudanca {
    return this.json('marca_dagua', { texto, cor });
  }

  bordaPagina(cor: string, espessura: number, distancia: number): Mudanca {
    return this.json('borda_pagina', { cor, espessura, distancia });
  }

  /** Mostra ou esconde as marcas de formatação (¶ ·). */
  marcas(sim: boolean): Mudanca {
    return this.json('marcas', { sim });
  }

  desfazer(): Mudanca | null {
    return this.json('desfazer');
  }

  refazer(): Mudanca | null {
    return this.json('refazer');
  }

  // --- tabela (a ponte com o FeatherSheet) ------------------------------------------

  /** A tabela em CSV (o que o FeatherSheet abre). */
  tabelaCsv(id: number): string {
    return utf8.decode(this.canal.chamar('tabela_csv', { id }));
  }

  /** Troca as células da tabela pelo CSV que voltou do FeatherSheet. */
  tabelaCsvSubstituir(id: number, csv: string): Mudanca {
    const corpo = new Uint8Array(8 + csv.length * 3);
    new DataView(corpo.buffer).setBigUint64(0, BigInt(id), true);
    const r = new TextEncoder().encodeInto(csv, corpo.subarray(8));
    return this.json('tabela_csv_substituir', corpo.subarray(0, 8 + r.written));
  }
}
