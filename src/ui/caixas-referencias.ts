/**
 * As caixas de Referências: sumário personalizado, marcador e referência cruzada. Só coletam e
 * mostram; quem guarda, numera e valida é o motor (o erro dele aparece na própria caixa).
 */

import { t } from '../idiomas/index.ts';
import type { AlvoDeReferencia } from '../ponte/tipos.ts';
import { Dialogo, rotuloFechar } from './dialogo.ts';

export const CAIXAS_REFERENCIAS_CSS = `
.caixa-form select { padding: 5px 8px; border: 1px solid var(--borda); border-radius: 6px; background: var(--painel); color: inherit; font-size: 13px; }
.caixa-form select[size] { min-height: 180px; }
.caixa-form .erro { color: var(--erro, #c5221f); font-size: 12.5px; min-height: 1em; }
.caixa-form .lado { display: flex; gap: 8px; align-items: flex-end; }
.caixa-form .lado > label { flex: 1; }
`;

export function rotulado(texto: string, el: HTMLElement): HTMLLabelElement {
  const l = document.createElement('label');
  const s = document.createElement('span');
  s.textContent = texto;
  l.append(s, el);
  return l;
}

export function marcar(texto: string, ligado: boolean): { el: HTMLLabelElement; caixa: HTMLInputElement } {
  const el = document.createElement('label');
  el.className = 'marcar';
  const caixa = document.createElement('input');
  caixa.type = 'checkbox';
  caixa.checked = ligado;
  el.append(caixa, document.createTextNode(texto));
  return { el, caixa };
}

export function botao(texto: string, ao: () => void, principal = false): HTMLButtonElement {
  const b = document.createElement('button');
  b.type = 'button';
  b.textContent = texto;
  if (principal) b.className = 'principal';
  b.addEventListener('click', ao);
  return b;
}

export function botoes(...lista: HTMLButtonElement[]): HTMLElement {
  const c = document.createElement('div');
  c.className = 'botoes';
  c.append(...lista);
  return c;
}

/** Referências → Sumário personalizado: até que nível, com número de página e pontilhado. */
export function caixaDeSumario(aplicar: (niveis: number, numeros: boolean, pontilhado: boolean) => void): void {
  const d: Dialogo = new Dialogo(t('cmd.sumario_personalizado'), rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const niveis = document.createElement('select');
    for (let n = 1; n <= 6; n++) niveis.add(new Option(String(n), String(n), false, n === 3));
    const numeros = marcar(t('ref.mostrar_numeros'), true);
    const pontilhado = marcar(t('ref.pontilhado'), true);
    numeros.caixa.addEventListener('change', () => (pontilhado.caixa.disabled = !numeros.caixa.checked));
    const ok = (): void => {
      d.fechar();
      aplicar(Number(niveis.value), numeros.caixa.checked, pontilhado.caixa.checked);
    };
    form.append(rotulado(t('ref.niveis'), niveis), numeros.el, pontilhado.el, botoes(botao(t('estilo.cancelar'), () => d.fechar()), botao(t('estilo.ok'), ok, true)));
    corpo.append(form);
  });
  d.abrir();
}

/**
 * Inserir → Marcador: um nome para o parágrafo do cursor; a lista dos que existem, com "Ir para" e
 * "Excluir". `adicionar` devolve a mensagem de erro do motor (ou `null`).
 */
export function caixaDeMarcador(
  existentes: AlvoDeReferencia[],
  adicionar: (nome: string) => string | null,
  irPara: (id: number) => void,
  remover: (nome: string) => void,
): void {
  const d: Dialogo = new Dialogo(t('cmd.marcador'), rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const nome = document.createElement('input');
    nome.type = 'text';
    nome.placeholder = t('ref.nome_marcador');
    const lista = document.createElement('select');
    lista.size = 8;
    for (const m of existentes) lista.add(new Option(`${m.texto} — ${t('ref.pagina_n', { n: m.pagina })}`, m.texto));
    lista.addEventListener('change', () => (nome.value = lista.value));
    const erro = document.createElement('div');
    erro.className = 'erro';
    const escolhido = (): AlvoDeReferencia | undefined => existentes.find((m) => m.texto === (lista.value || nome.value.trim()));
    const ok = (): void => {
      const msg = adicionar(nome.value);
      if (msg) erro.textContent = msg;
      else d.fechar();
    };
    nome.addEventListener('keydown', (e) => e.key === 'Enter' && ok());
    form.append(
      rotulado(t('ref.nome'), nome),
      lista,
      erro,
      botoes(
        botao(t('ref.excluir'), () => {
          const m = escolhido();
          if (!m) return;
          d.fechar();
          remover(m.texto);
        }),
        botao(t('ref.ir_para'), () => {
          const m = escolhido();
          if (!m) return;
          d.fechar();
          irPara(m.id);
        }),
        botao(t('ref.adicionar'), ok, true),
      ),
    );
    corpo.append(form);
    setTimeout(() => nome.focus());
  });
  d.abrir();
}

const TIPOS: Array<AlvoDeReferencia['tipo']> = ['titulo', 'figura', 'tabela', 'equacao', 'marcador'];

/** Referências → Referência cruzada: o tipo, o item e o que mostrar (texto, número ou página). */
export function caixaDeReferencia(alvos: AlvoDeReferencia[], inserir: (id: number, modo: 'texto' | 'numero' | 'pagina') => string | null): void {
  const d: Dialogo = new Dialogo(t('cmd.referencia_cruzada'), rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const tipo = document.createElement('select');
    for (const tp of TIPOS) tipo.add(new Option(t(`ref.tipo_${tp}`), tp));
    const modo = document.createElement('select');
    const lista = document.createElement('select');
    lista.size = 9;
    const erro = document.createElement('div');
    erro.className = 'erro';
    const encher = (): void => {
      lista.replaceChildren();
      for (const a of alvos.filter((x) => x.tipo === tipo.value)) {
        const prefixo = a.numero ? `${a.numero}  ` : '';
        lista.add(new Option(`${prefixo}${a.texto}`, String(a.id)));
      }
      if (lista.options.length > 0) lista.selectedIndex = 0;
      const modos: Array<['texto' | 'numero' | 'pagina', string]> =
        tipo.value === 'marcador'
          ? [['texto', t('ref.modo_marcador')], ['pagina', t('ref.modo_pagina')]]
          : [['texto', t('ref.modo_texto')], ['numero', t('ref.modo_numero')], ['pagina', t('ref.modo_pagina')]];
      modo.replaceChildren(...modos.map(([v, r]) => new Option(r, v)));
      erro.textContent = lista.options.length === 0 ? t('ref.nada') : '';
    };
    tipo.addEventListener('change', encher);
    const ok = (): void => {
      if (!lista.value) return;
      const msg = inserir(Number(lista.value), modo.value as 'texto' | 'numero' | 'pagina');
      if (msg) erro.textContent = msg;
      else d.fechar();
    };
    lista.addEventListener('dblclick', ok);
    const lado = document.createElement('div');
    lado.className = 'lado';
    lado.append(rotulado(t('ref.tipo'), tipo), rotulado(t('ref.inserir_para'), modo));
    form.append(lado, lista, erro, botoes(botao(t('estilo.cancelar'), () => d.fechar()), botao(t('ref.inserir'), ok, true)));
    corpo.append(form);
    encher();
  });
  d.abrir();
}
