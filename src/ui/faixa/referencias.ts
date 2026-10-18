/** Aba Referências: sumário, notas, citações e bibliografia, legendas e índice remissivo. */

import { t } from '../../idiomas/index.ts';
import type { Contexto } from '../comandos-lista.ts';
import type { Grupo, Kit } from './kit.ts';

export function abaReferencias(k: Kit, _c: Contexto): Grupo[] {
  return [
    { titulo: t('faixa.g_sumario'), itens: [k.cmd('sumario', 'grande'), k.pilha([k.cmd('adicionar_texto')], [k.cmd('numeracao_titulos')], [k.cmd('sumario_personalizado')])] },
    { titulo: t('faixa.g_notas'), itens: [k.cmd('nota_rodape', 'grande'), k.pilha([k.cmd('nota_fim')], [k.cmd('formato_notas')])] },
    { titulo: t('faixa.g_citacoes'), itens: [k.cmd('citacao_inserir', 'grande'), k.pilha([k.cmd('fontes')], [k.cmd('estilo_citacao')], [k.cmd('bibliografia')])] },
    { titulo: t('faixa.g_legendas'), itens: [k.cmd('legenda', 'grande'), k.pilha([k.cmd('lista_figuras')], [k.cmd('referencia_cruzada')])] },
    { titulo: t('faixa.g_indice'), itens: [k.cmd('marcar_entrada', 'grande'), k.pilha([k.cmd('indice')])] },
  ];
}
