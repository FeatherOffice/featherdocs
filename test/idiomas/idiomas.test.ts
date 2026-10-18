import assert from 'node:assert/strict';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { IDIOMAS, REFERENCIA, tabelaDe, formatar } from '../../src/idiomas/index.ts';

function arquivos(pasta: string): string[] {
  return readdirSync(pasta).flatMap((n) => {
    const p = join(pasta, n);
    return statSync(p).isDirectory() ? arquivos(p) : p.endsWith('.ts') ? [p] : [];
  });
}

test('toda língua tem exatamente as chaves da referência', () => {
  const ref = Object.keys(tabelaDe(REFERENCIA)).sort();
  for (const codigo of Object.keys(IDIOMAS)) {
    const chaves = Object.keys(tabelaDe(codigo)).sort();
    assert.deepEqual(chaves, ref, `${codigo} difere da referência`);
  }
});

test('toda chave usada no código existe na referência', () => {
  const ref = tabelaDe(REFERENCIA);
  const usadas = new Set<string>();
  for (const f of arquivos(new URL('../../src', import.meta.url).pathname)) {
    for (const m of readFileSync(f, 'utf8').matchAll(/\bt\('([\w.]+)'/g)) usadas.add(m[1]!);
  }
  assert.ok(usadas.size > 10);
  for (const k of usadas) assert.ok(k in ref, `chave sem texto: ${k}`);
});

test('formatar preenche as variáveis e deixa as que faltam', () => {
  assert.equal(formatar('Página {atual} de {total}', { atual: 2, total: 9 }), 'Página 2 de 9');
  assert.equal(formatar('{x} e {y}', { x: 1 }), '1 e {y}');
});
