/** Aba Inserir: páginas, tabelas, ilustrações, links, cabeçalho e rodapé, texto, símbolos e código. */

import { t } from '../../idiomas/index.ts';
import type { Contexto } from '../comandos-lista.ts';
import { abrirSeletorDeTabela, abrirSimbolos } from './componentes.ts';
import type { Grupo, Kit } from './kit.ts';

export function abaInserir(k: Kit, c: Contexto): Grupo[] {
  const g = c.g;
  // Tabela e Símbolo abrem um seletor visual (a matriz 10 × 8, a grade de símbolos), não um menu.
  const seletor = k.grande(`${t('cmd.tabela')} ▾`, 'tabela', () => abrirSeletorDeTabela(seletor, (l, col) => g.inserirTabela(l, col), () => g.tabelaDialogo()));
  seletor.title = t('cmd.tabela');
  const simbolo = k.pequeno(`${t('cmd.simbolo')} ▾`, 'simbolo', () => abrirSimbolos(simbolo, (s) => g.simbolo(s)));
  simbolo.title = t('cmd.simbolo');
  return [
    { titulo: t('faixa.g_paginas'), itens: [k.pilha([k.cmd('quebra_pagina')], [k.cmd('quebra_secao')])] },
    { titulo: t('faixa.g_tabela_ins'), itens: [seletor] },
    { titulo: t('faixa.g_ilustracoes'), itens: [k.cmd('imagem', 'grande'), k.pilha([k.cmd('formas')], [k.cmd('grafico')], [k.cmd('desenho')])] },
    { titulo: t('faixa.g_links'), itens: [k.cmd('link', 'grande'), k.pilha([k.cmd('link_remover')], [k.cmd('marcador')])] },
    { titulo: t('faixa.g_cabrod'), itens: [k.cmd('cabecalho', 'grande'), k.cmd('rodape', 'grande'), k.cmd('numero_pagina', 'grande')] },
    {
      titulo: t('faixa.g_elementos'),
      itens: [k.cmd('chamada', 'grande'), k.pilha([k.cmd('data_hora')], [k.cmd('linha_horizontal')], [k.cmd('caixa_texto')]), k.pilha([k.cmd('campo_dados')], [k.cmd('recolhivel')])],
    },
    { titulo: t('faixa.g_simbolos'), itens: [k.cmd('equacao', 'grande'), k.pilha([simbolo], [k.cmd('bloco_codigo')], [k.cmd('codigo_linha')])] },
  ];
}
