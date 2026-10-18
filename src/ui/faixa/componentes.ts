/**
 * Peças da faixa que não são um botão simples: campo numérico com setas (recuo, espaçamento),
 * grade de campos, botão grande dividido (Colar), seletor visual de tabela, grade de símbolos e
 * zoom deslizante. Só coletam o que o usuário quer; quem faz é o motor, pelos ganchos.
 */

import { formatar, t } from '../../idiomas/index.ts';
import { icon } from '../icones.ts';
import { openPopup, type MenuItem } from '../menu.ts';
import { Z } from '../z.ts';
import type { Kit } from './kit.ts';

export const COMPONENTES_CSS = `
.campos { display: grid; grid-template-columns: auto auto; column-gap: 18px; row-gap: 2px; align-items: center; padding-top: 2px; }
.campos-titulo { font-size: 11.5px; font-weight: 600; color: var(--chrome-ink-soft); padding: 0 0 2px 2px; }
.giro { display: grid; grid-template-columns: 18px 64px auto; align-items: center; gap: 6px; height: 26px; font-size: 12.5px; color: var(--chrome-ink); }
.giro svg { color: var(--chrome-ink-soft); }
.giro-caixa { display: inline-flex; align-items: stretch; height: 24px; border: 1px solid var(--chrome-rule); border-radius: 5px; background: var(--surface); overflow: hidden; }
.giro-caixa:focus-within { border-color: var(--chrome-accent); }
.giro-caixa input { width: 64px; border: 0; background: transparent; color: var(--chrome-ink); font-size: 12.5px; padding: 0 6px; text-align: right; outline: none; }
.giro-setas { display: flex; flex-direction: column; border-left: 1px solid var(--chrome-rule); }
.giro-setas button { flex: 1; width: 18px; padding: 0; border: 0; background: transparent; color: var(--chrome-ink-soft); font-size: 8px; line-height: 1; cursor: pointer; }
.giro-setas button:hover { background: var(--chrome-hover); color: var(--chrome-ink); }
.faixa-grande-dividido { display: flex; flex-direction: column; align-items: stretch; width: 56px; height: 64px; border-radius: 6px; }
.faixa-grande-dividido > button { border: 0; background: transparent; color: var(--chrome-ink); cursor: pointer; border-radius: 6px; padding: 0; }
.faixa-grande-dividido > button:hover { background: var(--chrome-hover); }
.faixa-grande-dividido .topo-div { flex: 1; display: grid; place-items: center; }
.faixa-grande-dividido .topo-div svg { width: 24px; height: 24px; color: var(--chrome-accent); }
.faixa-grande-dividido .base-div { height: 26px; font-size: 12px; line-height: 1.1; }
.seletor-tabela, .simbolos {
  position: fixed; z-index: ${Z.menu + 1}; padding: 8px; border: 1px solid var(--chrome-rule); border-radius: 8px;
  background: var(--surface); color: var(--chrome-ink); box-shadow: 0 6px 24px rgba(0, 0, 0, 0.22); font-size: 12.5px;
}
.seletor-tabela .titulo-sel { padding: 2px 2px 6px; color: var(--chrome-ink-soft); }
.seletor-tabela .grade-sel { display: grid; grid-template-columns: repeat(10, 18px); gap: 2px; }
.seletor-tabela .cel { width: 18px; height: 18px; border: 1px solid var(--chrome-rule); border-radius: 2px; background: var(--surface); cursor: pointer; padding: 0; }
.seletor-tabela .cel.on { border-color: var(--chrome-accent); background: var(--chrome-active); }
.seletor-tabela .mais-sel { display: flex; align-items: center; gap: 8px; width: 100%; margin-top: 8px; padding: 5px 4px; border: 0; border-top: 1px solid var(--chrome-rule); background: transparent; color: var(--chrome-ink); cursor: pointer; font-size: 12.5px; }
.seletor-tabela .mais-sel:hover { background: var(--chrome-hover); }
.simbolos { width: 340px; }
.simbolos input { width: 100%; box-sizing: border-box; padding: 4px 8px; margin-bottom: 6px; border: 1px solid var(--chrome-rule); border-radius: 5px; background: var(--surface); color: var(--chrome-ink); }
.simbolos .abas-sim { display: flex; flex-wrap: wrap; gap: 4px; margin-bottom: 6px; }
.simbolos .abas-sim button { padding: 2px 8px; border: 1px solid var(--chrome-rule); border-radius: 12px; background: transparent; color: var(--chrome-ink-soft); cursor: pointer; font-size: 12px; }
.simbolos .abas-sim button.on { border-color: var(--chrome-accent); color: var(--chrome-accent); }
.simbolos .grade-sim { display: grid; grid-template-columns: repeat(10, 30px); gap: 2px; max-height: 200px; overflow-y: auto; }
.simbolos .grade-sim button { width: 30px; height: 30px; padding: 0; border: 1px solid transparent; border-radius: 4px; background: transparent; color: var(--chrome-ink); font-size: 17px; cursor: pointer; }
.simbolos .grade-sim button:hover { border-color: var(--chrome-accent); background: var(--chrome-hover); }
.zoom-desliza { display: inline-flex; align-items: center; gap: 6px; }
.zoom-desliza input[type=range] { width: 120px; accent-color: var(--chrome-accent); }
`;

interface OpcoesDoGiro {
  rotulo: string;
  icone: string;
  unidade: 'cm' | 'pt';
  passo: number;
  min: number;
  max: number;
  ler(): number | undefined;
  aplicar(v: number): void;
}

const formatarNumero = (v: number): string => String(Math.round(v * 100) / 100).replace('.', ',');

/** Campo numérico com rótulo, ícone, unidade e setas ▲▼ (como o grupo Parágrafo do Layout do Word). */
export function campoGiratorio(kit: Kit, o: OpcoesDoGiro): HTMLElement {
  const linha = document.createElement('label');
  linha.className = 'giro';
  linha.title = o.rotulo;
  const rotulo = document.createElement('span');
  rotulo.textContent = o.rotulo;
  const caixa = document.createElement('span');
  caixa.className = 'giro-caixa';
  const campo = document.createElement('input');
  campo.type = 'text';
  campo.inputMode = 'decimal';
  campo.setAttribute('aria-label', o.rotulo);
  const mostrar = (): void => {
    const v = o.ler();
    if (v !== undefined && document.activeElement !== campo) campo.value = `${formatarNumero(v)} ${o.unidade}`;
  };
  const valor = (): number => Number.parseFloat(campo.value.replace(',', '.'));
  const aplicar = (v: number): void => {
    if (!Number.isFinite(v)) return mostrar();
    o.aplicar(Math.min(o.max, Math.max(o.min, Math.round(v / o.passo) * o.passo)));
  };
  campo.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      aplicar(valor());
      campo.blur();
    } else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
      e.preventDefault();
      aplicar((o.ler() ?? 0) + (e.key === 'ArrowUp' ? o.passo : -o.passo));
    } else if (e.key === 'Escape') {
      campo.blur();
    }
  });
  campo.addEventListener('focus', () => campo.select());
  campo.addEventListener('blur', mostrar);
  const setas = document.createElement('span');
  setas.className = 'giro-setas';
  for (const [rot, d] of [['▲', 1], ['▼', -1]] as const) {
    const b = document.createElement('button');
    b.type = 'button';
    b.textContent = rot;
    b.tabIndex = -1;
    b.addEventListener('mousedown', (e) => e.preventDefault());
    b.addEventListener('click', () => aplicar((o.ler() ?? 0) + d * o.passo));
    setas.append(b);
  }
  caixa.append(campo, setas);
  linha.append(icon(o.icone), rotulo, caixa);
  // A grade é: ícone, rótulo, caixa — o rótulo fica na 2ª coluna, a caixa na 3ª.
  linha.style.gridTemplateColumns = '18px 62px auto';
  kit.aoSincronizar(mostrar);
  return linha;
}

/** Colunas de campos com título em cima (Recuo · Espaçamento). */
export function grupoDeCampos(colunas: Array<{ titulo: string; campos: HTMLElement[] }>): HTMLElement {
  const g = document.createElement('div');
  g.className = 'campos';
  g.style.gridTemplateColumns = `repeat(${colunas.length}, auto)`;
  const linhas = Math.max(...colunas.map((c) => c.campos.length));
  for (const c of colunas) {
    const h = document.createElement('div');
    h.className = 'campos-titulo';
    h.textContent = c.titulo;
    g.append(h);
  }
  for (let i = 0; i < linhas; i++) {
    for (const c of colunas) g.append(c.campos[i] ?? document.createElement('span'));
  }
  return g;
}

/** Botão grande dividido: em cima faz a ação, embaixo (rótulo ▾) abre as opções (o Colar do Word). */
export function grandeDividido(rotulo: string, nome: string, dica: string, ao: () => void, opcoes: () => MenuItem[]): HTMLElement {
  const caixa = document.createElement('div');
  caixa.className = 'faixa-grande-dividido';
  const topo = document.createElement('button');
  topo.type = 'button';
  topo.className = 'topo-div';
  topo.title = dica;
  topo.setAttribute('aria-label', rotulo);
  topo.append(icon(nome));
  topo.addEventListener('mousedown', (e) => e.preventDefault());
  topo.addEventListener('click', ao);
  const base = document.createElement('button');
  base.type = 'button';
  base.className = 'base-div';
  base.title = dica;
  base.textContent = `${rotulo} ▾`;
  base.addEventListener('mousedown', (e) => e.preventDefault());
  base.addEventListener('click', () => openPopup(base, opcoes()));
  caixa.append(topo, base);
  return caixa;
}

/** Um painel flutuante junto de `ancora` que fecha ao clicar fora ou com Esc. */
function flutuante(classe: string, ancora: HTMLElement): { el: HTMLElement; fechar: () => void; posicionar: () => void } {
  const el = document.createElement('div');
  el.className = classe;
  const fechar = (): void => {
    el.remove();
    document.removeEventListener('mousedown', fora, true);
    document.removeEventListener('keydown', tecla, true);
  };
  const fora = (e: MouseEvent): void => {
    if (!el.contains(e.target as Node) && !ancora.contains(e.target as Node)) fechar();
  };
  const tecla = (e: KeyboardEvent): void => {
    if (e.key === 'Escape') fechar();
  };
  const posicionar = (): void => {
    document.body.append(el);
    const r = ancora.getBoundingClientRect();
    el.style.left = `${Math.max(8, Math.min(r.left, window.innerWidth - el.offsetWidth - 8))}px`;
    el.style.top = `${Math.min(r.bottom + 4, Math.max(8, window.innerHeight - el.offsetHeight - 8))}px`;
    document.addEventListener('mousedown', fora, true);
    document.addEventListener('keydown', tecla, true);
  };
  return { el, fechar, posicionar };
}

/** A matriz 10 × 8: passar o mouse destaca, clicar insere; embaixo, "Inserir tabela…". */
export function abrirSeletorDeTabela(ancora: HTMLElement, aoEscolher: (linhas: number, colunas: number) => void, maisOpcoes: () => void): void {
  const { el, fechar, posicionar } = flutuante('seletor-tabela', ancora);
  const titulo = document.createElement('div');
  titulo.className = 'titulo-sel';
  titulo.textContent = t('cmd.tabela');
  const grade = document.createElement('div');
  grade.className = 'grade-sel';
  const celulas: HTMLButtonElement[] = [];
  for (let l = 1; l <= 8; l++) {
    for (let c = 1; c <= 10; c++) {
      const b = document.createElement('button');
      b.type = 'button';
      b.className = 'cel';
      b.setAttribute('aria-label', formatar(t('faixa.tabela_grade'), { l, c }));
      b.addEventListener('mouseenter', () => {
        titulo.textContent = formatar(t('faixa.tabela_grade'), { l, c });
        celulas.forEach((x, i) => x.classList.toggle('on', Math.floor(i / 10) < l && (i % 10) < c));
      });
      b.addEventListener('mousedown', (e) => e.preventDefault());
      b.addEventListener('click', () => {
        fechar();
        aoEscolher(l, c);
      });
      celulas.push(b);
      grade.append(b);
    }
  }
  const mais = document.createElement('button');
  mais.type = 'button';
  mais.className = 'mais-sel';
  mais.append(icon('tabela'), document.createTextNode(t('cmd.tabela_dialogo')));
  mais.addEventListener('click', () => {
    fechar();
    maisOpcoes();
  });
  el.append(titulo, grade, mais);
  posicionar();
}

/** Os símbolos, por categoria, com o nome para a busca. */
const SIMBOLOS: Record<string, Array<[string, string]>> = {
  sim_setas: [['←', 'seta esquerda'], ['→', 'seta direita'], ['↑', 'seta cima'], ['↓', 'seta baixo'], ['↔', 'seta dupla'], ['↕', 'seta vertical'], ['⇐', 'implica esquerda'], ['⇒', 'implica'], ['⇔', 'equivale'], ['↗', 'nordeste'], ['↘', 'sudeste'], ['↙', 'sudoeste'], ['↖', 'noroeste'], ['➔', 'seta grossa'], ['↺', 'anti-horário'], ['↻', 'horário']],
  sim_gregas: [['α', 'alfa'], ['β', 'beta'], ['γ', 'gama'], ['δ', 'delta'], ['ε', 'epsilon'], ['ζ', 'zeta'], ['η', 'eta'], ['θ', 'teta'], ['λ', 'lambda'], ['μ', 'mi'], ['π', 'pi'], ['ρ', 'rô'], ['σ', 'sigma'], ['τ', 'tau'], ['φ', 'fi'], ['χ', 'qui'], ['ψ', 'psi'], ['ω', 'ômega'], ['Γ', 'gama maiúsculo'], ['Δ', 'delta maiúsculo'], ['Θ', 'teta maiúsculo'], ['Λ', 'lambda maiúsculo'], ['Π', 'pi maiúsculo'], ['Σ', 'sigma maiúsculo'], ['Φ', 'fi maiúsculo'], ['Ψ', 'psi maiúsculo'], ['Ω', 'ômega maiúsculo']],
  sim_moedas: [['R$', 'real'], ['$', 'dólar'], ['€', 'euro'], ['£', 'libra'], ['¥', 'iene'], ['₹', 'rupia'], ['₿', 'bitcoin'], ['¢', 'centavo'], ['₽', 'rublo'], ['₩', 'won']],
  sim_matematicos: [['±', 'mais ou menos'], ['×', 'vezes'], ['÷', 'dividido'], ['≠', 'diferente'], ['≈', 'aproximadamente'], ['≤', 'menor ou igual'], ['≥', 'maior ou igual'], ['∞', 'infinito'], ['√', 'raiz'], ['∑', 'somatório'], ['∏', 'produtório'], ['∫', 'integral'], ['∂', 'derivada parcial'], ['∆', 'incremento'], ['∇', 'nabla'], ['∈', 'pertence'], ['∉', 'não pertence'], ['⊂', 'contido'], ['∪', 'união'], ['∩', 'interseção'], ['∀', 'para todo'], ['∃', 'existe'], ['¬', 'não'], ['∧', 'e'], ['∨', 'ou'], ['°', 'grau'], ['‰', 'por mil'], ['²', 'ao quadrado'], ['³', 'ao cubo'], ['½', 'meio'], ['¼', 'um quarto'], ['¾', 'três quartos']],
  sim_outros: [['©', 'copyright'], ['®', 'registrado'], ['™', 'marca'], ['§', 'parágrafo'], ['¶', 'pilcrow'], ['†', 'cruz'], ['•', 'marcador'], ['…', 'reticências'], ['—', 'travessão'], ['–', 'meia-risca'], ['«', 'aspas angulares'], ['»', 'aspas angulares fecha'], ['“', 'aspas'], ['”', 'aspas fecha'], ['✓', 'visto'], ['✗', 'xis'], ['★', 'estrela'], ['☆', 'estrela vazia'], ['♥', 'coração'], ['☺', 'sorriso'], ['№', 'número'], ['ª', 'ordinal feminino'], ['º', 'ordinal masculino']],
};

/** A grade de símbolos com abas de categoria e busca por nome. */
export function abrirSimbolos(ancora: HTMLElement, aoEscolher: (s: string) => void): void {
  const { el, fechar, posicionar } = flutuante('simbolos', ancora);
  const busca = document.createElement('input');
  busca.type = 'search';
  busca.placeholder = t('faixa.simbolos_busca');
  const abas = document.createElement('div');
  abas.className = 'abas-sim';
  const grade = document.createElement('div');
  grade.className = 'grade-sim';
  let categoria = 'sim_matematicos';
  const desenhar = (): void => {
    const q = busca.value.trim().toLowerCase();
    const lista = q ? Object.values(SIMBOLOS).flat().filter(([s, nome]) => nome.includes(q) || s === q) : SIMBOLOS[categoria] ?? [];
    grade.replaceChildren(
      ...lista.map(([s, nome]) => {
        const b = document.createElement('button');
        b.type = 'button';
        b.textContent = s;
        b.title = nome;
        b.addEventListener('mousedown', (e) => e.preventDefault());
        b.addEventListener('click', () => {
          fechar();
          aoEscolher(s);
        });
        return b;
      }),
    );
    for (const b of abas.children) b.classList.toggle('on', !q && (b as HTMLElement).dataset.cat === categoria);
  };
  for (const cat of Object.keys(SIMBOLOS)) {
    const b = document.createElement('button');
    b.type = 'button';
    b.dataset.cat = cat;
    b.textContent = t(`faixa.${cat}`);
    b.addEventListener('click', () => {
      categoria = cat;
      busca.value = '';
      desenhar();
    });
    abas.append(b);
  }
  busca.addEventListener('input', desenhar);
  el.append(busca, abas, grade);
  desenhar();
  posicionar();
  busca.focus();
}

/** O zoom deslizante (50–200 %) com o valor ao lado. */
export function zoomDeslizante(kit: Kit, ler: () => number, aplicar: (z: number) => void): HTMLElement {
  const caixa = document.createElement('span');
  caixa.className = 'zoom-desliza';
  const r = document.createElement('input');
  r.type = 'range';
  r.min = '50';
  r.max = '200';
  r.step = '5';
  r.setAttribute('aria-label', t('faixa.zoom_deslizante'));
  const valor = document.createElement('span');
  valor.className = 'zoom-valor';
  r.addEventListener('input', () => {
    valor.textContent = `${r.value}%`;
    aplicar(Number(r.value) / 100);
  });
  kit.aoSincronizar(() => {
    const z = Math.round(ler() * 100);
    if (document.activeElement !== r) r.value = String(z);
    valor.textContent = `${z}%`;
  });
  caixa.append(r, valor);
  return caixa;
}
