/**
 * Abrir a Ajuda (F1) a partir da janela principal: no aplicativo, a janela própria (`ajuda.html`, como o Monitor);
 * no navegador, uma aba com a mesma página. Onde ela abre vai por `ajuda/pedido.ts`.
 */

import { pedirAjuda, type SecaoDaAjuda } from '../ajuda/pedido.ts';
import { abrirJanelaAjuda } from '../ponte/nativo.ts';

/** Abre a Ajuda na seção (e no tópico) pedidos. */
export function abrirAjuda(secao: SecaoDaAjuda = 'uso', topico?: string): void {
  pedirAjuda(secao, topico);
  void abrirJanelaAjuda().then((aberta) => {
    if (!aberta) window.open('ajuda.html', 'featherdocs-ajuda');
  });
}
