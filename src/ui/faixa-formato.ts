/**
 * Os controles de formatação da faixa Página inicial: fonte, tamanho, cor, realce, maiúsculas,
 * limpar, pincel, listas, entrelinha, sombreamento, bordas e Localizar/Substituir. Cada controle
 * chama um comando do motor; o que aparece marcado vem do `Formato` que o motor devolve.
 */

import { t } from '../idiomas/index.ts';
import type { EstiloDeLista, Formato } from '../ponte/tipos.ts';
import { FAMILIA_CODIGO, FAMILIA_SERIFA, FAMILIA_TEXTO } from './familias.ts';
import { openPopup, type MenuItem } from './menu.ts';
import { abrirPaleta } from './paleta.ts';
import type { Kit } from './faixa/kit.ts';

export const FORMATO_CSS = `
.faixa-select, .faixa-tamanho {
  height: 26px; border: 1px solid var(--chrome-rule); border-radius: 5px; background: var(--surface); color: var(--chrome-ink); font-size: 12.5px; padding: 0 6px;
}
.faixa-select { width: 150px; }
.faixa-tamanho { width: 52px; text-align: center; }
.faixa-dividido { display: inline-flex; align-items: stretch; }
.faixa-dividido .faixa-pequeno:first-child { border-top-right-radius: 0; border-bottom-right-radius: 0; position: relative; }
.faixa-dividido .faixa-pequeno:last-child { padding: 0 3px; border-top-left-radius: 0; border-bottom-left-radius: 0; }
.faixa-cor-barra { position: absolute; left: 5px; right: 5px; bottom: 2px; height: 3px; border-radius: 1px; }
`;

/** O que os controles pedem ao resto do aplicativo. */
export interface GanchosDeFormato {
  formatar(estilo: 'negrito' | 'italico' | 'sublinhado' | 'tachado' | 'codigo'): void;
  fonte(f: 'sans' | 'serifa' | 'mono'): void;
  tamanho(pt: number): void;
  tamanhoPasso(delta: number): void;
  cor(c: string): void;
  realce(c: string): void;
  posicao(p: 1 | -1): void;
  maiusculas(modo: string): void;
  limparFormatacao(): void;
  pincel(persistente: boolean): void;
  entrelinha(fator: number): void;
  espaco(antes: number | null, depois: number | null): void;
  fundo(c: string): void;
  borda(lados: number): void;
  lista(nome: EstiloDeLista): void;
  listaReiniciar(): void;
  /** Recuos esquerdo e direito do parágrafo, em pontos (a primeira linha continua como está). */
  recuos(esq: number, dir: number): void;
  /** Abre a pergunta da entrelinha personalizada. */
  entrelinhaPersonalizada(): void;
  localizar(): void;
  substituir(): void;
}

type Aplicar = () => Formato | null;

function opcao(valor: string, rotulo: string, familia: string): HTMLOptionElement {
  const o = document.createElement('option');
  o.value = valor;
  o.textContent = rotulo;
  o.style.fontFamily = familia;
  return o;
}

/** A lista de fontes: cada nome escrito na própria fonte. */
function seletorDeFonte(kit: Kit, g: GanchosDeFormato, f: Aplicar): HTMLSelectElement {
  const s = document.createElement('select');
  s.className = 'faixa-select';
  s.title = t('faixa.fonte_familia');
  s.setAttribute('aria-label', t('faixa.fonte_familia'));
  s.append(opcao('sans', t('faixa.fonte_sans'), FAMILIA_TEXTO), opcao('serifa', t('faixa.fonte_serifa'), FAMILIA_SERIFA), opcao('mono', t('faixa.fonte_mono'), FAMILIA_CODIGO));
  s.addEventListener('change', () => g.fonte(s.value as 'sans' | 'serifa' | 'mono'));
  kit.aoSincronizar(() => {
    s.value = f()?.fonte ?? 'sans';
  });
  return s;
}

const TAMANHOS = [8, 9, 10, 10.5, 11, 12, 14, 16, 18, 20, 24, 28, 32, 36, 48, 60, 72];

/** O tamanho: campo editável com a lista de 8 a 72 pt. */
function campoDeTamanho(kit: Kit, g: GanchosDeFormato, f: Aplicar): HTMLElement {
  const c = document.createElement('input');
  c.className = 'faixa-tamanho';
  c.type = 'text';
  c.inputMode = 'decimal';
  c.title = t('faixa.tamanho');
  c.setAttribute('aria-label', t('faixa.tamanho'));
  const lista = document.createElement('datalist');
  lista.id = 'featherdocs-tamanhos';
  lista.append(...TAMANHOS.map((n) => Object.assign(document.createElement('option'), { value: String(n) })));
  c.setAttribute('list', lista.id);
  const aplicar = (): void => {
    const n = Number.parseFloat(c.value.replace(',', '.'));
    if (Number.isFinite(n) && n > 0) g.tamanho(Math.min(400, n));
    else sincronizar();
  };
  c.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      aplicar();
    }
  });
  c.addEventListener('change', aplicar);
  const sincronizar = (): void => {
    const n = f()?.tamanho;
    if (n !== undefined && document.activeElement !== c) c.value = Number.isInteger(n) ? String(n) : n.toFixed(1).replace('.', ',');
  };
  kit.aoSincronizar(sincronizar);
  const dono = document.createElement('span');
  dono.append(c, lista);
  return dono;
}

/** Botão dividido: o principal aplica a última cor; a seta abre a paleta. */
function botaoDeCor(kit: Kit, rotulo: string, nome: string, inicial: string, semCor: string, aplicar: (cor: string) => void): HTMLElement {
  let ultima = inicial;
  const barra = document.createElement('span');
  barra.className = 'faixa-cor-barra';
  barra.style.background = ultima;
  const principal = kit.pequeno(rotulo, nome, () => aplicar(ultima), { soIcone: true });
  principal.append(barra);
  const seta = kit.pequeno(rotulo, 'seta-baixo', () => {
    abrirPaleta(principal, {
      semCor,
      aoEscolher: (cor) => {
        if (cor) {
          ultima = cor;
          barra.style.background = cor;
        }
        aplicar(cor);
      },
    });
  }, { soIcone: true });
  const caixa = document.createElement('span');
  caixa.className = 'faixa-dividido';
  caixa.append(principal, seta);
  return caixa;
}

function menuDe(ancora: HTMLElement, itens: MenuItem[]): void {
  openPopup(ancora, itens);
}

/** O botão que abre um menu suspenso. */
function botaoDeMenu(kit: Kit, rotulo: string, nome: string, itens: () => MenuItem[]): HTMLButtonElement {
  const b = kit.pequeno(rotulo, nome, () => menuDe(b, itens()), { soIcone: true });
  return b;
}

/** O grupo Fonte: família, tamanho, aumentar/diminuir, maiúsculas, limpar; negrito…código, sobrescrito, cor e realce. */
export function itensDaFonte(kit: Kit, g: GanchosDeFormato, f: Aplicar): HTMLElement[] {
  const marca = (rotulo: string, nome: string, estilo: Parameters<GanchosDeFormato['formatar']>[0]): HTMLButtonElement =>
    kit.pequeno(rotulo, nome, () => g.formatar(estilo), { soIcone: true, classe: 'letra', ativo: () => Boolean(f()?.[estilo]) });
  const maiusculas = (rotulo: string, modo: string): MenuItem => ({ label: rotulo, action: () => g.maiusculas(modo) });
  const caixa = botaoDeMenu(kit, t('faixa.maiusculas'), 'maiusculas', () => [
    maiusculas(t('faixa.cx_frase'), 'frase'),
    maiusculas(t('faixa.cx_minusculas'), 'minusculas'),
    maiusculas(t('faixa.cx_maiusculas'), 'maiusculas'),
    maiusculas(t('faixa.cx_palavras'), 'palavras'),
    maiusculas(t('faixa.cx_alternar'), 'alternar'),
  ]);
  return [
    kit.pilha(
      [
        seletorDeFonte(kit, g, f),
        campoDeTamanho(kit, g, f),
        kit.pequeno(t('faixa.aumentar_fonte'), 'fonte-mais', () => g.tamanhoPasso(1), { soIcone: true }),
        kit.pequeno(t('faixa.diminuir_fonte'), 'fonte-menos', () => g.tamanhoPasso(-1), { soIcone: true }),
        caixa,
        kit.pequeno(t('faixa.limpar'), 'limpar', () => g.limparFormatacao(), { soIcone: true }),
      ],
      [
        marca(t('faixa.negrito'), 'negrito', 'negrito'),
        marca(t('faixa.italico'), 'italico', 'italico'),
        marca(t('faixa.sublinhado'), 'sublinhado', 'sublinhado'),
        marca(t('faixa.tachado'), 'tachado', 'tachado'),
        marca(t('faixa.codigo'), 'codigo', 'codigo'),
        kit.pequeno(t('faixa.sobrescrito'), 'sobrescrito', () => g.posicao(1), { soIcone: true, ativo: () => Boolean(f()?.sobrescrito) }),
        kit.pequeno(t('faixa.subscrito'), 'subscrito', () => g.posicao(-1), { soIcone: true, ativo: () => Boolean(f()?.subscrito) }),
        botaoDeCor(kit, t('faixa.cor_fonte'), 'cor-texto', '#d93025', t('faixa.cor_auto'), (c) => g.cor(c)),
        botaoDeCor(kit, t('faixa.realce'), 'realce', '#ffe600', t('faixa.sem_realce'), (c) => g.realce(c)),
      ],
    ),
  ];
}

/** O pincel de formatação: um clique aplica uma vez, duplo clique fica ativo até Esc. */
export function botaoDoPincel(kit: Kit, g: GanchosDeFormato, f: Aplicar): HTMLButtonElement {
  const b = kit.pequeno(t('faixa.pincel'), 'pincel', () => g.pincel(false), { ativo: () => Boolean(f()?.pincel) });
  b.title = `${t('faixa.pincel')} — ${t('faixa.pincel_dica')}`;
  b.addEventListener('dblclick', () => g.pincel(true));
  return b;
}

function itemDeLista(g: GanchosDeFormato, f: Aplicar, nome: EstiloDeLista, rotulo: string): MenuItem {
  return { label: rotulo, action: () => g.lista(nome), checked: () => (f()?.estilo === 'item' || f()?.estilo === 'item_ordenado') && f()?.lista === nome };
}

/** Botão dividido: o principal faz a ação padrão, a seta abre a galeria. */
function botaoDividido(kit: Kit, rotulo: string, nome: string, principal: () => void, ativo: () => boolean, itens: () => MenuItem[]): HTMLElement {
  const p = kit.pequeno(rotulo, nome, principal, { soIcone: true, ativo });
  const seta = kit.pequeno(rotulo, 'seta-baixo', () => menuDe(p, itens()), { soIcone: true });
  const caixa = document.createElement('span');
  caixa.className = 'faixa-dividido';
  caixa.append(p, seta);
  return caixa;
}

/** Os botões de lista (principal + galeria) e de tarefas. */
export function itensDeLista(kit: Kit, g: GanchosDeFormato, f: Aplicar): HTMLElement[] {
  const marcadores = botaoDividido(kit, t('faixa.lista_marcadores'), 'lista', () => g.lista('padrao'), () => f()?.estilo === 'item', () =>
    (['padrao', 'circulo', 'quadrado', 'seta'] as const).map((n) => itemDeLista(g, f, n, t(`faixa.lista_${n}`))),
  );
  const numerada = botaoDividido(kit, t('faixa.lista_numerada_gal'), 'lista-numerada', () => g.lista('numerica'), () => f()?.estilo === 'item_ordenado', () => [
    itemDeLista(g, f, 'numerica', t('faixa.lista_num')),
    itemDeLista(g, f, 'alfabetica', t('faixa.lista_alfa')),
    itemDeLista(g, f, 'romana', t('faixa.lista_romana')),
    { separator: true },
    { label: t('faixa.lista_reiniciar'), action: () => g.listaReiniciar() },
  ]);
  const tarefas = kit.pequeno(t('faixa.lista_tarefas'), 'tarefa', () => g.lista('tarefa'), { soIcone: true, ativo: () => f()?.lista === 'tarefa' });
  return [marcadores, numerada, tarefas];
}

/** Sombreamento (cor de fundo do parágrafo) e bordas. */
export function itensDeFundoEBorda(kit: Kit, g: GanchosDeFormato, f: Aplicar): HTMLElement[] {
  const lado = (bit: number, rotulo: string): MenuItem => ({ label: rotulo, action: () => g.borda((f()?.borda ?? 0) ^ bit), checked: () => ((f()?.borda ?? 0) & bit) !== 0 });
  const bordas = botaoDeMenu(kit, t('faixa.bordas'), 'bordas', () => [
    lado(1, t('faixa.borda_esq')),
    lado(2, t('faixa.borda_dir')),
    lado(4, t('faixa.borda_cima')),
    lado(8, t('faixa.borda_baixo')),
    { separator: true },
    { label: t('faixa.borda_todas'), action: () => g.borda(15) },
    { label: t('faixa.borda_nenhuma'), action: () => g.borda(0) },
  ]);
  return [botaoDeCor(kit, t('faixa.sombreamento'), 'sombreamento', '#fff2cc', t('faixa.sem_realce'), (c) => g.fundo(c)), bordas];
}


