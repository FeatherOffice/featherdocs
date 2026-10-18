/**
 * A escala de `z-index` — a única. Nenhum componente escreve número solto:
 * `test/ui/z.test.ts` trava a ordem (cada camada acima da anterior).
 */
export const Z = {
  /** As folhas (canvas das páginas). */
  folhas: 1,
  /** Seleção e cursor, em cima da página. */
  sobreposicao: 5,
  /** A barra de ferramentas e o painel lateral. */
  barra: 10,
  /** A página inicial, por cima do editor. */
  inicio: 20,
  /** Menus e listas suspensas. */
  menu: 30,
  /** Janelas de Configurações e Ajuda. */
  dialogo: 35,
  /** Avisos (toast). */
  aviso: 40,
} as const;
