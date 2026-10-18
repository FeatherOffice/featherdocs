/**
 * A paleta de cores dos botões Cor da fonte, Realce e Sombreamento: cores do tema (cinzas e
 * tons), cores padrão, "automática", e "Mais cores" com hexadecimal, seletor do sistema e
 * conta-gotas (onde o navegador tem). Só escolhe: a cor vai ao motor como `#rrggbb`.
 */

import { t } from '../idiomas/index.ts';
import { Z } from './z.ts';

export const PALETA_CSS = `
.paleta {
  position: fixed; z-index: ${Z.menu + 1}; padding: 8px; border: 1px solid var(--chrome-rule); border-radius: 8px;
  background: var(--surface); box-shadow: 0 6px 24px rgba(0, 0, 0, 0.22); width: 208px; color: var(--chrome-ink); font-size: 12px;
}
.paleta-grade { display: grid; grid-template-columns: repeat(10, 18px); gap: 2px; margin-bottom: 6px; }
.paleta-cor { width: 18px; height: 18px; padding: 0; border: 1px solid rgba(128, 128, 128, 0.35); border-radius: 3px; cursor: pointer; }
.paleta-cor:hover { outline: 2px solid var(--chrome-accent); outline-offset: 1px; }
.paleta-linha { display: flex; gap: 6px; align-items: center; margin-top: 4px; }
.paleta-botao { flex: 1; padding: 4px 6px; border: 1px solid var(--chrome-rule); border-radius: 5px; background: transparent; color: var(--chrome-ink); cursor: pointer; font-size: 12px; }
.paleta-botao:hover { background: var(--chrome-hover); }
.paleta-hex { flex: 1; min-width: 0; padding: 3px 6px; border: 1px solid var(--chrome-rule); border-radius: 5px; background: var(--surface); color: var(--chrome-ink); font-size: 12px; }
.paleta input[type=color] { width: 28px; height: 24px; padding: 0; border: 1px solid var(--chrome-rule); border-radius: 4px; background: transparent; }
`;

const CINZAS = ['#000000', '#434343', '#666666', '#999999', '#b7b7b7', '#cccccc', '#d9d9d9', '#efefef', '#f3f3f3', '#ffffff'];
const PADRAO = ['#980000', '#ff0000', '#ff9900', '#ffff00', '#00ff00', '#00ffff', '#4a86e8', '#0000ff', '#9900ff', '#ff00ff'];
const TONS = [
  ['#e6b8af', '#f4cccc', '#fce5cd', '#fff2cc', '#d9ead3', '#d0e0e3', '#c9daf8', '#cfe2f3', '#d9d2e9', '#ead1dc'],
  ['#cc4125', '#e06666', '#f6b26b', '#ffd966', '#93c47d', '#76a5af', '#6d9eeb', '#6fa8dc', '#8e7cc3', '#c27ba0'],
  ['#85200c', '#990000', '#b45f06', '#bf9000', '#38761d', '#134f5c', '#1155cc', '#0b5394', '#351c75', '#741b47'],
];

interface OpcoesDaPaleta {
  /** O rótulo do botão que escolhe "sem cor" (Automática, Sem realce). */
  semCor: string;
  aoEscolher(cor: string): void;
}

function amostra(cor: string, ao: (c: string) => void): HTMLButtonElement {
  const b = document.createElement('button');
  b.type = 'button';
  b.className = 'paleta-cor';
  b.style.background = cor;
  b.title = cor;
  b.setAttribute('aria-label', cor);
  b.addEventListener('click', () => ao(cor));
  return b;
}

/** Abre a paleta junto de `ancora`; fecha ao escolher, ao clicar fora ou com Esc. */
export function abrirPaleta(ancora: HTMLElement, o: OpcoesDaPaleta): void {
  const painel = document.createElement('div');
  painel.className = 'paleta';
  const fechar = (): void => {
    painel.remove();
    document.removeEventListener('mousedown', fora, true);
    document.removeEventListener('keydown', tecla, true);
  };
  const escolher = (cor: string): void => {
    fechar();
    o.aoEscolher(cor);
  };
  const fora = (e: MouseEvent): void => {
    if (!painel.contains(e.target as Node) && !ancora.contains(e.target as Node)) fechar();
  };
  const tecla = (e: KeyboardEvent): void => {
    if (e.key === 'Escape') fechar();
  };
  const grade = document.createElement('div');
  grade.className = 'paleta-grade';
  grade.append(...[...CINZAS, ...PADRAO, ...TONS.flat()].map((c) => amostra(c, escolher)));
  const sem = document.createElement('button');
  sem.type = 'button';
  sem.className = 'paleta-botao';
  sem.textContent = o.semCor;
  sem.addEventListener('click', () => escolher(''));
  painel.append(sem, grade, maisCores(escolher));
  painel.addEventListener('mousedown', (e) => {
    if ((e.target as HTMLElement).tagName !== 'INPUT') e.preventDefault();
  });
  document.body.append(painel);
  const caixa = ancora.getBoundingClientRect();
  painel.style.left = `${Math.max(8, Math.min(caixa.left, window.innerWidth - painel.offsetWidth - 8))}px`;
  const abaixo = caixa.bottom + 4;
  painel.style.top = `${abaixo + painel.offsetHeight > window.innerHeight - 8 ? Math.max(8, caixa.top - painel.offsetHeight - 4) : abaixo}px`;
  document.addEventListener('mousedown', fora, true);
  document.addEventListener('keydown', tecla, true);
}

/** "Mais cores": o seletor do sistema, o hexadecimal e o conta-gotas. */
function maisCores(escolher: (cor: string) => void): HTMLElement {
  const linha = document.createElement('div');
  linha.className = 'paleta-linha';
  const seletor = document.createElement('input');
  seletor.type = 'color';
  seletor.title = t('faixa.mais_cores');
  seletor.addEventListener('change', () => escolher(seletor.value));
  const hex = document.createElement('input');
  hex.type = 'text';
  hex.className = 'paleta-hex';
  hex.placeholder = t('faixa.cor_hex');
  hex.maxLength = 7;
  const aplicar = (): void => {
    const v = hex.value.trim();
    if (/^#?[0-9a-fA-F]{6}$/.test(v)) escolher(v.startsWith('#') ? v : `#${v}`);
    else hex.setCustomValidity('#rrggbb');
  };
  hex.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') aplicar();
  });
  linha.append(seletor, hex);
  const Conta = (window as unknown as { EyeDropper?: new () => { open(): Promise<{ sRGBHex: string }> } }).EyeDropper;
  if (Conta) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'paleta-botao';
    b.style.flex = 'none';
    b.textContent = '⌖';
    b.title = t('faixa.conta_gotas');
    b.addEventListener('click', () => {
      new Conta().open().then((r) => escolher(r.sRGBHex), () => undefined);
    });
    linha.append(b);
  }
  return linha;
}
