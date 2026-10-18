/** Aba Exibir: modos, o que mostrar, zoom e tema. */

import { t } from '../../idiomas/index.ts';
import type { Contexto } from '../comandos-lista.ts';
import { zoomDeslizante } from './componentes.ts';
import type { Grupo, Kit } from './kit.ts';

export function abaExibir(k: Kit, c: Contexto, zoom: () => number): Grupo[] {
  return [
    {
      titulo: t('faixa.g_modos'),
      itens: [
        k.cmd('modo_paginas', 'grande'),
        k.cmd('modo_continuo', 'grande'),
        k.cmd('codigo_latex', 'grande'),
        k.cmd('codigo_markdown', 'grande'),
        k.cmd('modo_foco', 'grande'),
      ],
    },
    { titulo: t('faixa.g_mostrar'), itens: [k.pilha([k.cmd('regua')], [k.cmd('estrutura')], [k.cmd('marcas')]), k.pilha([k.cmd('grade')], [k.cmd('miniaturas')])] },
    {
      titulo: t('faixa.g_zoom'),
      itens: [
        k.pilha([k.cmd('zoom_menos', 'icone'), zoomDeslizante(k, zoom, (z) => c.g.zoomPara(z)), k.cmd('zoom_mais', 'icone')], [k.cmd('zoom_100'), k.cmd('zoom_varias')]),
        k.cmd('zoom_pagina', 'grande'),
        k.cmd('zoom_largura', 'grande'),
      ],
    },
    { titulo: t('faixa.g_janela'), itens: [k.cmd('tema', 'grande')] },
  ];
}
