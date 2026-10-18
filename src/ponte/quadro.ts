/**
 * Lê o quadro binário de uma página (formato em `crates/featherdocs-core/src/quadro.rs`).
 * Só decodifica: nada aqui decide o que desenhar.
 */

import type { Operacao, Quadro } from './tipos.ts';

const utf8 = new TextDecoder();

/** `null` se o quadro diz "nada mudou" (a versão que a interface tem ainda vale). */
export function decodificarQuadro(bytes: Uint8Array): Quadro | null {
  if (bytes.length === 0 || bytes[0] === 0) return null;
  const v = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const versao = Number(v.getBigUint64(1, true));
  const largura = v.getFloat32(9, true);
  const altura = v.getFloat32(13, true);
  const n = v.getUint32(17, true);
  let p = 21;
  const f = (): number => {
    const x = v.getFloat32(p, true);
    p += 4;
    return x;
  };
  const u32 = (): number => {
    const x = v.getUint32(p, true);
    p += 4;
    return x;
  };
  const texto = (): string => {
    const len = v.getUint16(p, true);
    p += 2;
    const s = utf8.decode(bytes.subarray(p, p + len));
    p += len;
    return s;
  };
  const operacoes: Operacao[] = [];
  for (let i = 0; i < n; i++) {
    const codigo = v.getUint8(p++);
    if (codigo === 1) {
      const x = f();
      const y = f();
      const tamanho = f();
      const estilo = v.getUint8(p++);
      const cor = u32();
      operacoes.push({ op: 'texto', x, y, tamanho, estilo, cor, texto: texto() });
    } else if (codigo === 2) {
      operacoes.push({ op: 'retangulo', x: f(), y: f(), w: f(), h: f(), cor: u32() });
    } else if (codigo === 3) {
      operacoes.push({ op: 'traco', x1: f(), y1: f(), x2: f(), y2: f(), espessura: f(), cor: u32() });
    } else if (codigo === 4) {
      operacoes.push({ op: 'imagem', x: f(), y: f(), w: f(), h: f(), origem: texto() });
    } else if (codigo === 5) {
      const x = f();
      const y = f();
      const tamanho = f();
      const angulo = f();
      const estilo = v.getUint8(p++);
      const cor = u32();
      operacoes.push({ op: 'girado', x, y, tamanho, angulo, estilo, cor, texto: texto() });
    } else {
      throw new Error(`Quadro ilegível: operação ${codigo} desconhecida.`);
    }
  }
  return { versao, largura, altura, operacoes };
}
