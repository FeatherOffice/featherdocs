/**
 * As caixas das notas: escrever (ou corrigir) o texto de uma nota de rodapé ou de fim, e o formato
 * do número. O texto vai como fonte em linha (`**negrito**`, `*itálico*`); quem numera, compõe e
 * põe a nota no pé da página é o motor.
 */

import { t } from '../idiomas/index.ts';
import { botao, botoes, rotulado } from './caixas-referencias.ts';
import { Dialogo, rotuloFechar } from './dialogo.ts';

export const NOTAS_CSS = `
.caixa-form textarea.nota { min-height: 90px; resize: vertical; padding: 6px 8px; border: 1px solid var(--borda); border-radius: 6px; background: var(--painel); color: inherit; font: inherit; font-size: 13px; }
.caixa-form .tipos-de-nota { display: flex; gap: 16px; font-size: 13px; }
.caixa-form .tipos-de-nota label { display: flex; flex-direction: row; align-items: center; gap: 6px; }
.caixa-form .dica { color: var(--texto-2, #5f6368); font-size: 12px; }
`;

/** O texto e o tipo de uma nota (a de fim vai para o fim do documento). */
export interface DadosDaNota {
  fonte: string;
  fim: boolean;
}

/**
 * Inserir ou editar uma nota. `aplicar` devolve a mensagem de erro do motor (ou `null`), que aparece
 * na própria caixa.
 */
export function caixaDeNota(titulo: string, inicial: DadosDaNota, aplicar: (d: DadosDaNota) => string | null): void {
  const d: Dialogo = new Dialogo(titulo, rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const texto = document.createElement('textarea');
    texto.className = 'nota';
    texto.value = inicial.fonte;
    const tipos = document.createElement('div');
    tipos.className = 'tipos-de-nota';
    const opcao = (rotulo: string, fim: boolean): HTMLInputElement => {
      const l = document.createElement('label');
      const r = document.createElement('input');
      r.type = 'radio';
      r.name = 'tipo-de-nota';
      r.checked = inicial.fim === fim;
      l.append(r, document.createTextNode(` ${rotulo}`));
      tipos.append(l);
      return r;
    };
    opcao(t('nota.rodape'), false);
    const fim = opcao(t('nota.fim'), true);
    const dica = document.createElement('div');
    dica.className = 'dica';
    dica.textContent = t('nota.dica');
    const erro = document.createElement('div');
    erro.className = 'erro';
    const ok = (): void => {
      const msg = aplicar({ fonte: texto.value, fim: fim.checked });
      if (msg) erro.textContent = msg;
      else d.fechar();
    };
    texto.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) ok();
    });
    form.append(rotulado(t('nota.texto'), texto), tipos, dica, erro, botoes(botao(t('estilo.cancelar'), () => d.fechar()), botao(t('estilo.ok'), ok, true)));
    corpo.append(form);
    setTimeout(() => texto.focus());
  });
  d.abrir();
}

/** Os formatos do número: 1, 2, 3 · a, b, c · i, ii, iii · *, †, ‡. */
const FORMATOS = ['1, 2, 3', 'a, b, c', 'i, ii, iii', '*, †, ‡'];

/** Referências → Formato das notas: o número das de rodapé e das de fim. */
export function caixaDeFormatoDasNotas(rodape: number, fim: number, aplicar: (rodape: number, fim: number) => void): void {
  const d: Dialogo = new Dialogo(t('cmd.formato_notas'), rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form';
    const seletor = (valor: number): HTMLSelectElement => {
      const s = document.createElement('select');
      FORMATOS.forEach((f, k) => s.add(new Option(f, String(k), false, k === valor)));
      return s;
    };
    const r = seletor(rodape);
    const f = seletor(fim);
    const ok = (): void => {
      d.fechar();
      aplicar(Number(r.value), Number(f.value));
    };
    form.append(rotulado(t('nota.formato_rodape'), r), rotulado(t('nota.formato_fim'), f), botoes(botao(t('estilo.cancelar'), () => d.fechar()), botao(t('estilo.ok'), ok, true)));
    corpo.append(form);
  });
  d.abrir();
}
