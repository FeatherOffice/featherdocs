/**
 * O painel Localizar e Substituir (Ctrl+F e Ctrl+H): flutua no canto da área do texto. Só
 * pede ao motor — quem acha, conta e troca é o Rust; aqui há campos, opções e botões.
 */

import { formatar, t } from '../idiomas/index.ts';
import type { OpcoesDeBusca, ResultadoDeBusca, Substituido } from '../ponte/tipos.ts';
import { icon } from './icones.ts';
import { Z } from './z.ts';

export const BUSCA_CSS = `
.busca {
  position: absolute; top: 8px; right: 22px; z-index: ${Z.barra}; display: flex; flex-direction: column; gap: 6px; padding: 8px;
  border: 1px solid var(--chrome-rule); border-radius: 8px; background: var(--surface); color: var(--chrome-ink);
  box-shadow: 0 6px 24px rgba(0, 0, 0, 0.22); font-size: 12.5px; width: 360px; max-width: calc(100% - 44px);
}
.busca[hidden] { display: none; }
.busca-linha { display: flex; align-items: center; gap: 4px; }
.busca-campo { flex: 1; min-width: 0; padding: 4px 8px; border: 1px solid var(--chrome-rule); border-radius: 5px; background: var(--surface); color: var(--chrome-ink); font-size: 13px; }
.busca-campo.erro { border-color: #d93025; }
.busca-contagem { min-width: 64px; text-align: right; color: var(--chrome-ink-soft); white-space: nowrap; }
.busca-botao { display: inline-flex; align-items: center; justify-content: center; min-width: 26px; height: 26px; padding: 0 6px; border: 0; border-radius: 5px; background: transparent; color: var(--chrome-ink); cursor: pointer; font-size: 12.5px; }
.busca-botao:hover { background: var(--chrome-hover); }
.busca-botao.on { background: var(--chrome-active); color: var(--chrome-accent); }
.busca-opcoes { display: flex; flex-wrap: wrap; gap: 10px; color: var(--chrome-ink-soft); }
.busca-opcoes label { display: inline-flex; align-items: center; gap: 4px; cursor: pointer; }
.busca-aviso { color: var(--chrome-ink-soft); min-height: 1em; }
.busca-aviso.erro { color: #d93025; }
`;

export interface GanchosDaBusca {
  definir(consulta: string, opcoes: OpcoesDeBusca): ResultadoDeBusca;
  ir(delta: number): ResultadoDeBusca;
  substituir(por: string): Substituido;
  substituirTudo(por: string): Substituido;
  /** O painel fechou: tirar o realce e devolver o foco ao texto. */
  encerrar(): void;
  /** Depois de qualquer pedido: repintar a seleção e o realce. */
  repintar(m: Substituido['mudanca'] | null): void;
}

export class PainelDeBusca {
  readonly el = document.createElement('div');
  private readonly consulta = document.createElement('input');
  private readonly por = document.createElement('input');
  private readonly linhaPor = document.createElement('div');
  private readonly contagem = document.createElement('span');
  private readonly aviso = document.createElement('div');
  private readonly opcoes: OpcoesDeBusca = { maiusculas: false, palavra: false, regex: false };

  constructor(private readonly g: GanchosDaBusca) {
    this.el.className = 'busca';
    this.el.hidden = true;
    this.el.setAttribute('role', 'search');
    this.consulta.className = 'busca-campo';
    this.consulta.placeholder = t('busca.localizar');
    this.consulta.setAttribute('aria-label', t('busca.localizar'));
    this.por.className = 'busca-campo';
    this.por.placeholder = t('busca.substituir_por');
    this.por.setAttribute('aria-label', t('busca.substituir_por'));
    this.contagem.className = 'busca-contagem';
    this.aviso.className = 'busca-aviso';
    this.montar();
  }

  private botao(rotulo: string, ao: () => void, conteudo: string | SVGElement): HTMLButtonElement {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'busca-botao';
    b.title = rotulo;
    b.setAttribute('aria-label', rotulo);
    if (typeof conteudo === 'string') b.textContent = conteudo;
    else b.append(conteudo);
    b.addEventListener('click', ao);
    return b;
  }

  private montar(): void {
    const topo = document.createElement('div');
    topo.className = 'busca-linha';
    const alternar = this.botao(t('busca.alternar_substituir'), () => this.mostrarSubstituir(this.linhaPor.hidden), icon('substituir'));
    topo.append(
      alternar,
      this.consulta,
      this.contagem,
      this.botao(t('busca.anterior'), () => this.ir(-1), icon('seta-cima')),
      this.botao(t('busca.proximo'), () => this.ir(1), icon('seta-baixo')),
      this.botao(t('busca.fechar'), () => this.fechar(), icon('cancelar')),
    );
    this.linhaPor.className = 'busca-linha';
    this.linhaPor.hidden = true;
    this.linhaPor.append(this.por, this.botao(t('busca.substituir'), () => this.trocar(false), t('busca.substituir')), this.botao(t('busca.todos'), () => this.trocar(true), t('busca.todos')));
    const opcoes = document.createElement('div');
    opcoes.className = 'busca-opcoes';
    for (const [chave, rotulo] of [['maiusculas', 'busca.maiusculas'], ['palavra', 'busca.palavra'], ['regex', 'busca.regex']] as const) {
      const l = document.createElement('label');
      const c = document.createElement('input');
      c.type = 'checkbox';
      c.addEventListener('change', () => {
        this.opcoes[chave] = c.checked;
        this.procurar();
      });
      l.append(c, document.createTextNode(t(rotulo)));
      opcoes.append(l);
    }
    this.el.append(topo, this.linhaPor, opcoes, this.aviso);
    this.consulta.addEventListener('input', () => this.procurar());
    for (const campo of [this.consulta, this.por]) {
      campo.addEventListener('keydown', (e) => this.aoTeclar(e, campo === this.por));
    }
  }

  private aoTeclar(e: KeyboardEvent, noCampoPor: boolean): void {
    if (e.key === 'Escape') {
      e.preventDefault();
      this.fechar();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (noCampoPor) this.trocar(false);
      else this.ir(e.shiftKey ? -1 : 1);
    }
  }

  get aberto(): boolean {
    return !this.el.hidden;
  }

  /** Abre o painel (com ou sem a linha de substituir) e põe o texto selecionado, se houver, como consulta. */
  abrir(comSubstituir: boolean, selecionado: string): void {
    this.el.hidden = false;
    this.mostrarSubstituir(comSubstituir);
    if (selecionado && !selecionado.includes('\n')) this.consulta.value = selecionado;
    this.consulta.focus();
    this.consulta.select();
    this.procurar();
  }

  private mostrarSubstituir(sim: boolean): void {
    this.linhaPor.hidden = !sim;
    if (sim) this.por.focus();
  }

  fechar(): void {
    this.el.hidden = true;
    this.g.encerrar();
  }

  private mostrar(r: ResultadoDeBusca): void {
    this.contagem.textContent = r.total > 0 ? formatar(t('busca.contagem'), { atual: String(r.atual || '–'), total: String(r.total) }) : '';
    this.consulta.classList.toggle('erro', r.erro !== null);
    this.aviso.classList.toggle('erro', r.erro !== null);
    this.aviso.textContent = r.erro ?? (this.consulta.value && r.total === 0 ? t('busca.nenhum') : '');
  }

  private procurar(): void {
    this.mostrar(this.g.definir(this.consulta.value, this.opcoes));
    this.g.repintar(null);
  }

  private ir(delta: number): void {
    if (!this.consulta.value) return;
    this.mostrar(this.g.ir(delta));
    this.g.repintar(null);
  }

  private trocar(todos: boolean): void {
    if (!this.consulta.value) return;
    const r = todos ? this.g.substituirTudo(this.por.value) : this.g.substituir(this.por.value);
    if (todos) {
      this.mostrar(this.g.definir(this.consulta.value, this.opcoes));
      this.aviso.textContent = formatar(t('busca.trocados'), { n: String(r.trocados) });
    } else {
      this.mostrar({ total: r.total, atual: r.atual, erro: null });
    }
    this.g.repintar(r.mudanca);
  }
}
