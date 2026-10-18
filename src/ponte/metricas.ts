/**
 * Métricas de depuração (RECEITA.md §7): todo comando que atravessa o canal é
 * contado, e todo gesto/quadro imprime o total e quantos comandos disparou.
 *
 * `[motor]`  — um comando (só imprime o que passa de um quadro, 16 ms);
 * `[ação]`   — um gesto do usuário, com quantos comandos e quanto foi motor;
 * `[quadro]` — um desenho, com quantos comandos ele fez (mais de um é bug).
 */

import type { Canal } from './canal.ts';

export const LIMITE_MS = 16;

const acumulado = { comandos: 0, ms: 0 };

const agora = (): number => (typeof performance === 'undefined' ? Date.now() : performance.now());

/** O canal, contando cada comando que passa por ele. */
export function canalMedido(canal: Canal): Canal {
  return {
    chamar(comando, args) {
      const inicio = agora();
      try {
        return canal.chamar(comando, args);
      } finally {
        const ms = agora() - inicio;
        acumulado.comandos++;
        acumulado.ms += ms;
        if (ms > LIMITE_MS) console.log(`[motor] ${comando}: ${ms.toFixed(1)} ms`);
      }
    },
  };
}

/** Embrulha uma medida: devolve o resultado e imprime `[<tipo>] <nome>` se passou do limite. */
export function medir<T>(tipo: 'ação' | 'quadro', nome: string, fn: () => T, detalhe?: (r: T) => string): T {
  const antes = { ...acumulado };
  const inicio = agora();
  const r = fn();
  const total = agora() - inicio;
  const comandos = acumulado.comandos - antes.comandos;
  if (total > LIMITE_MS || (tipo === 'quadro' && comandos > 3)) {
    const motor = acumulado.ms - antes.ms;
    console.log(`[${tipo}] ${nome}: ${total.toFixed(1)} ms (${comandos} comando(s), motor ${motor.toFixed(1)} ms)${detalhe ? ' ' + detalhe(r) : ''}`);
  }
  return r;
}

/** Um gesto do usuário (clique, tecla, menu): mede e imprime se passar do limite. */
export function medirAcao<T>(nome: string, fn: () => T): T {
  return medir('ação', nome, fn);
}
