/**
 * O painel de configurações do FeatherDocs, adaptado do FeatherSheet:
 * janela modal arrastável com seções de Aparência, Salvamento, Documento,
 * Atalhos e Sobre.
 */

import { IDIOMAS, codigoAtual, definirIdioma, t } from '../idiomas/index.ts';
import { icon } from './icones.ts';
import { PALETAS } from './paletas.ts';
import {
  ACENTOS,
  acentoSalvo,
  definirAcento,
  definirModo,
  definirPaleta,
  modoSalvo,
  paletaSalva,
  fonteSalva,
  definirFonte,
  corpoSalvo,
  definirCorpo,
  zoomSalvo,
  definirZoomSalvo,
  type ModoDoTema,
} from './tema.ts';
import { avisar } from './toast.ts';
import { Z } from './z.ts';
import type { Papel } from '../ponte/tipos.ts';

export const CONFIG_CSS = String.raw`
/* Modal central, acima de tudo */
.set-backdrop {
  position: fixed;
  inset: 0;
  z-index: ${Z.dialogo};
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
  pointer-events: auto;
}

.set-backdrop[hidden] {
  display: none !important;
}

.set-modal {
  position: relative;
  display: flex;
  width: min(840px, calc(100vw - 36px));
  height: min(620px, calc(100vh - 60px));
  border-radius: 14px;
  background: var(--surface);
  box-shadow: 0 12px 40px rgb(0 0 0 / 35%);
  overflow: hidden;
  border: 1px solid var(--rule);
  pointer-events: auto;
  color: var(--ink);
}

@media (prefers-reduced-motion: reduce) {
  .set-backdrop, .set-modal { animation: none; }
}

/* Coluna das seções */
.set-nav {
  flex: none;
  width: 200px;
  padding: 18px 10px;
  border-right: 1px solid var(--rule-soft);
  background: var(--pill);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.set-nav-title {
  padding: 0 10px 14px;
  font-size: 16px;
  font-weight: 600;
  cursor: grab;
  user-select: none;
  color: var(--ink);
}

.set-nav button {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--ink-soft);
  font-size: 13.5px;
  text-align: left;
  cursor: pointer;
  min-height: 0;
}

.set-nav button:hover { background: var(--hover); color: var(--ink); }
.set-nav button[aria-current='true'] { background: var(--accent-soft); color: var(--accent-ink); font-weight: 500; }
.set-nav svg { width: 16px; height: 16px; flex: none; }

/* Conteúdo principal */
.set-main { flex: 1; min-width: 0; display: flex; flex-direction: column; }

.set-head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 20px 12px;
  border-bottom: 1px solid var(--rule-soft);
}

.set-head strong { flex: 1; font-size: 15px; font-weight: 600; cursor: grab; user-select: none; }

.set-close {
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--ink-faint);
  font-size: 15px;
  cursor: pointer;
  display: grid;
  place-items: center;
  padding: 0;
  min-height: 0;
}

.set-close:hover { background: var(--hover); color: var(--ink); }

.set-body { flex: 1; overflow-y: auto; padding: 8px 20px 24px; scrollbar-width: thin; }

.set-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 13px 0;
  border-bottom: 1px solid var(--rule-soft);
}

.set-row:last-child { border-bottom: none; }
.set-row-text { flex: 1; min-width: 0; }
.set-row-label { font-size: 13.5px; font-weight: 500; color: var(--ink); }
.set-row-hint { margin-top: 3px; color: var(--ink-faint); font-size: 12px; line-height: 1.5; }

.set-seg {
  display: inline-flex;
  flex: none;
  padding: 3px;
  border: 1px solid var(--rule);
  border-radius: 18px;
  background: var(--surface);
}

.set-seg button {
  padding: 5px 12px;
  border: none;
  border-radius: 14px;
  background: transparent;
  color: var(--ink-faint);
  font-size: 12.5px;
  white-space: nowrap;
  cursor: pointer;
  min-height: 0;
}

.set-seg button:hover { color: var(--ink); }
.set-seg button[aria-pressed='true'] { background: var(--accent); color: #fff; font-weight: 500; }

.set-switch { flex: none; position: relative; width: 40px; height: 22px; cursor: pointer; }
.set-switch input { position: absolute; opacity: 0; width: 100%; height: 100%; margin: 0; cursor: pointer; }

.set-switch span {
  position: absolute;
  inset: 0;
  border-radius: 11px;
  background: var(--rule);
  transition: background 120ms;
  pointer-events: none;
}

.set-switch span::after {
  content: '';
  position: absolute;
  top: 3px;
  left: 3px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  transition: transform 120ms;
}

.set-switch input:checked + span { background: var(--accent); }
.set-switch input:checked + span::after { transform: translateX(18px); }
.set-switch input:focus-visible + span { outline: 2px solid var(--accent); outline-offset: 2px; }

.set-cores { display: flex; gap: 8px; flex: none; align-items: center; }

.set-esquemas {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(106px, 1fr));
  gap: 8px;
  margin-top: 10px;
}

.set-esquema {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 7px;
  border: 2px solid var(--rule-soft);
  border-radius: 10px;
  background: transparent;
  color: inherit;
  cursor: pointer;
  min-height: 0;
}

.set-esquema:hover { border-color: var(--rule); }
.set-esquema[aria-pressed='true'] { border-color: var(--accent); }

.set-esquema-amostra {
  position: relative;
  display: block;
  height: 42px;
  border: 1px solid var(--rule-soft);
  border-radius: 6px;
  overflow: hidden;
}

.set-esquema-amostra i { position: absolute; inset: 0 0 auto 0; height: 13px; display: block; }

.set-esquema-amostra u {
  position: absolute;
  left: 7px;
  bottom: 7px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  display: block;
}

.set-esquema-nome { font-size: 12px; text-align: center; }

.set-cor-auto {
  background: conic-gradient(#0b57d0, #0f8c57, #b35309, #b3255e, #6b3fa0, #0b57d0);
}

.set-cor {
  width: 26px;
  height: 26px;
  border: 2px solid transparent;
  border-radius: 50%;
  cursor: pointer;
  padding: 0;
  min-height: 0;
}

.set-cor[aria-pressed='true'] { border-color: var(--ink); transform: scale(1.1); }

.set-button {
  flex: none;
  padding: 7px 15px;
  border: 1px solid var(--rule);
  border-radius: 18px;
  background: transparent;
  color: inherit;
  font-size: 12.5px;
  white-space: nowrap;
  cursor: pointer;
  min-height: 0;
}

.set-button:hover { background: var(--hover); }
.set-button.principal { background: var(--accent); border-color: var(--accent); color: #fff; }
.set-button.perigo { color: var(--perigo); border-color: var(--perigo); }
.set-button:disabled { opacity: 0.45; cursor: default; }

.set-path {
  margin-top: 6px;
  padding: 8px 10px;
  border-radius: 7px;
  background: var(--pill);
  color: var(--ink-faint);
  font-family: monospace;
  font-size: 11.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.set-nota { padding: 14px 0 0; color: var(--ink-faint); font-size: 12.5px; line-height: 1.65; }
.set-dupla { display: flex; gap: 8px; flex: none; }

.set-kbd {
  display: inline-block;
  padding: 3px 7px;
  border: 1px solid var(--rule);
  border-radius: 5px;
  background: var(--pill);
  color: var(--ink);
  font-family: inherit;
  font-size: 12px;
  box-shadow: 0 1px 2px rgba(0,0,0,0.1);
}

@media (max-width: 680px) {
  .set-modal { flex-direction: column; height: min(640px, 100%); }
  .set-nav { width: auto; flex-direction: row; overflow-x: auto; padding: 10px; border-right: none; border-bottom: 1px solid var(--rule-soft); }
  .set-nav-title { display: none; }
}
`;

export type SecaoConfig = 'aparencia' | 'salvamento' | 'documento' | 'atalhos' | 'sobre';

const SECOES: [SecaoConfig, string, string][] = [
  ['aparencia', 'Aparência', 'paleta'],
  ['salvamento', 'Salvamento', 'salvar'],
  ['documento', 'Documento', 'pagina'],
  ['atalhos', 'Atalhos', 'teclado'],
  ['sobre', 'Sobre', 'ajuda'],
];

const FONTES_BASE: [string, string][] = [
  ['sans', 'Sem serifa (Liberation Sans)'],
  ['serif', 'Com serifa (Serif)'],
  ['mono', 'Monoespaçada (Liberation Mono)'],
];

const CORPOS_PADRAO: [number, string][] = [
  [10, '10 pt'],
  [11, '11 pt'],
  [12, '12 pt'],
  [14, '14 pt'],
  [16, '16 pt'],
];

const ZOOMS_PADRAO: [number, string][] = [
  [0.75, '75%'],
  [0.9, '90%'],
  [1, '100%'],
  [1.25, '125%'],
  [1.5, '150%'],
];

const INTERVALOS_SALVAMENTO: [number, string][] = [
  [2, '2 s'],
  [5, '5 s'],
  [15, '15 s'],
  [30, '30 s'],
  [60, '1 min'],
];

const ATALHOS_FEATHERDOCS: { id: string; rotulo: string; padrao: string }[] = [
  { id: 'negrito', rotulo: 'Negrito', padrao: 'Ctrl+B' },
  { id: 'italico', rotulo: 'Itálico', padrao: 'Ctrl+I' },
  { id: 'sublinhado', rotulo: 'Sublinhado', padrao: 'Ctrl+U' },
  { id: 'tachado', rotulo: 'Tachado', padrao: 'Ctrl+Shift+X' },
  { id: 'codigo', rotulo: 'Código em linha', padrao: 'Ctrl+E' },
  { id: 'desfazer', rotulo: 'Desfazer', padrao: 'Ctrl+Z' },
  { id: 'refazer', rotulo: 'Refazer', padrao: 'Ctrl+Y' },
  { id: 'salvar', rotulo: 'Salvar', padrao: 'Ctrl+S' },
  { id: 'salvar_como', rotulo: 'Salvar como…', padrao: 'Ctrl+Shift+S' },
  { id: 'abrir', rotulo: 'Abrir documento', padrao: 'Ctrl+O' },
  { id: 'novo', rotulo: 'Novo documento', padrao: 'Ctrl+N' },
  { id: 'localizar', rotulo: 'Localizar', padrao: 'Ctrl+F' },
  { id: 'substituir', rotulo: 'Substituir', padrao: 'Ctrl+H' },
  { id: 'pesquisar_comando', rotulo: 'Pesquisar comando', padrao: 'Ctrl+/' },
  { id: 'quebra_pagina', rotulo: 'Quebra de página', padrao: 'Ctrl+Enter' },
  { id: 'faixa', rotulo: 'Alternar faixa de opções', padrao: 'Ctrl+F1' },
  { id: 'ajuda', rotulo: 'Ajuda', padrao: 'F1' },
];

export interface ConfiguracoesAcoes {
  // Aparência
  aoMudarTema?: () => void;
  tema?: () => ModoDoTema;
  setTema?: (m: ModoDoTema) => void;
  paleta?: () => string;
  setPaleta?: (id: string) => void;
  acento?: () => number;
  setAcento?: (i: number) => void;
  fonte?: () => string;
  setFonte?: (f: string) => void;
  corpo?: () => number;
  setCorpo?: (n: number) => void;
  zoom?: () => number;
  setZoom?: (z: number) => void;

  // Salvamento
  noDisco?: () => boolean;
  arquivoAberto?: () => string;
  salvarAgora?: () => void;
  salvarComo?: () => void;
  pastaPadrao?: () => string;
  trocarPasta?: () => void;
  autoSalvar?: () => boolean;
  setAutoSalvar?: (ligado: boolean) => void;
  intervalo?: () => number;
  setIntervalo?: (s: number) => void;
  temCopia?: () => boolean;
  descartarCopia?: () => void;

  // Documento
  papel?: () => Papel;
  setPapel?: (p: Papel) => void;
  margens?: () => 'normal' | 'estreita' | 'larga';
  setMargens?: (m: 'normal' | 'estreita' | 'larga') => void;
  regua?: () => boolean;
  setRegua?: (mostrar: boolean) => void;
  estrutura?: () => boolean;
  setEstrutura?: (mostrar: boolean) => void;
  hifenizacao?: () => boolean;
  setHifenizacao?: (ligada: boolean) => void;
  marcas?: () => boolean;
  setMarcas?: (mostrar: boolean) => void;

  aoFechar?: () => void;
}

function makeDraggable(element: HTMLElement, handle: HTMLElement = element): void {
  let isDragging = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let initialLeft = 0;
  let initialTop = 0;

  handle.style.cursor = 'grab';

  handle.addEventListener('mousedown', (e: MouseEvent) => {
    const target = e.target as HTMLElement;
    if (target.closest('button, input, select, textarea, a, label, .no-drag')) return;

    isDragging = true;
    handle.style.cursor = 'grabbing';
    dragStartX = e.clientX;
    dragStartY = e.clientY;

    const rect = element.getBoundingClientRect();
    initialLeft = rect.left;
    initialTop = rect.top;

    element.style.position = 'fixed';
    element.style.left = `${initialLeft}px`;
    element.style.top = `${initialTop}px`;
    element.style.transform = 'none';
    element.style.margin = '0';

    const onMouseMove = (moveEvt: MouseEvent) => {
      if (!isDragging) return;
      const dx = moveEvt.clientX - dragStartX;
      const dy = moveEvt.clientY - dragStartY;
      const maxLeft = Math.max(10, window.innerWidth - 60);
      const maxTop = Math.max(10, window.innerHeight - 40);
      const newLeft = Math.max(10, Math.min(maxLeft, initialLeft + dx));
      const newTop = Math.max(10, Math.min(maxTop, initialTop + dy));
      element.style.left = `${newLeft}px`;
      element.style.top = `${newTop}px`;
    };

    const onMouseUp = () => {
      if (isDragging) {
        isDragging = false;
        handle.style.cursor = 'grab';
        window.removeEventListener('mousemove', onMouseMove);
        window.removeEventListener('mouseup', onMouseUp);
      }
    };

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  });
}

function eventToShortcut(e: KeyboardEvent): string | null {
  if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return null;
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push('Ctrl');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');
  let k = e.key.toUpperCase();
  if (k === ' ') k = 'Space';
  parts.push(k);
  return parts.join('+');
}

/** Painel de configurações do FeatherDocs. */
export class SettingsPanel {
  private readonly fundo = document.createElement('div');
  private readonly nav = document.createElement('nav');
  private readonly navTitulo = document.createElement('div');
  private readonly fecharBtn = document.createElement('button');
  private readonly corpoEl = document.createElement('div');
  private readonly titulo = document.createElement('strong');
  private secao: SecaoConfig = 'aparencia';

  constructor(
    host: HTMLElement,
    private readonly actions: ConfiguracoesAcoes,
  ) {
    this.fundo.className = 'set-backdrop';
    this.fundo.hidden = true;
    this.fundo.style.display = 'none';

    this.fundo.addEventListener('mousedown', (e) => {
      if (e.target === this.fundo) {
        e.preventDefault();
        this.fechar();
      }
    });

    const modal = document.createElement('div');
    modal.className = 'set-modal';
    modal.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        e.stopPropagation();
        this.fechar();
      }
    });

    this.nav.className = 'set-nav';
    this.navTitulo.className = 'set-nav-title';
    this.navTitulo.textContent = t('config.titulo');
    this.nav.append(this.navTitulo);

    for (const [id, rotulo, ico] of SECOES) {
      const b = document.createElement('button');
      b.type = 'button';
      b.dataset['secao'] = id;
      b.append(icon(ico), document.createTextNode(rotulo));
      b.addEventListener('click', () => {
        this.secao = id;
        this.render();
      });
      this.nav.append(b);
    }

    const main = document.createElement('div');
    main.className = 'set-main';
    const head = document.createElement('div');
    head.className = 'set-head';
    this.fecharBtn.type = 'button';
    this.fecharBtn.className = 'set-close';
    this.fecharBtn.textContent = '✕';
    this.fecharBtn.title = t('config.fechar');
    this.fecharBtn.setAttribute('aria-label', t('config.fechar'));
    this.fecharBtn.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.fechar();
    });
    head.append(this.titulo, this.fecharBtn);

    this.corpoEl.className = 'set-body';
    main.append(head, this.corpoEl);
    modal.append(this.nav, main);
    this.fundo.append(modal);
    host.append(this.fundo);

    makeDraggable(modal, this.navTitulo);
    makeDraggable(modal, head);
  }

  get aberto(): boolean {
    return !this.fundo.hidden && this.fundo.style.display !== 'none';
  }

  get visible(): boolean {
    return this.aberto;
  }

  abrir(secao?: SecaoConfig): void {
    if (secao) this.secao = secao;
    this.render();
    this.fundo.hidden = false;
    this.fundo.style.display = 'flex';
    this.nav.querySelector<HTMLElement>('button[aria-current="true"]')?.focus();
  }

  fechar(): void {
    this.fundo.hidden = true;
    this.fundo.style.display = 'none';
    this.actions.aoFechar?.();
  }

  alternar(): void {
    if (this.aberto) this.fechar();
    else this.abrir();
  }

  show(secao?: SecaoConfig): void {
    this.abrir(secao);
  }

  hide(): void {
    this.fechar();
  }

  toggle(): void {
    this.alternar();
  }

  private render(): void {
    const en = codigoAtual() === 'en-US';
    const rotulos: Record<SecaoConfig, string> = {
      aparencia: en ? 'Appearance' : 'Aparência',
      salvamento: en ? 'Saving' : 'Salvamento',
      documento: en ? 'Document' : 'Documento',
      atalhos: en ? 'Shortcuts' : 'Atalhos',
      sobre: en ? 'About' : 'Sobre',
    };
    this.navTitulo.textContent = t('config.titulo');
    this.fecharBtn.title = t('config.fechar');
    this.fecharBtn.setAttribute('aria-label', t('config.fechar'));
    for (const b of this.nav.querySelectorAll<HTMLElement>('button[data-secao]')) {
      const sec = b.dataset['secao'] as SecaoConfig;
      b.setAttribute('aria-current', sec === this.secao ? 'true' : 'false');
      const ico = SECOES.find(([id]) => id === sec)?.[2] ?? 'paleta';
      b.replaceChildren(icon(ico), document.createTextNode(rotulos[sec] ?? sec));
    }
    this.titulo.textContent = rotulos[this.secao] ?? t('config.titulo');
    this.corpoEl.replaceChildren();

    const desenhar = {
      aparencia: () => this.desenharAparencia(),
      salvamento: () => this.desenharSalvamento(),
      documento: () => this.desenharDocumento(),
      atalhos: () => this.desenharAtalhos(),
      sobre: () => this.desenharSobre(),
    }[this.secao];
    desenhar();
  }

  // ------------------------------------------------------------- 1. Aparência
  private desenharAparencia(): void {
    const a = this.actions;
    const esquemaAtual = a.paleta ? a.paleta() : paletaSalva();
    const acentoAtual = a.acento ? a.acento() : acentoSalvo();
    const temaAtual = a.tema ? a.tema() : modoSalvo();
    const fonteAtual = a.fonte ? a.fonte() : fonteSalva();
    const corpoAtual = a.corpo ? a.corpo() : corpoSalvo();
    const zoomAtual = a.zoom ? a.zoom() : zoomSalvo();

    // Galeria de temas com amostras visuais
    const esquemas = document.createElement('div');
    esquemas.className = 'set-esquemas';
    for (const p of PALETAS) {
      const b = document.createElement('button');
      b.type = 'button';
      b.className = 'set-esquema';
      b.title = p.nome;
      b.setAttribute('aria-pressed', esquemaAtual === p.id ? 'true' : 'false');
      const amostra = document.createElement('span');
      amostra.className = 'set-esquema-amostra';
      const claro = p.claro;
      amostra.style.background = claro.fundo;
      amostra.style.borderColor = claro.acento;
      const faixa = document.createElement('i');
      faixa.style.background = claro.cromo;
      const ponto = document.createElement('u');
      ponto.style.background = claro.acento;
      amostra.append(faixa, ponto);
      const nome = document.createElement('span');
      nome.className = 'set-esquema-nome';
      nome.textContent = p.nome;
      b.append(amostra, nome);
      b.addEventListener('click', () => {
        if (a.setPaleta) a.setPaleta(p.id);
        else definirPaleta(p.id);
        a.aoMudarTema?.();
        this.render();
      });
      esquemas.append(b);
    }

    // Cores de destaque
    const cores = document.createElement('div');
    cores.className = 'set-cores';
    const doEsquema = document.createElement('button');
    doEsquema.type = 'button';
    doEsquema.className = 'set-cor set-cor-auto';
    doEsquema.title = 'Cor do esquema';
    doEsquema.setAttribute('aria-label', 'Usar a cor do esquema');
    doEsquema.setAttribute('aria-pressed', acentoAtual < 0 ? 'true' : 'false');
    doEsquema.addEventListener('click', () => {
      if (a.setAcento) a.setAcento(-1);
      else definirAcento(-1);
      a.aoMudarTema?.();
      this.render();
    });
    cores.append(doEsquema);

    ACENTOS.forEach(([nome, clara], i) => {
      const b = document.createElement('button');
      b.type = 'button';
      b.className = 'set-cor';
      b.style.background = clara;
      b.title = nome;
      b.setAttribute('aria-label', `Cor de destaque ${nome}`);
      b.setAttribute('aria-pressed', acentoAtual === i ? 'true' : 'false');
      b.addEventListener('click', () => {
        if (a.setAcento) a.setAcento(i);
        else definirAcento(i);
        a.aoMudarTema?.();
        this.render();
      });
      cores.append(b);
    });

    const blocoPaleta = linha('Esquema de cores', 'Muda a paleta da interface e das ferramentas.', document.createElement('span'));
    blocoPaleta.querySelector('.set-row-text')!.append(esquemas);
    blocoPaleta.lastElementChild!.remove();

    // Idioma
    const linguas = Object.keys(IDIOMAS).map((codigo) => {
      const nome = codigo === 'pt-BR' ? 'Português (Brasil)' : 'English (US)';
      return [codigo, nome] as [string, string];
    });

    this.corpoEl.append(
      linha('Tema', 'Automático acompanha a preferência do sistema operacional.', seg(
        [['auto', 'Automático'], ['light', 'Claro'], ['dark', 'Escuro']],
        temaAtual,
        (v) => {
          const modo = v as ModoDoTema;
          if (a.setTema) a.setTema(modo);
          else definirModo(modo);
          a.aoMudarTema?.();
          this.render();
        },
      )),
      blocoPaleta,
      linha('Cor de destaque', 'Sobrepõe a cor de realce do esquema ativo.', cores),
      linha('Fonte do editor', 'Família tipográfica padrão para escrita.', seg(
        FONTES_BASE,
        fonteAtual,
        (v) => {
          if (a.setFonte) a.setFonte(v);
          else definirFonte(v);
          this.render();
        },
      )),
      linha('Corpo padrão da letra', 'Tamanho base da tipografia do texto corrido.', seg(
        CORPOS_PADRAO.map(([n, r]) => [String(n), r]),
        String(corpoAtual),
        (v) => {
          const n = Number(v);
          if (a.setCorpo) a.setCorpo(n);
          else definirCorpo(n);
          this.render();
        },
      )),
      linha('Zoom padrão', 'Nível inicial de aproximação das páginas.', seg(
        ZOOMS_PADRAO.map(([n, r]) => [String(n), r]),
        String(zoomAtual),
        (v) => {
          const z = Number(v);
          if (a.setZoom) a.setZoom(z);
          else definirZoomSalvo(z);
          this.render();
        },
      )),
      linha(
        codigoAtual() === 'en-US' ? 'Interface language' : 'Idioma da interface',
        t('config.idioma_dica'),
        seg(
          linguas,
          codigoAtual(),
          (codigo) => {
            definirIdioma(codigo);
            const nome = codigo === 'pt-BR' ? 'Português (Brasil)' : 'English (US)';
            avisar(t('config.idioma_guardado', { nome }));
            this.render();
          },
        ),
      ),
    );
  }

  // ------------------------------------------------------------- 2. Salvamento
  private desenharSalvamento(): void {
    const a = this.actions;
    const aberto = a.arquivoAberto ? a.arquivoAberto() : '';

    const botoes = document.createElement('div');
    botoes.className = 'set-dupla';
    const agora = document.createElement('button');
    agora.type = 'button';
    agora.className = 'set-button principal';
    agora.textContent = 'Salvar agora';
    agora.addEventListener('click', () => {
      a.salvarAgora?.();
      this.render();
    });
    const como = document.createElement('button');
    como.type = 'button';
    como.className = 'set-button';
    como.textContent = 'Salvar como…';
    como.addEventListener('click', () => a.salvarComo?.());
    botoes.append(agora, como);

    this.corpoEl.append(
      linha(
        'Salvar',
        aberto ? `Arquivo atual: ${aberto}` : 'Documento ainda não gravado no disco.',
        botoes,
      ),
      linha(
        'Salvamento automático',
        'Grava as alterações no arquivo periodicamente enquanto você digita.',
        interruptor(Boolean(a.autoSalvar?.() ?? false), (v) => {
          a.setAutoSalvar?.(v);
          this.render();
        }),
      ),
    );

    if (a.autoSalvar?.()) {
      this.corpoEl.append(
        linha('Gravar após inatividade', 'Intervalo após a última alteração do teclado.', seg(
          INTERVALOS_SALVAMENTO.map(([v, r]) => [String(v), r]),
          String(a.intervalo?.() ?? 5),
          (v) => {
            a.setIntervalo?.(Number(v));
            this.render();
          },
        )),
      );
    }

    if (a.noDisco?.()) {
      const trocar = document.createElement('button');
      trocar.type = 'button';
      trocar.className = 'set-button';
      trocar.textContent = 'Alterar…';
      trocar.addEventListener('click', () => a.trocarPasta?.());
      const bloco = linha('Pasta padrão dos documentos', 'Local sugerido para novos arquivos .fdoz.', trocar);
      const onde = document.createElement('div');
      onde.className = 'set-path';
      onde.textContent = a.pastaPadrao?.() || '(pasta padrão do sistema)';
      bloco.querySelector('.set-row-text')!.append(onde);
      this.corpoEl.append(bloco);
    }

    const temCopia = Boolean(a.temCopia?.() ?? false);
    const descartar = document.createElement('button');
    descartar.type = 'button';
    descartar.className = 'set-button perigo';
    descartar.textContent = 'Descartar cópia';
    descartar.disabled = !temCopia;
    descartar.addEventListener('click', () => {
      a.descartarCopia?.();
      avisar('Cópia de segurança descartada');
      this.render();
    });

    this.corpoEl.append(
      linha(
        'Cópia de segurança e recuperação',
        temCopia
          ? 'Existe uma cópia de segurança salva com edições recentes não salvas.'
          : 'Nenhuma cópia pendente: o arquivo está sincronizado.',
        descartar,
      ),
    );

    const nota = document.createElement('p');
    nota.className = 'set-nota';
    nota.textContent =
      'O FeatherDocs roda inteiro na sua máquina: sem telemetria, sem contas e sem envio de dados para servidores externos. Suas cópias de segurança ficam guardadas exclusivamente no seu computador.';
    this.corpoEl.append(nota);
  }

  // ------------------------------------------------------------- 3. Documento
  private desenharDocumento(): void {
    const a = this.actions;
    const papelAtual = a.papel ? a.papel() : 'a4';
    const margensAtual = a.margens ? a.margens() : 'normal';

    this.corpoEl.append(
      linha('Papel padrão', 'Dimensões da folha física para novos documentos.', seg(
        [['a4', 'A4 (210 × 297 mm)'], ['carta', 'Carta (216 × 279 mm)']],
        papelAtual,
        (v) => {
          a.setPapel?.(v as Papel);
          this.render();
        },
      )),
      linha('Margens padrão', 'Predefinições de espaçamento lateral e vertical da página.', seg(
        [['normal', 'Normal (2,54 cm)'], ['estreita', 'Estreita (1,27 cm)'], ['larga', 'Larga (3,18 cm)']],
        margensAtual,
        (v) => {
          a.setMargens?.(v as 'normal' | 'estreita' | 'larga');
          this.render();
        },
      )),
      linha(
        'Régua horizontal',
        'Exibe a régua em centímetros no topo da folha para controle de recuos e margens.',
        interruptor(Boolean(a.regua?.() ?? true), (v) => a.setRegua?.(v)),
      ),
      linha(
        'Painel lateral de estrutura',
        'Mostra a lista de títulos e tópicos na barra lateral esquerda para navegação rápida.',
        interruptor(Boolean(a.estrutura?.() ?? false), (v) => a.setEstrutura?.(v)),
      ),
      linha(
        'Hifenização automática',
        'Separa palavras no fim das linhas respeitando as regras ortográficas do português.',
        interruptor(Boolean(a.hifenizacao?.() ?? false), (v) => a.setHifenizacao?.(v)),
      ),
      linha(
        'Marcas de formatação',
        'Exibe caracteres ocultos (espaços, quebras de página, fim de parágrafo).',
        interruptor(Boolean(a.marcas?.() ?? false), (v) => a.setMarcas?.(v)),
      ),
    );
  }

  // ------------------------------------------------------------- 4. Atalhos
  private desenharAtalhos(): void {
    const CHAVE_ATALHOS = 'featherdocs.atalhos';
    let mapa: Record<string, string> = {};
    try {
      const salvo = localStorage.getItem(CHAVE_ATALHOS);
      if (salvo) mapa = JSON.parse(salvo);
    } catch {}

    const resetBtn = document.createElement('button');
    resetBtn.type = 'button';
    resetBtn.className = 'set-button';
    resetBtn.textContent = 'Restaurar padrões';
    resetBtn.addEventListener('click', () => {
      try {
        localStorage.removeItem(CHAVE_ATALHOS);
      } catch {}
      avisar('Atalhos restaurados para o padrão');
      this.render();
    });

    const topo = linha('Atalhos de teclado', 'Clique no botão do atalho para regravar uma nova combinação de teclas.', resetBtn);
    this.corpoEl.append(topo);

    for (const item of ATALHOS_FEATHERDOCS) {
      const atalhoAtual = mapa[item.id] || item.padrao;
      const keyBtn = document.createElement('button');
      keyBtn.type = 'button';
      keyBtn.className = 'set-button';
      keyBtn.style.minWidth = '120px';
      keyBtn.style.textAlign = 'center';
      keyBtn.innerHTML = `<kbd class="set-kbd">${atalhoAtual}</kbd>`;

      let gravando = false;
      keyBtn.addEventListener('click', () => {
        if (gravando) return;
        gravando = true;
        keyBtn.textContent = 'Pressione teclas…';
        keyBtn.style.borderColor = 'var(--accent)';

        const aoPressionar = (e: KeyboardEvent) => {
          e.preventDefault();
          e.stopPropagation();
          const novo = eventToShortcut(e);
          if (!novo) return;

          document.removeEventListener('keydown', aoPressionar, true);
          gravando = false;
          mapa[item.id] = novo;
          try {
            localStorage.setItem(CHAVE_ATALHOS, JSON.stringify(mapa));
          } catch {}
          this.render();
        };

        requestAnimationFrame(() => {
          document.addEventListener('keydown', aoPressionar, true);
        });
      });

      this.corpoEl.append(linha(item.rotulo, `Padrão: ${item.padrao}`, keyBtn));
    }
  }

  // ------------------------------------------------------------- 5. Sobre
  private desenharSobre(): void {
    const topo = document.createElement('div');
    topo.style.cssText = 'padding:14px 0 10px;display:flex;align-items:center;gap:12px';
    const logo = icon('pagina');
    logo.style.cssText = 'width:36px;height:36px;color:var(--accent)';
    const info = document.createElement('div');
    info.innerHTML = `<h3 style="margin:0;font-size:17px;font-weight:600">FeatherDocs</h3>
      <p style="margin:2px 0 0;color:var(--ink-faint);font-size:12.5px">Versão 0.1.0 · A suíte FeatherOffice</p>`;
    topo.append(logo, info);

    const texto = document.createElement('p');
    texto.className = 'set-nota';
    texto.style.marginTop = '0';
    texto.textContent =
      'O FeatherDocs é o editor de documentos de texto da família FeatherOffice, desenvolvido com arquitetura rigorosa: o motor de dados e o motor de paginação determinística são 100% em Rust puro; a interface visual é 100% em TypeScript empacotada com Tauri.';

    const pontos = document.createElement('ul');
    pontos.style.cssText = 'color:var(--ink-soft);font-size:12.5px;line-height:1.7;padding-left:18px;margin:12px 0';
    pontos.innerHTML = `
      <li><strong>Sem quebra acidental de layout</strong>: Paginação determinística página a página com cálculo exato em Rust.</li>
      <li><strong>Privacidade total</strong>: Sem telemetria, sem contas obrigatórias e sem tráfego de dados para nuvem.</li>
      <li><strong>Formato .fdoz</strong>: Contêiner moderno e aberto, com suporte de leitura e exportação para Markdown (.md), LaTeX (.tex) e Word (.docx).</li>
      <li><strong>Integração FeatherOffice</strong>: Conexão direta com planilhas FeatherSheet e elementos vetoriais.</li>
    `;

    this.corpoEl.append(topo, texto, pontos);
  }
}

// ---------------------------------------------------------------- Funções auxiliares

function linha(rotulo: string, dica: string, controle: HTMLElement): HTMLElement {
  const el = document.createElement('div');
  el.className = 'set-row';
  const texto = document.createElement('div');
  texto.className = 'set-row-text';
  const r = document.createElement('div');
  r.className = 'set-row-label';
  r.textContent = rotulo;
  const d = document.createElement('div');
  d.className = 'set-row-hint';
  d.textContent = dica;
  texto.append(r, d);
  el.append(texto, controle);
  return el;
}

function seg(opcoes: [string, string][], atual: string, onPick: (valor: string) => void): HTMLElement {
  const el = document.createElement('div');
  el.className = 'set-seg';
  for (const [valor, rotulo] of opcoes) {
    const b = document.createElement('button');
    b.type = 'button';
    b.textContent = rotulo;
    b.setAttribute('aria-pressed', valor === atual ? 'true' : 'false');
    b.addEventListener('click', () => onPick(valor));
    el.append(b);
  }
  return el;
}

function interruptor(ligado: boolean, onChange: (valor: boolean) => void): HTMLElement {
  const el = document.createElement('label');
  el.className = 'set-switch';
  const caixa = document.createElement('input');
  caixa.type = 'checkbox';
  caixa.checked = ligado;
  caixa.addEventListener('change', () => onChange(caixa.checked));
  el.append(caixa, document.createElement('span'));
  return el;
}

/** Cria a instância das configurações ligada ao app ou a um callback simples. */
export function painelDeConfiguracoes(aoMudarTemaOuAcoes: (() => void) | ConfiguracoesAcoes, raiz: HTMLElement = document.body): SettingsPanel {
  const acoes: ConfiguracoesAcoes =
    typeof aoMudarTemaOuAcoes === 'function'
      ? { aoMudarTema: aoMudarTemaOuAcoes }
      : aoMudarTemaOuAcoes;
  return new SettingsPanel(raiz, acoes);
}
