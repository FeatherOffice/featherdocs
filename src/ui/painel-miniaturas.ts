/**
 * Exibir → Miniaturas: as páginas em ponto pequeno numa coluna à esquerda; clicar leva até a página.
 * Só pinta o que está à vista (o mesmo quadro do motor, em escala reduzida) e guarda a versão de cada
 * miniatura, então rolar não pede nada de novo ao motor.
 */

import { t } from '../idiomas/index.ts';
import type { Motor } from '../ponte/index.ts';
import { pintarQuadro } from './pintor.ts';

/** Largura de uma miniatura em px. */
const LARGURA = 132;

export const MINIATURAS_CSS = `
.miniaturas { flex: none; width: ${LARGURA + 36}px; overflow-y: auto; border-right: 1px solid var(--chrome-rule); background: var(--chrome-bg); padding: 12px 0; box-sizing: border-box; }
.miniaturas[hidden] { display: none; }
.miniatura { display: block; margin: 0 auto 10px; padding: 0; border: 2px solid transparent; border-radius: 3px; background: none; cursor: pointer; }
.miniatura canvas { display: block; background: #fff; box-shadow: var(--sombra); }
.miniatura.on { border-color: var(--chrome-accent); }
.miniatura span { display: block; margin-top: 3px; color: var(--chrome-ink-soft); font-size: 11px; text-align: center; }
`;

interface Mini {
  botao: HTMLButtonElement;
  canvas: HTMLCanvasElement;
  versao: number;
}

export class PainelDeMiniaturas {
  readonly el = document.createElement('div');
  private minis: Mini[] = [];
  private versao = 0;
  private atual = 0;
  private relogio = 0;
  private larguraPt = 595.28;
  private alturaPt = 841.89;
  private readonly vista: IntersectionObserver;

  constructor(
    private readonly motor: Motor,
    private readonly irPara: (pagina: number) => void,
  ) {
    this.el.className = 'miniaturas';
    this.el.hidden = true;
    this.el.setAttribute('aria-label', t('cmd.miniaturas'));
    this.vista = new IntersectionObserver((itens) => {
      for (const i of itens) if (i.isIntersecting) this.pintar(Number((i.target as HTMLElement).dataset.pagina));
    }, { root: this.el, rootMargin: '200px' });
  }

  get visivel(): boolean {
    return !this.el.hidden;
  }

  alternar(): void {
    this.el.hidden = !this.el.hidden;
  }

  /** O documento mudou (ou o papel, ou o número de páginas): refaz as miniaturas à vista, sem pressa. */
  atualizar(paginas: number, versao: number, largura: number, altura: number): void {
    this.versao = versao;
    if (this.el.hidden) return;
    clearTimeout(this.relogio);
    this.relogio = window.setTimeout(() => this.refazer(paginas, largura, altura), 250);
  }

  /** A página do meio da tela fica marcada. */
  marcar(pagina: number): void {
    this.minis[this.atual]?.botao.classList.remove('on');
    this.atual = pagina;
    const m = this.minis[pagina];
    if (!m) return;
    m.botao.classList.add('on');
    const r = m.botao.getBoundingClientRect();
    const caixa = this.el.getBoundingClientRect();
    if (r.top < caixa.top || r.bottom > caixa.bottom) m.botao.scrollIntoView({ block: 'nearest' });
  }

  private refazer(paginas: number, largura: number, altura: number): void {
    const papelMudou = largura !== this.larguraPt || altura !== this.alturaPt;
    this.larguraPt = largura;
    this.alturaPt = altura;
    while (this.minis.length > paginas) {
      const m = this.minis.pop()!;
      this.vista.unobserve(m.botao);
      m.botao.remove();
    }
    const h = Math.round((LARGURA * altura) / largura);
    while (this.minis.length < paginas) {
      const p = this.minis.length;
      const botao = document.createElement('button');
      botao.type = 'button';
      botao.className = 'miniatura';
      botao.dataset.pagina = String(p);
      const canvas = document.createElement('canvas');
      const n = document.createElement('span');
      n.textContent = String(p + 1);
      botao.append(canvas, n);
      botao.addEventListener('click', () => this.irPara(p));
      this.el.append(botao);
      this.minis.push({ botao, canvas, versao: 0 });
      this.vista.observe(botao);
    }
    for (const m of this.minis) {
      if (papelMudou || m.canvas.style.height !== `${h}px`) {
        m.canvas.style.width = `${LARGURA}px`;
        m.canvas.style.height = `${h}px`;
        m.versao = 0;
      }
    }
    // As que estão à vista repintam já; as outras, quando aparecerem.
    const caixa = this.el.getBoundingClientRect();
    this.minis.forEach((m, p) => {
      const r = m.botao.getBoundingClientRect();
      if (r.bottom >= caixa.top - 200 && r.top <= caixa.bottom + 200) this.pintar(p);
    });
    this.marcar(Math.min(this.atual, paginas - 1));
  }

  private pintar(p: number): void {
    const m = this.minis[p];
    if (!m || m.versao === this.versao) return;
    const q = this.motor.quadro(p, 0);
    if (!q) return;
    const dpr = window.devicePixelRatio || 1;
    const escala = (LARGURA / this.larguraPt) * dpr;
    m.canvas.width = Math.ceil(this.larguraPt * escala);
    m.canvas.height = Math.ceil(this.alturaPt * escala);
    const ctx = m.canvas.getContext('2d');
    if (ctx) pintarQuadro(ctx, q, escala);
    m.versao = this.versao;
  }
}
