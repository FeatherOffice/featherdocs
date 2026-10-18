import assert from 'node:assert/strict';
import { test } from 'node:test';
import { decodificarQuadro } from '../../src/ponte/quadro.ts';
import { CODIGO, NEGRITO } from '../../src/ponte/tipos.ts';
import { corCss, fonteCss } from '../../src/ui/pintor.ts';
import { motorNovo } from '../motor-wasm.ts';

test('cor 0xRRGGBBAA vira rgba()', () => {
  assert.equal(corCss(0x1a56c4ff), 'rgba(26, 86, 196, 1.000)');
  assert.equal(corCss(0xffffff80), 'rgba(255, 255, 255, 0.502)');
});

test('fonte: negrito e código', () => {
  assert.match(fonteCss(NEGRITO, 11), /^bold 11px "FeatherDocs Sans"/);
  assert.match(fonteCss(CODIGO, 9.5), /FeatherDocs Mono/);
  assert.doesNotMatch(fonteCss(CODIGO | NEGRITO, 9.5), /bold/);
});

test('o quadro do motor decodifica inteiro, sem sobra de bytes', () => {
  const m = motorNovo();
  m.abrir('md', new TextEncoder().encode('# T\n\n- a\n\n| x | y |\n|---|---|\n| 1 | 2 |\n\n```\ncod\n```\n\n![f](a.png)\n'));
  for (let p = 0; p < m.estado().paginas; p++) {
    const q = m.quadro(p, 0);
    assert.ok(q && q.operacoes.length > 0);
    assert.ok(q.operacoes.some((o) => o.op === 'texto' && /^\d+$/.test(o.texto)), 'o número da página está no rodapé');
  }
});

test('quadro corrompido dá erro, não lixo', () => {
  const bytes = new Uint8Array(40);
  bytes[0] = 1;
  new DataView(bytes.buffer).setUint32(17, 1, true);
  bytes[21] = 99;
  assert.throws(() => decodificarQuadro(bytes), /desconhecida/);
});
