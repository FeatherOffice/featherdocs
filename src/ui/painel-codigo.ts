/**
 * O modo dividido: o código (Markdown ou LaTeX) à esquerda das páginas, os dois editáveis.
 *
 * Só desenha e repassa: o texto digitado vai ao motor depois de uma pausa (o motor relê e troca
 * só os blocos que mudaram); quando o documento muda pelo papel, o código novo chega pronto do
 * motor, com as linhas de cada bloco — é assim que o cursor de um lado acha o lugar no outro.
 * Enquanto a pessoa digita no código, ele **não** é reescrito (o motor normalizaria o que ainda
 * está pela metade); o código volta à forma do motor quando o foco sai dele.
 */

import { t } from '../idiomas/index.ts';
import type { FormatoDoCodigo, MapaDoCodigo } from '../ponte/tipos.ts';
import { FAMILIA_CODIGO } from './familias.ts';
import { icon } from './icones.ts';

/** A pausa na digitação antes de mandar o código ao motor. */
const PAUSA_MS = 300;
/** Altura de uma linha do código em px (o número da linha e a marca do bloco usam a mesma). */
const LINHA_PX = 19;

export const CODIGO_CSS = `
.codigo { position: relative; display: flex; flex-direction: column; min-width: 220px; border-right: 1px solid var(--chrome-rule); background: var(--surface); }
.codigo[hidden] { display: none; }
.codigo-topo { display: flex; align-items: center; gap: 8px; height: 30px; padding: 0 6px 0 12px; border-bottom: 1px solid var(--chrome-rule); color: var(--chrome-ink); font-size: 12px; font-weight: 600; letter-spacing: 0.04em; }
.codigo-topo span { flex: 1; }
.codigo-topo button { display: inline-flex; align-items: center; justify-content: center; width: 24px; height: 22px; padding: 0; border: 0; border-radius: 4px; background: transparent; color: var(--chrome-ink-soft); cursor: pointer; }
.codigo-topo button:hover { background: var(--chrome-hover); color: var(--chrome-ink); }
.codigo-topo svg { width: 14px; height: 14px; }
.codigo-corpo { position: relative; display: flex; flex: 1; min-height: 0; overflow: hidden; }
.codigo-linhas {
  position: relative; flex: none; width: 44px; margin: 0; padding: 8px 8px 8px 0; overflow: hidden; box-sizing: border-box;
  color: var(--chrome-ink-soft); text-align: right; user-select: none; background: var(--chrome-bg);
}
.codigo-marca { position: absolute; left: 0; right: 0; background: var(--chrome-accent); opacity: 0.16; pointer-events: none; }
.codigo-area { position: relative; flex: 1; min-width: 0; overflow: hidden; }
.codigo-texto {
  position: absolute; inset: 0; width: 100%; height: 100%; margin: 0; padding: 8px 10px; border: 0; outline: none; resize: none; box-sizing: border-box;
  background: transparent; color: transparent; caret-color: var(--chrome-ink); white-space: pre; overflow: auto; tab-size: 2;
}
.codigo-texto::selection { background: color-mix(in srgb, var(--chrome-accent) 30%, transparent); color: transparent; }
/* As cores ficam numa camada por baixo da caixa de texto (que tem a letra transparente) e rolam junto com ela. */
.codigo-realce {
  position: absolute; top: 0; left: 0; margin: 0; padding: 8px 10px; pointer-events: none; box-sizing: border-box;
  white-space: pre; tab-size: 2; color: var(--chrome-ink); will-change: transform;
}
.codigo-linhas, .codigo-texto, .codigo-realce { font-family: ${FAMILIA_CODIGO}; font-size: 12.5px; line-height: ${LINHA_PX}px; }
.cr-1 { color: #1f4fbf; }
.cr-2 { color: #6a8a5a; font-style: italic; }
.cr-3 { color: #b05a00; }
.cr-4 { color: #8a2be2; }
.cr-5 { color: #0b7f7f; font-weight: 600; }
.cr-6 { color: #1f4fbf; font-weight: 700; }
.cr-7 { color: #a0306a; }
.cr-8 { color: #b05a00; }
.cr-9 { color: #0b7f7f; text-decoration: underline; }
.cr-10, .cr-11 { color: #b05a00; font-weight: 700; }
html[data-theme='dark'] .cr-1, html[data-theme='dark'] .cr-6 { color: #7aa7ff; }
html[data-theme='dark'] .cr-2 { color: #8aa47c; }
html[data-theme='dark'] .cr-3, html[data-theme='dark'] .cr-8, html[data-theme='dark'] .cr-10, html[data-theme='dark'] .cr-11 { color: #e5a35a; }
html[data-theme='dark'] .cr-4 { color: #c39bff; }
html[data-theme='dark'] .cr-5, html[data-theme='dark'] .cr-9 { color: #5fd0c8; }
html[data-theme='dark'] .cr-7 { color: #e88abf; }
.codigo-divisor { position: absolute; top: 0; right: -3px; bottom: 0; width: 6px; cursor: col-resize; z-index: 1; }
.codigo-divisor:hover, .codigo-divisor.arrastando { background: var(--chrome-accent); opacity: 0.35; }
`;

export interface GanchosDoCodigo {
  /** O código mudou (depois da pausa): o motor relê e troca os blocos. */
  aplicar(formato: FormatoDoCodigo, texto: string): void;
  /** O cursor do código está no bloco `id`: o papel acompanha. */
  irParaBloco(id: number): void;
  /** O código saiu de foco: pede de novo ao motor (na forma dele). */
  recarregar(): void;
  /** As cores do código (o motor separa comando, comentário, chave…): trios `[início, fim, tipo]`. */
  cores(formato: FormatoDoCodigo, texto: string): number[];
  fechar(): void;
}

export class PainelDeCodigo {
  readonly el = document.createElement('div');
  private readonly titulo = document.createElement('span');
  private readonly linhas = document.createElement('div');
  private readonly numeros = document.createElement('div');
  private readonly marca = document.createElement('div');
  private readonly texto = document.createElement('textarea');
  private readonly realce = document.createElement('pre');
  private formatoAtual: FormatoDoCodigo = 'md';
  private mapa: MapaDoCodigo = { texto: '', blocos: [] };
  private relogio = 0;
  private marcado: { de: number; ate: number } | null = null;
  private ultimoBloco = -1;

  constructor(private readonly g: GanchosDoCodigo) {
    this.el.className = 'codigo';
    this.el.hidden = true;
    this.el.style.width = `${larguraSalva()}px`;
    const topo = document.createElement('div');
    topo.className = 'codigo-topo';
    const fechar = document.createElement('button');
    fechar.type = 'button';
    fechar.title = t('codigo.fechar');
    fechar.setAttribute('aria-label', t('codigo.fechar'));
    fechar.append(icon('fechar'));
    fechar.addEventListener('click', () => this.g.fechar());
    topo.append(this.titulo, fechar);
    const corpo = document.createElement('div');
    corpo.className = 'codigo-corpo';
    this.linhas.className = 'codigo-linhas';
    this.marca.className = 'codigo-marca';
    this.marca.hidden = true;
    this.linhas.append(this.marca, this.numeros);
    this.texto.className = 'codigo-texto';
    this.texto.spellcheck = false;
    this.texto.wrap = 'off';
    this.texto.setAttribute('aria-label', t('codigo.aria'));
    this.texto.addEventListener('input', () => this.aoDigitar());
    this.texto.addEventListener('scroll', () => this.acompanharRolagem());
    this.realce.className = 'codigo-realce';
    this.realce.setAttribute('aria-hidden', 'true');
    this.texto.addEventListener('keydown', (e) => this.aoTeclar(e));
    for (const ev of ['click', 'keyup'] as const) this.texto.addEventListener(ev, () => this.cursorMudou());
    this.texto.addEventListener('blur', () => this.aoSair());
    const area = document.createElement('div');
    area.className = 'codigo-area';
    area.append(this.realce, this.texto);
    corpo.append(this.linhas, area);
    const divisor = document.createElement('div');
    divisor.className = 'codigo-divisor';
    divisor.addEventListener('mousedown', (e) => this.arrastar(e, divisor));
    this.el.append(topo, corpo, divisor);
  }

  get visivel(): boolean {
    return !this.el.hidden;
  }

  get formato(): FormatoDoCodigo {
    return this.formatoAtual;
  }

  /** O código está sendo editado agora (o papel não o reescreve). */
  get editando(): boolean {
    return document.activeElement === this.texto;
  }

  get textoAtual(): string {
    return this.texto.value;
  }

  mostrar(formato: FormatoDoCodigo, mapa: MapaDoCodigo): void {
    if (this.relogio && this.formatoAtual !== formato) {
      clearTimeout(this.relogio);
      this.relogio = 0;
      this.g.aplicar(this.formatoAtual, this.texto.value);
    }
    this.formatoAtual = formato;
    this.titulo.textContent = t(formato === 'md' ? 'codigo.markdown' : 'codigo.latex');
    this.el.hidden = false;
    this.carregar(mapa);
  }

  esconder(): void {
    if (this.relogio) {
      clearTimeout(this.relogio);
      this.relogio = 0;
      this.g.aplicar(this.formatoAtual, this.texto.value);
    }
    this.el.hidden = true;
  }

  /** O código novo do motor (o documento mudou pelo papel). Guarda a rolagem e, se der, o cursor. */
  carregar(mapa: MapaDoCodigo): void {
    this.mapa = mapa;
    if (this.texto.value !== mapa.texto) {
      const { scrollTop, selectionStart, selectionEnd } = this.texto;
      this.texto.value = mapa.texto;
      this.texto.setSelectionRange(Math.min(selectionStart, mapa.texto.length), Math.min(selectionEnd, mapa.texto.length));
      this.texto.scrollTop = scrollTop;
    }
    this.numerar();
    this.colorir();
    this.desenharMarca();
  }

  /** Pinta a camada de cores com os trechos que o motor separou. */
  private colorir(): void {
    const texto = this.texto.value;
    const trios = this.g.cores(this.formatoAtual, texto);
    const frag = document.createDocumentFragment();
    let pos = 0;
    for (let k = 0; k + 2 < trios.length; k += 3) {
      const [ini, fim, tipo] = [trios[k] as number, trios[k + 1] as number, trios[k + 2] as number];
      if (ini < pos) continue;
      if (ini > pos) frag.append(texto.slice(pos, ini));
      const s = document.createElement('span');
      s.className = `cr-${tipo}`;
      s.textContent = texto.slice(ini, fim);
      frag.append(s);
      pos = fim;
    }
    // A quebra de linha a mais garante a mesma altura de rolagem que a caixa de texto.
    frag.append(`${texto.slice(pos)}\n`);
    this.realce.replaceChildren(frag);
    this.acompanharRolagem();
  }

  /** Marca as linhas do bloco do cursor do papel (e rola até elas se estão fora de vista). */
  marcarBloco(id: number, rolar: boolean): void {
    const b = this.mapa.texto === this.texto.value ? this.mapa.blocos.find((x) => x.id === id) : undefined;
    this.marcado = b ? { de: b.de, ate: b.ate } : null;
    this.ultimoBloco = id;
    if (b && rolar && !this.editando) {
      const topo = b.de * LINHA_PX;
      const fim = b.ate * LINHA_PX;
      const visto = this.texto.scrollTop;
      if (topo < visto || fim > visto + this.texto.clientHeight - 16) this.texto.scrollTop = Math.max(0, topo - this.texto.clientHeight / 3);
    }
    this.desenharMarca();
  }

  private numerar(): void {
    const n = this.texto.value.split('\n').length;
    if (Number(this.numeros.dataset.n) !== n) {
      this.numeros.dataset.n = String(n);
      this.numeros.textContent = Array.from({ length: n }, (_, i) => String(i + 1)).join('\n');
      this.numeros.style.whiteSpace = 'pre';
    }
    this.acompanharRolagem();
  }

  private acompanharRolagem(): void {
    this.numeros.style.transform = `translateY(${-this.texto.scrollTop}px)`;
    this.realce.style.transform = `translate(${-this.texto.scrollLeft}px, ${-this.texto.scrollTop}px)`;
    this.desenharMarca();
  }

  private desenharMarca(): void {
    const m = this.marcado;
    this.marca.hidden = !m;
    if (!m) return;
    // O padding de cima (8 px) é o mesmo da caixa de texto.
    this.marca.style.top = `${8 + m.de * LINHA_PX - this.texto.scrollTop}px`;
    this.marca.style.height = `${(m.ate - m.de) * LINHA_PX}px`;
  }

  private aoDigitar(): void {
    this.numerar();
    this.colorir();
    this.marcado = null;
    this.desenharMarca();
    clearTimeout(this.relogio);
    this.relogio = window.setTimeout(() => this.enviar(), PAUSA_MS);
  }

  private enviar(): void {
    clearTimeout(this.relogio);
    this.relogio = 0;
    this.g.aplicar(this.formatoAtual, this.texto.value);
  }

  private aoSair(): void {
    if (this.relogio) this.enviar();
  }

  /** Tab recua com dois espaços (no código, o Tab não sai da caixa). */
  private aoTeclar(e: KeyboardEvent): void {
    if (e.key !== 'Tab' || e.ctrlKey || e.altKey || e.metaKey) return;
    e.preventDefault();
    this.texto.setRangeText('  ', this.texto.selectionStart, this.texto.selectionEnd, 'end');
    this.aoDigitar();
  }

  /** O cursor do código andou: se o código é o do motor, o papel vai para o bloco daquela linha. */
  private cursorMudou(): void {
    if (this.mapa.texto !== this.texto.value) return;
    const linha = this.texto.value.slice(0, this.texto.selectionStart).split('\n').length - 1;
    const b = this.mapa.blocos.find((x) => linha >= x.de && linha < x.ate);
    if (!b || b.id === this.ultimoBloco) return;
    this.ultimoBloco = b.id;
    this.marcado = { de: b.de, ate: b.ate };
    this.desenharMarca();
    this.g.irParaBloco(b.id);
  }

  private arrastar(e: MouseEvent, divisor: HTMLElement): void {
    e.preventDefault();
    const x0 = e.clientX;
    const w0 = this.el.offsetWidth;
    divisor.classList.add('arrastando');
    const mover = (m: MouseEvent): void => {
      const max = Math.max(260, window.innerWidth - 360);
      this.el.style.width = `${Math.min(max, Math.max(220, w0 + m.clientX - x0))}px`;
    };
    const soltar = (): void => {
      divisor.classList.remove('arrastando');
      document.removeEventListener('mousemove', mover);
      document.removeEventListener('mouseup', soltar);
      try {
        localStorage.setItem('featherdocs.codigo.largura', String(this.el.offsetWidth));
      } catch {
        // sem armazenamento: a largura vale só nesta sessão
      }
    };
    document.addEventListener('mousemove', mover);
    document.addEventListener('mouseup', soltar);
  }
}

function larguraSalva(): number {
  try {
    const n = Number(localStorage.getItem('featherdocs.codigo.largura'));
    if (n >= 220) return n;
  } catch {
    // sem armazenamento
  }
  return Math.round(Math.min(560, Math.max(320, window.innerWidth * 0.36)));
}
