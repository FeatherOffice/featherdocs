/** O painel lateral de estrutura: os títulos do documento; clicar leva até eles. */

import { t } from '../idiomas/index.ts';
import type { Titulo } from '../ponte/tipos.ts';
import { Z } from './z.ts';

export const ESTRUTURA_CSS = `
.estrutura {
  width: 250px; flex: none; overflow: auto; padding: 10px 6px; background: var(--painel);
  border-right: 1px solid var(--borda); z-index: ${Z.barra};
}
.estrutura[hidden] { display: none; }
.estrutura h2 { margin: 0 6px 8px; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--texto-2); }
.estrutura button { display: flex; width: 100%; text-align: left; gap: 6px; justify-content: space-between; border-radius: 4px; }
.estrutura .pg { color: var(--texto-2); flex: none; }
.estrutura .tx { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.estrutura p { margin: 6px; color: var(--texto-2); }
`;

export class PainelEstrutura {
  readonly el = document.createElement('aside');
  private titulosCache: Titulo[] = [];

  constructor(private readonly ir: (t: Titulo) => void) {
    this.el.className = 'estrutura';
    this.el.setAttribute('aria-label', t('estrutura.titulo'));
  }

  get visivel(): boolean {
    return !this.el.hidden;
  }

  definirVisivel(v: boolean): void {
    this.el.hidden = !v;
  }

  /** Redesenha a lista (só chamar com o painel visível). */
  atualizar(titulos: Titulo[]): void {
    this.titulosCache = titulos;
    const cabecalho = document.createElement('h2');
    cabecalho.textContent = t('estrutura.titulo');
    const itens: HTMLElement[] = titulos.map((ti) => {
      const b = document.createElement('button');
      b.type = 'button';
      b.style.paddingLeft = `${9 + (ti.nivel - 1) * 14}px`;
      const tx = document.createElement('span');
      tx.className = 'tx';
      tx.textContent = ti.texto || '…';
      const pg = document.createElement('span');
      pg.className = 'pg';
      pg.textContent = String(ti.pagina + 1);
      b.append(tx, pg);
      b.addEventListener('click', () => this.ir(ti));
      return b;
    });
    if (itens.length === 0) {
      const vazio = document.createElement('p');
      vazio.textContent = t('estrutura.vazio');
      itens.push(vazio);
    }
    this.el.replaceChildren(cabecalho, ...itens);
  }

  /** Atualiza o rótulo e o título do painel para o novo idioma em tempo real. */
  atualizarIdioma(): void {
    this.el.setAttribute('aria-label', t('estrutura.titulo'));
    if (this.visivel) this.atualizar(this.titulosCache);
  }
}
