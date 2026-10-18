/**
 * O que só existe dentro da janela nativa (Tauri): abrir e salvar arquivo pelas
 * caixas do sistema e a ponte com o FeatherSheet. Vai direto em
 * `window.__TAURI_INTERNALS__.invoke`, sem o pacote `@tauri-apps/api` (a regra da
 * casa é não ter dependência de execução). No navegador nada disto existe: `noApp()` dá `false`.
 *
 * Isto NÃO é a ponte do motor (`canal.ts`, um XHR síncrono por comando): é a
 * conversa com a casca do aplicativo, e é assíncrona. Aqui só há disco e caixas;
 * o conteúdo do documento é do motor.
 */

import type { Estado, Salvo } from './tipos.ts';

interface TauriInternals {
  invoke(cmd: string, args?: Record<string, unknown>, options?: unknown): Promise<unknown>;
}

function internals(): TauriInternals | null {
  const g = globalThis as unknown as { __TAURI_INTERNALS__?: TauriInternals };
  return g.__TAURI_INTERNALS__ && typeof g.__TAURI_INTERNALS__.invoke === 'function' ? g.__TAURI_INTERNALS__ : null;
}

/** Estamos dentro da janela nativa? */
export function noApp(): boolean {
  return internals() !== null;
}

async function chamar<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const t = internals();
  if (!t) throw new Error('Esta janela não é o aplicativo (sem acesso à casca nativa)');
  return (await t.invoke(cmd, args)) as T;
}

/** Um documento aberto no disco. */
export interface Aberto {
  estado: Estado;
  nome: string;
  textoBruto?: string;
  texto_bruto?: string;
}

/** Abre um documento (a caixa de abrir, se não vier `caminho`). `null` se o usuário cancelou. */
export function abrirNoDisco(caminho?: string): Promise<Aberto | null> {
  return chamar('documento_abrir', { caminho: caminho ?? null });
}

/**
 * Grava no arquivo do documento (ou pergunta, se `como`, se ainda não há ou se ele é de outra vertente).
 * `texto`/`formato` são o código do modo dividido. `null` se cancelou; com `perdas`, falta confirmar.
 */
export function salvarNoDisco(como: boolean, texto?: string, formato?: 'md' | 'tex'): Promise<Salvo | null> {
  return chamar('documento_salvar', { como, texto: texto ?? null, formato: formato ?? null });
}

/** Grava no `caminho` que `salvarNoDisco` devolveu com `perdas`, depois que o usuário aceitou. */
export function salvarConfirmado(caminho: string, texto?: string, formato?: 'md' | 'tex'): Promise<Salvo> {
  return chamar('documento_salvar_confirmado', { caminho, texto: texto ?? null, formato: formato ?? null });
}

/** Exporta (`md`, `tex`, `docx`) pela caixa de salvar. Devolve o nome, ou `null` se cancelou. */
export function exportarNoDisco(formato: 'md' | 'tex' | 'docx', texto?: string): Promise<string | null> {
  return chamar('documento_exportar', { formato, texto: texto ?? null });
}

/** Abre um arquivo de texto (um `.bib`) pela caixa do sistema; `null` se cancelou. */
export function textoAbrir(filtro: string, extensoes: string[]): Promise<string | null> {
  return chamar('texto_abrir', { filtro, extensoes });
}

/** Grava um texto (um `.bib`) onde o usuário escolher; devolve o nome, ou `null` se cancelou. */
export function textoSalvar(nome: string, filtro: string, extensoes: string[], texto: string): Promise<string | null> {
  return chamar('texto_salvar', { nome, filtro, extensoes, texto });
}

/** Uma versão salva dentro do `.fdoz` (o Git embutido). */
export interface VersaoDoArquivo {
  id: string;
  /** Segundos desde 1970 (UTC). */
  quando: number;
  autor: string;
  mensagem: string;
}

/** As versões do `.fdoz` aberto, da mais nova para a mais velha (vazio se não é um `.fdoz` no disco). */
export function versoes(): Promise<VersaoDoArquivo[]> {
  return chamar('documento_versoes');
}

/** Abre (restaura) uma versão antiga do `.fdoz` aberto. */
export function abrirVersao(id: string): Promise<Aberto> {
  return chamar('documento_abrir_versao', { id });
}

/** Marca no documento aberto o que mudou desde a versão `id`; devolve a mudança. */
export function compararVersao(id: string): Promise<import('./tipos.ts').Mudanca> {
  return chamar('documento_comparar_versao', { id });
}

/** Um documento novo: esquece o caminho do anterior. */
export function esquecerCaminho(): Promise<void> {
  return chamar('documento_esquecer_caminho');
}

/** Grava a tabela (CSV) num arquivo temporário e abre no FeatherSheet. */
export function abrirTabelaNoFeatherSheet(id: number, csv: string): Promise<void> {
  return chamar('tabela_editar_no_feathersheet', { id, csv });
}

/** O CSV da tabela, se o FeatherSheet o salvou desde a última vez; senão `null`. */
export function verificarTabela(id: number): Promise<string | null> {
  return chamar('tabela_verificar', { id });
}

/** A janela da Ajuda (F1), própria no aplicativo. `false` no navegador: quem chama abre uma aba. */
export async function abrirJanelaAjuda(): Promise<boolean> {
  if (!noApp()) return false;
  await chamar<void>('abrir_janela_ajuda');
  return true;
}
