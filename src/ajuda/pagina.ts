/**
 * A janela da Ajuda (F1): como usar o FeatherDocs e os atalhos. Janela própria no aplicativo; no navegador, uma
 * aba. Mesmo jeito do FeatherSheet: cada seção abre numa introdução e se divide em subitens (menu lateral em
 * árvore); as setas "anterior / próximo" percorrem tudo em ordem; a busca procura nas duas seções de uma vez.
 *
 * O idioma e o tema são os do app (o mesmo `localStorage`, mesma origem), e a janela principal pede "abra em tal
 * lugar" por `ajuda/pedido.ts`.
 */

import { codigoAtual, t, usarIdioma } from '../idiomas/index.ts';
import { iniciarTema } from '../ui/tema.ts';
import { TODO_O_CSS } from '../ui/styles.ts';
import { AJUDA_CSS } from './css.ts';
import { CANAL_DA_AJUDA, ultimoPedido, type PedidoDeAjuda, type SecaoDaAjuda } from './pedido.ts';
import { normalizar, secaoAtalhos, secaoUso, subsecoes, type SecaoMontada } from './secoes.ts';

const SECOES: SecaoDaAjuda[] = ['uso', 'atalhos'];
/** Pedido mais velho que isto, achado ao abrir, é de outra vez: começa do início. */
const PEDIDO_RECENTE_MS = 10_000;

/** Onde se está: a seção e o subitem (`null` = a introdução da seção). */
interface Lugar {
  secao: SecaoDaAjuda;
  sub: string | null;
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, classe: string, texto?: string): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  if (classe) e.className = classe;
  if (texto !== undefined) e.textContent = texto;
  return e;
}

/** Monta a Ajuda em `raiz`. Devolve quem fecha o canal e quem abre num lugar. */
export function montarAjuda(raiz: HTMLElement): { fechar: () => void; abrir: (p: PedidoDeAjuda) => void } {
  usarIdioma(codigoAtual());
  const estilo = document.createElement('style');
  estilo.textContent = `${TODO_O_CSS}\n${AJUDA_CSS}`;
  document.head.append(estilo);
  iniciarTema();
  document.title = t('ajuda.rotulos.titulo');

  const estado: Lugar = { secao: 'uso', sub: null };

  raiz.className = 'aj';
  const topo = el('header', 'aj-topo');
  const busca = el('input', 'aj-busca');
  busca.type = 'search';
  busca.placeholder = t('ajuda.rotulos.busca');
  topo.append(el('h1', '', t('ajuda.rotulos.titulo')), busca);
  const nav = el('nav', 'aj-nav');
  const corpo = el('main', 'aj-corpo');
  const meio = el('div', 'aj-meio');
  meio.append(nav, corpo);
  raiz.append(topo, meio);

  const tituloDe = (l: Lugar): string => (l.sub === null ? t(`ajuda.rotulos.${l.secao}`) : (subsecoes(l.secao).find((x) => x.id === l.sub)?.titulo ?? l.sub));

  /** Todas as páginas em ordem: a introdução de cada seção e os subitens dela. */
  const ordem = (): Lugar[] => SECOES.flatMap((s) => [{ secao: s, sub: null }, ...subsecoes(s).map((x) => ({ secao: s, sub: x.id }))]);

  const montar = (secao: SecaoDaAjuda, termo: string, so?: string): SecaoMontada => (secao === 'uso' ? secaoUso(termo, so) : secaoAtalhos(termo, so));

  const ir = (l: Lugar): void => {
    busca.value = '';
    estado.secao = l.secao;
    estado.sub = l.sub;
    desenhar();
    corpo.scrollTop = 0;
  };

  const botaoDeIr = (classe: string, texto: string, l: Lugar, atual: boolean): HTMLButtonElement => {
    const b = el('button', classe, texto);
    if (atual) b.setAttribute('aria-current', 'true');
    b.addEventListener('click', () => ir(l));
    return b;
  };

  /** O menu lateral: as seções e, embaixo da aberta, os subitens dela. */
  const desenharNav = (buscando: boolean): void => {
    const itens: HTMLElement[] = [];
    for (const s of SECOES) {
      const aberta = s === estado.secao && !buscando;
      const b = botaoDeIr('aj-secao', t(`ajuda.rotulos.${s}`), { secao: s, sub: null }, aberta && estado.sub === null);
      b.setAttribute('aria-expanded', String(aberta));
      itens.push(b);
      if (!aberta) continue;
      for (const x of subsecoes(s)) itens.push(botaoDeIr('aj-sub', x.titulo, { secao: s, sub: x.id }, estado.sub === x.id));
    }
    nav.replaceChildren(...itens);
  };

  /** "‹ Anterior: X" e "Próximo: Y ›", atravessando de uma seção para a outra. */
  const passos = (): HTMLElement => {
    const todas = ordem();
    const i = todas.findIndex((l) => l.secao === estado.secao && l.sub === estado.sub);
    const barra = el('nav', 'aj-passos');
    const antes = todas[i - 1];
    const depois = todas[i + 1];
    barra.append(antes ? botaoDeIr('aj-anterior', `‹ ${t('ajuda.rotulos.anterior')}: ${tituloDe(antes)}`, antes, false) : el('span', ''));
    if (depois) barra.append(botaoDeIr('aj-proximo', `${t('ajuda.rotulos.proximo')}: ${tituloDe(depois)} ›`, depois, false));
    return barra;
  };

  /** A introdução da seção: o texto, o índice dos subitens e a seta. */
  const paginaDeEntrada = (): HTMLElement[] => {
    const indice = el('div', 'aj-indice');
    for (const x of subsecoes(estado.secao)) indice.append(botaoDeIr('aj-cartao', x.titulo, { secao: estado.secao, sub: x.id }, false));
    return [el('h2', '', t(`ajuda.rotulos.${estado.secao}`)), el('p', 'aj-intro', t(`ajuda.introducoes.${estado.secao}`)), el('h3', '', t('ajuda.rotulos.nestaSecao')), indice, passos()];
  };

  const paginaDoSub = (sub: string): HTMLElement[] => {
    const migalha = el('button', 'aj-migalha', t(`ajuda.rotulos.${estado.secao}`));
    migalha.addEventListener('click', () => ir({ secao: estado.secao, sub: null }));
    return [migalha, el('h2', '', tituloDe(estado)), ...montar(estado.secao, '', sub).elementos, passos()];
  };

  /** Buscando: as duas seções, uma depois da outra, só com o que achou. */
  const paginaDaBusca = (termo: string): HTMLElement[] => {
    const partes: HTMLElement[] = [];
    for (const s of SECOES) {
      const m = montar(s, termo);
      if (m.achados > 0) partes.push(el('h2', '', `${t(`ajuda.rotulos.${s}`)} (${m.achados})`), ...m.elementos);
    }
    return partes.length ? partes : [el('p', 'aj-vazio', t('ajuda.rotulos.semResultado'))];
  };

  function desenhar(): void {
    const termo = normalizar(busca.value.trim());
    desenharNav(termo !== '');
    if (termo) corpo.replaceChildren(...paginaDaBusca(termo));
    else if (estado.sub === null) corpo.replaceChildren(...paginaDeEntrada());
    else corpo.replaceChildren(...paginaDoSub(estado.sub));
  }

  const abrir = (p: PedidoDeAjuda): void => {
    const secao = SECOES.includes(p.secao) ? p.secao : 'uso';
    const sub = p.topico && subsecoes(secao).some((x) => x.id === p.topico) ? p.topico : null;
    ir({ secao, sub });
  };

  busca.addEventListener('input', desenhar);
  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
      e.preventDefault();
      busca.focus();
      busca.select();
    }
  });

  const inicial = ultimoPedido();
  if (inicial && Date.now() - inicial.quando < PEDIDO_RECENTE_MS) abrir(inicial);
  else desenhar();

  let canal: BroadcastChannel | null = null;
  if (typeof BroadcastChannel !== 'undefined') {
    canal = new BroadcastChannel(CANAL_DA_AJUDA);
    canal.onmessage = (e: MessageEvent<PedidoDeAjuda>) => abrir(e.data);
  }
  busca.focus();
  return { fechar: () => canal?.close(), abrir };
}
