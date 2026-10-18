/**
 * Todos os comandos da faixa e dos menus, na ordem das abas da especificação
 * (`docs/MENU-ESPECIFICACAO.md`). Quem executa é o aplicativo (os ganchos); aqui só se diz
 * o que cada comando é, quando aparece marcado e o que ainda está "em breve".
 */

import { t } from '../idiomas/index.ts';
import type { Alinhamento, EstiloDeCitacao, EstiloDeLista, ModoDeEdicao, Estado, Formato, FormatoDoCodigo, Numeracao, Papel } from '../ponte/tipos.ts';
import type { Comando, Registro } from './comandos.ts';
import { itemEmBreve } from './comandos.ts';
import type { GanchosDeFormato } from './faixa-formato.ts';
import type { MenuItem } from './menu.ts';
import type { ModoDoTema } from './tema.ts';

/** Margens prontas, em pontos: [esquerda, direita, superior, inferior]. */
export const MARGENS_PRONTAS: Record<'normal' | 'estreita' | 'moderada' | 'larga', [number, number, number, number]> = {
  normal: [72, 72, 72, 72],
  estreita: [36, 36, 36, 36],
  moderada: [54, 54, 72, 72],
  larga: [144, 144, 72, 72],
};

/** Os estilos de citação (Referências → Estilo). */
export const ESTILOS_DE_CITACAO: EstiloDeCitacao[] = ['abnt', 'apa', 'vancouver', 'ieee', 'chicago'];

/** Os campos de Inserir → Campo de dados (o motor resolve o valor e o atualiza sozinho). */
export const CAMPOS = ['autor', 'titulo', 'data', 'pagina', 'paginas', 'arquivo'];

/** Os estilos de parágrafo da galeria e do menu (todos têm comando no motor). */
export const ESTILOS = ['paragrafo', 'titulo1', 'titulo2', 'titulo3', 'titulo4', 'titulo5', 'titulo6', 'citacao', 'codigo', 'equacao'];

export interface GanchosDosComandos extends GanchosDeFormato {
  inicio(): void;
  configurar(): void;
  ajuda(): void;
  novo(): void;
  abrir(): void;
  salvar(): void;
  salvarComo(): void;
  exportar(formato: 'md' | 'tex' | 'docx'): void;
  desfazer(): void;
  refazer(): void;
  recortar(): void;
  copiar(): void;
  colar(semFormatacao: boolean): void;
  selecionarTudo(): void;
  estilo(tipo: string): void;
  /** Aplica o estilo com nome `id`. */
  estiloComNome(id: number): void;
  /** A caixa "Novo estilo", preenchida com o parágrafo do cursor. */
  estiloNovo(): void;
  estiloModificar(id: number): void;
  estiloExcluir(id: number): void;
  numeracao(n: Numeracao): void;
  tituloSemNumero(): void;
  /** Liga o modo dividido (código | páginas) em Markdown ou LaTeX; `null` desliga. */
  modoDividido(f: FormatoDoCodigo | null): void;
  sumarioPersonalizado(): void;
  numerarLinhas(n: 'nenhuma' | 'continua' | 'por_pagina'): void;
  vertical(v: 'topo' | 'centro' | 'base'): void;
  alternarGrade(): void;
  legenda(tipo: 'figura' | 'tabela' | 'equacao'): void;
  listaDeLegendas(k: 1 | 2): void;
  marcador(): void;
  referenciaCruzada(): void;
  referenciasAtualizar(): void;
  /** A caixa de texto de uma nota nova de rodapé (ou de fim). */
  notaInserir(fim: boolean): void;
  formatoDasNotas(): void;
  /** Um campo que se atualiza: `autor`, `titulo`, `data`, `pagina`, `paginas`, `arquivo`. */
  campo(nome: string): void;
  citacaoInserir(): void;
  fontes(): void;
  estiloCitacao(nome: EstiloDeCitacao): void;
  bibliografiaInserir(todas: boolean): void;
  marcarEntrada(): void;
  indiceInserir(): void;
  comentarioNovo(): void;
  /** O painel de revisão: todos os comentários, só os abertos, ou fechado (`null`). */
  comentariosMostrar(filtro: 'todos' | 'abertos' | null): void;
  /** O filtro do painel de agora (`null` = fechado). */
  comentariosFiltro(): 'todos' | 'abertos' | null;
  modoEdicao(modo: ModoDeEdicao): void;
  revisoesResolver(aceitar: boolean, tudo: boolean): void;
  comparar(): void;
  historico(): void;
  proteger(sim: boolean): void;
  imagemUrl(): void;
  modoContinuo(): void;
  zoomVarias(): void;
  alternarMiniaturas(): void;
  inserir(tipo: string): void;
  nivel(delta: number): void;
  alinhar(como: Alinhamento): void;
  papel(papel: Papel): void;
  papelPersonalizado(): void;
  orientacao(paisagem: boolean): void;
  margensPersonalizadas(): void;
  margens(pronta: keyof typeof MARGENS_PRONTAS): void;
  zoom(delta: number): void;
  zoomPara(z: number): void;
  zoomLargura(): void;
  zoomPagina(): void;
  alternarEstrutura(): void;
  alternarRegua(): void;
  alternarMarcas(): void;
  modoFoco(): void;
  tema(modo: ModoDoTema): void;
  inserirTabela(linhas: number, colunas: number): void;
  tabelaDialogo(): void;
  inserirImagem(): void;
  link(): void;
  linkRemover(): void;
  simbolo(texto: string): void;
  /** Abre a caixa de cabeçalho e rodapé (com o foco num dos dois). */
  cabecalhoRodape(foco: 'cabecalho' | 'rodape'): void;
  numeroDePagina(modelo: string, onde: string): void;
  numeroRemover(): void;
  dataHora(formato: 'curta' | 'longa' | 'hora' | 'iso'): void;
  chamada(tipo: string): void;
  corPagina(cor: string): void;
  marcaDagua(texto: string): void;
  marcaDaguaPersonalizada(): void;
  /** Espessura em pontos; 0 tira a borda. */
  bordaPagina(espessura: number): void;
  inserirSumario(niveis: number): void;
  removerSumario(): void;
  contagem(): void;
  tabelaLinha(abaixo: boolean): void;
  tabelaColuna(direita: boolean): void;
  tabelaExcluir(o: 'linha' | 'coluna' | 'tabela'): void;
  tabelaCabecalho(): void;
  /** Só no aplicativo: abre a tabela do cursor no FeatherSheet. */
  tabelaNoFeatherSheet: (() => void) | null;
}

/** O que os comandos leem para saber se estão marcados. */
export interface Contexto {
  g: GanchosDosComandos;
  formato(): Formato | null;
  estado(): Estado | null;
  estrutura(): boolean;
  regua(): boolean;
  foco(): boolean;
  tema(): ModoDoTema;
  /** O modo dividido ligado (e em que língua), ou `null`. */
  codigo(): FormatoDoCodigo | null;
  /** O papel é desenhado sem vão entre as páginas (Exibir → Contínuo). */
  continuo(): boolean;
  /** Várias páginas lado a lado. */
  varias(): boolean;
  miniaturas(): boolean;
}

const ALINHAMENTOS: Alinhamento[] = ['esquerda', 'centro', 'direita', 'justificado'];
const ATALHO_ALINHAR: Record<Alinhamento, string> = { esquerda: 'Ctrl+L', centro: 'Ctrl+E', direita: 'Ctrl+R', justificado: 'Ctrl+J' };

type Par = [string, Comando];

/** Cores prontas do papel (a paleta completa está no botão da faixa). */
const CORES_DE_PAGINA: Array<[string, string]> = [['branca', ''], ['creme', '#fff8e1'], ['cinza', '#f1f3f4'], ['azul', '#e8f0fe'], ['verde', '#e6f4ea'], ['rosa', '#fce8e6']];

/** `0xRRGGBBAA` como `#rrggbb` (vazio = 0), igual ao motor. */
export function corEmTexto(cor: number): string {
  return cor === 0 ? '' : `#${(cor >>> 8).toString(16).padStart(6, '0')}`;
}

/** Um comando que funciona. */
function cmd(id: string, icone: string, o: Partial<Comando> = {}): Par {
  return [id, { rotulo: t(`cmd.${id}`), icone, ...o }];
}

/** Um comando da especificação que ainda não tem motor. */
function breve(id: string, icone: string): Par {
  return [id, { rotulo: t(`cmd.${id}`), icone, emBreve: t(`breve.${id}`) }];
}

function item(rotulo: string, action: () => void, extra: Partial<MenuItem> = {}): MenuItem {
  return { label: rotulo, action, ...extra };
}

const SEP: MenuItem = { separator: true };

function arquivoEEdicao(c: Contexto): Par[] {
  const g = c.g;
  return [
    cmd('inicio', 'inicio', { executar: () => g.inicio() }),
    cmd('novo', 'arquivo', { atalho: 'Ctrl+N', executar: () => g.novo() }),
    cmd('abrir', 'abrir', { atalho: 'Ctrl+O', executar: () => g.abrir() }),
    cmd('salvar', 'salvar', { atalho: 'Ctrl+S', executar: () => g.salvar() }),
    cmd('salvar_como', 'salvar-como', { atalho: 'Ctrl+Shift+S', executar: () => g.salvarComo() }),
    cmd('exportar', 'baixar', { menu: () => [item(t('arquivo.exportar_docx'), () => g.exportar('docx')), item(t('arquivo.exportar_md'), () => g.exportar('md')), item(t('arquivo.exportar_tex'), () => g.exportar('tex'))] }),
    cmd('desfazer', 'desfazer', { atalho: 'Ctrl+Z', executar: () => g.desfazer(), desativado: () => !c.estado()?.pode_desfazer }),
    cmd('refazer', 'refazer', { atalho: 'Ctrl+Y', executar: () => g.refazer(), desativado: () => !c.estado()?.pode_refazer }),
    cmd('recortar', 'recortar', { atalho: 'Ctrl+X', executar: () => g.recortar() }),
    cmd('copiar', 'copiar', { atalho: 'Ctrl+C', executar: () => g.copiar() }),
    cmd('colar', 'colar', { atalho: 'Ctrl+V', executar: () => g.colar(false) }),
    cmd('colar_sem', 'colar-texto', { atalho: 'Ctrl+Shift+V', executar: () => g.colar(true) }),
    cmd('selecionar_tudo', 'selecionar', { atalho: 'Ctrl+A', executar: () => g.selecionarTudo() }),
    breve('selecionar_semelhante', 'selecionar'),
    breve('selecionar_titulos', 'selecionar'),
    cmd('selecionar', 'selecionar', {
      menu: () => [item(t('cmd.selecionar_tudo'), () => g.selecionarTudo(), { shortcut: 'Ctrl+A' }), itemEmBreve(t('cmd.selecionar_semelhante')), itemEmBreve(t('cmd.selecionar_titulos'))],
    }),
    cmd('localizar', 'buscar', { atalho: 'Ctrl+F', executar: () => g.localizar() }),
    cmd('substituir', 'substituir', { atalho: 'Ctrl+H', executar: () => g.substituir() }),
    cmd('configuracoes', 'config', { executar: () => g.configurar() }),
    cmd('ajuda', 'ajuda', { atalho: 'F1', executar: () => g.ajuda() }),
  ];
}

function fonteEParagrafo(c: Contexto): Par[] {
  const g = c.g;
  const f = c.formato;
  const lista = (n: EstiloDeLista, rotulo: string): MenuItem => item(rotulo, () => g.lista(n), { checked: () => (f()?.estilo === 'item' || f()?.estilo === 'item_ordenado') && f()?.lista === n });
  const fator = (n: number): MenuItem => item(String(n).replace('.', ','), () => g.entrelinha(n), { checked: () => Math.abs((f()?.entrelinha ?? 0) - n) < 0.001 });
  return [
    cmd('negrito', 'negrito', { atalho: 'Ctrl+B', executar: () => g.formatar('negrito'), ativo: () => Boolean(f()?.negrito) }),
    cmd('italico', 'italico', { atalho: 'Ctrl+I', executar: () => g.formatar('italico'), ativo: () => Boolean(f()?.italico) }),
    cmd('sublinhado', 'sublinhado', { atalho: 'Ctrl+U', executar: () => g.formatar('sublinhado'), ativo: () => Boolean(f()?.sublinhado) }),
    cmd('tachado', 'tachado', { atalho: 'Ctrl+Shift+X', executar: () => g.formatar('tachado'), ativo: () => Boolean(f()?.tachado) }),
    cmd('codigo_linha', 'codigo', { atalho: 'Ctrl+Shift+E', executar: () => g.formatar('codigo'), ativo: () => Boolean(f()?.codigo) }),
    cmd('sobrescrito', 'sobrescrito', { atalho: 'Ctrl+Shift+=', executar: () => g.posicao(1), ativo: () => Boolean(f()?.sobrescrito) }),
    cmd('subscrito', 'subscrito', { atalho: 'Ctrl+=', executar: () => g.posicao(-1), ativo: () => Boolean(f()?.subscrito) }),
    cmd('fonte_familia', 'fonte', {
      menu: () => (['sans', 'serifa', 'mono'] as const).map((fm) => item(t(`faixa.fonte_${fm}`), () => g.fonte(fm), { checked: () => f()?.fonte === fm })),
    }),
    cmd('aumentar_fonte', 'fonte-mais', { atalho: 'Ctrl+Shift+>', executar: () => g.tamanhoPasso(1) }),
    cmd('diminuir_fonte', 'fonte-menos', { atalho: 'Ctrl+Shift+<', executar: () => g.tamanhoPasso(-1) }),
    cmd('limpar', 'limpar', { atalho: 'Ctrl+Espaço', executar: () => g.limparFormatacao() }),
    cmd('pincel', 'pincel', { atalho: 'Ctrl+Shift+C', executar: () => g.pincel(false), ativo: () => Boolean(f()?.pincel) }),
    cmd('maiusculas', 'maiusculas', { menu: () => (['frase', 'minusculas', 'maiusculas', 'palavras', 'alternar'] as const).map((m) => item(t(`faixa.cx_${m}`), () => g.maiusculas(m))) }),
    ...ALINHAMENTOS.map((a): Par => cmd(`alinhar_${a}`, `alinhar-${a}`, { atalho: ATALHO_ALINHAR[a], executar: () => g.alinhar(a), ativo: () => f()?.alinhamento === a })),
    cmd('recuo_menos', 'recuo-menos', { atalho: 'Shift+Tab', executar: () => g.nivel(-1) }),
    cmd('recuo_mais', 'recuo-mais', { atalho: 'Tab', executar: () => g.nivel(1) }),
    cmd('lista_marcadores', 'lista', { atalho: 'Ctrl+Shift+8', menu: () => (['padrao', 'circulo', 'quadrado', 'seta'] as const).map((n) => lista(n, t(`faixa.lista_${n}`))) }),
    cmd('lista_numerada', 'lista-numerada', {
      atalho: 'Ctrl+Shift+7',
      menu: () => [lista('numerica', t('faixa.lista_num')), lista('alfabetica', t('faixa.lista_alfa')), lista('romana', t('faixa.lista_romana')), SEP, item(t('faixa.lista_reiniciar'), () => g.listaReiniciar())],
    }),
    cmd('lista_tarefas', 'tarefa', { executar: () => g.lista('tarefa'), ativo: () => f()?.lista === 'tarefa' && f()?.estilo === 'item' }),
    cmd('citacao', 'citacao', { executar: () => g.estilo('citacao'), ativo: () => f()?.estilo === 'citacao' }),
    cmd('entrelinha', 'entrelinha', {
      menu: () => {
        const antes = f()?.antes ?? 0;
        const depois = f()?.depois ?? 0;
        return [
          ...[1, 1.15, 1.5, 2].map(fator),
          item(t('faixa.entrelinha_outro'), () => g.entrelinhaPersonalizada()),
          SEP,
          item(antes > 0 ? t('faixa.espaco_antes_menos') : t('faixa.espaco_antes_mais'), () => g.espaco(antes > 0 ? 0 : 12, depois), { icon: 'espaco-antes' }),
          item(depois > 0 ? t('faixa.espaco_depois_menos') : t('faixa.espaco_depois_mais'), () => g.espaco(antes, depois > 0 ? 0 : 12), { icon: 'espaco-depois' }),
        ];
      },
    }),
    breve('direcao_texto', 'direcao-texto'),
    cmd('estilos', 'texto', { menu: () => menuDeEstilos(c) }),
    cmd('estilo_novo', 'estilo-novo', { executar: () => g.estiloNovo() }),
    cmd('estilo_modificar', 'estilo-modificar', { executar: () => g.estiloModificar(f()?.estilo_proprio ?? 0), desativado: () => !f()?.estilo_proprio }),
    cmd('numeracao_titulos', 'numeracao-titulos', {
      menu: () => {
        const n = (nome: Numeracao): MenuItem => item(t(`faixa.num_tit_${nome}`), () => g.numeracao(nome), { checked: () => (c.estado()?.estilos.numeracao ?? 'nenhuma') === nome });
        return [
          n('nenhuma'),
          n('decimal'),
          n('capitulo'),
          SEP,
          item(t('faixa.titulo_sem_numero'), () => g.tituloSemNumero(), { checked: () => Boolean(f()?.sem_numero), disabled: () => !f()?.estilo.startsWith('titulo') }),
        ];
      },
    }),
  ];
}

/** Estilos: os de fábrica, os com nome do documento, e criar/modificar. */
function menuDeEstilos(c: Contexto): MenuItem[] {
  const g = c.g;
  const f = c.formato;
  const fabrica = ESTILOS.map((id) => item(t(`tipos.${id}`), () => g.estilo(id), { checked: () => f()?.estilo === id && !f()?.estilo_proprio }));
  const proprios = (c.estado()?.estilos.lista ?? []).map((e) => item(e.nome, () => g.estiloComNome(e.id), { checked: () => f()?.estilo_proprio === e.id }));
  return [
    ...fabrica,
    ...(proprios.length > 0 ? [SEP, ...proprios] : []),
    SEP,
    item(t('cmd.estilo_novo'), () => g.estiloNovo(), { icon: 'estilo-novo' }),
    item(t('cmd.estilo_modificar'), () => g.estiloModificar(f()?.estilo_proprio ?? 0), { icon: 'estilo-modificar', disabled: () => !f()?.estilo_proprio }),
  ];
}

function numeroDePagina(g: GanchosDosComandos): MenuItem[] {
  const modelos: Array<[string, string, string]> = [
    [t('faixa.num_simples'), '{pagina}', 'esquerda'],
    [t('faixa.num_simples'), '{pagina}', 'centro'],
    [t('faixa.num_simples'), '{pagina}', 'direita'],
    [t('faixa.num_pagina'), t('faixa.modelo_pagina'), 'centro'],
    [t('faixa.num_de'), t('faixa.modelo_de'), 'direita'],
    [t('faixa.num_romano'), '{pagina_romano}', 'centro'],
  ];
  const lugar = (topo: boolean): MenuItem => ({
    label: t(topo ? 'faixa.topo' : 'faixa.base'),
    icon: topo ? 'cabecalho' : 'rodape',
    submenu: modelos.map(([rotulo, modelo, lado]) => item(`${rotulo} · ${t(`faixa.lugar_${lado}`)}`, () => g.numeroDePagina(modelo, `${topo ? 'topo' : 'base'}_${lado}`))),
  });
  return [lugar(true), lugar(false), SEP, item(t('faixa.num_remover'), () => g.numeroRemover())];
}

function inserir(c: Contexto): Par[] {
  const g = c.g;
  const f = c.formato;
  return [
    cmd('quebra_pagina', 'quebra-pagina', { atalho: 'Ctrl+Enter', executar: () => g.inserir('quebra_de_pagina') }),
    breve('quebra_secao', 'quebra-secao'),
    cmd('tabela_dialogo', 'tabela', { executar: () => g.tabelaDialogo() }),
    cmd('imagem', 'imagem', { menu: () => [item(t('faixa.imagem_computador'), () => g.inserirImagem(), { icon: 'imagem' }), item(t('cmd.imagem_url'), () => g.imagemUrl(), { icon: 'mundo' })] }),
    cmd('imagem_url', 'mundo', { executar: () => g.imagemUrl() }),
    breve('formas', 'formas'),
    breve('grafico', 'grafico'),
    breve('desenho', 'lapis'),
    cmd('link', 'link', { atalho: 'Ctrl+K', executar: () => g.link(), ativo: () => Boolean(f()?.link) }),
    cmd('link_remover', 'link-remover', { executar: () => g.linkRemover(), desativado: () => !f()?.link && !f()?.tem_selecao }),
    cmd('marcador', 'marcador', { executar: () => g.marcador() }),
    cmd('equacao', 'equacao', { executar: () => g.inserir('equacao') }),
    cmd('simbolo', 'simbolo', { executar: () => g.simbolo('') }),
    cmd('cabecalho', 'cabecalho', { executar: () => g.cabecalhoRodape('cabecalho') }),
    cmd('rodape', 'rodape', { executar: () => g.cabecalhoRodape('rodape') }),
    cmd('numero_pagina', 'numero-pagina', { menu: () => numeroDePagina(g) }),
    cmd('data_hora', 'data', {
      menu: () => (['curta', 'longa', 'hora', 'iso'] as const).map((fm) => item(t(`faixa.data_${fm === 'hora' ? 'hora_f' : fm}`), () => g.dataHora(fm))),
    }),
    cmd('bloco_codigo', 'codigo-bloco', { executar: () => g.inserir('codigo') }),
    cmd('chamada', 'chamada', {
      menu: () => [
        ...(['info', 'dica', 'aviso', 'atencao'] as const).map((tp) => item(t(`faixa.chamada_${tp}`), () => g.chamada(tp), { checked: () => f()?.chamada === tp })),
        SEP,
        item(t('faixa.chamada_nenhuma'), () => g.chamada('nenhuma')),
      ],
    }),
    breve('caixa_texto', 'caixa-texto'),
    breve('recolhivel', 'recolhivel'),
    cmd('campo_dados', 'campo', { menu: () => CAMPOS.map((nome) => item(t(`campo.${nome}`), () => g.campo(nome))) }),
    cmd('linha_horizontal', 'regua', { executar: () => g.inserir('regua') }),
  ];
}

function layout(c: Contexto): Par[] {
  const g = c.g;
  const e = c.estado;
  const igual = (p: keyof typeof MARGENS_PRONTAS) => (): boolean => {
    const s = e();
    const [esq, dir, sup, inf] = MARGENS_PRONTAS[p];
    return !!s && [s.margem_esq - esq, s.margem_dir - dir, s.margem_sup - sup, s.margem_inf - inf].every((d) => Math.abs(d) < 0.5);
  };
  const margem = (id: keyof typeof MARGENS_PRONTAS, cm: string): MenuItem => item(`${t(`faixa.margem_${id}`)} (${cm})`, () => g.margens(id), { checked: igual(id) });
  const papel = (id: Papel, medida: string): MenuItem => item(`${t(`papeis.${id}`)} (${medida})`, () => g.papel(id), { checked: () => e()?.papel === id });
  const marca = (chave: string): MenuItem => item(t(`faixa.${chave}`), () => g.marcaDagua(t(`faixa.${chave}`)), { checked: () => e()?.extras.marca === t(`faixa.${chave}`) });
  const borda = (chave: string, w: number): MenuItem => item(t(`faixa.${chave}`), () => g.bordaPagina(w), { checked: () => (w === 0 ? !e()?.extras.borda_cor : Boolean(e()?.extras.borda_cor) && Math.abs((e()?.extras.borda_espessura ?? 0) - w) < 0.01) });
  return [
    cmd('margens', 'margens', {
      menu: () => [margem('normal', '2,54 cm'), margem('estreita', '1,27 cm'), margem('moderada', '1,91 cm'), margem('larga', '5,08 cm'), SEP, item(t('faixa.margens_personalizadas'), () => g.margensPersonalizadas())],
    }),
    cmd('orientacao', 'orientacao', {
      menu: () => [
        item(t('faixa.retrato'), () => g.orientacao(false), { icon: 'retrato', checked: () => !e()?.paisagem }),
        item(t('faixa.paisagem'), () => g.orientacao(true), { icon: 'paisagem', checked: () => Boolean(e()?.paisagem) }),
      ],
    }),
    cmd('tamanho_papel', 'papel', {
      menu: () => [
        papel('a4', '21 × 29,7 cm'),
        papel('carta', '21,6 × 27,9 cm'),
        papel('oficio', '21,6 × 35,6 cm'),
        papel('a3', '29,7 × 42 cm'),
        papel('a5', '14,8 × 21 cm'),
        SEP,
        item(t('faixa.papel_personalizado'), () => g.papelPersonalizado(), { checked: () => e()?.papel === 'personalizado' }),
      ],
    }),
    breve('colunas', 'colunas'),
    cmd('quebras', 'quebras', { menu: () => [item(t('cmd.quebra_pagina'), () => g.inserir('quebra_de_pagina'), { icon: 'quebra-pagina', shortcut: 'Ctrl+Enter' }), itemEmBreve(t('faixa.quebra_coluna')), itemEmBreve(t('cmd.quebra_secao'))] }),
    cmd('numeracao_linhas', 'numeracao-linhas', {
      menu: () =>
        (['nenhuma', 'continua', 'por_pagina'] as const).map((n) => item(t(`faixa.linhas_${n}`), () => g.numerarLinhas(n), { checked: () => (e()?.extras.numerar_linhas ?? 'nenhuma') === n })),
    }),
    breve('hifenizacao', 'hifenizacao'),
    cmd('cor_pagina', 'cor-pagina', {
      menu: () =>
        CORES_DE_PAGINA.map(([chave, cor]) =>
          item(chave === 'branca' ? t('faixa.cor_branca') : cor, () => g.corPagina(cor), { checked: () => corEmTexto(e()?.extras.cor_pagina ?? 0) === cor }),
        ),
    }),
    cmd('marca_dagua', 'marca-dagua', {
      menu: () => [marca('marca_rascunho'), marca('marca_confidencial'), marca('marca_urgente'), marca('marca_copia'), SEP, item(t('faixa.marca_personalizada'), () => g.marcaDaguaPersonalizada()), item(t('faixa.marca_remover'), () => g.marcaDagua(''))],
    }),
    cmd('bordas_pagina', 'borda-pagina', { menu: () => [borda('borda_fina', 0.75), borda('borda_media', 1.5), borda('borda_grossa', 3), SEP, borda('borda_remover', 0)] }),
    cmd('alinhar_vertical', 'alinhar-vertical', {
      menu: () => (['topo', 'centro', 'base'] as const).map((v) => item(t(`faixa.vertical_${v}`), () => g.vertical(v), { checked: () => (e()?.extras.vertical ?? 'topo') === v })),
    }),
  ];
}

function referenciasERevisao(c: Contexto): Par[] {
  const g = c.g;
  const f = c.formato;
  return [
    cmd('sumario', 'sumario', {
      menu: () => [...[2, 3, 4].map((n) => item(t('faixa.sumario_n', { n }), () => g.inserirSumario(n))), SEP, item(t('faixa.sumario_remover'), () => g.removerSumario())],
    }),
    cmd('adicionar_texto', 'adicionar-texto', {
      menu: () => [
        item(t('faixa.nao_mostrar'), () => g.estilo('paragrafo'), { checked: () => f()?.estilo === 'paragrafo' }),
        ...[1, 2, 3].map((n) => item(t('faixa.nivel_n', { n }), () => g.estilo(`titulo${n}`), { checked: () => f()?.estilo === `titulo${n}` })),
      ],
    }),
    cmd('sumario_personalizado', 'sumario-personalizado', { executar: () => g.sumarioPersonalizado() }),
    cmd('nota_rodape', 'nota-rodape', { atalho: 'Ctrl+Alt+F', executar: () => g.notaInserir(false) }),
    cmd('nota_fim', 'nota-fim', { atalho: 'Ctrl+Alt+D', executar: () => g.notaInserir(true) }),
    cmd('formato_notas', 'formato-notas', { executar: () => g.formatoDasNotas() }),
    cmd('legenda', 'legenda', { menu: () => (['figura', 'tabela', 'equacao'] as const).map((tp) => item(t(`ref.tipo_${tp}`), () => g.legenda(tp))) }),
    cmd('lista_figuras', 'lista-figuras', { menu: () => [item(t('faixa.lista_de_figuras'), () => g.listaDeLegendas(1)), item(t('faixa.lista_de_tabelas'), () => g.listaDeLegendas(2))] }),
    cmd('referencia_cruzada', 'ref-cruzada', { menu: () => [item(t('cmd.referencia_cruzada') + '…', () => g.referenciaCruzada(), { icon: 'ref-cruzada' }), item(t('faixa.referencias_atualizar'), () => g.referenciasAtualizar(), { shortcut: 'F9' })] }),
    cmd('citacao_inserir', 'citacao-inserir', { executar: () => g.citacaoInserir() }),
    cmd('fontes', 'fontes', { executar: () => g.fontes() }),
    cmd('estilo_citacao', 'estilo-citacao', {
      menu: () => ESTILOS_DE_CITACAO.map((nome) => item(t(`citacao.estilo_${nome}`), () => g.estiloCitacao(nome), { checked: () => (c.estado()?.estilo_citacao ?? 'abnt') === nome })),
    }),
    cmd('bibliografia', 'bibliografia', { menu: () => [item(t('citacao.bib_citadas'), () => g.bibliografiaInserir(false)), item(t('citacao.bib_todas'), () => g.bibliografiaInserir(true))] }),
    cmd('marcar_entrada', 'marcar-entrada', { atalho: 'Alt+Shift+X', executar: () => g.marcarEntrada() }),
    cmd('indice', 'indice', { executar: () => g.indiceInserir() }),
    breve('ortografia', 'ortografia'),
    breve('idioma', 'idioma'),
    cmd('contagem', 'contagem', { atalho: 'Ctrl+Shift+G', executar: () => g.contagem() }),
    cmd('comentario_novo', 'comentario', { atalho: 'Ctrl+Alt+M', executar: () => g.comentarioNovo() }),
    cmd('comentarios_mostrar', 'comentarios', {
      menu: () => [
        item(t('revisao.mostrar_todos'), () => g.comentariosMostrar('todos'), { checked: () => g.comentariosFiltro() === 'todos' }),
        item(t('revisao.mostrar_abertos'), () => g.comentariosMostrar('abertos'), { checked: () => g.comentariosFiltro() === 'abertos' }),
        SEP,
        item(t('revisao.esconder'), () => g.comentariosMostrar(null), { checked: () => g.comentariosFiltro() === null }),
      ],
    }),
    cmd('modo_edicao', 'modo-edicao', {
      menu: () => (['editando', 'sugerindo', 'leitura'] as const).map((m) => item(t(`revisao.modo_${m}`), () => g.modoEdicao(m), { checked: () => (c.estado()?.modo_edicao ?? 'editando') === m })),
    }),
    cmd('aceitar', 'check', { menu: () => [item(t('revisao.aceitar_esta'), () => g.revisoesResolver(true, false)), item(t('revisao.aceitar_todas'), () => g.revisoesResolver(true, true))], desativado: () => !c.estado()?.tem_revisoes }),
    cmd('rejeitar', 'cancelar', { menu: () => [item(t('revisao.rejeitar_esta'), () => g.revisoesResolver(false, false)), item(t('revisao.rejeitar_todas'), () => g.revisoesResolver(false, true))], desativado: () => !c.estado()?.tem_revisoes }),
    cmd('comparar', 'comparar', { executar: () => g.comparar() }),
    cmd('historico', 'relogio', { executar: () => g.historico() }),
    cmd('proteger', 'proteger', { executar: () => g.proteger(!c.estado()?.extras.protegido), ativo: () => Boolean(c.estado()?.extras.protegido) }),
  ];
}

function exibirETabela(c: Contexto): Par[] {
  const g = c.g;
  const f = c.formato;
  const semTabela = (): boolean => f()?.tabela == null;
  const tema = (m: ModoDoTema, chave: string): MenuItem => item(t(`faixa.${chave}`), () => g.tema(m), { checked: () => c.tema() === m });
  return [
    cmd('modo_paginas', 'modo-paginas', {
      executar: () => {
        if (c.codigo() !== null) g.modoDividido(null);
        if (c.foco()) g.modoFoco();
        if (c.continuo()) g.modoContinuo();
      },
      ativo: () => !c.foco() && c.codigo() === null && !c.continuo(),
    }),
    cmd('codigo_latex', 'codigo', {
      executar: () => {
        if (c.codigo() === 'tex') g.modoDividido(null);
        else g.modoDividido('tex');
      },
      ativo: () => c.codigo() === 'tex',
    }),
    cmd('codigo_markdown', 'codigo-bloco', {
      executar: () => {
        if (c.codigo() === 'md') g.modoDividido(null);
        else g.modoDividido('md');
      },
      ativo: () => c.codigo() === 'md',
    }),
    cmd('modo_continuo', 'modo-continuo', { executar: () => g.modoContinuo(), ativo: () => c.continuo() }),
    cmd('modo_foco', 'foco', { executar: () => g.modoFoco(), ativo: () => c.foco() }),
    cmd('modo_dividido', 'modo-dividido', {
      ativo: () => c.codigo() !== null,
      menu: () => [
        item(t('cmd.modo_paginas'), () => g.modoDividido(null), { checked: () => c.codigo() === null }),
        item(t('cmd.codigo_latex'), () => g.modoDividido('tex'), { checked: () => c.codigo() === 'tex' }),
        item(t('cmd.codigo_markdown'), () => g.modoDividido('md'), { checked: () => c.codigo() === 'md' }),
        SEP,
        item(t('codigo.fechar'), () => g.modoDividido(null), { disabled: () => c.codigo() === null }),
      ],
    }),
    cmd('regua', 'regua', { executar: () => g.alternarRegua(), ativo: () => c.regua() }),
    cmd('estrutura', 'estrutura', { executar: () => g.alternarEstrutura(), ativo: () => c.estrutura() }),
    cmd('marcas', 'marcas', { atalho: 'Ctrl+*', executar: () => g.alternarMarcas(), ativo: () => Boolean(c.estado()?.marcas) }),
    cmd('grade', 'grade', { executar: () => g.alternarGrade(), ativo: () => Boolean(c.estado()?.grade) }),
    cmd('miniaturas', 'miniaturas', { executar: () => g.alternarMiniaturas(), ativo: () => c.miniaturas() }),
    cmd('zoom_100', 'zoom', { executar: () => g.zoomPara(1) }),
    cmd('zoom_largura', 'zoom-largura', { executar: () => g.zoomLargura() }),
    cmd('zoom_pagina', 'zoom-pagina', { executar: () => g.zoomPagina() }),
    cmd('zoom_varias', 'zoom-varias', { executar: () => g.zoomVarias(), ativo: () => c.varias() }),
    cmd('zoom_mais', 'zoom-mais', { atalho: 'Ctrl++', executar: () => g.zoom(1) }),
    cmd('zoom_menos', 'zoom-menos', { atalho: 'Ctrl+-', executar: () => g.zoom(-1) }),
    cmd('tema', 'tema', { menu: () => [tema('auto', 'tema_auto'), tema('light', 'tema_claro'), tema('dark', 'tema_escuro')] }),
    cmd('tab_linha_acima', 'linha-acima', { executar: () => g.tabelaLinha(false), desativado: semTabela }),
    cmd('tab_linha_abaixo', 'linha-abaixo', { executar: () => g.tabelaLinha(true), desativado: semTabela }),
    cmd('tab_coluna_esq', 'coluna-esq', { executar: () => g.tabelaColuna(false), desativado: semTabela }),
    cmd('tab_coluna_dir', 'coluna-dir', { executar: () => g.tabelaColuna(true), desativado: semTabela }),
    cmd('tab_excluir', 'excluir-tabela', {
      menu: () => [
        item(t('faixa.excluir_linha'), () => g.tabelaExcluir('linha'), { disabled: semTabela }),
        item(t('faixa.excluir_coluna'), () => g.tabelaExcluir('coluna'), { disabled: semTabela }),
        item(t('faixa.excluir_tabela'), () => g.tabelaExcluir('tabela'), { disabled: semTabela }),
      ],
    }),
    cmd('tab_cabecalho', 'cabecalho-tabela', { executar: () => g.tabelaCabecalho(), ativo: () => Boolean(f()?.tabela_cabecalho), desativado: semTabela }),
    breve('tab_mesclar', 'mesclar'),
    breve('tab_dividir', 'dividir'),
    cmd('tab_feathersheet', 'planilha', { executar: () => g.tabelaNoFeatherSheet?.(), desativado: () => semTabela() || g.tabelaNoFeatherSheet === null }),
  ];
}

/** Monta o registro (de novo a cada troca de idioma: os rótulos já vêm traduzidos). */
export function criarRegistro(c: Contexto): Registro {
  return new Map<string, Comando>([...arquivoEEdicao(c), ...fonteEParagrafo(c), ...inserir(c), ...layout(c), ...referenciasERevisao(c), ...exibirETabela(c)]);
}
