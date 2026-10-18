/**
 * O pedido que a janela principal faz à janela da Ajuda: em que seção abrir e, opcionalmente, em que tópico.
 * As duas janelas têm a mesma origem, então o pedido vai pelo `localStorage` (para a Ajuda que ainda vai abrir
 * ler) e pelo `BroadcastChannel` (para a que já está aberta mudar na hora). Mesmo esquema do FeatherSheet.
 */

export type SecaoDaAjuda = 'uso' | 'atalhos';

export interface PedidoDeAjuda {
  secao: SecaoDaAjuda;
  /** O id de um tópico (`tabelas`, `pagina`…) para abrir já nele. */
  topico?: string;
  /** Marca de tempo: dois pedidos iguais seguidos ainda são dois pedidos. */
  quando: number;
}

export const CANAL_DA_AJUDA = 'featherdocs_ajuda';
const CHAVE_DO_PEDIDO = 'featherdocs.ajuda.pedido';

/** Deixa o pedido para a janela da Ajuda — a aberta e a que vai abrir. */
export function pedirAjuda(secao: SecaoDaAjuda, topico?: string): PedidoDeAjuda {
  const pedido: PedidoDeAjuda = { secao, quando: Date.now(), ...(topico ? { topico } : {}) };
  try {
    localStorage.setItem(CHAVE_DO_PEDIDO, JSON.stringify(pedido));
  } catch {
    // sem armazenamento: a Ajuda aberta ainda recebe pelo canal
  }
  if (typeof BroadcastChannel !== 'undefined') {
    const canal = new BroadcastChannel(CANAL_DA_AJUDA);
    canal.postMessage(pedido);
    canal.close();
  }
  return pedido;
}

/** O último pedido deixado, se houver (a Ajuda lê ao abrir). */
export function ultimoPedido(): PedidoDeAjuda | null {
  try {
    const bruto = localStorage.getItem(CHAVE_DO_PEDIDO);
    return bruto ? (JSON.parse(bruto) as PedidoDeAjuda) : null;
  } catch {
    return null;
  }
}
