/** O que cada aba recebe para montar seus grupos: os construtores da `Barra` e o registro de comandos. */

import type { Registro } from '../comandos.ts';
import type { Contexto } from '../comandos-lista.ts';

export interface Kit {
  pequeno(rotulo: string, icone: string, ao: () => void, opcoes?: { soIcone?: boolean; classe?: string; ativo?: () => boolean; desativado?: () => boolean }): HTMLButtonElement;
  grande(rotulo: string, icone: string, ao: () => void, ativo?: () => boolean, desativado?: () => boolean): HTMLButtonElement;
  pilha(...linhas: HTMLElement[][]): HTMLElement;
  /** Liga um botão feito à mão (clique sem roubar o foco, marcado quando `ativo`). */
  ligar(b: HTMLButtonElement, ao: () => void, ativo?: () => boolean): void;
  /** Chama `f` toda vez que o estado ou o formato mudam (para pôr o valor nos campos). */
  aoSincronizar(f: () => void): void;
  /** Um botão do registro: `grande` (ícone em cima, rótulo embaixo), `pequeno` (ícone + rótulo) ou `icone`. */
  cmd(id: string, forma?: 'grande' | 'pequeno' | 'icone'): HTMLButtonElement;
  readonly registro: Registro;
}

export interface Grupo {
  titulo: string;
  itens: HTMLElement[];
}

/** Uma aba monta seus grupos com o kit e o contexto (ganchos, formato, estado). */
export type MontarAba = (k: Kit, c: Contexto) => Grupo[];
