/** Junta o CSS de todos os componentes numa folha só (nada de folha global solta). */

import { BARRA_CSS } from './barra.ts';
import { BASE_CSS } from './base.ts';
import { REGUA_CSS } from './regua.ts';
import { CONFIG_CSS } from './configuracoes.ts';
import { DIALOGO_CSS } from './dialogo.ts';
import { INICIO_CSS } from './inicio.ts';
import { ENTRADA_CSS } from './entrada.ts';
import { FOLHAS_CSS } from './folhas.ts';
import { ESTRUTURA_CSS } from './painel-estrutura.ts';
import { RODAPE_CSS } from './rodape.ts';
import { TOAST_CSS } from './toast.ts';
import { BUSCA_CSS } from './busca.ts';
import { PALETA_CSS } from './paleta.ts';
import { CAIXAS_CSS } from './caixas.ts';
import { CAIXA_ESTILO_CSS } from './caixa-estilo.ts';
import { CODIGO_CSS } from './painel-codigo.ts';
import { CAIXAS_REFERENCIAS_CSS } from './caixas-referencias.ts';
import { NOTAS_CSS } from './notas.ts';
import { CITACOES_CSS } from './citacoes.ts';
import { REVISAO_CSS } from './painel-revisao.ts';
import { VERSOES_CSS } from './versoes.ts';
import { MINIATURAS_CSS } from './painel-miniaturas.ts';

export const TODO_O_CSS = [BASE_CSS, BARRA_CSS, ESTRUTURA_CSS, FOLHAS_CSS, ENTRADA_CSS, RODAPE_CSS, TOAST_CSS, INICIO_CSS, DIALOGO_CSS, CONFIG_CSS, REGUA_CSS, PALETA_CSS, BUSCA_CSS, CAIXAS_CSS, CAIXA_ESTILO_CSS, CODIGO_CSS, CAIXAS_REFERENCIAS_CSS, NOTAS_CSS, CITACOES_CSS, REVISAO_CSS, VERSOES_CSS, MINIATURAS_CSS].join('\n');

/** Põe o CSS na página (uma vez). */
export function injetarEstilos(): void {
  if (document.getElementById('featherdocs-css')) return;
  const s = document.createElement('style');
  s.id = 'featherdocs-css';
  s.textContent = TODO_O_CSS;
  document.head.append(s);
}
