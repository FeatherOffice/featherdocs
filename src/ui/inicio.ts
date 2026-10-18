/**
 * A página inicial: o que criar, o que abrir, e de onde se chega às Configurações e à
 * Ajuda (F1). Cobre o app inteiro sem desmontar o editor — voltar é só esconder esta camada.
 */

import { t } from '../idiomas/index.ts';
import { Z } from './z.ts';

export const INICIO_CSS = `
.inicio { position: absolute; inset: 0; z-index: ${Z.inicio}; overflow-y: auto; background: var(--fundo); }
.inicio[hidden] { display: none; }
.inicio-dentro { width: min(760px, 100%); margin: 0 auto; padding: 56px 24px 64px; }
.inicio-cabeca { display: flex; align-items: center; gap: 14px; margin-bottom: 34px; }
.inicio-marca {
  flex: none; width: 44px; height: 44px; border-radius: 10px; background: var(--acento); color: var(--painel);
  display: grid; place-items: center; font-size: 22px; font-weight: 700;
}
.inicio-textos { flex: 1; min-width: 0; }
.inicio-titulo { font-size: 26px; font-weight: 500; letter-spacing: -0.4px; }
.inicio-sub { color: var(--texto-2); }
.inicio-ferramentas { display: flex; gap: 8px; flex-wrap: wrap; }
.inicio-ferramentas button { border: 1px solid var(--borda); background: var(--painel); border-radius: 20px; padding: 7px 15px; }
.inicio-acoes { display: flex; flex-wrap: wrap; gap: 12px; margin-bottom: 40px; }
.inicio-acoes button {
  border: 1px solid var(--borda); background: var(--painel); border-radius: 10px; padding: 12px 20px; font-size: 14px;
}
.inicio-acoes button.principal { background: var(--acento); border-color: var(--acento); color: var(--painel); }
.inicio-nota { padding-top: 18px; border-top: 1px solid var(--borda); color: var(--texto-2); font-size: 12.5px; }
@media (max-width: 560px) { .inicio-cabeca { flex-wrap: wrap; } .inicio-ferramentas { width: 100%; } }
`;

export interface GanchosDoInicio {
  novo(): void;
  abrir(): void;
  /** Volta ao documento em edição. */
  continuar(): void;
  /** Se há documento para voltar (alterado, ou já visto). */
  podeContinuar(): boolean;
  configurar(): void;
}

export class PaginaInicial {
  readonly el = document.createElement('div');
  private readonly acoes = document.createElement('div');

  private readonly tituloEl = document.createElement('div');
  private readonly subEl = document.createElement('div');
  private readonly configEl: HTMLButtonElement;
  private readonly notaEl = document.createElement('p');

  constructor(private readonly g: GanchosDoInicio) {
    this.el.className = 'inicio';
    this.el.hidden = true;
    const dentro = document.createElement('div');
    dentro.className = 'inicio-dentro';

    const cabeca = document.createElement('div');
    cabeca.className = 'inicio-cabeca';
    const marca = document.createElement('div');
    marca.className = 'inicio-marca';
    marca.textContent = 'F';
    const textos = document.createElement('div');
    textos.className = 'inicio-textos';
    this.tituloEl.className = 'inicio-titulo';
    this.tituloEl.textContent = t('app.nome');
    this.subEl.className = 'inicio-sub';
    this.subEl.textContent = t('inicio.subtitulo');
    textos.append(this.tituloEl, this.subEl);

    const ferramentas = document.createElement('div');
    ferramentas.className = 'inicio-ferramentas';
    this.configEl = this.botao(t('inicio.configuracoes'), () => g.configurar());
    this.configEl.title = t('inicio.configuracoes');
    ferramentas.append(this.configEl);
    cabeca.append(marca, textos, ferramentas);

    this.acoes.className = 'inicio-acoes';
    this.notaEl.className = 'inicio-nota';
    this.notaEl.textContent = t('inicio.nota');
    dentro.append(cabeca, this.acoes, this.notaEl);
    this.el.append(dentro);
  }

  get visivel(): boolean {
    return !this.el.hidden;
  }

  mostrar(): void {
    this.acoes.replaceChildren();
    const novo = this.botao(t('inicio.novo'), () => this.g.novo());
    novo.classList.add('principal');
    const abrir = this.botao(t('inicio.abrir'), () => this.g.abrir());
    this.acoes.append(novo, abrir);
    if (this.g.podeContinuar()) this.acoes.append(this.botao(t('inicio.voltar'), () => this.g.continuar()));
    this.el.hidden = false;
    (this.acoes.firstElementChild as HTMLElement | null)?.focus();
  }

  esconder(): void {
    this.el.hidden = true;
  }

  /** Atualiza todos os textos da página inicial para o novo idioma em tempo real. */
  atualizarIdioma(): void {
    this.tituloEl.textContent = t('app.nome');
    this.subEl.textContent = t('inicio.subtitulo');
    this.configEl.textContent = t('inicio.configuracoes');
    this.configEl.title = t('inicio.configuracoes');
    this.notaEl.textContent = t('inicio.nota');
    if (this.visivel) this.mostrar();
  }

  private botao(rotulo: string, ao: () => void): HTMLButtonElement {
    const b = document.createElement('button');
    b.type = 'button';
    b.textContent = rotulo;
    b.addEventListener('click', ao);
    return b;
  }
}
