/**
 * O tema: **automático** (segue o sistema), **claro** ou **escuro** — o mesmo do
 * FeatherSheet. A escolha fica em `office.theme`, a chave da **suíte** (não do
 * app), para os aplicativos acompanharem uns aos outros. O tema é o atributo
 * `data-theme` do `<html>`; todo componente só usa as variáveis de cor de
 * `base.ts`, nunca uma cor solta. Vale também para as janelas à parte (Monitor):
 * elas ouvem o evento `storage` e trocam junto.
 */

import { PALETAS, paletaPorId, variaveisCSS } from './paletas.ts';

export type ModoDoTema = 'auto' | 'light' | 'dark';

/** A ordem em que o botão gira. */
export const ORDEM_DOS_TEMAS: ModoDoTema[] = ['auto', 'light', 'dark'];

export const CHAVE_DO_TEMA = 'office.theme';

const CONSULTA_ESCURO = '(prefers-color-scheme: dark)';

/** O modo guardado (`auto` se nada, ou se o armazenamento falhar). */
export function modoSalvo(): ModoDoTema {
  try {
    const v = localStorage.getItem(CHAVE_DO_TEMA);
    return ORDEM_DOS_TEMAS.find((m) => m === v) ?? 'auto';
  } catch {
    return 'auto';
  }
}

/** O próximo modo na ordem (auto → claro → escuro → auto). */
export function proximoModo(atual: ModoDoTema): ModoDoTema {
  return ORDEM_DOS_TEMAS[(ORDEM_DOS_TEMAS.indexOf(atual) + 1) % ORDEM_DOS_TEMAS.length]!;
}

/** O tema é escuro? O automático pergunta ao sistema. */
export function ehEscuro(modo: ModoDoTema, sistemaEscuro: boolean): boolean {
  return modo === 'auto' ? sistemaEscuro : modo === 'dark';
}

type Ouvinte = () => void;
const ouvintes: Ouvinte[] = [];

/** Avisa `f` sempre que o tema visível mudar (quem desenha em canvas precisa reler as cores). */
export function aoMudarTema(f: Ouvinte): void {
  ouvintes.push(f);
}

const sistemaEscuro = (): boolean => window.matchMedia?.(CONSULTA_ESCURO).matches ?? true;

/** O tema visível agora. */
export function temaVisivel(): 'light' | 'dark' {
  return document.documentElement.dataset['theme'] === 'light' ? 'light' : 'dark';
}

export const CHAVE_DA_PALETA = 'featherdocs.paleta';

/** O id da paleta guardada (`padrao` se nada). */
export function paletaSalva(): string {
  try {
    const v = localStorage.getItem(CHAVE_DA_PALETA);
    return PALETAS.find((p) => p.id === v)?.id ?? 'padrao';
  } catch {
    return 'padrao';
  }
}

/** Escolhe e guarda a paleta; as cores da casca (`--chrome-*`) são refeitas. */
export function definirPaleta(id: string): void {
  try {
    localStorage.setItem(CHAVE_DA_PALETA, id);
  } catch {
    // sem armazenamento: vale até fechar a janela
  }
  aplicarModo(modoSalvo());
}

export const ACENTOS: [string, string, string, string][] = [
  ['Azul', '#0b57d0', '#8ab4f8', '#d3e3fd'],
  ['Verde', '#0f8c57', '#7fd0a6', '#d7f0e2'],
  ['Roxo', '#6b3fa0', '#c6a8ec', '#e8dcf7'],
  ['Laranja', '#b35309', '#f5b183', '#fde4cf'],
  ['Rosa', '#b3255e', '#f2a0c0', '#fbd9e6'],
  ['Grafite', '#3c4043', '#b0b5ba', '#e1e3e5'],
];

export const CHAVE_DO_ACENTO = 'featherdocs.acento';
export const CHAVE_DA_FONTE = 'featherdocs.fonte';
export const CHAVE_DO_CORPO = 'featherdocs.corpo';
export const CHAVE_DO_ZOOM = 'featherdocs.zoom';

export function acentoSalvo(): number {
  try {
    const v = localStorage.getItem(CHAVE_DO_ACENTO);
    return v !== null ? Number(v) : -1;
  } catch {
    return -1;
  }
}

export function definirAcento(indice: number): void {
  try {
    if (indice < 0) localStorage.removeItem(CHAVE_DO_ACENTO);
    else localStorage.setItem(CHAVE_DO_ACENTO, String(indice));
  } catch {
    // sem armazenamento
  }
  aplicarModo(modoSalvo());
}

export function fonteSalva(): string {
  try {
    return localStorage.getItem(CHAVE_DA_FONTE) || 'sans';
  } catch {
    return 'sans';
  }
}

export function definirFonte(fonte: string): void {
  try {
    localStorage.setItem(CHAVE_DA_FONTE, fonte);
  } catch {}
}

export function corpoSalvo(): number {
  try {
    const v = localStorage.getItem(CHAVE_DO_CORPO);
    return v ? Number(v) : 11;
  } catch {
    return 11;
  }
}

export function definirCorpo(n: number): void {
  try {
    localStorage.setItem(CHAVE_DO_CORPO, String(n));
  } catch {}
}

export function zoomSalvo(): number {
  try {
    const v = localStorage.getItem(CHAVE_DO_ZOOM);
    return v ? Number(v) : 1;
  } catch {
    return 1;
  }
}

export function definirZoomSalvo(z: number): void {
  try {
    localStorage.setItem(CHAVE_DO_ZOOM, String(z));
  } catch {}
}

/** Aplica o modo ao `<html>` e avisa quem desenha. */
export function aplicarModo(modo: ModoDoTema): void {
  const antes = document.documentElement.dataset['theme'];
  const depois = ehEscuro(modo, sistemaEscuro()) ? 'dark' : 'light';
  document.documentElement.dataset['theme'] = depois;
  const paleta = paletaPorId(paletaSalva());
  const escuro = depois === 'dark';
  const base = { ...(escuro ? paleta.escuro : paleta.claro) };
  const acentoIdx = acentoSalvo();
  if (acentoIdx >= 0 && ACENTOS[acentoIdx]) {
    base.acento = escuro ? ACENTOS[acentoIdx][2] : ACENTOS[acentoIdx][1];
  }
  for (const [k, v] of Object.entries(variaveisCSS(base, escuro))) {
    document.documentElement.style.setProperty(k, v);
  }
  if (antes !== depois) for (const f of ouvintes) f();
}

/** Escolhe e guarda o modo. */
export function definirModo(modo: ModoDoTema): void {
  try {
    localStorage.setItem(CHAVE_DO_TEMA, modo);
  } catch {
    // sem armazenamento: vale até fechar a janela
  }
  aplicarModo(modo);
}

/**
 * Liga o tema na página: aplica o guardado, segue o sistema no modo automático e
 * acompanha a troca feita em outra janela do mesmo app. Chamar uma vez por página.
 */
export function iniciarTema(): void {
  aplicarModo(modoSalvo());
  window.matchMedia?.(CONSULTA_ESCURO).addEventListener('change', () => aplicarModo(modoSalvo()));
  window.addEventListener('storage', (e) => {
    if (e.key === CHAVE_DO_TEMA) aplicarModo(modoSalvo());
  });
}

/** Lê uma variável de cor do tema (`--cursor`…) para desenhar em canvas. */
export function corDoTema(variavel: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(variavel).trim();
}
