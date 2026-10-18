/**
 * Abrir, salvar e exportar. No aplicativo, é a casca nativa (caixas do sistema e
 * disco, `src-tauri/src/arquivos.rs`); no navegador (`npm run dev`), um seletor de
 * arquivo e um download. Em nenhum dos dois a interface olha dentro do arquivo: os
 * bytes vão ao motor, que lê (Markdown, LaTeX, .docx, .fdoz) e devolve o estado.
 */

import { t } from '../idiomas/index.ts';
import type { Motor } from '../ponte/index.ts';
import { abrirNoDisco, esquecerCaminho, exportarNoDisco, noApp, salvarConfirmado, salvarNoDisco, textoAbrir, textoSalvar, type Aberto } from '../ponte/nativo.ts';
import type { FormatoDeArquivo, Salvo } from '../ponte/tipos.ts';

const TIPO_DOCX = 'application/vnd.openxmlformats-officedocument.wordprocessingml.document';

const EXTENSOES: Record<string, FormatoDeArquivo> = { fdoz: 'fdoz', md: 'md', markdown: 'md', tex: 'tex', latex: 'tex', docx: 'docx' };

function baixar(nome: string, dados: BlobPart, tipo: string): void {
  const url = URL.createObjectURL(new Blob([dados], { type: tipo }));
  const a = document.createElement('a');
  a.href = url;
  a.download = nome;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function escolherArquivo(): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = Object.keys(EXTENSOES)
      .map((e) => `.${e}`)
      .join(',');
    input.addEventListener('change', () => resolve(input.files?.[0] ?? null));
    input.addEventListener('cancel', () => resolve(null));
    input.click();
  });
}

/** Um documento escolhido para comparar (não abre: os bytes e o formato vão ao motor). */
export async function escolherDocumento(): Promise<{ formato: FormatoDeArquivo; bytes: Uint8Array } | null> {
  const arq = await escolherArquivo();
  if (!arq) return null;
  const formato = EXTENSOES[arq.name.split('.').pop()?.toLowerCase() ?? ''];
  if (!formato) throw new Error(t('arquivo.formatos'));
  return { formato, bytes: new Uint8Array(await arq.arrayBuffer()) };
}

/** Abre um arquivo de texto com a extensão `ext` (um `.bib`): a caixa do sistema no aplicativo, o seletor no navegador. */
export async function abrirTexto(filtro: string, ext: string): Promise<string | null> {
  if (noApp()) return textoAbrir(filtro, [ext]);
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = `.${ext}`;
    input.addEventListener('change', () => {
      const arq = input.files?.[0];
      if (!arq) return resolve(null);
      void arq.text().then(resolve, () => resolve(null));
    });
    input.addEventListener('cancel', () => resolve(null));
    input.click();
  });
}

/** Grava um texto (um `.bib`): a caixa do sistema no aplicativo, um download no navegador. */
export async function salvarTexto(nome: string, filtro: string, ext: string, texto: string): Promise<string | null> {
  if (noApp()) return textoSalvar(nome, filtro, [ext], texto);
  baixar(nome, texto, 'text/plain;charset=utf-8');
  return nome;
}

export class Arquivos {
  private nomeNoNavegador = '';

  constructor(private readonly motor: Motor) {}

  /** O nome do documento, para o título da janela. */
  get nome(): string {
    return this.nomeNoNavegador || t('app.sem_titulo');
  }

  /** Esquece o arquivo do documento anterior. */
  async novo(): Promise<void> {
    this.nomeNoNavegador = '';
    if (noApp()) await esquecerCaminho();
  }

  /** Abre um documento. `null` se o usuário cancelou. */
  async abrir(): Promise<Aberto | null> {
    if (noApp()) return abrirNoDisco();
    const arq = await escolherArquivo();
    if (!arq) return null;
    const ext = arq.name.split('.').pop()?.toLowerCase() ?? '';
    const formato = EXTENSOES[ext];
    if (!formato) throw new Error(t('arquivo.formatos'));
    const buffer = await arq.arrayBuffer();
    const bytes = new Uint8Array(buffer);
    const estado = this.motor.abrir(formato, bytes);
    this.nomeNoNavegador = arq.name.replace(/\.[^.]+$/, '');
    let textoBruto: string | undefined;
    if (ext === 'tex' || ext === 'latex' || ext === 'md' || ext === 'markdown') {
      try {
        textoBruto = new TextDecoder().decode(bytes);
      } catch {
        // sem texto decodificado
      }
    }
    return { estado, nome: arq.name, textoBruto };
  }

  /**
   * Salva nos formatos da vertente do documento (convencional: `.fdoz`/`.docx`; código: `.tex`/`.md`).
   * `texto`/`formato` são o código do modo dividido. `null` se cancelou; com `perdas`, falta `confirmar`.
   * No navegador baixa no formato padrão da vertente.
   */
  async salvar(como: boolean, texto?: string, formato?: 'md' | 'tex'): Promise<Salvo | null> {
    if (noApp()) return salvarNoDisco(como, texto, formato);
    const padrao = this.motor.vertente().formatos[0] ?? 'fdoz';
    const nome = `${this.nome}.${padrao}`;
    if (padrao === 'tex' || padrao === 'md') {
      baixar(nome, formato === padrao && texto !== undefined ? texto : this.motor.exportarTexto(padrao), 'text/plain;charset=utf-8');
    } else if (padrao === 'docx') {
      baixar(nome, new Uint8Array(this.motor.gravarDocx()), TIPO_DOCX);
    } else {
      baixar(nome, new Uint8Array(this.motor.gravarFdoz()), 'application/zip');
    }
    this.motor.marcarGravado();
    return { nome, formato: padrao };
  }

  /** Grava depois que o usuário aceitou o aviso de conversão que `salvar` devolveu. */
  async confirmar(salvo: Salvo, texto?: string, formato?: 'md' | 'tex'): Promise<Salvo> {
    if (noApp() && salvo.caminho) return salvarConfirmado(salvo.caminho, texto, formato);
    if (salvo.formato !== 'md' && salvo.formato !== 'tex') throw new Error(t('arquivo.formatos'));
    baixar(salvo.nome, this.motor.converter(salvo.formato), 'text/plain;charset=utf-8');
    this.motor.marcarGravado();
    return { nome: salvo.nome, formato: salvo.formato };
  }

  /** Exporta como Markdown, LaTeX ou Word (não muda o arquivo nem a vertente do documento). */
  async exportar(formato: 'md' | 'tex' | 'docx', texto?: string): Promise<string | null> {
    if (noApp()) return exportarNoDisco(formato, texto);
    const nome = `${this.nome}.${formato}`;
    if (formato === 'docx') baixar(nome, new Uint8Array(this.motor.gravarDocx()), TIPO_DOCX);
    else baixar(nome, texto ?? this.motor.exportarTexto(formato), 'text/plain;charset=utf-8');
    return nome;
  }
}
