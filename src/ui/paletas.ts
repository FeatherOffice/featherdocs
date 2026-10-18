/**
 * Esquemas de cor (copiados do FeatherSheet — regra 3 da suíte: sem import entre apps).
 *
 * Um esquema não é só o CSS: a grade é um canvas com a própria paleta, e se só
 * a interface mudasse, a planilha ficaria destoando da moldura. Por isso cada
 * esquema gera as duas coisas.
 *
 * Para não escrever quarenta valores por tema, tudo é derivado de quatro
 * cores-base. O resto sai de misturas — é o mesmo raciocínio de um tema de
 * editor de texto: fundo, tinta, um traço e um acento.
 */

export interface Base {
  /** Fundo das células e das superfícies. */
  fundo: string;
  /** Faixa do topo: título, menus, ferramentas. */
  cromo: string;
  /** Cor do texto. */
  tinta: string;
  /** Seleção, botões, links. */
  acento: string;
}

export interface Paleta {
  id: string;
  nome: string;
  claro: Base;
  escuro: Base;
}

/** Mistura duas cores hexadecimais; `t` vai de 0 (a) a 1 (b). */
export function mistura(a: string, b: string, t: number): string {
  const ler = (c: string): [number, number, number] => [
    parseInt(c.slice(1, 3), 16),
    parseInt(c.slice(3, 5), 16),
    parseInt(c.slice(5, 7), 16),
  ];
  const [r1, g1, b1] = ler(a);
  const [r2, g2, b2] = ler(b);
  const hex = (n: number) => Math.round(Math.min(255, Math.max(0, n))).toString(16).padStart(2, '0');
  return `#${hex(r1 + (r2 - r1) * t)}${hex(g1 + (g2 - g1) * t)}${hex(b1 + (b2 - b1) * t)}`;
}

/** Claridade percebida de 0 a 1, para decidir o que escrever por cima. */
/** A cor `#rrggbb` com transparência, para pintar por cima sem esconder. */
export function translucido(cor: string, alfa: number): string {
  const n = (i: number) => parseInt(cor.slice(i, i + 2), 16);
  return `rgba(${n(1)}, ${n(3)}, ${n(5)}, ${alfa})`;
}

export function claridade(cor: string): number {
  const r = parseInt(cor.slice(1, 3), 16) / 255;
  const g = parseInt(cor.slice(3, 5), 16) / 255;
  const b = parseInt(cor.slice(5, 7), 16) / 255;
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

export const PALETAS: Paleta[] = [
  {
    id: 'padrao',
    nome: 'Padrão',
    claro: { fundo: '#ffffff', cromo: '#f4f6f8', tinta: '#1f1f1f', acento: '#0b57d0' },
    escuro: { fundo: '#202124', cromo: '#21252b', tinta: '#e8eaed', acento: '#8ab4f8' },
  },
  {
    id: 'sepia',
    nome: 'Sépia',
    claro: { fundo: '#fbf4e6', cromo: '#f3e8d2', tinta: '#4a3a24', acento: '#9a6b2f' },
    escuro: { fundo: '#2a2419', cromo: '#221d14', tinta: '#eadcc2', acento: '#d9a760' },
  },
  {
    id: 'floresta',
    nome: 'Floresta',
    claro: { fundo: '#f6faf6', cromo: '#e6f0e6', tinta: '#1c2e22', acento: '#0f8c57' },
    escuro: { fundo: '#18201b', cromo: '#131a16', tinta: '#dbe8de', acento: '#5fc28d' },
  },
  {
    id: 'nord',
    nome: 'Ártico',
    claro: { fundo: '#f7f9fc', cromo: '#e7edf5', tinta: '#2e3440', acento: '#4c78a8' },
    escuro: { fundo: '#2e3440', cromo: '#272c36', tinta: '#e5e9f0', acento: '#88c0d0' },
  },
  {
    id: 'ameixa',
    nome: 'Ameixa',
    claro: { fundo: '#faf7fd', cromo: '#efe8f8', tinta: '#2e2338', acento: '#6b3fa0' },
    escuro: { fundo: '#221a2b', cromo: '#1c1523', tinta: '#e8ddf5', acento: '#c6a8ec' },
  },
  {
    id: 'contraste',
    nome: 'Alto contraste',
    claro: { fundo: '#ffffff', cromo: '#ffffff', tinta: '#000000', acento: '#0033cc' },
    escuro: { fundo: '#000000', cromo: '#000000', tinta: '#ffffff', acento: '#ffd400' },
  },
];

export function paletaPorId(id: string): Paleta {
  return PALETAS.find((p) => p.id === id) ?? PALETAS[0]!;
}

/**
 * As variáveis do CSS de um esquema. Tudo sai das quatro cores-base: o que é
 * "mais claro" ou "mais escuro" é uma mistura com o oposto da tinta, então o
 * mesmo cálculo serve para o tema claro e para o escuro.
 */
export function variaveisCSS(base: Base, escuro: boolean): Record<string, string> {
  const { fundo, cromo, tinta, acento } = base;
  // O lado para onde "desbotar": no claro, em direção ao branco do fundo.
  const fraco = escuro ? '#ffffff' : '#000000';
  const chapa = escuro ? '#000000' : '#ffffff';

  return {
    '--surface': fundo,
    '--chrome': cromo,
    '--chrome-bg': cromo,
    '--chrome-bg-2': mistura(cromo, fraco, 0.04),
    '--pill': mistura(cromo, fraco, 0.05),
    '--hover': escuro ? 'rgb(255 255 255 / 9%)' : 'rgb(0 0 0 / 6%)',
    '--chrome-hover': escuro ? 'rgb(255 255 255 / 9%)' : 'rgb(0 0 0 / 6%)',
    '--chrome-active': mistura(cromo, acento, 0.18),
    '--rule': mistura(fundo, tinta, 0.2),
    '--rule-soft': mistura(fundo, tinta, 0.1),
    '--chrome-rule': mistura(cromo, tinta, 0.16),
    '--ink': tinta,
    '--ink-soft': mistura(tinta, fundo, 0.25),
    '--ink-faint': mistura(tinta, fundo, 0.45),
    '--chrome-ink': tinta,
    '--chrome-ink-soft': mistura(tinta, cromo, 0.4),
    '--accent': acento,
    '--chrome-accent': acento,
    '--accent-soft': mistura(acento, fundo, escuro ? 0.7 : 0.82),
    '--accent-ink': escuro ? mistura(acento, chapa, 0.3) : mistura(acento, tinta, 0.4),
    '--tab-active': mistura(acento, fundo, escuro ? 0.75 : 0.86),
    '--tab-active-ink': escuro ? mistura(acento, chapa, 0.2) : mistura(acento, tinta, 0.3),
    '--scroll-thumb': mistura(fundo, tinta, 0.35),
  };
}
