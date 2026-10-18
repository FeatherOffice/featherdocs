/** Aba contextual Tabela (aparece com o cursor numa tabela): linhas e colunas, cabeçalho, mesclar, dados. */

import { t } from '../../idiomas/index.ts';
import type { Contexto } from '../comandos-lista.ts';
import type { Grupo, Kit } from './kit.ts';

export function abaTabela(k: Kit, _c: Contexto): Grupo[] {
  return [
    {
      titulo: t('faixa.g_linhas_colunas'),
      itens: [k.cmd('tab_linha_acima', 'grande'), k.cmd('tab_linha_abaixo', 'grande'), k.cmd('tab_coluna_esq', 'grande'), k.cmd('tab_coluna_dir', 'grande'), k.cmd('tab_excluir', 'grande')],
    },
    { titulo: t('faixa.g_estilos'), itens: [k.cmd('tab_cabecalho', 'grande')] },
    { titulo: t('faixa.g_mesclar'), itens: [k.cmd('tab_mesclar', 'grande'), k.cmd('tab_dividir', 'grande')] },
    { titulo: t('faixa.g_dados'), itens: [k.cmd('tab_feathersheet', 'grande')] },
  ];
}
