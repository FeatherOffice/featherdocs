/** A barra de status: página, contagem de palavras e se há mudança não salva. */

import { t } from '../idiomas/index.ts';
import type { Estado } from '../ponte/tipos.ts';

export const RODAPE_CSS = `
.rodape {
  display: flex; gap: 18px; align-items: center; padding: 3px 12px; background: var(--painel);
  border-top: 1px solid var(--borda); color: var(--texto-2); font-size: 12px;
}
.rodape .espaco { flex: 1; }
.rodape .sujo { color: var(--perigo); }
`;

export class Rodape {
  readonly el = document.createElement('footer');
  private readonly pagina = document.createElement('span');
  private readonly palavras = document.createElement('span');
  private readonly caracteres = document.createElement('span');
  private readonly salvo = document.createElement('span');

  constructor() {
    this.el.className = 'rodape';
    const espaco = document.createElement('span');
    espaco.className = 'espaco';
    this.el.append(this.pagina, this.palavras, this.caracteres, espaco, this.salvo);
  }

  /** `pagina` começa em 0. */
  atualizar(e: Estado, pagina: number): void {
    this.pagina.textContent = t('status.pagina_de', { atual: Math.min(pagina + 1, e.paginas), total: e.paginas });
    this.palavras.textContent = t('status.palavras', { n: e.palavras.toLocaleString() });
    this.caracteres.textContent = t('status.caracteres', { n: e.caracteres.toLocaleString() });
    this.salvo.textContent = e.sujo ? t('status.nao_salvo') : t('status.salvo');
    this.salvo.classList.toggle('sujo', e.sujo);
  }
}
