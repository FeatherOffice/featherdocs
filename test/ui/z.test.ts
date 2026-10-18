import assert from 'node:assert/strict';
import { test } from 'node:test';
import { Z } from '../../src/ui/z.ts';

test('a escala de z-index só sobe: folhas < sobreposição < barra < início < menu < diálogo < aviso', () => {
  const ordem = [Z.folhas, Z.sobreposicao, Z.barra, Z.inicio, Z.menu, Z.dialogo, Z.aviso];
  for (let i = 1; i < ordem.length; i++) assert.ok(ordem[i]! > ordem[i - 1]!);
});
