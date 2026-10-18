/** Aba Revisão: revisão de texto, comentários, controle de alterações, comparar e proteger. */

import { t } from '../../idiomas/index.ts';
import type { Contexto } from '../comandos-lista.ts';
import type { Grupo, Kit } from './kit.ts';

export function abaRevisao(k: Kit, _c: Contexto): Grupo[] {
  return [
    { titulo: t('faixa.g_ortografia'), itens: [k.cmd('ortografia', 'grande'), k.cmd('contagem', 'grande'), k.pilha([k.cmd('idioma')])] },
    { titulo: t('faixa.g_comentarios'), itens: [k.cmd('comentario_novo', 'grande'), k.pilha([k.cmd('comentarios_mostrar')])] },
    { titulo: t('faixa.g_alteracoes'), itens: [k.cmd('modo_edicao', 'grande'), k.pilha([k.cmd('aceitar')], [k.cmd('rejeitar')])] },
    { titulo: t('faixa.g_comparar'), itens: [k.cmd('comparar', 'grande'), k.cmd('historico', 'grande')] },
    { titulo: t('faixa.g_protecao'), itens: [k.cmd('proteger', 'grande')] },
  ];
}
