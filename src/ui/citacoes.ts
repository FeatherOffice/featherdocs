/**
 * As caixas de Referências: gerenciar fontes (cadastro, importar e exportar .bib), editar uma
 * fonte, inserir (ou corrigir) uma citação e marcar uma entrada do índice remissivo. Só coletam e
 * mostram; quem formata no estilo, numera e valida é o motor (o erro dele aparece na caixa).
 */

import { t } from '../idiomas/index.ts';
import type { Autor, Bibliografia, Fonte, TipoDeFonte } from '../ponte/tipos.ts';
import { botao, botoes, rotulado } from './caixas-referencias.ts';
import { Dialogo, rotuloFechar } from './dialogo.ts';

export const CITACOES_CSS = `
.caixa-form .grade-fonte { display: grid; grid-template-columns: 1fr 1fr; gap: 8px 12px; }
.caixa-form .grade-fonte .largo { grid-column: 1 / -1; }
.caixa-form .grade-fonte input, .caixa-form .grade-fonte textarea { width: 100%; box-sizing: border-box; }
.caixa-form textarea.autores { min-height: 56px; resize: vertical; padding: 5px 8px; border: 1px solid var(--borda); border-radius: 6px; background: var(--painel); color: inherit; font: inherit; font-size: 13px; }
.caixa-form .lista-fontes { max-height: 220px; overflow-y: auto; border: 1px solid var(--borda); border-radius: 6px; padding: 4px 0; }
.caixa-form .lista-fontes label { display: flex; flex-direction: row; align-items: flex-start; gap: 8px; padding: 4px 10px; font-size: 13px; cursor: pointer; }
.caixa-form .lista-fontes label:hover { background: var(--hover, rgba(0,0,0,.05)); }
.caixa-form .lista-fontes .chave { color: var(--texto-2, #5f6368); font-size: 12px; }
.caixa-form .fontes-lado { display: flex; gap: 10px; }
.caixa-form .fontes-lado select { flex: 1; min-height: 240px; }
.caixa-form .fontes-lado .acoes { display: flex; flex-direction: column; gap: 6px; }
`;

const TIPOS: TipoDeFonte[] = ['livro', 'capitulo', 'artigo', 'site', 'tese', 'relatorio', 'outro'];

/** "Silva, A. (2023). Título" — a linha de uma fonte nas listas. */
export function resumoDaFonte(f: Fonte): string {
  const quem = f.autores.length ? f.autores.map((a) => a.sobrenome).join('; ') : '—';
  return `${quem} (${f.ano || 's.d.'}). ${f.titulo}`;
}

/** Uma fonte vazia do tipo livro. */
export function fonteVazia(): Fonte {
  return { chave: '', tipo: 'livro', autores: [], titulo: '', subtitulo: '', ano: '', edicao: '', local: '', editora: '', veiculo: '', volume: '', numero: '', paginas: '', url: '', acesso: '', doi: '' };
}

/** "Silva, João" por linha → autores. */
function lerAutores(texto: string): Autor[] {
  return texto
    .split('\n')
    .map((l) => l.trim())
    .filter(Boolean)
    .map((l) => {
      const [sobrenome, ...resto] = l.split(',');
      return { sobrenome: sobrenome!.trim(), nome: resto.join(',').trim() };
    });
}

/** A chave sugerida: o primeiro sobrenome (sem acento, minúsculo) e o ano. */
function chaveSugerida(autores: Autor[], ano: string): string {
  const s = (autores[0]?.sobrenome ?? 'fonte').normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().replace(/[^a-z0-9]/g, '');
  return `${s}${ano.replace(/\D/g, '')}`;
}

function entrada(valor: string): HTMLInputElement {
  const i = document.createElement('input');
  i.type = 'text';
  i.value = valor;
  return i;
}

/** Editar uma fonte (ou cadastrar, com `inicial` nulo). `salvar` devolve o erro do motor ou `null`. */
export function caixaDeFonte(inicial: Fonte | null, salvar: (f: Fonte) => string | null): void {
  const f0 = inicial ?? fonteVazia();
  const d: Dialogo = new Dialogo(inicial ? t('fonte.editar') : t('fonte.nova'), rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const grade = document.createElement('div');
    grade.className = 'grade-fonte';
    const tipo = document.createElement('select');
    for (const tp of TIPOS) tipo.add(new Option(t(`fonte.tipo_${tp}`), tp, false, tp === f0.tipo));
    const chave = entrada(f0.chave);
    chave.placeholder = 'silva2023';
    const autores = document.createElement('textarea');
    autores.className = 'autores';
    autores.value = f0.autores.map((a) => (a.nome ? `${a.sobrenome}, ${a.nome}` : a.sobrenome)).join('\n');
    autores.placeholder = t('fonte.autores_dica');
    const campos: Record<string, HTMLInputElement> = {};
    const veiculo = document.createElement('span');
    const rotuloDoVeiculo = (): void => {
      veiculo.textContent = t(`fonte.veiculo_${tipo.value}`);
    };
    tipo.addEventListener('change', rotuloDoVeiculo);
    rotuloDoVeiculo();
    const linha = (chaveCampo: keyof Fonte, largo = false, rotulo?: HTMLElement): void => {
      const i = entrada(String(f0[chaveCampo] ?? ''));
      campos[chaveCampo] = i;
      const l = rotulado(t(`fonte.${chaveCampo}`), i);
      if (rotulo) l.firstElementChild!.replaceWith(rotulo);
      if (largo) l.classList.add('largo');
      grade.append(l);
    };
    grade.append(rotulado(t('fonte.tipo'), tipo), rotulado(t('fonte.chave'), chave));
    const la = rotulado(t('fonte.autores'), autores);
    la.classList.add('largo');
    grade.append(la);
    linha('titulo', true);
    linha('subtitulo', true);
    linha('ano');
    linha('edicao');
    linha('local');
    linha('editora');
    linha('veiculo', true, veiculo);
    linha('volume');
    linha('numero');
    linha('paginas');
    linha('doi');
    linha('url', true);
    linha('acesso');
    const erro = document.createElement('div');
    erro.className = 'erro';
    const ok = (): void => {
      const lidos = lerAutores(autores.value);
      const f: Fonte = { ...f0, tipo: tipo.value as TipoDeFonte, autores: lidos, chave: chave.value.trim() || chaveSugerida(lidos, campos.ano!.value) };
      for (const [k, i] of Object.entries(campos)) (f as unknown as Record<string, string>)[k] = i.value.trim();
      const msg = salvar(f);
      if (msg) erro.textContent = msg;
      else d.fechar();
    };
    form.append(grade, erro, botoes(botao(t('estilo.cancelar'), () => d.fechar()), botao(t('estilo.ok'), ok, true)));
    corpo.append(form);
    setTimeout(() => (inicial ? campos.titulo : autores)?.focus());
  });
  d.abrir();
}

/** O que a caixa "Gerenciar fontes" faz (quem executa é o aplicativo). */
export interface AcoesDasFontes {
  lista(): Bibliografia;
  salvar(f: Fonte, antiga: string): string | null;
  remover(chave: string): void;
  /** Importa um .bib; devolve a mensagem para mostrar (quantas, ou o erro). */
  importar(): Promise<string | null>;
  exportar(): void;
}

/** Referências → Gerenciar fontes: a lista, com nova, editar, excluir, importar e exportar .bib. */
export function caixaDeFontes(a: AcoesDasFontes): void {
  const d: Dialogo = new Dialogo(t('cmd.fontes'), rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const lado = document.createElement('div');
    lado.className = 'fontes-lado';
    const lista = document.createElement('select');
    lista.size = 12;
    const aviso = document.createElement('div');
    aviso.className = 'dica';
    const encher = (escolher?: string): void => {
      const fontes = a.lista().fontes;
      lista.replaceChildren(...fontes.map((f) => new Option(`[${f.chave}] ${resumoDaFonte(f)}`, f.chave, false, f.chave === escolher)));
      aviso.textContent = fontes.length ? '' : t('fonte.nenhuma');
    };
    const atual = (): Fonte | undefined => a.lista().fontes.find((f) => f.chave === lista.value);
    const editar = (): void => {
      const f = atual();
      if (f) caixaDeFonte(f, (novo) => a.salvar(novo, f.chave) ?? (encher(novo.chave), null));
    };
    lista.addEventListener('dblclick', editar);
    const acoes = document.createElement('div');
    acoes.className = 'acoes';
    acoes.append(
      botao(t('fonte.nova') + '…', () => caixaDeFonte(null, (novo) => a.salvar(novo, '') ?? (encher(novo.chave), null))),
      botao(t('fonte.editar') + '…', editar),
      botao(t('fonte.excluir'), () => {
        if (!lista.value) return;
        a.remover(lista.value);
        encher();
      }),
      botao(t('fonte.importar'), () => {
        void a.importar().then((msg) => {
          encher();
          if (msg) aviso.textContent = msg;
        });
      }),
      botao(t('fonte.exportar'), () => a.exportar()),
    );
    lado.append(lista, acoes);
    form.append(lado, aviso, botoes(botao(t('estilo.ok'), () => d.fechar(), true)));
    corpo.append(form);
    encher();
  });
  d.abrir();
}

/**
 * Inserir (ou corrigir) uma citação: as fontes marcadas e a página. `novaFonte` abre o cadastro e
 * depois volta a esta caixa com a lista atualizada.
 */
export function caixaDeCitacao(titulo: string, fontes: Fonte[], inicial: { chaves: string[]; pagina: string }, aplicar: (chaves: string[], pagina: string) => string | null, novaFonte: () => void): void {
  const d: Dialogo = new Dialogo(titulo, rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const lista = document.createElement('div');
    lista.className = 'lista-fontes';
    const caixas: Array<[string, HTMLInputElement]> = [];
    for (const f of fontes) {
      const l = document.createElement('label');
      const c = document.createElement('input');
      c.type = 'checkbox';
      c.checked = inicial.chaves.includes(f.chave);
      const texto = document.createElement('span');
      const chave = document.createElement('span');
      chave.className = 'chave';
      chave.textContent = ` [${f.chave}]`;
      texto.append(resumoDaFonte(f), chave);
      l.append(c, texto);
      lista.append(l);
      caixas.push([f.chave, c]);
    }
    if (!fontes.length) lista.textContent = t('fonte.nenhuma');
    const pagina = entrada(inicial.pagina);
    pagina.placeholder = '42';
    const erro = document.createElement('div');
    erro.className = 'erro';
    const ok = (): void => {
      const msg = aplicar(caixas.filter(([, c]) => c.checked).map(([k]) => k), pagina.value);
      if (msg) erro.textContent = msg;
      else d.fechar();
    };
    form.append(rotulado(t('citacao.fontes'), lista), rotulado(t('citacao.pagina'), pagina), erro, botoes(botao(t('fonte.nova') + '…', () => (d.fechar(), novaFonte())), botao(t('estilo.cancelar'), () => d.fechar()), botao(t('estilo.ok'), ok, true)));
    corpo.append(form);
  });
  d.abrir();
}

/** Referências → Marcar entrada: o termo (já com o texto selecionado), o subtermo e "marcar todas". */
export function caixaDeEntrada(termo: string, aplicar: (termo: string, sub: string, todas: boolean) => string | null): void {
  const d: Dialogo = new Dialogo(t('cmd.marcar_entrada'), rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const principal = entrada(termo);
    const sub = entrada('');
    const erro = document.createElement('div');
    erro.className = 'erro';
    const marcar = (todas: boolean): void => {
      const msg = aplicar(principal.value, sub.value, todas);
      if (msg) erro.textContent = msg;
      else d.fechar();
    };
    principal.addEventListener('keydown', (e) => e.key === 'Enter' && marcar(false));
    const dica = document.createElement('div');
    dica.className = 'dica';
    dica.textContent = t('entrada.dica');
    form.append(rotulado(t('entrada.termo'), principal), rotulado(t('entrada.sub'), sub), dica, erro, botoes(botao(t('estilo.cancelar'), () => d.fechar()), botao(t('entrada.todas'), () => marcar(true)), botao(t('entrada.marcar'), () => marcar(false), true)));
    corpo.append(form);
    setTimeout(() => principal.select());
  });
  d.abrir();
}
