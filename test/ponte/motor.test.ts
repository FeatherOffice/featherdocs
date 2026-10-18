import assert from 'node:assert/strict';
import { test } from 'node:test';
import { COMANDOS_DA_PONTE_ESPERADOS } from '../../src/ponte/index.ts';
import { motorNovo } from '../motor-wasm.ts';

const enc = new TextEncoder();

test('a lista de comandos da interface é a que o motor atende', () => {
  const m = motorNovo();
  assert.deepEqual([...m.comandosDaPonte()].sort(), [...COMANDOS_DA_PONTE_ESPERADOS].sort());
  assert.deepEqual(m.comandosFaltando(), []);
});

test('documento novo: uma página, o cursor no começo, limpo', () => {
  const m = motorNovo();
  const e = m.estado();
  assert.equal(e.paginas, 1);
  assert.equal(e.sujo, false);
  const v = m.visao(0, 0);
  assert.ok(v.cursor && v.cursor.x >= 72 && v.cursor.pagina === 0);
  assert.equal(v.retangulos.length, 0);
});

test('digitar muda o quadro e o cursor anda; desfazer volta tudo', () => {
  const m = motorNovo();
  const antes = m.visao(0, 0).cursor!;
  const mu = m.inserirTexto('Olá mundo');
  assert.equal(mu.afetados.length, 1);
  const depois = m.visao(0, 0).cursor!;
  assert.ok(depois.x > antes.x + 30, 'o cursor andou para a direita');
  const q = m.quadro(0, 0);
  assert.ok(q?.operacoes.some((o) => o.op === 'texto' && o.texto === 'Olá mundo'));
  assert.equal(m.quadro(0, q!.versao), null, 'versão conhecida: nada mudou');
  assert.equal(m.estado().sujo, true);
  m.desfazer();
  assert.equal(m.estado().palavras, 0);
});

test('Enter parte o parágrafo, Backspace une, Ctrl+B formata a seleção', () => {
  const m = motorNovo();
  m.inserirTexto('um dois');
  m.tecla('Home', false, false, false);
  for (let i = 0; i < 3; i++) m.tecla('ArrowRight', false, false, false);
  m.tecla('Enter', false, false, false);
  assert.equal(m.estado().blocos, 2);
  m.tecla('Backspace', false, false, false);
  assert.equal(m.estado().blocos, 1);
  m.tecla('Home', false, false, false);
  for (let i = 0; i < 2; i++) m.tecla('ArrowRight', false, true, false);
  assert.equal(m.visao(0, 0).formato.tem_selecao, true);
  assert.ok(m.visao(0, 0).retangulos.length > 0);
  m.tecla('b', true, false, false);
  assert.equal(m.visao(0, 0).formato.negrito, true);
  assert.ok(m.quadro(0, 0)?.operacoes.some((o) => o.op === 'texto' && o.texto === 'um' && (o.estilo & 1) === 1));
});

test('clicar põe o cursor onde o motor desenhou a letra, e duplo clique seleciona a palavra', () => {
  const m = motorNovo();
  m.inserirTexto('alfa beta gama');
  const g = m.visao(0, 0).cursor!;
  m.clicar(0, g.x - 10, g.y + g.h / 2);
  const v = m.visao(0, 0);
  assert.ok(v.cursor!.x < g.x && v.cursor!.x > g.x - 30);
  m.clicar(0, v.cursor!.x, g.y + g.h / 2, false, 2);
  assert.equal(m.visao(0, 0).formato.tem_selecao, true);
});

test('Ctrl+A, copiar e colar (a formatação volta quando é o texto copiado aqui)', () => {
  const m = motorNovo();
  m.inserirTexto('abc');
  m.tecla('b', true, false, false);
  m.tecla('a', true, false, false);
  const texto = m.copiar();
  assert.equal(texto, 'abc');
  m.tecla('End', true, false, false);
  m.colar(texto);
  assert.equal(m.estado().caracteres, 6);
});

test('atalhos Markdown ao digitar e estilo de parágrafo', () => {
  const m = motorNovo();
  m.inserirTexto('#');
  m.inserirTexto(' ');
  m.inserirTexto('Título');
  assert.equal(m.visao(0, 0).formato.estilo, 'titulo1');
  m.tecla('Enter', false, false, false);
  assert.equal(m.visao(0, 0).formato.estilo, 'paragrafo');
  m.paragrafoEstilo('item');
  assert.equal(m.visao(0, 0).formato.estilo, 'item');
  assert.equal(m.estrutura()[0]?.texto, 'Título');
});

test('sumário lista os títulos e o número da página sai no quadro', () => {
  const m = motorNovo();
  m.inserirBloco('sumario');
  m.inserirTexto('#');
  m.inserirTexto(' ');
  m.inserirTexto('Resultados');
  assert.equal(m.estrutura()[0]?.texto, 'Resultados');
  assert.ok(m.quadro(0, 0)?.operacoes.some((o) => o.op === 'texto' && o.texto === '1'), 'o título está na página 1');
});

test('abre Markdown, LaTeX e fdoz; erro de arquivo vem em português', () => {
  const m = motorNovo();
  assert.equal(m.abrir('md', enc.encode('# A\n\ntexto\n\n- x\n- y\n')).blocos, 4);
  assert.equal(m.abrir('tex', enc.encode('\\section{B}\nTexto $x^2$.')).blocos, 2);
  const bytes = m.gravarFdoz();
  const outro = motorNovo();
  const e = outro.abrir('fdoz', bytes);
  assert.equal(e.blocos, 2);
  assert.equal(e.sujo, false);
  assert.throws(() => outro.abrir('docx', enc.encode('lixo')), /docx/);
});

test('exporta markdown e latex', () => {
  const m = motorNovo();
  m.abrir('md', enc.encode('# Título\n\nUm **par**.\n'));
  assert.match(m.exportarTexto('md'), /^# Título/);
  // Sem numeração dos títulos, a divisão sai com estrela; com ela, numerada.
  assert.match(m.exportarTexto('tex'), /\\section\*\{Título\}/);
  m.numeracao('decimal');
  assert.match(m.exportarTexto('tex'), /\\section\{Título\}/);
});

test('tabela: o cursor entra na célula e as células vão e voltam por CSV (a ponte com o FeatherSheet)', () => {
  const m = motorNovo();
  m.inserirBloco('tabela');
  const id = m.visao(0, 0).formato.tabela;
  assert.ok(id != null, 'o cursor está numa célula');
  m.inserirTexto('nome');
  m.tecla('Tab', false, false, false);
  assert.equal(m.tabelaCsv(id!).split('\n')[0], 'nome,,');
  const mu = m.tabelaCsvSubstituir(id!, 'a;b\nação;42\n');
  assert.deepEqual(mu.afetados, [id]);
  assert.equal(m.tabelaCsv(id!), 'a,b\nação,42\n');
});

test('documento grande: digitar no fim só repinta as últimas páginas', () => {
  const m = motorNovo();
  const md = Array.from({ length: 300 }, (_, i) => `## Seção ${i}\n\n${'texto de enchimento '.repeat(40)}\n`).join('\n');
  m.abrir('md', enc.encode(md));
  assert.ok(m.estado().paginas > 50);
  m.tecla('End', true, false, false);
  const mu = m.inserirTexto('Fim.');
  assert.ok(mu.primeira_pagina > 40, `só as últimas páginas mudaram (primeira = ${mu.primeira_pagina})`);
});

test('digitar no meio de um documento de 300 páginas é rápido (o limite de um quadro é 16 ms)', () => {
  const m = motorNovo();
  const md = Array.from({ length: 300 }, (_, i) => `## Seção ${i}\n\n${'texto de enchimento '.repeat(40)}\n`).join('\n');
  m.abrir('md', enc.encode(md));
  m.tecla('End', true, false, false);
  for (let i = 0; i < 20; i++) m.tecla('ArrowUp', false, false, false);
  const inicio = performance.now();
  for (let i = 0; i < 50; i++) {
    m.inserirTexto('x');
    m.visao(0, 3);
  }
  const por = (performance.now() - inicio) / 50;
  console.log(`# digitar + visão em 300 páginas: ${por.toFixed(2)} ms por tecla`);
  assert.ok(por < 16, `${por.toFixed(2)} ms por tecla`);
});

test('fonte, tamanho, cor e realce chegam ao quadro e à barra', () => {
  const m = motorNovo();
  m.inserirTexto('Olá');
  m.selecionarTudo();
  m.tamanho(24);
  m.fonte('serifa');
  m.cor('#ff0000');
  m.realce('#ffff00');
  const v = m.visao(0, 0);
  assert.equal(v.formato.tamanho, 24);
  assert.equal(v.formato.fonte, 'serifa');
  assert.equal(v.formato.cor, '#ff0000');
  assert.equal(v.formato.realce, '#ffff00');
  const ops = m.quadro(0, 0)!.operacoes;
  const texto = ops.find((o) => o.op === 'texto' && o.texto === 'Olá');
  assert.ok(texto && texto.op === 'texto' && texto.tamanho === 24 && (texto.estilo & 64) !== 0 && texto.cor === 0xff00_00ff);
  assert.ok(ops.some((o) => o.op === 'retangulo' && o.cor === 0xffff_00ff), 'o realce vira um retângulo atrás do texto');
  m.limparFormatacao();
  assert.equal(m.visao(0, 0).formato.tamanho, 11);
  assert.throws(() => m.tamanho(999), /inválido/);
});

test('listas, tarefas e entrelinha', () => {
  const m = motorNovo();
  m.inserirTexto('comprar pão');
  m.lista('tarefa');
  assert.equal(m.visao(0, 0).formato.lista, 'tarefa');
  m.lista('romana');
  assert.equal(m.visao(0, 0).formato.lista, 'romana');
  m.lista('romana');
  assert.equal(m.visao(0, 0).formato.estilo, 'paragrafo');
  m.entrelinha(2);
  assert.equal(m.visao(0, 0).formato.entrelinha, 2);
});

test('localizar, contar, trocar um e trocar todos', () => {
  const m = motorNovo();
  m.inserirTexto('gato e Gato e gatos');
  const op = { maiusculas: false, palavra: false, regex: false };
  const r = m.buscaDefinir('gato', op);
  assert.deepEqual([r.total, r.atual], [3, 1]);
  assert.ok(m.visao(0, 0).achados.length >= 2, 'os achados à vista ganham realce');
  assert.equal(m.buscaDefinir('gato', { ...op, palavra: true }).total, 2);
  assert.ok(m.buscaDefinir('(', { ...op, regex: true }).erro);
  m.buscaDefinir('gato', op);
  const s = m.substituirTudo('cão');
  assert.equal(s.trocados, 3);
  assert.equal(m.estado().caracteres > 0, true);
  m.buscaLimpar();
  assert.equal(m.visao(0, 0).achados.length, 0);
});

test('cabeçalho, número de página, marca d’água e marcas de formatação chegam ao quadro', () => {
  const m = motorNovo();
  m.inserirTexto('a b');
  m.numeroDePagina('Página {pagina} de {total}', 'base_direita');
  m.cabecalho('Relatório', '', '');
  m.marcaDagua('RASCUNHO');
  m.marcas(true);
  const ops = m.quadro(0, 0)!.operacoes;
  const textos = ops.filter((o) => o.op === 'texto').map((o) => (o as { texto: string }).texto);
  assert.ok(textos.includes('Página 1 de 1') && textos.includes('Relatório') && textos.includes('¶'));
  assert.ok(ops.some((o) => o.op === 'girado' && o.texto === 'RASCUNHO'));
  assert.equal(m.estado().marcas, true);
  assert.equal(m.estado().extras.cabecalho.esquerda, 'Relatório');
});

test('tabela N×M, operações de tabela, link e contagem pela ponte', () => {
  const m = motorNovo();
  m.inserirTabela(2, 3);
  assert.notEqual(m.visao(0, 0).formato.tabela, null);
  m.tabelaLinha(true);
  m.tabelaColuna(true);
  m.tabelaExcluir('tabela');
  assert.equal(m.visao(0, 0).formato.tabela, null);
  m.linkDefinir('exemplo.com', 'site');
  assert.equal(m.visao(0, 0).formato.link, 'https://exemplo.com');
  assert.equal(m.contagem().palavras, 1);
});
