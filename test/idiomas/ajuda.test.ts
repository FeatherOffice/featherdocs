import assert from 'node:assert/strict';
import { test } from 'node:test';
import { GRUPOS_DE_ATALHOS, TOPICOS } from '../../src/ajuda/conteudo.ts';
import { IDIOMAS, tabelaDe } from '../../src/idiomas/index.ts';

test('todo tópico e todo atalho da Ajuda tem texto em todas as línguas', () => {
  for (const codigo of Object.keys(IDIOMAS)) {
    const tab = tabelaDe(codigo);
    for (const id of TOPICOS) {
      assert.ok(tab[`ajuda.comoUsar.${id}.titulo`], `${codigo}: falta o título de ${id}`);
      assert.ok(tab[`ajuda.comoUsar.${id}.corpo.0`], `${codigo}: falta o corpo de ${id}`);
    }
    for (const g of GRUPOS_DE_ATALHOS) {
      assert.ok(tab[`ajuda.rotulos.g_${g.id}`], `${codigo}: falta o título do grupo ${g.id}`);
      for (const a of g.atalhos) assert.ok(tab[`ajuda.teclas.${a.id}`], `${codigo}: falta o nome do atalho ${a.id}`);
    }
    for (const s of ['uso', 'atalhos']) assert.ok(tab[`ajuda.introducoes.${s}`], `${codigo}: falta a introdução de ${s}`);
  }
});
