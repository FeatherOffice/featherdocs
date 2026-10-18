/**
 * O painel de revisão, à direita das páginas: os comentários (com as respostas, responder,
 * resolver e excluir) e as alterações sugeridas (aceitar ou rejeitar todas). Só mostra o que o
 * motor manda e pede ao motor o que o usuário fizer.
 */

import { t } from '../idiomas/index.ts';
import type { InfoComentario, InfoRevisao } from '../ponte/tipos.ts';
import { Z } from './z.ts';

export const REVISAO_CSS = `
.painel-revisao {
  width: 290px; flex: none; overflow: auto; padding: 10px 8px; background: var(--painel);
  border-left: 1px solid var(--borda); z-index: ${Z.barra}; font-size: 13px;
}
.painel-revisao[hidden] { display: none; }
.painel-revisao h2 { margin: 4px 4px 8px; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--texto-2); display: flex; justify-content: space-between; align-items: center; }
.painel-revisao .cartao { border: 1px solid var(--borda); border-left: 3px solid #fbbc04; border-radius: 6px; padding: 8px; margin: 0 2px 8px; background: var(--fundo, transparent); cursor: pointer; }
.painel-revisao .cartao.resolvido { border-left-color: #9aa0a6; opacity: 0.75; }
.painel-revisao .quem { font-weight: 600; }
.painel-revisao .quando, .painel-revisao .trecho { color: var(--texto-2); font-size: 12px; }
.painel-revisao .trecho { font-style: italic; margin: 2px 0 4px; }
.painel-revisao .resposta { margin: 6px 0 0 10px; padding-left: 8px; border-left: 2px solid var(--borda); }
.painel-revisao .acoes { display: flex; gap: 6px; margin-top: 6px; flex-wrap: wrap; }
.painel-revisao .acoes button { font-size: 12px; padding: 2px 8px; }
.painel-revisao input { width: 100%; box-sizing: border-box; margin-top: 6px; padding: 4px 6px; border: 1px solid var(--borda); border-radius: 4px; background: var(--painel); color: inherit; font: inherit; }
.painel-revisao .mudanca { padding: 4px 6px; border-radius: 4px; cursor: pointer; }
.painel-revisao .mudanca:hover { background: var(--hover, rgba(0,0,0,.05)); }
.painel-revisao .ins { text-decoration: underline; }
.painel-revisao .del { text-decoration: line-through; }
.painel-revisao p { margin: 6px; color: var(--texto-2); }
`;

export interface AcoesDaRevisao {
  irComentario(id: number): void;
  responder(id: number, texto: string): void;
  resolver(id: number, sim: boolean): void;
  excluir(id: number): void;
  irBloco(id: number): void;
  resolverTodas(aceitar: boolean): void;
  fechar(): void;
}

/** O que o painel mostra: todos os comentários ou só os abertos. */
export type FiltroDeComentarios = 'todos' | 'abertos';

function el<K extends keyof HTMLElementTagNameMap>(tag: K, classe = '', texto = ''): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  if (classe) e.className = classe;
  if (texto) e.textContent = texto;
  return e;
}

function botao(texto: string, ao: () => void): HTMLButtonElement {
  const b = el('button', '', texto);
  b.type = 'button';
  b.addEventListener('click', (e) => {
    e.stopPropagation();
    ao();
  });
  return b;
}

export class PainelDeRevisao {
  readonly el = document.createElement('aside');
  filtro: FiltroDeComentarios = 'todos';

  constructor(private readonly a: AcoesDaRevisao) {
    this.el.className = 'painel-revisao';
    this.el.hidden = true;
    this.el.setAttribute('aria-label', t('revisao.painel'));
  }

  get visivel(): boolean {
    return !this.el.hidden;
  }

  definirVisivel(v: boolean): void {
    this.el.hidden = !v;
  }

  private cartao(c: InfoComentario): HTMLElement {
    const k = el('div', `cartao${c.resolvido ? ' resolvido' : ''}`);
    k.dataset.id = String(c.id);
    k.append(el('div', 'quem', c.autor || t('revisao.sem_autor')), el('div', 'quando', `${c.quando}${c.pagina ? ` · ${t('revisao.pagina', { n: c.pagina })}` : ''}`));
    if (c.trecho) k.append(el('div', 'trecho', `“${c.trecho}”`));
    k.append(el('div', '', c.texto));
    for (const r of c.respostas) {
      const d = el('div', 'resposta');
      d.append(el('div', 'quem', r.autor || t('revisao.sem_autor')), el('div', '', r.texto));
      k.append(d);
    }
    const resposta = el('input');
    resposta.placeholder = t('revisao.responder');
    resposta.addEventListener('click', (e) => e.stopPropagation());
    resposta.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && resposta.value.trim()) this.a.responder(c.id, resposta.value);
      e.stopPropagation();
    });
    const acoes = el('div', 'acoes');
    acoes.append(botao(c.resolvido ? t('revisao.reabrir') : t('revisao.resolver'), () => this.a.resolver(c.id, !c.resolvido)), botao(t('revisao.excluir'), () => this.a.excluir(c.id)));
    k.append(resposta, acoes);
    k.addEventListener('click', () => this.a.irComentario(c.id));
    return k;
  }

  /** Redesenha (só chamar com o painel visível). */
  atualizar(comentarios: InfoComentario[], revisoes: InfoRevisao[], destaque?: number): void {
    const titulo = el('h2', '', t('revisao.comentarios'));
    titulo.append(botao('✕', () => this.a.fechar()));
    const lista = comentarios.filter((c) => this.filtro === 'todos' || !c.resolvido);
    const itens: HTMLElement[] = lista.map((c) => this.cartao(c));
    if (!itens.length) itens.push(el('p', '', t('revisao.sem_comentarios')));
    const titulo2 = el('h2', '', t('revisao.alteracoes'));
    const mudancas: HTMLElement[] = revisoes.map((r) => {
      const d = el('div', 'mudanca');
      const quem = el('span', 'quem', `${r.autor || t('revisao.sem_autor')}: `);
      d.append(quem, el('span', r.tipo === 'insercao' ? 'ins' : 'del', r.texto.length > 80 ? `${r.texto.slice(0, 79)}…` : r.texto));
      d.title = r.tipo === 'insercao' ? t('revisao.inserido') : t('revisao.apagado');
      d.addEventListener('click', () => this.a.irBloco(r.bloco));
      return d;
    });
    if (!mudancas.length) mudancas.push(el('p', '', t('revisao.sem_alteracoes')));
    else {
      const acoes = el('div', 'acoes');
      acoes.append(botao(t('revisao.aceitar_todas'), () => this.a.resolverTodas(true)), botao(t('revisao.rejeitar_todas'), () => this.a.resolverTodas(false)));
      mudancas.unshift(acoes);
    }
    this.el.replaceChildren(titulo, ...itens, titulo2, ...mudancas);
    if (destaque !== undefined) this.el.querySelector<HTMLElement>(`.cartao[data-id="${destaque}"]`)?.scrollIntoView({ block: 'nearest' });
  }
}
