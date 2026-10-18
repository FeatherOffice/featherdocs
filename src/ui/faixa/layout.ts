/** Aba Layout: configurar página, plano de fundo, parágrafo (recuo e espaçamento) e disposição. */

import { t } from '../../idiomas/index.ts';
import { comando, dicaDe } from '../comandos.ts';
import type { Contexto } from '../comandos-lista.ts';
import { abrirPaleta } from '../paleta.ts';
import { campoGiratorio, grupoDeCampos } from './componentes.ts';
import type { Grupo, Kit } from './kit.ts';

const PT_POR_CM = 72 / 2.54;

export function abaLayout(k: Kit, c: Contexto): Grupo[] {
  const g = c.g;
  const f = c.formato;
  const cor = comando(k.registro, 'cor_pagina');
  const corPagina = k.grande(`${cor.rotulo} ▾`, cor.icone, () => abrirPaleta(corPagina, { semCor: t('faixa.cor_branca'), aoEscolher: (x) => g.corPagina(x) }));
  corPagina.title = dicaDe(cor);
  const cm = (pt: number | undefined): number | undefined => (pt === undefined ? undefined : pt / PT_POR_CM);
  const recuos = (esq: number, dir: number): void => g.recuos(esq * PT_POR_CM, dir * PT_POR_CM);
  const campos = grupoDeCampos([
    {
      titulo: t('faixa.recuo'),
      campos: [
        campoGiratorio(k, { rotulo: t('faixa.esquerda'), icone: 'recuo-esq', unidade: 'cm', passo: 0.1, min: 0, max: 20, ler: () => cm(f()?.recuo), aplicar: (v) => recuos(v, cm(f()?.recuo_dir) ?? 0) }),
        campoGiratorio(k, { rotulo: t('faixa.direita'), icone: 'recuo-dir', unidade: 'cm', passo: 0.1, min: 0, max: 20, ler: () => cm(f()?.recuo_dir), aplicar: (v) => recuos(cm(f()?.recuo) ?? 0, v) }),
      ],
    },
    {
      titulo: t('faixa.espacamento'),
      campos: [
        campoGiratorio(k, { rotulo: t('faixa.antes'), icone: 'espaco-antes', unidade: 'pt', passo: 6, min: 0, max: 200, ler: () => f()?.antes, aplicar: (v) => g.espaco(v, f()?.depois ?? 0) }),
        campoGiratorio(k, { rotulo: t('faixa.depois'), icone: 'espaco-depois', unidade: 'pt', passo: 6, min: 0, max: 200, ler: () => f()?.depois, aplicar: (v) => g.espaco(f()?.antes ?? 0, v) }),
      ],
    },
  ]);
  return [
    {
      titulo: t('faixa.g_pagina'),
      itens: [
        k.cmd('margens', 'grande'),
        k.cmd('orientacao', 'grande'),
        k.cmd('tamanho_papel', 'grande'),
        k.cmd('colunas', 'grande'),
        k.pilha([k.cmd('quebras')], [k.cmd('numeracao_linhas')], [k.cmd('hifenizacao')]),
      ],
    },
    { titulo: t('faixa.g_fundo'), itens: [k.cmd('marca_dagua', 'grande'), corPagina, k.cmd('bordas_pagina', 'grande')] },
    { titulo: t('faixa.g_paragrafo_layout'), itens: [campos] },
    { titulo: t('faixa.g_disposicao'), itens: [k.cmd('alinhar_vertical', 'grande')] },
  ];
}
