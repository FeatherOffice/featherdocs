/**
 * O menu do botão direito no papel, no lugar do menu do navegador. Os itens saem do registro
 * de comandos (o mesmo rótulo, ícone e atalho da faixa); o que aparece depende de onde está o
 * cursor: com link, também "Remover link"; numa tabela, as operações de tabela.
 */

import { t } from '../idiomas/index.ts';
import type { Formato } from '../ponte/tipos.ts';
import { itemDeMenu, type Registro } from './comandos.ts';
import type { MenuItem } from './menu.ts';

/** Os ids do menu, na ordem (`'|'` é separador). */
export function idsDoMenuDeContexto(f: Formato | null, comFeatherSheet: boolean): string[] {
  const ids = ['recortar', 'copiar', 'colar', 'colar_sem', '|', 'link'];
  if (f?.link) ids.push('link_remover');
  ids.push('|', 'estilos', 'entrelinha');
  if (f?.tabela != null) {
    ids.push('|', 'tab_linha_acima', 'tab_linha_abaixo', 'tab_coluna_esq', 'tab_coluna_dir', 'tab_excluir', 'tab_cabecalho');
    if (comFeatherSheet) ids.push('tab_feathersheet');
  }
  return ids;
}

/** Fora da faixa não há o nome do grupo ("Linhas e colunas"): estes itens ganham o rótulo inteiro. */
const ROTULO_PROPRIO = new Set(['tab_linha_acima', 'tab_linha_abaixo', 'tab_coluna_esq', 'tab_coluna_dir']);

/** Os itens prontos para o `openPopup`. */
export function menuDeContexto(r: Registro, f: Formato | null, comFeatherSheet: boolean): MenuItem[] {
  return idsDoMenuDeContexto(f, comFeatherSheet)
    .filter((id) => id === '|' || r.has(id))
    .map((id) => {
      const item = itemDeMenu(r, id);
      if (ROTULO_PROPRIO.has(id)) return { ...item, label: t(`contexto.${id}`) };
      if (id === 'link' && f?.link) return { ...item, label: t('contexto.link_editar') };
      return item;
    });
}
