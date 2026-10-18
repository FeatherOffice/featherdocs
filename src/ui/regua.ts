/**
 * A régua horizontal: centímetros a partir da borda da página e as duas margens (esquerda e direita) como
 * alças que se arrastam. Só desenha e traduz o arrasto em pedido: quem decide se a margem vale é o motor
 * (`margens_definir`), e o que ele recusa volta como aviso.
 */

import { formatar, t } from '../idiomas/index.ts';
import { Z } from './z.ts';

export const REGUA_CSS = `
.regua { position: relative; flex: none; height: 22px; background: var(--chrome-bg-2); border-bottom: 1px solid var(--chrome-rule); overflow: hidden; z-index: ${Z.sobreposicao}; user-select: none; }
.regua canvas { position: absolute; inset: 0; width: 100%; height: 100%; }
.regua-alca { position: absolute; top: 0; width: 9px; height: 22px; margin-left: -4px; cursor: ew-resize; touch-action: none; }
.regua-alca::after { content: ''; position: absolute; left: 3px; top: 4px; width: 3px; height: 14px; border-radius: 1px; background: var(--chrome-accent); opacity: 0.8; }
.regua-alca:hover::after, .regua-alca.arrastando::after { opacity: 1; }
.regua-dica { position: absolute; top: 24px; padding: 2px 6px; border-radius: 4px; background: var(--ink); color: var(--surface); font-size: 11px; pointer-events: none; z-index: ${Z.menu}; white-space: nowrap; }
`;

const PT_POR_CM = 72 / 2.54;
/** Passo do arrasto: 1 mm. */
const PASSO_PT = 72 / 25.4;
/** Quanto de texto o arrasto preserva entre as duas alças (o motor exige 100 pt). */
const MIN_TEXTO_PT = 100;

export interface GanchosDaRegua {
  /** Pede ao motor as novas margens laterais (em pontos). */
  margens(esq: number, dir: number): void;
}

export interface MedidaDaRegua {
  /** O `x` da borda esquerda da página, em px, dentro da régua. */
  xDaPagina: number;
  /** px por ponto (zoom incluído). */
  escala: number;
  larguraPt: number;
  margemEsq: number;
  margemDir: number;
  /** Largura visível da área de rolagem, em px. */
  larguraVisivel: number;
}

export class Regua {
  readonly el = document.createElement('div');
  private readonly canvas = document.createElement('canvas');
  private readonly alcaEsq = document.createElement('div');
  private readonly alcaDir = document.createElement('div');
  private readonly dica = document.createElement('div');
  private medida: MedidaDaRegua | null = null;

  constructor(private readonly g: GanchosDaRegua) {
    this.el.className = 'regua';
    this.el.setAttribute('aria-label', t('regua.dica'));
    this.dica.className = 'regua-dica';
    this.dica.hidden = true;
    for (const [alca, lado] of [[this.alcaEsq, 'esq'], [this.alcaDir, 'dir']] as const) {
      alca.className = 'regua-alca';
      alca.title = t('regua.dica');
      this.ligarArrasto(alca, lado);
    }
    this.el.append(this.canvas, this.alcaEsq, this.alcaDir, this.dica);
  }

  /** Redesenha para a posição e o zoom atuais. */
  atualizar(m: MedidaDaRegua): void {
    this.medida = m;
    this.desenhar(m, m.margemEsq, m.margemDir);
  }

  private desenhar(m: MedidaDaRegua, esq: number, dir: number): void {
    const dpr = window.devicePixelRatio || 1;
    const w = Math.max(1, Math.round(m.larguraVisivel));
    const h = 22;
    if (this.canvas.width !== Math.round(w * dpr) || this.canvas.height !== Math.round(h * dpr)) {
      this.canvas.width = Math.round(w * dpr);
      this.canvas.height = Math.round(h * dpr);
    }
    const c = this.canvas.getContext('2d');
    if (!c) return;
    const estilo = getComputedStyle(document.documentElement);
    const tinta = estilo.getPropertyValue('--chrome-ink-soft').trim() || '#666';
    const area = estilo.getPropertyValue('--surface').trim() || '#fff';
    const sombra = estilo.getPropertyValue('--chrome-active').trim() || '#ddd';
    c.setTransform(dpr, 0, 0, dpr, 0, 0);
    c.clearRect(0, 0, w, h);
    const x0 = m.xDaPagina;
    const px = (pt: number): number => x0 + pt * m.escala;
    // Fora do papel e dentro das margens ficam sombreados; a área de texto fica clara.
    c.fillStyle = sombra;
    c.fillRect(0, 4, w, h - 8);
    c.fillStyle = area;
    c.fillRect(px(esq), 4, (m.larguraPt - esq - dir) * m.escala, h - 8);
    c.fillStyle = tinta;
    c.strokeStyle = tinta;
    c.font = '10px system-ui, sans-serif';
    c.textAlign = 'center';
    c.textBaseline = 'middle';
    c.lineWidth = 1;
    // Graduação a cada meio centímetro, número a cada centímetro (contado da margem esquerda, como no Word).
    const meio = PT_POR_CM / 2;
    const de = Math.ceil(-esq / meio);
    const ate = Math.floor((m.larguraPt - esq) / meio);
    for (let i = de; i <= ate; i++) {
      const x = Math.round(px(esq + i * meio)) + 0.5;
      if (x < 0 || x > w) continue;
      if (i % 2 === 0) {
        if (i !== 0 && m.escala * PT_POR_CM > 16) c.fillText(String(Math.abs(i / 2)), x, h / 2);
        else if (i === 0) {
          c.beginPath();
          c.moveTo(x, 6);
          c.lineTo(x, h - 6);
          c.stroke();
        }
      } else {
        c.beginPath();
        c.moveTo(x, h / 2 - 2);
        c.lineTo(x, h / 2 + 2);
        c.stroke();
      }
    }
    this.alcaEsq.style.left = `${px(esq)}px`;
    this.alcaDir.style.left = `${px(m.larguraPt - dir)}px`;
  }

  private ligarArrasto(alca: HTMLElement, lado: 'esq' | 'dir'): void {
    alca.addEventListener('pointerdown', (e) => {
      const m = this.medida;
      if (!m) return;
      e.preventDefault();
      alca.setPointerCapture(e.pointerId);
      alca.classList.add('arrastando');
      let esq = m.margemEsq;
      let dir = m.margemDir;
      const mover = (ev: PointerEvent): void => {
        const caixa = this.el.getBoundingClientRect();
        const pt = (ev.clientX - caixa.left - m.xDaPagina) / m.escala;
        const arredonda = (v: number): number => Math.round(v / PASSO_PT) * PASSO_PT;
        if (lado === 'esq') esq = Math.min(Math.max(0, arredonda(pt)), m.larguraPt - dir - MIN_TEXTO_PT);
        else dir = Math.min(Math.max(0, arredonda(m.larguraPt - pt)), m.larguraPt - esq - MIN_TEXTO_PT);
        this.desenhar(m, esq, dir);
        const valor = lado === 'esq' ? esq : dir;
        this.dica.hidden = false;
        this.dica.textContent = formatar(t('regua.cm'), { n: (valor / PT_POR_CM).toFixed(2).replace('.', ',') });
        this.dica.style.left = `${ev.clientX - caixa.left + 8}px`;
      };
      const soltar = (): void => {
        alca.removeEventListener('pointermove', mover);
        alca.removeEventListener('pointerup', soltar);
        alca.removeEventListener('pointercancel', soltar);
        alca.classList.remove('arrastando');
        this.dica.hidden = true;
        if (Math.abs(esq - m.margemEsq) > 0.01 || Math.abs(dir - m.margemDir) > 0.01) this.g.margens(esq, dir);
        else this.atualizar(m);
      };
      alca.addEventListener('pointermove', mover);
      alca.addEventListener('pointerup', soltar);
      alca.addEventListener('pointercancel', soltar);
    });
  }
}
