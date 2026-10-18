/**
 * Pinta o quadro de uma página num canvas. Só desenha o que o motor mandou:
 * posição, tamanho, cor e estilo de cada texto já vêm resolvidos do Rust.
 *
 * O papel é sempre branco (nos dois temas), como no Google Docs.
 */

import { CODIGO, ITALICO, NEGRITO, SERIFA, SUBLINHADO, TACHADO, type Operacao, type Quadro } from '../ponte/tipos.ts';

import { FAMILIA_CODIGO as FONTE_CODIGO, FAMILIA_SERIFA as FONTE_SERIFA, FAMILIA_TEXTO as FONTE_TEXTO } from './familias.ts';

/** `0xRRGGBBAA` (como o motor manda) para uma cor CSS. */
export function corCss(rgba: number): string {
  const n = rgba >>> 0;
  return `rgba(${(n >>> 24) & 255}, ${(n >>> 16) & 255}, ${(n >>> 8) & 255}, ${((n & 255) / 255).toFixed(3)})`;
}

/** A fonte CSS de um texto do quadro. */
export function fonteCss(estilo: number, tamanho: number): string {
  const familia = estilo & CODIGO ? FONTE_CODIGO : estilo & SERIFA ? FONTE_SERIFA : FONTE_TEXTO;
  return `${estilo & ITALICO ? 'italic ' : ''}${estilo & NEGRITO && !(estilo & CODIGO) ? 'bold ' : ''}${tamanho}px ${familia}`;
}

/**
 * A fonte é desenhada `AMPLIAR` vezes maior e reduzida pela escala: em corpo pequeno (9 px) o
 * navegador arredonda o avanço de cada letra para o pixel e o texto sai mais largo que o medido
 * pelo motor (o negrito encavala no que vem depois). Grande, o arredondamento some.
 */
const AMPLIAR = 8;

function pintarTexto(ctx: CanvasRenderingContext2D, o: Extract<Operacao, { op: 'texto' }>): void {
  ctx.save();
  ctx.translate(o.x, o.y);
  ctx.scale(1 / AMPLIAR, 1 / AMPLIAR);
  ctx.font = fonteCss(o.estilo, o.tamanho * AMPLIAR);
  ctx.fillStyle = corCss(o.cor);
  ctx.fillText(o.texto, 0, 0);
  const w = o.estilo & (SUBLINHADO | TACHADO) ? ctx.measureText(o.texto).width / AMPLIAR : 0;
  ctx.restore();
  if (o.estilo & (SUBLINHADO | TACHADO)) {
    ctx.strokeStyle = corCss(o.cor);
    ctx.lineWidth = Math.max(0.5, o.tamanho / 16);
    ctx.beginPath();
    if (o.estilo & SUBLINHADO) {
      ctx.moveTo(o.x, o.y + o.tamanho * 0.12);
      ctx.lineTo(o.x + w, o.y + o.tamanho * 0.12);
    }
    if (o.estilo & TACHADO) {
      ctx.moveTo(o.x, o.y - o.tamanho * 0.3);
      ctx.lineTo(o.x + w, o.y - o.tamanho * 0.3);
    }
    ctx.stroke();
  }
}

/** A marca d'água: texto girado em volta do próprio centro. */
function pintarGirado(ctx: CanvasRenderingContext2D, o: Extract<Operacao, { op: 'girado' }>): void {
  ctx.save();
  ctx.translate(o.x, o.y);
  ctx.rotate((-o.angulo * Math.PI) / 180);
  ctx.font = fonteCss(o.estilo, o.tamanho);
  ctx.fillStyle = corCss(o.cor);
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(o.texto, 0, 0);
  ctx.restore();
}

const cacheImagens = new Map<string, HTMLImageElement>();
let ouvinteImagemCarregada: (() => void) | null = null;

export function definirAoCarregarImagem(cb: () => void): void {
  ouvinteImagemCarregada = cb;
}

/** Desenha a imagem no canvas se já carregou, ou desenha a moldura de marcação enquanto carrega. */
function pintarImagem(ctx: CanvasRenderingContext2D, o: Extract<Operacao, { op: 'imagem' }>): void {
  if (o.origem) {
    let img = cacheImagens.get(o.origem);
    if (!img) {
      img = new Image();
      img.src = o.origem;
      img.onload = () => {
        ouvinteImagemCarregada?.();
      };
      cacheImagens.set(o.origem, img);
    }
    if (img.complete && img.naturalWidth > 0) {
      try {
        ctx.drawImage(img, o.x, o.y, o.w, o.h);
        return;
      } catch {
        // segue para o marcador caso a imagem falhe
      }
    }
  }
  ctx.fillStyle = 'rgba(128, 134, 139, 0.12)';
  ctx.fillRect(o.x, o.y, o.w, o.h);
  ctx.strokeStyle = 'rgba(128, 134, 139, 0.6)';
  ctx.lineWidth = 0.75;
  ctx.setLineDash([4, 3]);
  ctx.strokeRect(o.x, o.y, o.w, o.h);
  ctx.setLineDash([]);
  ctx.font = fonteCss(0, 10);
  ctx.fillStyle = 'rgba(95, 99, 104, 1)';
  ctx.textAlign = 'center';
  const texto = o.origem?.startsWith('data:') ? '(imagem)' : (o.origem || '(imagem)');
  ctx.fillText(texto, o.x + o.w / 2, o.y + o.h / 2);
  ctx.textAlign = 'left';
}

/** Pinta a página inteira. `escala` = pixels do canvas por ponto. */
export function pintarQuadro(ctx: CanvasRenderingContext2D, q: Quadro, escala: number): void {
  ctx.save();
  ctx.setTransform(escala, 0, 0, escala, 0, 0);
  // Sem kerning: o motor soma a largura de cada letra; o kerning do canvas deslocaria o cursor.
  ctx.fontKerning = 'none';
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, q.largura, q.altura);
  ctx.textBaseline = 'alphabetic';
  ctx.textAlign = 'left';
  for (const o of q.operacoes) {
    if (o.op === 'texto') pintarTexto(ctx, o);
    else if (o.op === 'retangulo') {
      ctx.fillStyle = corCss(o.cor);
      ctx.fillRect(o.x, o.y, o.w, o.h);
    } else if (o.op === 'traco') {
      ctx.strokeStyle = corCss(o.cor);
      ctx.lineWidth = o.espessura;
      ctx.beginPath();
      ctx.moveTo(o.x1, o.y1);
      ctx.lineTo(o.x2, o.y2);
      ctx.stroke();
    } else if (o.op === 'girado') pintarGirado(ctx, o);
    else pintarImagem(ctx, o);
  }
  ctx.restore();
}
