/**
 * As duas seções da Ajuda ("Como usar" e "Atalhos"), cada uma filtrada pela busca. Devolvem os elementos já
 * montados; a página (`pagina.ts`) só decide qual mostrar. Os textos vêm dos arquivos de língua.
 */

import { t } from '../idiomas/index.ts';
import { GRUPOS_DE_ATALHOS, TOPICOS } from './conteudo.ts';
import type { SecaoDaAjuda } from './pedido.ts';

/** O que uma seção montou, e quantos itens passaram pela busca. */
export interface SecaoMontada {
  elementos: HTMLElement[];
  achados: number;
}

/** Minúsculo e sem acento: "média" acha "MEDIA" e vice-versa. */
export function normalizar(texto: string): string {
  return texto.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();
}

function bate(termo: string, ...textos: string[]): boolean {
  return !termo || textos.some((x) => normalizar(x).includes(termo));
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, classe: string, texto?: string): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  if (classe) e.className = classe;
  if (texto !== undefined) e.textContent = texto;
  return e;
}

/** As linhas do corpo de um tópico (`ajuda.comoUsar.<id>.corpo.0`, `.1`…). */
function corpoDe(id: string): string[] {
  const linhas: string[] = [];
  for (let i = 0; ; i++) {
    const chave = `ajuda.comoUsar.${id}.corpo.${i}`;
    const texto = t(chave);
    if (texto === chave) return linhas;
    linhas.push(texto);
  }
}

/** "Como usar": os tópicos (ou só o `so`), com lista para as linhas que começam por "• ". */
export function secaoUso(termo: string, so?: string): SecaoMontada {
  const elementos: HTMLElement[] = [];
  for (const id of TOPICOS) {
    if (so && id !== so) continue;
    const titulo = t(`ajuda.comoUsar.${id}.titulo`);
    const corpo = corpoDe(id);
    if (!bate(termo, titulo, ...corpo)) continue;
    const bloco = el('section', 'aj-topico');
    bloco.id = `uso-${id}`;
    // Numa página de um tópico só, o título já está no topo da página.
    if (!so) bloco.append(el('h4', '', titulo));
    let lista: HTMLUListElement | null = null;
    for (const linha of corpo) {
      if (linha.startsWith('• ')) {
        lista ??= bloco.appendChild(el('ul', ''));
        lista.append(el('li', '', linha.slice(2)));
      } else {
        lista = null;
        bloco.append(el('p', '', linha));
      }
    }
    elementos.push(bloco);
  }
  return { elementos, achados: elementos.length };
}

/** "Atalhos": os grupos (ou só o `so`), com a tecla de cada um. */
export function secaoAtalhos(termo: string, so?: string): SecaoMontada {
  const elementos: HTMLElement[] = [];
  let achados = 0;
  for (const grupo of GRUPOS_DE_ATALHOS) {
    if (so && grupo.id !== so) continue;
    const linhas = grupo.atalhos.filter((a) => bate(termo, t(`ajuda.teclas.${a.id}`), a.tecla));
    if (linhas.length === 0) continue;
    const tabela = el('table', 'aj-atalhos');
    for (const a of linhas) {
      const tr = tabela.insertRow();
      tr.insertCell().textContent = t(`ajuda.teclas.${a.id}`);
      tr.insertCell().append(el('code', '', a.tecla));
    }
    elementos.push(...(so ? [] : [el('h3', '', t(`ajuda.rotulos.g_${grupo.id}`))]), tabela);
    achados += linhas.length;
  }
  return { elementos, achados };
}

/** Um subitem de uma seção: o que aparece embaixo dela no menu lateral. */
export interface Subsecao {
  id: string;
  titulo: string;
}

/** Os subitens de cada seção, na ordem em que a seta passa por eles. */
export function subsecoes(secao: SecaoDaAjuda): Subsecao[] {
  if (secao === 'uso') return TOPICOS.map((id) => ({ id, titulo: t(`ajuda.comoUsar.${id}.titulo`) }));
  return GRUPOS_DE_ATALHOS.map((g) => ({ id: g.id, titulo: t(`ajuda.rotulos.g_${g.id}`) }));
}
