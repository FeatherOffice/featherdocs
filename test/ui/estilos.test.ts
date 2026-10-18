import assert from 'node:assert/strict';
import { test } from 'node:test';
import { ESTILOS } from '../../src/ui/comandos-lista.ts';
import { AMOSTRAS } from '../../src/ui/estilos-galeria.ts';

test('todo estilo da galeria tem amostra (faixa e lista usam o mesmo bloco)', () => {
  for (const id of ESTILOS) assert.ok(AMOSTRAS[id], `sem amostra: ${id}`);
  assert.equal(Object.keys(AMOSTRAS).length, ESTILOS.length);
});

test('estilo com nome: do motor para a caixa e do cursor para um estilo novo', async () => {
  const { baseDoTipo, dadosDoEstilo, dadosDoCursor, cssDoEstilo } = await import('../../src/ui/caixa-estilo.ts');
  assert.equal(baseDoTipo('titulo3'), 3);
  assert.equal(baseDoTipo('item'), 0);
  const d = dadosDoEstilo({ id: 1, nome: 'Capa', base: 1, bits: 1 | 64, tamanho: 0, cor: 0x336699ff, alinhamento: 'centro', antes: null, depois: 24, entrelinha: 0 });
  assert.equal(d.fonte, 'serifa');
  assert.equal(d.negrito, true);
  assert.equal(d.tamanho, 24, 'tamanho 0 = o da base (Título 1)');
  assert.equal(d.cor, '#336699');
  assert.match(cssDoEstilo(d), /font-weight: 600/);
  const novo = dadosDoCursor({ estilo: 'titulo2', fonte: 'sans', negrito: false, tamanho: 18, alinhamento: 'esquerda', entrelinha: 1.15 } as never);
  assert.equal(novo.base, 2);
  assert.equal(novo.negrito, true, 'título nasce negrito como aparece');
  assert.equal(novo.entrelinha, 0, '1,15 é a padrão');
});
