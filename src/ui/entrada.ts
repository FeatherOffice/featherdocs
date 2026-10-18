/**
 * A entrada de teclado: um `<textarea>` invisível, posto junto ao cursor, que só
 * serve para o navegador entregar teclado, acento morto, IME e área de
 * transferência. Aqui nada é interpretado: cada tecla vai ao motor, que decide o
 * que ela faz com o texto.
 *
 * - texto digitado (`beforeinput`) → `inserirTexto`; acento morto e IME só entregam o
 *   texto pronto no fim da composição (`compositionend`), e enquanto compõe a interface
 *   mostra o rascunho junto ao cursor;
 * - teclas de comando (setas, Enter, Backspace, Ctrl+B…) → `tecla`; o motor diz se usou;
 * - copiar/recortar/colar: o texto vem e vai pela área de transferência do sistema.
 */

export interface GanchosDaEntrada {
  /** Uma tecla de comando; devolve se o motor a usou. */
  tecla(tecla: string, ctrl: boolean, shift: boolean, alt: boolean): boolean;
  /** Texto digitado, ou o texto pronto de uma composição. */
  texto(texto: string): void;
  /** Ctrl+C: o texto selecionado. */
  copiar(): string;
  /** Ctrl+X: o texto selecionado, já removido do documento. */
  recortar(): string;
  /** Ctrl+V. */
  colar(texto: string, semFormatacao: boolean): void;
  /** Colagem direta de imagem da área de transferência (Ctrl+V ou botão direito). */
  colarImagem?(dataUrl: string): void;
  /** O rascunho de uma composição (IME), ou `null` quando acaba. */
  composicao(rascunho: string | null): void;
  /** O foco entrou ou saiu (o cursor para de piscar e a seleção esmaece). */
  foco(tem: boolean): void;
}

export const ENTRADA_CSS = `
.entrada {
  position: absolute; left: 0; top: 0; width: 2px; height: 16px; padding: 0; margin: 0; border: 0;
  opacity: 0; resize: none; overflow: hidden; pointer-events: none; z-index: 0; caret-color: transparent;
}
`;

/** Escreve `texto` na área de transferência do sistema, pelo caminho que funciona dentro e fora do aplicativo. */
function levarParaOSistema(alvo: HTMLTextAreaElement, texto: string): void {
  alvo.value = texto;
  alvo.select();
  // O Ctrl+C/Ctrl+X padrão do navegador copia a seleção do textarea; limpa-se logo depois.
  setTimeout(() => {
    alvo.value = '';
  }, 0);
}

export class Entrada {
  readonly el = document.createElement('textarea');
  private compondo = false;
  private semFormatacao = false;

  constructor(private readonly g: GanchosDaEntrada) {
    this.el.className = 'entrada';
    this.el.setAttribute('aria-label', 'Texto do documento');
    this.el.setAttribute('autocapitalize', 'off');
    this.el.setAttribute('autocomplete', 'off');
    this.el.setAttribute('autocorrect', 'off');
    this.el.spellcheck = false;
    this.el.addEventListener('keydown', (e) => this.aoTeclar(e));
    this.el.addEventListener('beforeinput', (e) => this.antesDeInserir(e));
    this.el.addEventListener('compositionstart', () => {
      this.compondo = true;
    });
    this.el.addEventListener('compositionupdate', (e) => this.g.composicao(e.data));
    this.el.addEventListener('compositionend', (e) => this.aoFimDaComposicao(e));
    this.el.addEventListener('paste', (e) => this.aoColar(e));
    this.el.addEventListener('focus', () => this.g.foco(true));
    this.el.addEventListener('blur', () => this.g.foco(false));
  }

  focar(): void {
    this.el.focus({ preventScroll: true });
  }

  /** Põe o textarea no cursor (o candidato do IME aparece ali). `x` e `y` em pixels dentro do miolo. */
  posicionar(x: number, y: number, altura: number): void {
    this.el.style.transform = `translate(${Math.round(x)}px, ${Math.round(y)}px)`;
    this.el.style.height = `${Math.max(12, Math.round(altura))}px`;
  }

  private aoTeclar(e: KeyboardEvent): void {
    if (this.compondo || e.isComposing || e.keyCode === 229) return;
    const ctrl = e.ctrlKey || e.metaKey;
    const k = e.key.toLowerCase();
    if (ctrl && !e.altKey && !e.shiftKey && (k === 'c' || k === 'x')) {
      levarParaOSistema(this.el, k === 'c' ? this.g.copiar() : this.g.recortar());
      return;
    }
    if (ctrl && !e.altKey && k === 'v') {
      this.semFormatacao = e.shiftKey; // Ctrl+Shift+V cola sem formatação; o navegador dispara `paste`
      return;
    }
    // Texto digitável (inclui AltGr): vem por `beforeinput`.
    if (e.key.length === 1 && !ctrl) return;
    if (e.key === 'Dead' || e.key === 'Process') return;
    if (this.g.tecla(e.key, ctrl, e.shiftKey, e.altKey)) e.preventDefault();
  }

  private antesDeInserir(e: InputEvent): void {
    if (e.isComposing || this.compondo) return;
    if ((e.inputType === 'insertText' || e.inputType === 'insertReplacementText') && e.data) {
      e.preventDefault();
      this.g.texto(e.data);
    } else if (e.inputType.startsWith('insert')) {
      e.preventDefault();
    }
  }

  private aoFimDaComposicao(e: CompositionEvent): void {
    this.compondo = false;
    this.g.composicao(null);
    this.el.value = '';
    if (e.data) this.g.texto(e.data);
  }

  private aoColar(e: ClipboardEvent): void {
    const items = e.clipboardData?.items;
    if (items) {
      for (const item of items) {
        if (item.type.startsWith('image/')) {
          e.preventDefault();
          const file = item.getAsFile();
          if (file) {
            const reader = new FileReader();
            reader.onload = () => {
              const dataUrl = reader.result as string;
              if (dataUrl) this.g.colarImagem?.(dataUrl);
            };
            reader.readAsDataURL(file);
            return;
          }
        }
      }
    }
    e.preventDefault();
    const texto = e.clipboardData?.getData('text/plain') ?? '';
    const sem = this.semFormatacao;
    this.semFormatacao = false;
    if (texto) this.g.colar(texto, sem);
  }
}
