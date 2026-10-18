import assert from 'node:assert/strict';
import { test } from 'node:test';
import { itemDeMenu } from '../../src/ui/comandos.ts';
import { criarRegistro, type Contexto, type GanchosDosComandos } from '../../src/ui/comandos-lista.ts';
import { hasIcon } from '../../src/ui/icones.ts';
import { IDS_DOS_MENUS } from '../../src/ui/menus-suspensos.ts';

/** Ganchos de mentira: cada chamada só é anotada. */
function contexto(): { c: Contexto; chamadas: string[] } {
  const chamadas: string[] = [];
  const g = new Proxy({} as GanchosDosComandos, {
    get: (_alvo, nome) => (nome === 'tabelaNoFeatherSheet' ? null : (...args: unknown[]) => chamadas.push(`${String(nome)}(${args.join(',')})`)),
  });
  const c: Contexto = { g, formato: () => null, estado: () => null, estrutura: () => false, regua: () => true, foco: () => false, tema: () => 'auto', codigo: () => null, continuo: () => false, varias: () => false, miniaturas: () => false };
  return { c, chamadas };
}

test('todo comando tem rótulo traduzido e ícone que existe', () => {
  const r = criarRegistro(contexto().c);
  assert.ok(r.size > 100, `registro pequeno demais: ${r.size}`);
  for (const [id, c] of r) {
    assert.ok(c.rotulo && !c.rotulo.startsWith('cmd.'), `sem rótulo: ${id}`);
    assert.ok(hasIcon(c.icone), `ícone inexistente em ${id}: ${c.icone}`);
    if (c.emBreve) assert.ok(!c.emBreve.startsWith('breve.'), `sem dica de "em breve": ${id}`);
    assert.ok(c.emBreve || c.executar || c.menu, `comando que não faz nada nem é "em breve": ${id}`);
  }
});

test('os menus suspensos só citam comandos do registro, e "em breve" sai desabilitado', () => {
  const r = criarRegistro(contexto().c);
  for (const id of IDS_DOS_MENUS) assert.ok(r.has(id), `menu cita comando inexistente: ${id}`);
  for (const [id, c] of r) {
    if (!c.emBreve) continue;
    const breve = itemDeMenu(r, id);
    assert.equal(breve.disabled?.(), true);
    assert.equal(breve.action, undefined);
    assert.ok(breve.title?.includes('Em breve') || breve.title?.includes('Coming soon'));
  }
});

test('executar um comando chama o gancho certo', () => {
  const { c, chamadas } = contexto();
  const r = criarRegistro(c);
  r.get('negrito')!.executar!();
  r.get('alinhar_centro')!.executar!();
  r.get('contagem')!.executar!();
  const sumario = r.get('sumario')!.menu!();
  sumario[1]!.action!();
  assert.deepEqual(chamadas, ['formatar(negrito)', 'alinhar(centro)', 'contagem()', 'inserirSumario(3)']);
});

test('o menu do botão direito: edição sempre, link e tabela só onde cabem', async () => {
  const { menuDeContexto, idsDoMenuDeContexto } = await import('../../src/ui/menu-contexto.ts');
  const r = criarRegistro(contexto().c);
  const simples = idsDoMenuDeContexto(null, false);
  assert.deepEqual(simples.slice(0, 4), ['recortar', 'copiar', 'colar', 'colar_sem']);
  assert.ok(!simples.includes('link_remover') && !simples.includes('tab_linha_acima'));
  const naTabela = idsDoMenuDeContexto({ link: 'https://x', tabela: 3 } as never, true);
  assert.ok(naTabela.includes('link_remover') && naTabela.includes('tab_linha_abaixo') && naTabela.includes('tab_feathersheet'));
  const itens = menuDeContexto(r, null, false);
  assert.ok(itens.every((i) => i.separator || i.label), 'todo item tem rótulo');
});
