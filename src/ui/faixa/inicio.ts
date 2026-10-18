/** Aba Página inicial: área de transferência, fonte, parágrafo, estilos e edição. */

import { t } from '../../idiomas/index.ts';
import type { Formato } from '../../ponte/tipos.ts';
import { comando, dicaDe } from '../comandos.ts';
import { ESTILOS, type Contexto } from '../comandos-lista.ts';
import { dadosDoEstilo, cssDoEstilo } from '../caixa-estilo.ts';
import { galeriaDeEstilos, type Descricao } from '../estilos-galeria.ts';
import type { MenuItem } from '../menu.ts';
import { botaoDoPincel, itensDaFonte, itensDeFundoEBorda, itensDeLista } from '../faixa-formato.ts';
import { grandeDividido } from './componentes.ts';
import type { Grupo, Kit } from './kit.ts';

/** O id na galeria de um estilo com nome (`u3`); os de fábrica usam o tipo (`titulo1`). */
const ID_PROPRIO = /^u(\d+)$/;

/** A amostra de um estilo com nome na escala da faixa (o corpo cabe no bloco de 50 px). */
function descrever(c: Contexto, id: string): Descricao | null {
  const n = ID_PROPRIO.exec(id);
  const e = n ? c.estado()?.estilos.lista.find((x) => x.id === Number(n[1])) : undefined;
  if (!e) return null;
  const d = dadosDoEstilo(e);
  const px = Math.min(18, Math.max(11, 8 + d.tamanho * 0.42));
  return { nome: e.nome, css: `${cssDoEstilo(d)}; font-size: ${px.toFixed(1)}px` };
}

/** A galeria de estilos (moldura com setas; ▾ expande no lugar), com os estilos com nome do documento depois dos de fábrica. */
function galeria(k: Kit, c: Contexto): HTMLElement {
  const g = c.g;
  const proprio = (id: string): number => Number(ID_PROPRIO.exec(id)?.[1] ?? 0);
  return galeriaDeEstilos({
    ids: () => [...ESTILOS, ...(c.estado()?.estilos.lista ?? []).map((e) => `u${e.id}`)],
    atual: () => {
      const f = c.formato();
      return f?.estilo_proprio ? `u${f.estilo_proprio}` : f?.estilo;
    },
    escolher: (id) => (proprio(id) ? g.estiloComNome(proprio(id)) : g.estilo(id)),
    descrever: (id) => descrever(c, id),
    menu: (id) => {
      const novo: MenuItem = { label: t('estilo.criar'), icon: 'estilo-novo', action: () => g.estiloNovo() };
      const n = proprio(id);
      if (!n) return [novo];
      return [
        { label: t('estilo.modificar'), icon: 'estilo-modificar', action: () => g.estiloModificar(n) },
        { label: t('estilo.excluir'), action: () => g.estiloExcluir(n) },
        { separator: true },
        novo,
      ];
    },
    novo: () => g.estiloNovo(),
    aoSincronizar: (f) => k.aoSincronizar(f),
  });
}

export function abaInicio(k: Kit, c: Contexto): Grupo[] {
  const g = c.g;
  const f = (): Formato | null => c.formato();
  const colar = comando(k.registro, 'colar');
  return [
    {
      titulo: t('faixa.g_transferencia'),
      itens: [
        grandeDividido(colar.rotulo, 'colar', dicaDe(colar), () => g.colar(false), () => [
          { label: t('faixa.colar'), icon: 'colar', shortcut: 'Ctrl+V', action: () => g.colar(false) },
          { label: t('cmd.colar_sem'), icon: 'colar-texto', shortcut: 'Ctrl+Shift+V', action: () => g.colar(true) },
        ]),
        k.pilha([k.cmd('recortar')], [k.cmd('copiar')], [botaoDoPincel(k, g, f)]),
      ],
    },
    { titulo: t('faixa.g_fonte'), itens: itensDaFonte(k, g, f) },
    {
      titulo: t('faixa.g_paragrafo'),
      itens: [
        k.pilha(
          [...itensDeLista(k, g, f), k.cmd('numeracao_titulos', 'icone'), k.cmd('recuo_menos', 'icone'), k.cmd('recuo_mais', 'icone'), k.cmd('marcas', 'icone')],
          [
            k.cmd('alinhar_esquerda', 'icone'),
            k.cmd('alinhar_centro', 'icone'),
            k.cmd('alinhar_direita', 'icone'),
            k.cmd('alinhar_justificado', 'icone'),
            k.cmd('entrelinha', 'icone'),
            ...itensDeFundoEBorda(k, g, f),
            k.cmd('citacao', 'icone'),
            k.cmd('direcao_texto', 'icone'),
          ],
        ),
      ],
    },
    { titulo: t('faixa.g_estilos'), itens: [galeria(k, c)] },
    { titulo: t('faixa.g_edicao'), itens: [k.pilha([k.cmd('localizar')], [k.cmd('substituir')], [k.cmd('selecionar')])] },
  ];
}
