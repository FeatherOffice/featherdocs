/**
 * A tabela editada no FeatherSheet, como o Google Docs faz com o Excel: o motor
 * entrega a tabela em CSV, a casca grava num arquivo temporário e abre no
 * FeatherSheet; enquanto a janela de lá estiver aberta, a cada salvar o CSV volta
 * ao motor, que troca as células. Nenhum app importa código do outro — o contrato é
 * o arquivo. (O vínculo direto com um `.fshz` e com consultas do OpenQuery pede o
 * crate comum de `featherdocs/docs/ARQUITETURA.md`, ainda não criado.)
 */

import { t } from '../idiomas/index.ts';
import type { Motor } from '../ponte/index.ts';
import { abrirTabelaNoFeatherSheet, verificarTabela } from '../ponte/nativo.ts';
import type { Mudanca } from '../ponte/tipos.ts';

const INTERVALO_MS = 900;

export class LigacaoComFeatherSheet {
  private readonly ativas = new Set<number>();
  private relogio = 0;
  private verificando = false;

  constructor(
    private readonly motor: Motor,
    private readonly aoMudar: (m: Mudanca) => void,
    private readonly avisar: (texto: string, erro?: boolean) => void,
  ) {}

  /** Abre a tabela do bloco `id` no FeatherSheet e passa a escutar as gravações de lá. */
  async editar(id: number): Promise<void> {
    try {
      await abrirTabelaNoFeatherSheet(id, this.motor.tabelaCsv(id));
    } catch (e) {
      this.avisar(t('editor.feathersheet_falhou', { erro: String(e) }), true);
      return;
    }
    this.ativas.add(id);
    this.avisar(t('editor.aguardando_feathersheet'));
    if (!this.relogio) this.relogio = window.setInterval(() => void this.verificar(), INTERVALO_MS);
  }

  private async verificar(): Promise<void> {
    if (this.verificando) return;
    this.verificando = true;
    try {
      for (const id of [...this.ativas]) await this.verificarUma(id);
    } finally {
      this.verificando = false;
    }
  }

  private async verificarUma(id: number): Promise<void> {
    try {
      const csv = await verificarTabela(id);
      if (csv === null) return;
      this.aoMudar(this.motor.tabelaCsvSubstituir(id, csv));
      this.avisar(t('editor.tabela_atualizada'));
    } catch {
      // o bloco foi apagado, ou o arquivo sumiu: para de escutar esta tabela
      this.ativas.delete(id);
      if (this.ativas.size === 0) {
        clearInterval(this.relogio);
        this.relogio = 0;
      }
    }
  }
}
