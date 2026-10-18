/** A janela por cima do app (Configurações, Ajuda): fundo escurecido, Esc e clique fora fecham. */

import { t } from '../idiomas/index.ts';
import { Z } from './z.ts';

export const DIALOGO_CSS = `
.dialogo-fundo {
  position: fixed; inset: 0; z-index: ${Z.dialogo}; display: grid; place-items: center;
  background: rgba(0, 0, 0, 0.4);
}
.dialogo {
  width: min(640px, calc(100vw - 32px)); max-height: calc(100vh - 48px); display: flex; flex-direction: column;
  background: var(--painel); border: 1px solid var(--borda); border-radius: 10px; box-shadow: var(--sombra);
}
.dialogo-topo { display: flex; align-items: center; padding: 12px 16px; border-bottom: 1px solid var(--borda); }
.dialogo-topo h2 { flex: 1; margin: 0; font-size: 16px; font-weight: 600; }
.dialogo-corpo { padding: 16px; overflow-y: auto; }
`;

export class Dialogo {
  private fundo: HTMLElement | null = null;

  constructor(
    private readonly titulo: string,
    private readonly rotuloFechar: string,
    private readonly montar: (corpo: HTMLElement) => void,
    /** Chamado ao fechar por qualquer caminho (✕, clique fora ou `fechar()`). */
    private readonly aoFechar?: () => void,
  ) {}

  get aberto(): boolean {
    return this.fundo !== null;
  }

  abrir(): void {
    if (this.fundo) return;
    const fundo = document.createElement('div');
    fundo.className = 'dialogo-fundo';
    fundo.addEventListener('mousedown', (e) => {
      if (e.target === fundo) this.fechar();
    });
    const caixa = document.createElement('div');
    caixa.className = 'dialogo';
    caixa.setAttribute('role', 'dialog');
    caixa.setAttribute('aria-label', this.titulo);
    const topo = document.createElement('div');
    topo.className = 'dialogo-topo';
    const h = document.createElement('h2');
    h.textContent = this.titulo;
    const x = document.createElement('button');
    x.type = 'button';
    x.textContent = '✕';
    x.title = this.rotuloFechar;
    x.setAttribute('aria-label', this.rotuloFechar);
    x.addEventListener('click', () => this.fechar());
    topo.append(h, x);
    const corpo = document.createElement('div');
    corpo.className = 'dialogo-corpo';
    this.montar(corpo);
    caixa.append(topo, corpo);
    fundo.append(caixa);
    document.body.append(fundo);
    this.fundo = fundo;
    x.focus();
  }

  fechar(): void {
    if (!this.fundo) return;
    this.fundo.remove();
    this.fundo = null;
    this.aoFechar?.();
  }
}

/** Atalho para quem só precisa do rótulo padrão de fechar. */
export const rotuloFechar = (): string => t('config.fechar');
