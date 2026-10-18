/**
 * As caixas de diálogo da faixa: link, cabeçalho e rodapé, texto da marca d'água, contagem de
 * palavras, e a escolha de imagem do computador. Só coletam e mostram; quem guarda e valida é o motor.
 */

import { codigoAtual, t } from '../idiomas/index.ts';
import type { Contagem, Extras, Faixas } from '../ponte/tipos.ts';
import { Dialogo, rotuloFechar } from './dialogo.ts';
import { Z } from './z.ts';

export const CAIXAS_CSS = `
.caixa-form { display: flex; flex-direction: column; gap: 10px; min-width: 360px; }
.caixa-form label { display: flex; flex-direction: column; gap: 4px; font-size: 13px; }
.caixa-form input[type=text], .caixa-form input:not([type]) { padding: 6px 8px; border: 1px solid var(--borda); border-radius: 6px; background: var(--painel); color: inherit; font-size: 13px; }
.caixa-form .linha-3 { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
.caixa-form .linha-3 label span { font-size: 11.5px; opacity: 0.75; }
.caixa-form h3 { margin: 6px 0 0; font-size: 13.5px; }
.caixa-form .campos-chip { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; font-size: 12px; }
.caixa-form .campos-chip button { padding: 2px 10px; border: 1px solid var(--borda); border-radius: 12px; background: transparent; color: inherit; cursor: pointer; }
.caixa-form .botoes { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
.caixa-form .botoes button { padding: 6px 14px; border-radius: 6px; border: 1px solid var(--borda); background: transparent; color: inherit; cursor: pointer; }
.caixa-form .botoes button.principal { background: var(--chrome-accent, #1a73e8); border-color: transparent; color: #fff; }
.caixa-form label.marcar { flex-direction: row; align-items: center; gap: 8px; }
.contagem-tabela { border-collapse: collapse; min-width: 300px; font-size: 13.5px; }
.contagem-tabela td { padding: 6px 4px; border-bottom: 1px solid var(--borda); }
.contagem-tabela td:last-child { text-align: right; font-variant-numeric: tabular-nums; font-weight: 600; }
.sair-foco {
  position: fixed; top: 10px; right: 14px; z-index: ${Z.barra}; padding: 6px 12px; border: 1px solid var(--chrome-rule);
  border-radius: 16px; background: var(--surface); color: var(--chrome-ink); cursor: pointer; font-size: 12.5px; opacity: 0.6;
}
.sair-foco:hover { opacity: 1; }
.foco-ligado .casca, .foco-ligado .rodape, .foco-ligado .regua, .foco-ligado .estrutura { display: none !important; }
`;

function campo(rotulo: string, valor: string): { el: HTMLLabelElement; input: HTMLInputElement } {
  const el = document.createElement('label');
  const span = document.createElement('span');
  span.textContent = rotulo;
  const input = document.createElement('input');
  input.type = 'text';
  input.value = valor;
  el.append(span, input);
  return { el, input };
}

function botoes(...lista: Array<[string, () => void, boolean]>): HTMLElement {
  const caixa = document.createElement('div');
  caixa.className = 'botoes';
  for (const [rotulo, ao, principal] of lista) {
    const b = document.createElement('button');
    b.type = 'button';
    b.textContent = rotulo;
    if (principal) b.className = 'principal';
    b.addEventListener('click', ao);
    caixa.append(b);
  }
  return caixa;
}

/** A caixa de link (Ctrl+K): texto a exibir (só sem seleção) e endereço; com link no cursor, "Remover link". */
export function caixaDeLink(atual: string | null, temSelecao: boolean, aplicar: (url: string, texto: string) => void, remover: () => void): void {
  const d: Dialogo = new Dialogo(t('cmd.link'), rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const texto = campo(t('faixa.link_texto'), '');
    const url = campo(t('faixa.link_endereco'), atual ?? '');
    url.input.placeholder = 'https://…';
    if (!temSelecao && !atual) form.append(texto.el);
    form.append(url.el);
    const ok = (): void => {
      if (!url.input.value.trim()) return url.input.focus();
      d.fechar();
      aplicar(url.input.value, texto.input.value);
    };
    for (const i of [texto.input, url.input]) i.addEventListener('keydown', (e) => e.key === 'Enter' && ok());
    const lista: Array<[string, () => void, boolean]> = [];
    if (atual) lista.push([t('cmd.link_remover'), () => (d.fechar(), remover()), false]);
    lista.push([t('faixa.aplicar'), ok, true]);
    form.append(botoes(...lista));
    corpo.append(form);
    setTimeout(() => (form.querySelector('input') as HTMLInputElement | null)?.focus());
  });
  d.abrir();
}

/** Cabeçalho e rodapé: três lugares cada, com botões que inserem os campos, e "primeira página diferente". */
export function caixaDeCabecalho(x: Extras, foco: 'cabecalho' | 'rodape', aplicar: (cab: Faixas, rod: Faixas, primeira: boolean) => void): void {
  const d: Dialogo = new Dialogo(t('faixa.cab_titulo'), rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    let ultimo: HTMLInputElement | null = null;
    const secao = (titulo: string, f: Faixas): HTMLInputElement[] => {
      const h = document.createElement('h3');
      h.textContent = titulo;
      const linha = document.createElement('div');
      linha.className = 'linha-3';
      const ins = (['esquerda', 'centro', 'direita'] as const).map((lado) => {
        const c = campo(t(`faixa.${lado}`), f[lado]);
        c.input.addEventListener('focus', () => (ultimo = c.input));
        linha.append(c.el);
        return c.input;
      });
      form.append(h, linha);
      return ins;
    };
    const cab = secao(t('cmd.cabecalho'), x.cabecalho);
    const rod = secao(t('cmd.rodape'), x.rodape);
    const chips = document.createElement('div');
    chips.className = 'campos-chip';
    chips.append(document.createTextNode(t('faixa.cab_campos')));
    for (const [rotulo, campoTxt] of [['campo_pagina', '{pagina}'], ['campo_total', '{total}'], ['campo_titulo', '{titulo}'], ['campo_autor', '{autor}']] as const) {
      const b = document.createElement('button');
      b.type = 'button';
      b.textContent = t(`faixa.${rotulo}`);
      b.addEventListener('mousedown', (e) => e.preventDefault());
      b.addEventListener('click', () => {
        const alvo = ultimo ?? (foco === 'cabecalho' ? cab[1] : rod[1])!;
        const p = alvo.selectionStart ?? alvo.value.length;
        alvo.value = alvo.value.slice(0, p) + campoTxt + alvo.value.slice(alvo.selectionEnd ?? p);
        alvo.focus();
      });
      chips.append(b);
    }
    const primeira = document.createElement('label');
    primeira.className = 'marcar';
    const marca = document.createElement('input');
    marca.type = 'checkbox';
    marca.checked = x.primeira_diferente;
    primeira.append(marca, document.createTextNode(t('faixa.cab_primeira')));
    const faixas = (ins: HTMLInputElement[]): Faixas => ({ esquerda: ins[0]!.value, centro: ins[1]!.value, direita: ins[2]!.value });
    form.append(chips, primeira, botoes([t('faixa.aplicar'), () => (d.fechar(), aplicar(faixas(cab), faixas(rod), marca.checked)), true]));
    corpo.append(form);
    setTimeout(() => (foco === 'cabecalho' ? cab[1] : rod[1])?.focus());
  });
  d.abrir();
}

/** Uma pergunta de texto (a marca d'água personalizada). */
export function pedirTexto(titulo: string, rotulo: string, valor: string, aplicar: (s: string) => void): void {
  const d: Dialogo = new Dialogo(titulo, rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const c = campo(rotulo, valor);
    const ok = (): void => (d.fechar(), aplicar(c.input.value));
    c.input.addEventListener('keydown', (e) => e.key === 'Enter' && ok());
    form.append(c.el, botoes([t('faixa.aplicar'), ok, true]));
    corpo.append(form);
    setTimeout(() => c.input.select());
  });
  d.abrir();
}

/** Revisão → Contagem de palavras. */
export function caixaDeContagem(c: Contagem): void {
  const titulo = c.selecao ? `${t('faixa.contagem_titulo')} ${t('faixa.contagem_selecao')}` : t('faixa.contagem_titulo');
  const d = new Dialogo(titulo, rotuloFechar(), (corpo) => {
    const tabela = document.createElement('table');
    tabela.className = 'contagem-tabela';
    const linhas: Array<[string, number]> = [
      ['cont_paginas', c.paginas],
      ['cont_palavras', c.palavras],
      ['cont_caracteres', c.caracteres],
      ['cont_sem_espaco', c.caracteres_sem_espaco],
      ['cont_paragrafos', c.paragrafos],
      ['cont_linhas', c.linhas],
    ];
    for (const [chave, n] of linhas) {
      const tr = tabela.insertRow();
      tr.insertCell().textContent = t(`faixa.${chave}`);
      tr.insertCell().textContent = n.toLocaleString(codigoAtual());
    }
    corpo.append(tabela);
  });
  d.abrir();
}

/** Escolhe uma imagem do computador: data URL e tamanho natural em pontos (px × 0,75). */
export function escolherImagem(): Promise<{ origem: string; largura: number; altura: number } | null> {
  return new Promise((resolver) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/*';
    input.addEventListener('change', () => {
      const arquivo = input.files?.[0];
      if (!arquivo) return resolver(null);
      const leitor = new FileReader();
      leitor.onload = () => {
        const origem = String(leitor.result);
        const img = new Image();
        img.onload = () => resolver({ origem, largura: img.naturalWidth * 0.75, altura: img.naturalHeight * 0.75 });
        img.onerror = () => resolver({ origem, largura: 0, altura: 0 });
        img.src = origem;
      };
      leitor.onerror = () => resolver(null);
      leitor.readAsDataURL(arquivo);
    });
    input.click();
  });
}

/** A data (e hora) de agora no formato pedido, na língua da interface. */
export function dataFormatada(formato: 'curta' | 'longa' | 'hora' | 'iso', agora = new Date()): string {
  const lingua = codigoAtual();
  if (formato === 'iso') {
    const p = (n: number): string => String(n).padStart(2, '0');
    return `${agora.getFullYear()}-${p(agora.getMonth() + 1)}-${p(agora.getDate())}`;
  }
  if (formato === 'longa') return agora.toLocaleDateString(lingua, { day: 'numeric', month: 'long', year: 'numeric' });
  if (formato === 'hora') return agora.toLocaleString(lingua, { day: '2-digit', month: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit' });
  return agora.toLocaleDateString(lingua, { day: '2-digit', month: '2-digit', year: 'numeric' });
}
