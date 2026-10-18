/** Um aviso rápido no canto da janela. */

import { Z } from './z.ts';

export const TOAST_CSS = `
.toast {
  position: fixed; left: 50%; bottom: 40px; transform: translateX(-50%); z-index: ${Z.aviso};
  background: var(--texto); color: var(--painel); padding: 8px 16px; border-radius: var(--raio);
  box-shadow: var(--sombra); max-width: 70vw; transition: opacity 0.2s;
}
.toast.erro { background: var(--perigo); color: #fff; }
`;

let atual: HTMLElement | null = null;
let relogio = 0;

/** Mostra um aviso por alguns segundos (erro fica mais). */
export function avisar(texto: string, erro = false): void {
  atual?.remove();
  clearTimeout(relogio);
  const el = document.createElement('div');
  el.className = erro ? 'toast erro' : 'toast';
  el.setAttribute('role', erro ? 'alert' : 'status');
  el.textContent = texto;
  document.body.append(el);
  atual = el;
  relogio = window.setTimeout(() => el.remove(), erro ? 6000 : 2500);
}
