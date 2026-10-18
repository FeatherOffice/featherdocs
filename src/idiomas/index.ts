/**
 * Os textos da interface (`t('arquivo.abrir')`). Nenhum texto de interface mora no
 * código: vai em `pt-BR.json` (a referência) e nas outras línguas, completas. O que
 * a língua não traduziu cai na referência.
 */

import enUS from './en-US.json' with { type: 'json' };
import ptBR from './pt-BR.json' with { type: 'json' };

export const REFERENCIA = 'pt-BR';
export const IDIOMAS: Record<string, unknown> = { 'pt-BR': ptBR, 'en-US': enUS };

const CHAVE = 'featherdocs.idioma';

function achatar(obj: unknown, prefixo = '', saida: Record<string, string> = {}): Record<string, string> {
  for (const [k, v] of Object.entries(obj as Record<string, unknown>)) {
    const chave = prefixo ? `${prefixo}.${k}` : k;
    if (typeof v === 'string') saida[chave] = v;
    else achatar(v, chave, saida);
  }
  return saida;
}

/** Todas as chaves de uma língua, achatadas (`arquivo.abrir` → texto). */
export function tabelaDe(codigo: string): Record<string, string> {
  return { ...achatar(IDIOMAS[REFERENCIA]), ...achatar(IDIOMAS[codigo] ?? {}) };
}

/** A língua em uso: a guardada (a mesma da suíte), senão a do sistema, senão a referência. */
export function codigoAtual(): string {
  try {
    const guardada = localStorage.getItem(CHAVE);
    if (guardada && guardada in IDIOMAS) return guardada;
  } catch {
    // sem armazenamento: segue para a do sistema
  }
  const sistema = typeof navigator === 'undefined' ? '' : navigator.language;
  return sistema in IDIOMAS ? sistema : REFERENCIA;
}

let atual = tabelaDe(REFERENCIA);

export type OuvinteIdioma = (codigo: string) => void;
const ouvintes: OuvinteIdioma[] = [];

/** Notifica sempre que a língua da interface mudar. */
export function aoMudarIdioma(ouvinte: OuvinteIdioma): () => void {
  ouvintes.push(ouvinte);
  return () => {
    const idx = ouvintes.indexOf(ouvinte);
    if (idx >= 0) ouvintes.splice(idx, 1);
  };
}

/** Troca a língua da interface e notifica os componentes. */
export function usarIdioma(codigo: string): void {
  atual = tabelaDe(codigo);
  for (const f of ouvintes) f(codigo);
}

/** Preenche `{nome}` no texto. */
export function formatar(texto: string, vars?: Record<string, string | number>): string {
  return vars ? texto.replace(/\{(\w+)\}/g, (m, k: string) => (k in vars ? String(vars[k]) : m)) : texto;
}

/** O texto da chave na língua em uso. Chave que não existe aparece como ela mesma (e o teste acusa). */
export function t(chave: string, vars?: Record<string, string | number>): string {
  return formatar(atual[chave] ?? chave, vars);
}

/** Guarda a língua no armazenamento local. */
export function guardarIdioma(codigo: string): void {
  try {
    localStorage.setItem(CHAVE, codigo);
  } catch {
    // sem armazenamento: vale só até fechar a janela
  }
}

/** Troca a língua e guarda a escolha para as próximas sessões. */
export function definirIdioma(codigo: string): void {
  guardarIdioma(codigo);
  usarIdioma(codigo);
}
