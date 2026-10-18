/**
 * A casca de cima, no jeito do FeatherSheet: o topo (pílula "Início", configurações, ajuda), a barra de
 * menus (quando a faixa está minimizada) e a faixa de abas: Página inicial, Inserir, Layout, Referências,
 * Revisão, Exibir, e a aba Tabela quando o cursor está numa. O que cada botão é e faz vem do registro de
 * comandos (`comandos-lista.ts`); cada aba monta seus grupos em `faixa/*.ts`. Os botões que mexem no texto
 * **não roubam o foco** do editor (`mousedown` sem ação padrão).
 */

import { t } from '../idiomas/index.ts';
import type { Estado, Formato, FormatoDoCodigo } from '../ponte/tipos.ts';
import { comando, dicaDe, type Registro } from './comandos.ts';
import { criarRegistro, type Contexto, type GanchosDosComandos } from './comandos-lista.ts';
import { ESTILOS_CSS } from './estilos-galeria.ts';
import { FORMATO_CSS } from './faixa-formato.ts';
import { abrirSimbolos, COMPONENTES_CSS } from './faixa/componentes.ts';
import { abaExibir } from './faixa/exibir.ts';
import { abaInicio } from './faixa/inicio.ts';
import { abaInserir } from './faixa/inserir.ts';
import type { Grupo, Kit } from './faixa/kit.ts';
import { abaLayout } from './faixa/layout.ts';
import { abaReferencias } from './faixa/referencias.ts';
import { abaRevisao } from './faixa/revisao.ts';
import { abaTabela } from './faixa/tabela.ts';
import { icon } from './icones.ts';
import { MENU_CSS, MenuBar, openPopup } from './menu.ts';
import { menusSuspensos } from './menus-suspensos.ts';
import { modoSalvo } from './tema.ts';
import { Z } from './z.ts';

export { MARGENS_PRONTAS } from './comandos-lista.ts';

export const BARRA_CSS = `
.casca { flex: none; background: var(--chrome-bg); z-index: ${Z.barra}; position: relative; }
.topo { display: flex; align-items: center; gap: 14px; padding: 9px 18px 0; }
.topo-bloco { flex: 1; min-width: 0; display: flex; align-items: center; gap: 10px; }
.botao-inicio {
  display: inline-flex; align-items: center; gap: 7px; height: 32px; padding: 0 14px 0 11px;
  border: 1px solid var(--chrome-rule); border-radius: 16px; background: transparent; color: var(--chrome-ink);
  font-size: 13.5px; line-height: 1; cursor: pointer; white-space: nowrap;
}
.botao-inicio:hover { background: var(--chrome-hover); }
.botao-inicio svg { width: 16px; height: 16px; flex: none; }
.topo-acoes { flex: none; display: flex; align-items: center; gap: 6px; }
.botao-redondo {
  display: grid; place-items: center; width: 34px; height: 34px; border: 0; border-radius: 50%;
  background: transparent; color: var(--chrome-ink-soft); cursor: pointer; padding: 0;
}
.botao-redondo:hover { background: var(--chrome-hover); color: var(--chrome-ink); }
.botao-redondo svg { width: 18px; height: 18px; }
.casca button { min-height: 0; }
${MENU_CSS}
${FORMATO_CSS}
${ESTILOS_CSS}
${COMPONENTES_CSS}
.faixa { flex: none; background: var(--chrome-bg); border-bottom: 1px solid var(--chrome-rule); }
/* Faixa aberta: os menus saem de cena. Faixa minimizada: sobram só os menus suspensos (e a faixa some). */
.casca.com-faixa .menus-host { display: none; }
.casca:not(.com-faixa) .faixa { display: none; }
.faixa-abas { display: flex; align-items: flex-end; gap: 2px; padding: 0 14px 0 12px; }
.faixa-aba {
  height: 30px; padding: 0 14px; border: 0; border-bottom: 2px solid transparent; background: transparent;
  color: var(--chrome-ink-soft); font-size: 13px; cursor: pointer; white-space: nowrap; border-radius: 4px 4px 0 0;
}
.faixa-aba:hover { color: var(--chrome-ink); background: var(--chrome-hover); }
.faixa-aba.on { color: var(--chrome-accent); border-bottom-color: var(--chrome-accent); font-weight: 600; }
.faixa-contexto {
  align-self: center; margin-left: 10px; padding: 2px 8px; border-radius: 4px; background: var(--chrome-accent);
  color: var(--chrome-bg); font-size: 11px; font-weight: 600; white-space: nowrap;
}
.faixa-aba.contextual { color: var(--chrome-accent); }
.faixa-lateral { display: flex; align-items: center; gap: 2px; margin-left: auto; align-self: center; }
.faixa-rapido {
  display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 26px; border: 0;
  border-radius: 4px; background: transparent; color: var(--chrome-ink-soft); cursor: pointer; padding: 0;
}
.faixa-rapido:hover:not(:disabled) { background: var(--chrome-hover); color: var(--chrome-ink); }
.faixa-rapido:disabled { opacity: 0.4; cursor: default; }
.faixa-painel {
  display: flex; align-items: stretch; min-height: 100px; padding: 6px 12px 0; border-top: 1px solid var(--chrome-rule);
  background: var(--chrome-bg-2); overflow-x: auto; overflow-y: hidden; scrollbar-width: thin;
}
.faixa-grupo { display: flex; flex-direction: column; flex: none; padding: 0 10px; border-right: 1px solid var(--chrome-rule); }
.faixa-grupo:first-child { padding-left: 0; }
.faixa-grupo:last-child { border-right: 0; }
.faixa-itens { display: flex; align-items: flex-start; gap: 2px; flex: 1; }
.faixa-rotulo { padding: 2px 0 4px; color: var(--chrome-ink-soft); font-size: 11px; text-align: center; white-space: nowrap; }
.faixa-grande {
  display: flex; flex-direction: column; align-items: center; justify-content: flex-start; gap: 4px; min-width: 56px;
  max-width: 84px; height: 64px; padding: 7px 6px 0; border: 0; border-radius: 6px; background: transparent;
  color: var(--chrome-ink); font-size: 12px; line-height: 1.15; text-align: center; cursor: pointer;
}
.faixa-grande svg { flex: none; width: 24px; height: 24px; color: var(--chrome-accent); }
.faixa-grande:hover:not(:disabled), .faixa-pequeno:hover:not(:disabled) { background: var(--chrome-hover); }
.faixa-grande.on, .faixa-pequeno.on { background: var(--chrome-active); color: var(--chrome-accent); }
.faixa-grande:disabled, .faixa-pequeno:disabled { opacity: 0.45; cursor: default; }
.faixa-pilha { display: flex; flex-direction: column; gap: 1px; }
.faixa-linha { display: flex; align-items: center; gap: 1px; height: 24px; }
.faixa-pequeno {
  display: inline-flex; align-items: center; gap: 6px; height: 23px; padding: 0 7px; border: 0; border-radius: 4px;
  background: transparent; color: var(--chrome-ink); font-size: 12.5px; white-space: nowrap; cursor: pointer;
}
.faixa-pequeno svg { color: var(--chrome-ink-soft); }
.faixa-pequeno.so-icone { padding: 0 5px; }
.faixa-pequeno.letra { min-width: 26px; justify-content: center; }
.faixa .caret { font-size: 9px; opacity: 0.75; }
.faixa-grande .caret { margin-left: 2px; font-size: 9px; color: var(--chrome-ink-soft); }
.faixa-pequeno .caret { font-size: 9px; color: var(--chrome-ink-soft); margin-left: -2px; }
/* Comando da especificação ainda sem motor: visível, desabilitado, com a dica do que fará. */
.faixa-grande.em-breve, .faixa-pequeno.em-breve { opacity: 0.4; cursor: help; }
.faixa-grande.em-breve svg, .faixa-pequeno.em-breve svg { color: var(--chrome-ink-soft); }
.faixa .zoom-valor { min-width: 44px; text-align: center; color: var(--chrome-ink-soft); font-size: 12.5px; }
`;


export interface GanchosDaBarra extends GanchosDosComandos {
  /** A régua está à vista? (para o botão aparecer marcado) */
  reguaVisivel(): boolean;
  /** O modo foco está ligado? */
  focoAtivo(): boolean;
  /** O zoom atual (1 = 100 %). */
  zoomAtual(): number;
  /** O modo dividido ligado (e em que língua), ou `null`. */
  codigoAtivo(): FormatoDoCodigo | null;
  continuoAtivo(): boolean;
  variasAtivo(): boolean;
  miniaturasAtivas(): boolean;
}

interface Observado {
  el: HTMLButtonElement;
  ativo?: () => boolean;
  desativado?: () => boolean;
}

interface Aba {
  titulo: string;
  grupos: () => Grupo[];
}

export class Barra implements Kit {
  readonly el = document.createElement('header');
  private readonly menus: MenuBar;
  private readonly abasEl = document.createElement('div');
  private readonly painel = document.createElement('div');
  private readonly faixa = document.createElement('div');
  private readonly observados: Observado[] = [];
  private readonly sincronizadores: Array<() => void> = [];
  private abas: Aba[] = [];
  private abaContextual: Aba | null = null;
  registro: Registro;
  private readonly contexto: Contexto;
  private rapidoDesfazer!: HTMLButtonElement;
  private rapidoRefazer!: HTMLButtonElement;
  private rapidoDesfazerMenu!: HTMLButtonElement;
  private rapidoRefazerMenu!: HTMLButtonElement;
  private atual = 0;
  private comTabela = false;
  private faixaAberta = true;
  private estrutura = false;
  private estado: Estado | null = null;
  private formato: Formato | null = null;
  private topoEl!: HTMLElement;

  constructor(private readonly g: GanchosDaBarra) {
    this.el.className = 'casca';
    this.contexto = {
      g,
      formato: () => this.formato,
      estado: () => this.estado,
      estrutura: () => this.estrutura,
      regua: () => g.reguaVisivel(),
      foco: () => g.focoAtivo(),
      tema: () => modoSalvo(),
      codigo: () => g.codigoAtivo(),
      continuo: () => g.continuoAtivo(),
      varias: () => g.variasAtivo(),
      miniaturas: () => g.miniaturasAtivas(),
    };
    this.registro = criarRegistro(this.contexto);
    this.topoEl = this.montarTopo();
    this.el.append(this.topoEl);

    const menubarHost = document.createElement('div');
    menubarHost.className = 'menus-host';
    this.menus = new MenuBar(menubarHost);
    this.montarMenus();
    this.el.append(menubarHost);
    try {
      this.faixaAberta = localStorage.getItem('featherdocs.faixa') !== 'fechada';
    } catch {
      // sem armazenamento: começa com a faixa aberta
    }
    this.el.classList.toggle('com-faixa', this.faixaAberta);

    this.faixa.className = 'faixa';
    this.faixa.setAttribute('role', 'toolbar');
    this.faixa.setAttribute('aria-label', t('faixa.aria'));
    this.abasEl.className = 'faixa-abas';
    this.abasEl.setAttribute('role', 'tablist');
    this.painel.className = 'faixa-painel';
    this.painel.setAttribute('role', 'tabpanel');
    this.faixa.append(this.abasEl, this.painel);
    this.el.append(this.faixa);
    this.montarAbas();
    this.mostrar(0);
  }

  private montarAbas(): void {
    const c = this.contexto;
    this.abas = [
      { titulo: t('faixa.inicio'), grupos: () => abaInicio(this, c) },
      { titulo: t('faixa.inserir'), grupos: () => abaInserir(this, c) },
      { titulo: t('faixa.layout'), grupos: () => abaLayout(this, c) },
      { titulo: t('faixa.referencias'), grupos: () => abaReferencias(this, c) },
      { titulo: t('faixa.revisao'), grupos: () => abaRevisao(this, c) },
      { titulo: t('faixa.exibir'), grupos: () => abaExibir(this, c, () => this.g.zoomAtual()) },
    ];
    this.abaContextual = { titulo: t('faixa.tabela'), grupos: () => abaTabela(this, c) };
  }

  // ---------------------------------------------------------------- topo e menus

  private montarTopo(): HTMLElement {
    const topo = document.createElement('div');
    topo.className = 'topo';
    const bloco = document.createElement('div');
    bloco.className = 'topo-bloco';
    const inicio = document.createElement('button');
    inicio.type = 'button';
    inicio.className = 'botao-inicio';
    inicio.title = t('inicio.botao_barra');
    inicio.setAttribute('aria-label', t('inicio.botao_barra'));
    inicio.append(icon('inicio'), document.createTextNode(t('cmd.inicio')));
    inicio.addEventListener('click', () => this.g.inicio());
    bloco.append(inicio);
    const acoes = document.createElement('div');
    acoes.className = 'topo-acoes';
    const redondo = (nome: string, titulo: string, ao: () => void): HTMLButtonElement => {
      const b = document.createElement('button');
      b.type = 'button';
      b.className = 'botao-redondo';
      b.title = titulo;
      b.setAttribute('aria-label', titulo);
      b.append(icon(nome));
      b.addEventListener('click', ao);
      return b;
    };
    acoes.append(redondo('ajuda', t('inicio.ajuda'), () => this.g.ajuda()), redondo('config', t('inicio.configuracoes'), () => this.g.configurar()));
    topo.append(bloco, acoes);
    return topo;
  }

  private montarMenus(): void {
    for (const [titulo, itens] of menusSuspensos(this.registro)) this.menus.add(titulo, itens);
    this.menus.onToggleToolbar = () => this.definirFaixa(true);
    this.rapidoDesfazerMenu = this.menus.addQuickAction('desfazer', t('barra.desfazer'), () => this.g.desfazer());
    this.rapidoRefazerMenu = this.menus.addQuickAction('refazer', t('barra.refazer'), () => this.g.refazer());
    this.menus.addQuickAction('seta-baixo', t('faixa.expandir'), () => this.definirFaixa(true));
  }

  // ---------------------------------------------------------------- faixa

  private mostrar(indice: number): void {
    this.atual = indice;
    const todas = this.comTabela && this.abaContextual ? [...this.abas, this.abaContextual] : this.abas;
    this.abasEl.replaceChildren(
      ...todas.flatMap((aba, i) => {
        const b = document.createElement('button');
        b.type = 'button';
        const contextual = i >= this.abas.length;
        b.className = contextual ? 'faixa-aba contextual' : 'faixa-aba';
        b.textContent = aba.titulo;
        b.setAttribute('role', 'tab');
        b.setAttribute('aria-selected', String(i === indice));
        b.classList.toggle('on', i === indice);
        b.addEventListener('mousedown', (e) => e.preventDefault());
        b.addEventListener('click', () => this.mostrar(i));
        // Como no Excel e no FeatherSheet: duplo clique na aba minimiza a faixa (viram os menus suspensos).
        b.addEventListener('dblclick', () => this.definirFaixa(false));
        return contextual ? [this.rotuloContexto(), b] : [b];
      }),
      this.lateral(),
    );
    this.observados.length = 0;
    this.sincronizadores.length = 0;
    this.painel.replaceChildren(
      ...todas[indice]!.grupos().map((grupo) => {
        const el = document.createElement('div');
        el.className = 'faixa-grupo';
        const itens = document.createElement('div');
        itens.className = 'faixa-itens';
        itens.append(...grupo.itens);
        const rotulo = document.createElement('div');
        rotulo.className = 'faixa-rotulo';
        rotulo.textContent = grupo.titulo;
        el.append(itens, rotulo);
        return el;
      }),
    );
    this.sincronizar();
  }

  private rotuloContexto(): HTMLElement {
    const el = document.createElement('span');
    el.className = 'faixa-contexto';
    el.textContent = t('faixa.g_tabela');
    return el;
  }

  /** Desfazer, refazer, salvar e o minimizar, à direita das abas. */
  private lateral(): HTMLElement {
    const lado = document.createElement('div');
    lado.className = 'faixa-lateral';
    const rapido = (nome: string, titulo: string, ao: () => void): HTMLButtonElement => {
      const b = document.createElement('button');
      b.type = 'button';
      b.className = 'faixa-rapido';
      b.title = titulo;
      b.setAttribute('aria-label', titulo);
      b.append(icon(nome));
      b.addEventListener('mousedown', (e) => e.preventDefault());
      b.addEventListener('click', ao);
      return b;
    };
    this.rapidoDesfazer = rapido('desfazer', t('barra.desfazer'), () => this.g.desfazer());
    this.rapidoRefazer = rapido('refazer', t('barra.refazer'), () => this.g.refazer());
    const minimizar = rapido('seta-cima', `${t('faixa.minimizar')}  Ctrl+F1`, () => this.definirFaixa(false));
    lado.append(rapido('salvar', t('faixa.salvar_rapido'), () => this.g.salvar()), this.rapidoDesfazer, this.rapidoRefazer, minimizar);
    return lado;
  }

  // ---------------------------------------------------------------- o kit das abas

  grande(rotulo: string, nome: string, ao: () => void, ativo?: () => boolean, desativado?: () => boolean): HTMLButtonElement {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'faixa-grande';
    b.title = rotulo;
    const texto = document.createElement('span');
    texto.className = 'rotulo-grande';
    texto.textContent = rotulo;
    b.append(icon(nome), texto);
    this.ligar(b, ao, ativo, desativado);
    return b;
  }

  pequeno(rotulo: string, nome: string, ao: () => void, opcoes: { soIcone?: boolean; classe?: string; ativo?: () => boolean; desativado?: () => boolean } = {}): HTMLButtonElement {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = `faixa-pequeno${opcoes.soIcone ? ' so-icone' : ''}${opcoes.classe ? ` ${opcoes.classe}` : ''}`;
    b.title = rotulo;
    b.setAttribute('aria-label', rotulo);
    b.append(icon(nome));
    if (!opcoes.soIcone) b.append(document.createTextNode(rotulo));
    this.ligar(b, ao, opcoes.ativo, opcoes.desativado);
    return b;
  }

  ligar(b: HTMLButtonElement, ao: () => void, ativo?: () => boolean, desativado?: () => boolean): void {
    b.addEventListener('mousedown', (e) => e.preventDefault());
    b.addEventListener('click', ao);
    this.observados.push({ el: b, ativo, desativado });
  }

  pilha(...linhas: HTMLElement[][]): HTMLElement {
    const p = document.createElement('div');
    p.className = 'faixa-pilha';
    for (const itens of linhas) {
      const l = document.createElement('div');
      l.className = 'faixa-linha';
      l.append(...itens);
      p.append(l);
    }
    return p;
  }

  aoSincronizar(f: () => void): void {
    this.sincronizadores.push(f);
  }

  /** Um botão do registro. Com menu, abre o menu (e ganha ▾); "em breve", sai desabilitado com a dica. */
  cmd(id: string, forma: 'grande' | 'pequeno' | 'icone' = 'pequeno'): HTMLButtonElement {
    const c = comando(this.registro, id);
    let b: HTMLButtonElement;
    const ao = (): void => {
      if (c.menu) openPopup(b, c.menu());
      else c.executar?.();
    };
    const desativado = c.emBreve ? () => true : c.desativado;
    if (forma === 'grande') {
      b = this.grande(c.rotulo, c.icone, ao, c.ativo, desativado);
    } else {
      b = this.pequeno(c.rotulo, c.icone, ao, { soIcone: forma === 'icone', ativo: c.ativo, desativado });
    }
    if (c.menu) {
      const caret = document.createElement('span');
      caret.className = 'caret';
      caret.textContent = '▾';
      (b.querySelector('.rotulo-grande') ?? b).append(caret);
    }
    b.title = dicaDe(c);
    b.classList.toggle('em-breve', Boolean(c.emBreve));
    return b;
  }

  // ---------------------------------------------------------------- estado

  /** Abre ou minimiza a faixa: aberta, os menus somem; minimizada, voltam como menus suspensos. */
  definirFaixa(aberta: boolean): void {
    this.faixaAberta = aberta;
    this.el.classList.toggle('com-faixa', aberta);
    this.menus.close();
    try {
      localStorage.setItem('featherdocs.faixa', aberta ? 'aberta' : 'fechada');
    } catch {
      // sem armazenamento: vale até fechar a janela
    }
  }

  /** Ctrl+F1: troca entre a faixa e os menus. */
  alternarFaixa(): void {
    this.definirFaixa(!this.faixaAberta);
  }

  private sincronizar(): void {
    for (const o of this.observados) {
      if (o.ativo) o.el.classList.toggle('on', o.ativo());
      if (o.desativado) o.el.disabled = o.desativado();
    }
    for (const f of this.sincronizadores) f();
    const e = this.estado;
    if (this.rapidoDesfazer) this.rapidoDesfazer.disabled = !e?.pode_desfazer;
    if (this.rapidoRefazer) this.rapidoRefazer.disabled = !e?.pode_refazer;
    this.rapidoDesfazerMenu.disabled = !e?.pode_desfazer;
    this.rapidoRefazerMenu.disabled = !e?.pode_refazer;
  }

  /** Reflete o estado do documento e a formatação no cursor. */
  atualizar(e: Estado, formato: Formato | null): void {
    this.estado = e;
    this.formato = formato;
    const tabela = formato?.tabela != null;
    if (tabela !== this.comTabela) {
      this.comTabela = tabela;
      // A aba de Tabela some junto com a tabela: se estava nela, volta à primeira.
      this.mostrar(!tabela && this.atual >= this.abas.length ? 0 : this.atual);
      return;
    }
    this.sincronizar();
  }

  /** A grade de símbolos pelo menu Inserir (sem botão da faixa por perto: abre junto do topo). */
  abrirSimbolos(): void {
    abrirSimbolos(this.el, (s) => this.g.simbolo(s));
  }

  /** Marca o botão da estrutura como ligado/desligado. */
  estruturaVisivel(visivel: boolean): void {
    this.estrutura = visivel;
    this.sincronizar();
  }

  /** Atualiza todos os textos e menus da barra para o novo idioma em tempo real. */
  atualizarIdioma(): void {
    const novoTopo = this.montarTopo();
    this.topoEl.replaceWith(novoTopo);
    this.topoEl = novoTopo;
    this.registro = criarRegistro(this.contexto);
    this.menus.clear();
    this.montarMenus();
    this.faixa.setAttribute('aria-label', t('faixa.aria'));
    this.montarAbas();
    this.mostrar(this.atual);
  }
}
