/**
 * As folhas: a rolagem com todas as páginas, **virtualizada** — só as páginas
 * visíveis (e uma de cada lado) têm canvas e quadro; as outras são só altura.
 * Um documento de 300 páginas custa o mesmo que um de 3.
 *
 * Cada folha tem dois canvas: o do texto (pintado do quadro do motor) e o da
 * sobreposição (seleção e cursor piscando, pintados do que o motor diz em `Visao`).
 * Mouse vira pedido ao motor (`aoPressionar`/`aoArrastar`); zoom é estado de interface.
 */

import type { Motor } from '../ponte/index.ts';
import { medir } from '../ponte/metricas.ts';
import type { Geo, Mudanca, Quadro, Visao } from '../ponte/tipos.ts';
import { pintarQuadro } from './pintor.ts';
import { Z } from './z.ts';

export const FOLHAS_CSS = `
.rolagem { position: relative; flex: 1; overflow: auto; background: var(--fundo); outline: none; }
.miolo { position: relative; margin: 0 auto; cursor: text; }
.folha, .sobre { position: absolute; left: 0; display: block; }
.folha { background: var(--papel); box-shadow: var(--sombra); z-index: ${Z.folhas}; }
.sobre { z-index: ${Z.sobreposicao}; pointer-events: none; }
.rascunho {
  position: absolute; z-index: ${Z.sobreposicao}; pointer-events: none; white-space: pre; padding: 0 2px;
  background: #fff8c5; color: #1a1a1a; border-bottom: 2px solid var(--acento); font-family: system-ui, sans-serif;
}
`;

/** Pixels CSS por ponto (96 dpi / 72 pt) com zoom 1. */
const PX_POR_PT = 96 / 72;
const ESPACO = 24;
export const ZOOM_MIN = 0.4;
export const ZOOM_MAX = 3;
const COR_SELECAO = 'rgba(26, 115, 232, 0.30)';
const COR_ACHADO = 'rgba(255, 213, 0, 0.45)';
const COR_SELECAO_SEM_FOCO = 'rgba(128, 134, 139, 0.30)';
const PISCAR_MS = 530;

interface Entrada {
  canvas: HTMLCanvasElement;
  sobre: HTMLCanvasElement;
  quadro: Quadro | null;
  /** Em que escala (px/pt × dpr) o canvas foi pintado pela última vez. */
  escalaPintada: number;
}

export interface GanchosDasFolhas {
  /** Botão apertado numa página (em pontos); `contagem` 1, 2 ou 3 (duplo e triplo clique). */
  aoPressionar(pagina: number, x: number, y: number, estender: boolean, contagem: number, ctrl: boolean): void;
  /** O mouse andou com o botão apertado. */
  aoArrastar(pagina: number, x: number, y: number): void;
  /** O botão direito numa página (em pontos), e onde o menu abre (na janela, em pixels). */
  aoMenu?(pagina: number, x: number, y: number, janela: { x: number; y: number }): void;
  /** O botão do mouse foi solto (depois de um clique ou de um arrasto). */
  aoSoltar?(): void;
  /** O que está à vista mudou (rolagem, zoom, tamanho da janela): a seleção a desenhar é outra. */
  aoMudarVisao(): void;
  /** A página no centro da janela mudou. */
  aoMudarPagina(pagina: number): void;
}

export class Folhas {
  readonly rolagem = document.createElement('div');
  readonly miolo = document.createElement('div');
  private readonly entradas = new Map<number, Entrada>();
  private readonly rascunhoEl = document.createElement('div');
  private larguraPt = 595.28;
  private alturaPt = 841.89;
  private paginas = 1;
  private versao = 0;
  private zoomAtual = 1;
  private paginaVisivel = 0;
  private pendente = 0;
  private visao: Visao | null = null;
  private comFoco = false;
  private piscando = true;
  private relogio = 0;
  private arrastando = false;
  /** Exibir → Contínuo: as páginas encostadas, sem vão. */
  private continuo = false;
  /** Exibir → Várias páginas: lado a lado, quantas couberem na largura. */
  private varias = false;

  constructor(
    private readonly motor: Motor,
    private readonly ganchos: GanchosDasFolhas,
  ) {
    this.rolagem.className = 'rolagem';
    this.rolagem.tabIndex = -1;
    this.miolo.className = 'miolo';
    this.rascunhoEl.className = 'rascunho';
    this.rascunhoEl.hidden = true;
    this.miolo.append(this.rascunhoEl);
    this.rolagem.append(this.miolo);
    this.rolagem.addEventListener('scroll', () => this.agendar(), { passive: true });
    new ResizeObserver(() => this.agendar()).observe(this.rolagem);
    this.miolo.addEventListener('mousedown', (e) => this.aoApertar(e));
    this.miolo.addEventListener('contextmenu', (e) => this.aoMenu(e));
    this.rolagem.addEventListener('wheel', (e) => this.aoRodar(e), { passive: false });
    this.medidas();
    this.relogio = window.setInterval(() => {
      this.piscando = !this.piscando;
      this.pintarCursor();
    }, PISCAR_MS);
  }

  get zoom(): number {
    return this.zoomAtual;
  }

  /** Pixels CSS por ponto, com o zoom. */
  get escala(): number {
    return PX_POR_PT * this.zoomAtual;
  }

  /** O vão entre uma página e a de baixo. */
  private get vao(): number {
    return this.continuo ? 0 : ESPACO;
  }

  /** A altura de uma fileira de páginas, com o vão. */
  private get passo(): number {
    return this.alturaPt * this.escala + this.vao;
  }

  /** Quantas páginas por fileira (1, ou as que cabem na largura em "Várias páginas"). */
  private get colunas(): number {
    if (!this.varias) return 1;
    const largura = this.larguraPt * this.escala;
    return Math.max(2, Math.floor((this.rolagem.clientWidth - ESPACO) / (largura + ESPACO)));
  }

  private topoDe(pagina: number): number {
    return ESPACO + Math.floor(pagina / this.colunas) * this.passo;
  }

  private esquerdaDe(pagina: number): number {
    return (pagina % this.colunas) * (this.larguraPt * this.escala + ESPACO);
  }

  /** Liga ou desliga o modo contínuo e as várias páginas (as folhas se rearrumam no lugar). */
  arrumar(continuo: boolean, varias: boolean): void {
    if (continuo === this.continuo && varias === this.varias) return;
    const atual = this.paginaVisivel;
    this.continuo = continuo;
    this.varias = varias;
    // Várias páginas: o zoom diminui até caberem duas lado a lado (como no Word).
    if (varias) {
      const cabe = (this.rolagem.clientWidth - 3 * ESPACO) / (2 * this.larguraPt * PX_POR_PT);
      if (this.zoomAtual > cabe) this.zoomAtual = Math.max(ZOOM_MIN, Math.floor(cabe * 100) / 100);
    }
    this.medidas();
    this.irParaPagina(atual);
    this.atualizar();
    this.ganchos.aoMudarVisao();
  }

  get modoContinuo(): boolean {
    return this.continuo;
  }

  get variasPaginas(): boolean {
    return this.varias;
  }

  /** Um documento novo ou aberto: tamanho do papel, páginas e versão; descarta tudo que estava pintado. */
  carregar(largura: number, altura: number, paginas: number, versao: number): void {
    this.larguraPt = largura;
    this.alturaPt = altura;
    this.paginas = paginas;
    this.versao = versao;
    this.limpar();
    this.rolagem.scrollTop = 0;
    this.medidas();
    this.atualizar();
  }

  private limpar(): void {
    for (const e of this.entradas.values()) {
      e.canvas.remove();
      e.sobre.remove();
    }
    this.entradas.clear();
  }

  /** Depois de uma edição: as páginas antes de `primeira_pagina` continuam como estavam; as outras repintam. */
  invalidar(m: Mudanca): void {
    this.versao = m.versao;
    this.paginas = m.paginas;
    for (const [p, e] of this.entradas) {
      if (p >= m.paginas) {
        e.canvas.remove();
        e.sobre.remove();
        this.entradas.delete(p);
      } else if (p < m.primeira_pagina && e.quadro) {
        e.quadro = { ...e.quadro, versao: m.versao };
      }
    }
    this.medidas();
    this.atualizar();
  }

  /** O papel mudou (A4 ↔ Carta): tudo repinta. */
  papel(largura: number, altura: number, m: Mudanca): void {
    this.larguraPt = largura;
    this.alturaPt = altura;
    this.limpar();
    this.invalidar(m);
  }

  private medidas(): void {
    const k = Math.min(this.colunas, Math.max(1, this.paginas));
    this.miolo.style.width = `${Math.round(k * this.larguraPt * this.escala + (k - 1) * ESPACO)}px`;
    this.miolo.style.height = `${Math.round(ESPACO + Math.ceil(this.paginas / this.colunas) * this.passo + (this.continuo ? ESPACO : 0))}px`;
  }

  private agendar(): void {
    if (this.pendente) return;
    this.pendente = requestAnimationFrame(() => {
      this.pendente = 0;
      // Em "Várias páginas", a largura da janela decide quantas cabem por fileira.
      if (this.varias) this.medidas();
      this.atualizar();
      this.ganchos.aoMudarVisao();
    });
  }

  /** As páginas com canvas (as visíveis e uma de cada lado): é onde a seleção precisa ser desenhada. */
  paginasVisiveis(): { de: number; ate: number } {
    const alto = this.rolagem.clientHeight;
    const topo = this.rolagem.scrollTop;
    const k = this.colunas;
    return {
      de: Math.max(0, (Math.floor((topo - ESPACO) / this.passo) - 1) * k),
      ate: Math.min(this.paginas - 1, (Math.floor((topo + alto) / this.passo) + 2) * k - 1),
    };
  }

  /** Cria, pinta e descarta as folhas conforme o que está à vista. */
  atualizar(): void {
    medir('quadro', 'folhas', () => {
      const { de, ate } = this.paginasVisiveis();
      for (const [p, e] of this.entradas) {
        if (p < de || p > ate) {
          e.canvas.remove();
          e.sobre.remove();
          this.entradas.delete(p);
        }
      }
      for (let p = de; p <= ate; p++) this.garantir(p);
      this.avisarPagina(this.rolagem.scrollTop + this.rolagem.clientHeight / 2);
    });
  }

  private avisarPagina(yCentro: number): void {
    const p = Math.max(0, Math.min(this.paginas - 1, Math.floor((yCentro - ESPACO / 2) / this.passo) * this.colunas));
    if (p !== this.paginaVisivel) {
      this.paginaVisivel = p;
      this.ganchos.aoMudarPagina(p);
    }
  }

  private garantir(p: number): void {
    let e = this.entradas.get(p);
    if (!e) {
      const canvas = document.createElement('canvas');
      canvas.className = 'folha';
      const sobre = document.createElement('canvas');
      sobre.className = 'sobre';
      this.miolo.append(canvas, sobre);
      e = { canvas, sobre, quadro: null, escalaPintada: 0 };
      this.entradas.set(p, e);
    }
    const larguraPx = this.larguraPt * this.escala;
    const alturaPx = this.alturaPt * this.escala;
    for (const c of [e.canvas, e.sobre]) {
      c.style.top = `${this.topoDe(p)}px`;
      c.style.left = `${this.esquerdaDe(p)}px`;
      c.style.width = `${larguraPx}px`;
      c.style.height = `${alturaPx}px`;
    }
    const dpr = window.devicePixelRatio || 1;
    const escalaPintura = this.escala * dpr;
    let repintar = e.escalaPintada !== escalaPintura;
    if (!e.quadro || e.quadro.versao !== this.versao) {
      const novo = this.motor.quadro(p, e.quadro?.versao ?? 0);
      if (novo) {
        e.quadro = novo;
        repintar = true;
      }
    }
    if (repintar && e.quadro) {
      for (const c of [e.canvas, e.sobre]) {
        c.width = Math.ceil(larguraPx * dpr);
        c.height = Math.ceil(alturaPx * dpr);
      }
      const ctx = e.canvas.getContext('2d');
      if (ctx) pintarQuadro(ctx, e.quadro, escalaPintura);
      e.escalaPintada = escalaPintura;
      this.pintarSobreposicao(p, e);
    }
  }

  // --- cursor e seleção ---------------------------------------------------------------

  /** O que o motor mandou: onde está o cursor e o que está selecionado. */
  definirVisao(v: Visao): void {
    this.visao = v;
    this.piscando = true;
    for (const [p, e] of this.entradas) this.pintarSobreposicao(p, e);
  }

  /** O foco do teclado entrou ou saiu: a seleção muda de cor e o cursor para de piscar. */
  definirFoco(tem: boolean): void {
    this.comFoco = tem;
    this.piscando = true;
    for (const [p, e] of this.entradas) this.pintarSobreposicao(p, e);
  }

  private pintarCursor(): void {
    const g = this.visao?.cursor;
    const e = g ? this.entradas.get(g.pagina) : undefined;
    if (g && e) this.pintarSobreposicao(g.pagina, e);
  }

  private pintarSobreposicao(p: number, e: Entrada): void {
    const ctx = e.sobre.getContext('2d');
    if (!ctx) return;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, e.sobre.width, e.sobre.height);
    const v = this.visao;
    if (!v) return;
    const s = e.escalaPintada || this.escala * (window.devicePixelRatio || 1);
    ctx.setTransform(s, 0, 0, s, 0, 0);
    ctx.fillStyle = COR_ACHADO;
    for (const r of v.achados) if (r.pagina === p) ctx.fillRect(r.x, r.y, r.w, r.h);
    ctx.fillStyle = this.comFoco ? COR_SELECAO : COR_SELECAO_SEM_FOCO;
    for (const r of v.retangulos) if (r.pagina === p) ctx.fillRect(r.x, r.y, r.w, r.h);
    const g = v.cursor;
    if (g && g.pagina === p && this.comFoco && this.piscando && v.retangulos.length === 0) {
      ctx.fillStyle = '#1a1a1a';
      ctx.fillRect(g.x - 0.4, g.y, 0.9, g.h);
    }
  }

  /** O cursor em pixels dentro do miolo (para pôr a entrada de teclado e o rascunho do IME). */
  posicaoEmPx(g: Geo): { x: number; y: number } {
    return { x: this.esquerdaDe(g.pagina) + g.x * this.escala, y: this.topoDe(g.pagina) + g.y * this.escala };
  }

  /** Rola só o necessário para o cursor ficar à vista. */
  mostrarCursor(g: Geo): void {
    const { y } = this.posicaoEmPx(g);
    const topo = this.rolagem.scrollTop;
    const alto = this.rolagem.clientHeight;
    const folga = 70;
    if (y < topo + folga) this.rolagem.scrollTop = Math.max(0, y - folga);
    else if (y + g.h * this.escala > topo + alto - folga) this.rolagem.scrollTop = y + g.h * this.escala - alto + folga;
  }

  /** O rascunho de uma composição (IME) junto ao cursor; `null` esconde. */
  rascunho(texto: string | null): void {
    const g = this.visao?.cursor;
    if (!texto || !g) {
      this.rascunhoEl.hidden = true;
      return;
    }
    const { x, y } = this.posicaoEmPx(g);
    this.rascunhoEl.textContent = texto;
    this.rascunhoEl.style.left = `${x}px`;
    this.rascunhoEl.style.top = `${y}px`;
    this.rascunhoEl.style.fontSize = `${11 * this.escala}px`;
    this.rascunhoEl.hidden = false;
  }

  // --- mouse --------------------------------------------------------------------------

  /** O ponto da página sob o mouse (a página mais perto, se está no vão entre elas). */
  private pontoDoMouse(e: MouseEvent): { pagina: number; x: number; y: number } {
    const caixa = this.miolo.getBoundingClientRect();
    const y = e.clientY - caixa.top;
    const x = e.clientX - caixa.left;
    const fileira = Math.max(0, Math.floor((y - ESPACO / 2) / this.passo));
    const coluna = Math.max(0, Math.min(this.colunas - 1, Math.floor((x + ESPACO / 2) / (this.larguraPt * this.escala + ESPACO))));
    const pagina = Math.max(0, Math.min(this.paginas - 1, fileira * this.colunas + coluna));
    return { pagina, x: (x - this.esquerdaDe(pagina)) / this.escala, y: (y - this.topoDe(pagina)) / this.escala };
  }

  private aoApertar(e: MouseEvent): void {
    if (e.button !== 0) return;
    e.preventDefault();
    const p = this.pontoDoMouse(e);
    this.arrastando = true;
    this.ganchos.aoPressionar(p.pagina, p.x, p.y, e.shiftKey, Math.min(3, Math.max(1, e.detail)), e.ctrlKey || e.metaKey);
    const mover = (m: MouseEvent): void => {
      if (!this.arrastando) return;
      this.rolarPelaBorda(m);
      const q = this.pontoDoMouse(m);
      this.ganchos.aoArrastar(q.pagina, q.x, q.y);
    };
    const soltar = (): void => {
      this.arrastando = false;
      this.ganchos.aoSoltar?.();
      document.removeEventListener('mousemove', mover);
      document.removeEventListener('mouseup', soltar);
    };
    document.addEventListener('mousemove', mover);
    document.addEventListener('mouseup', soltar);
  }

  /** O botão direito: nada do menu do navegador; o aplicativo abre o dele. */
  private aoMenu(e: MouseEvent): void {
    e.preventDefault();
    if (!this.ganchos.aoMenu) return;
    const p = this.pontoDoMouse(e);
    this.ganchos.aoMenu(p.pagina, p.x, p.y, { x: e.clientX, y: e.clientY });
  }

  /** Arrastando para fora da janela, a folha rola sozinha. */
  private rolarPelaBorda(e: MouseEvent): void {
    const r = this.rolagem.getBoundingClientRect();
    if (e.clientY < r.top + 24) this.rolagem.scrollTop -= 24;
    else if (e.clientY > r.bottom - 24) this.rolagem.scrollTop += 24;
  }

  private aoRodar(e: WheelEvent): void {
    if (!e.ctrlKey) return;
    e.preventDefault();
    this.definirZoom(this.zoomAtual * (e.deltaY < 0 ? 1.1 : 1 / 1.1));
  }

  /** Muda o zoom mantendo a posição que está no meio da janela. */
  definirZoom(z: number): void {
    const novo = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, Math.round(z * 100) / 100));
    if (novo === this.zoomAtual) return;
    const fracao = (this.rolagem.scrollTop + this.rolagem.clientHeight / 2) / Math.max(1, this.miolo.offsetHeight);
    this.zoomAtual = novo;
    this.medidas();
    this.rolagem.scrollTop = fracao * this.miolo.offsetHeight - this.rolagem.clientHeight / 2;
    this.atualizar();
    this.ganchos.aoMudarPagina(this.paginaVisivel);
    this.ganchos.aoMudarVisao();
  }

  /** Rola até o topo da página. */
  irParaPagina(p: number): void {
    this.rolagem.scrollTop = this.topoDe(Math.max(0, Math.min(this.paginas - 1, p))) - ESPACO;
  }

  get paginaAtual(): number {
    return this.paginaVisivel;
  }

  /** Para o relógio do cursor (testes e desmontagem). */
  parar(): void {
    clearInterval(this.relogio);
  }
}
