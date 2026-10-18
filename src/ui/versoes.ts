/**
 * Revisão → Histórico (Versões…): as versões salvas dentro do `.fdoz` (o Git embutido), da mais
 * nova para a mais velha, com "Abrir" (restaura: salvar grava uma versão nova por cima) e
 * "Comparar com a atual" (marca no documento o que mudou desde ela).
 */

import { t } from '../idiomas/index.ts';
import type { VersaoDoArquivo } from '../ponte/nativo.ts';
import { botao, botoes } from './caixas-referencias.ts';
import { Dialogo, rotuloFechar } from './dialogo.ts';

export const VERSOES_CSS = `
.versoes { max-height: 360px; overflow-y: auto; border: 1px solid var(--borda); border-radius: 6px; }
.versoes .versao { display: flex; align-items: center; gap: 8px; padding: 8px 10px; border-bottom: 1px solid var(--borda); }
.versoes .versao:last-child { border-bottom: none; }
.versoes .versao .info { flex: 1; min-width: 0; }
.versoes .versao .quando { font-weight: 600; }
.versoes .versao .msg { color: var(--texto-2); font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
`;

/** Os segundos desde 1970 em data e hora da língua da interface. */
function dataDe(segundos: number): string {
  return new Date(segundos * 1000).toLocaleString();
}

export function caixaDeVersoes(versoes: VersaoDoArquivo[], abrir: (id: string) => void, comparar: (id: string) => void): void {
  const d: Dialogo = new Dialogo(t('cmd.historico'), rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const lista = document.createElement('div');
    lista.className = 'versoes';
    for (const v of versoes) {
      const linha = document.createElement('div');
      linha.className = 'versao';
      const info = document.createElement('div');
      info.className = 'info';
      const quando = document.createElement('div');
      quando.className = 'quando';
      quando.textContent = dataDe(v.quando);
      const msg = document.createElement('div');
      msg.className = 'msg';
      msg.textContent = [v.autor, v.mensagem].filter(Boolean).join(' — ');
      info.append(quando, msg);
      linha.append(info, botao(t('versoes.comparar'), () => (d.fechar(), comparar(v.id))), botao(t('versoes.abrir'), () => (d.fechar(), abrir(v.id))));
      lista.append(linha);
    }
    if (!versoes.length) lista.textContent = t('versoes.nenhuma');
    form.append(lista, botoes(botao(t('estilo.ok'), () => d.fechar(), true)));
    corpo.append(form);
  });
  d.abrir();
}
