/**
 * A galeria de Estilos, no jeito do Word: uma moldura com blocos todos do mesmo tamanho e uma
 * coluna de setas (▲ ▼ rolam de 4 em 4, ▾ expande). Expandida, a moldura cresce para baixo **no
 * mesmo lugar**: a primeira linha fica exatamente onde estava. Faixa e lista usam o mesmo bloco.
 */

import { t } from '../idiomas/index.ts';
import { FAMILIA_CODIGO, FAMILIA_SERIFA } from './familias.ts';
import { icon } from './icones.ts';
import { openPopup, type MenuItem } from './menu.ts';
import { Z } from './z.ts';

/** Quantos blocos cabem na moldura fechada. */
const POR_LINHA = 4;

export const ESTILOS_CSS = `
.estilos-moldura {
  display: flex; align-items: flex-start; gap: 4px; padding: 4px; border: 1px solid var(--chrome-rule); border-radius: 6px;
  background: var(--surface); box-sizing: border-box;
}
.estilos-grade { display: grid; grid-template-columns: repeat(${POR_LINHA}, 88px); gap: 4px; }
.estilo-bloco {
  display: flex; align-items: center; justify-content: center; width: 88px; height: 50px; padding: 0 6px; box-sizing: border-box;
  border: 1px solid var(--chrome-rule); border-radius: 4px; background: var(--surface); color: var(--chrome-ink);
  cursor: pointer; overflow: hidden;
}
.estilo-bloco span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.estilo-bloco:hover:not(:disabled) { border-color: var(--chrome-ink-soft); background: var(--surface); }
.estilo-bloco.on, .estilo-bloco.on:hover:not(:disabled) { border: 2px solid var(--chrome-accent); padding: 0 5px; }
.estilos-setas { display: flex; flex-direction: column; width: 16px; height: 50px; }
.estilos-setas button {
  flex: 1; width: 16px; padding: 0; border: 0; border-radius: 3px; background: transparent; color: var(--chrome-ink-soft);
  font-size: 9px; line-height: 1; cursor: pointer;
}
.estilos-setas button:hover:not(:disabled) { background: var(--chrome-hover); color: var(--chrome-ink); }
.estilos-setas button:disabled { opacity: 0.35; cursor: default; }
.estilos-novo {
  grid-column: 1 / -1; display: flex; align-items: center; gap: 6px; height: 28px; padding: 0 8px; border: 0; border-top: 1px solid var(--chrome-rule);
  background: transparent; color: var(--chrome-ink); font-size: 12.5px; cursor: pointer; text-align: left;
}
.estilos-novo:hover { background: var(--chrome-hover); }
.estilos-moldura.expandida { position: fixed; z-index: ${Z.menu + 1}; box-shadow: 0 8px 20px rgba(0, 0, 0, 0.18); max-height: calc(100vh - 16px); overflow-y: auto; }
`;

/** O visual de cada estilo (o mesmo do papel, em escala de faixa). */
export const AMOSTRAS: Record<string, string> = {
  paragrafo: 'font-size: 13px',
  titulo1: 'font-size: 18px; font-weight: 600',
  titulo2: 'font-size: 15px; font-weight: 600',
  titulo3: 'font-size: 13.5px; font-weight: 600',
  titulo4: 'font-size: 13px; font-weight: 600',
  titulo5: 'font-size: 12px; font-weight: 600',
  titulo6: 'font-size: 11px; font-weight: 600',
  citacao: 'font-size: 13px; font-style: italic',
  codigo: `font-size: 12px; font-family: ${FAMILIA_CODIGO}`,
  equacao: `font-size: 13.5px; font-style: italic; font-family: ${FAMILIA_SERIFA}`,
};

/** O nome curto que aparece no bloco. */
function nomeDoEstilo(id: string): string {
  if (id === 'paragrafo') return t('faixa.normal');
  if (id === 'codigo') return t('faixa.codigo');
  if (id === 'equacao') return t('faixa.equacao');
  return t(`tipos.${id}`);
}

/** Um estilo com nome: o nome e o CSS da amostra. */
export interface Descricao {
  nome: string;
  css: string;
}

/** Um bloco de estilo: a mesma peça na faixa e na lista expandida. O botão direito abre `menu`. */
export function blocoDeEstilo(id: string, ativo: boolean, escolher: (id: string) => void, o?: Pick<OpcoesDaGaleria, 'descrever' | 'menu'>): HTMLButtonElement {
  const b = document.createElement('button');
  b.type = 'button';
  b.className = `estilo-bloco${ativo ? ' on' : ''}`;
  const proprio = o?.descrever?.(id) ?? null;
  b.title = proprio?.nome ?? t(`tipos.${id}`);
  b.dataset.estilo = id;
  const texto = document.createElement('span');
  texto.textContent = proprio?.nome ?? nomeDoEstilo(id);
  texto.style.cssText = proprio?.css ?? AMOSTRAS[id] ?? '';
  b.append(texto);
  b.addEventListener('mousedown', (e) => e.preventDefault());
  b.addEventListener('click', () => escolher(id));
  const menu = o?.menu;
  if (menu) {
    b.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      openPopup({ x: e.clientX, y: e.clientY }, menu(id));
    });
  }
  return b;
}

function seta(rotulo: string, dica: string, ao: () => void): HTMLButtonElement {
  const b = document.createElement('button');
  b.type = 'button';
  b.textContent = rotulo;
  b.title = dica;
  b.setAttribute('aria-label', dica);
  b.addEventListener('mousedown', (e) => e.preventDefault());
  b.addEventListener('click', ao);
  return b;
}

export interface OpcoesDaGaleria {
  /** Os de fábrica e os com nome do documento (a lista muda com o documento). */
  ids(): string[];
  /** Um estilo com nome (`null` = de fábrica, que usa `AMOSTRAS`). */
  descrever?(id: string): Descricao | null;
  /** O menu do botão direito sobre o bloco. */
  menu?(id: string): MenuItem[];
  /** "Novo estilo…", no pé da lista aberta. */
  novo?(): void;
  atual(): string | undefined;
  escolher(id: string): void;
  /** Chamado a cada mudança de formato (o bloco ativo muda). */
  aoSincronizar(f: () => void): void;
}

/** A moldura da faixa: 4 blocos visíveis, ▲ ▼ rolam, ▾ expande no lugar. */
export function galeriaDeEstilos(o: OpcoesDaGaleria): HTMLElement {
  const moldura = document.createElement('div');
  moldura.className = 'estilos-moldura';
  const grade = document.createElement('div');
  grade.className = 'estilos-grade';
  let inicio = 0;
  const subir = seta('▲', t('faixa.estilos_acima'), () => mover(-POR_LINHA));
  const descer = seta('▼', t('faixa.estilos_abaixo'), () => mover(POR_LINHA));
  const expandir = seta('▾', t('faixa.mais_estilos'), () => expandirNoLugar(moldura, o));
  const setas = document.createElement('div');
  setas.className = 'estilos-setas';
  setas.append(subir, descer, expandir);
  const desenhar = (): void => {
    const ids = o.ids();
    inicio = Math.min(inicio, Math.floor(Math.max(0, ids.length - 1) / POR_LINHA) * POR_LINHA);
    grade.replaceChildren(...ids.slice(inicio, inicio + POR_LINHA).map((id) => blocoDeEstilo(id, o.atual() === id, o.escolher, o)));
    subir.disabled = inicio === 0;
    descer.disabled = inicio + POR_LINHA >= ids.length;
  };
  const mover = (d: number): void => {
    inicio = Math.max(0, Math.min(inicio + d, Math.floor((o.ids().length - 1) / POR_LINHA) * POR_LINHA));
    desenhar();
  };
  moldura.append(grade, setas);
  o.aoSincronizar(desenhar);
  desenhar();
  return moldura;
}

/** A lista aberta: fecha a que estiver aberta antes (a seta ▴ ou um novo ▾). */
let fecharAberta: (() => void) | null = null;

/** Abre a galeria inteira por cima da moldura, com a primeira linha no mesmo lugar dos blocos visíveis. */
function expandirNoLugar(moldura: HTMLElement, o: OpcoesDaGaleria): void {
  if (fecharAberta) {
    fecharAberta();
    return;
  }
  const painel = document.createElement('div');
  painel.className = 'estilos-moldura expandida';
  painel.setAttribute('role', 'listbox');
  const fechar = (): void => {
    fecharAberta = null;
    painel.remove();
    document.removeEventListener('mousedown', fora, true);
    document.removeEventListener('keydown', tecla, true);
  };
  const fora = (e: MouseEvent): void => {
    if (!painel.contains(e.target as Node)) fechar();
  };
  const tecla = (e: KeyboardEvent): void => {
    if (e.key === 'Escape') fechar();
  };
  const grade = document.createElement('div');
  grade.className = 'estilos-grade';
  const comFechar = o.menu ? (id: string): MenuItem[] => o.menu!(id).map((m) => (m.action ? { ...m, action: () => (fechar(), m.action!()) } : m)) : undefined;
  grade.append(
    ...o.ids().map((id) =>
      blocoDeEstilo(
        id,
        o.atual() === id,
        (escolhido) => {
          fechar();
          o.escolher(escolhido);
        },
        { descrever: o.descrever, menu: comFechar },
      ),
    ),
  );
  if (o.novo) {
    const novo = document.createElement('button');
    novo.type = 'button';
    novo.className = 'estilos-novo';
    novo.append(icon('estilo-novo'), document.createTextNode(t('estilo.criar')));
    novo.addEventListener('mousedown', (e) => e.preventDefault());
    novo.addEventListener('click', () => {
      fechar();
      o.novo!();
    });
    grade.append(novo);
  }
  const setas = document.createElement('div');
  setas.className = 'estilos-setas';
  setas.append(seta('▴', t('faixa.recolher_estilos'), fechar));
  painel.append(grade, setas);
  const r = moldura.getBoundingClientRect();
  painel.style.left = `${r.left}px`;
  painel.style.top = `${r.top}px`;
  painel.style.width = `${r.width}px`;
  document.body.append(painel);
  fecharAberta = fechar;
  document.addEventListener('mousedown', fora, true);
  document.addEventListener('keydown', tecla, true);
}
