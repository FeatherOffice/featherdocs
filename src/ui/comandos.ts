/**
 * O registro de comandos: **um** lugar para cada ação da interface (rótulo, ícone, atalho,
 * o que faz, quando aparece marcada). A faixa de opções e os menus suspensos montam seus
 * botões e itens daqui, então nunca divergem. Um comando com `emBreve` existe na especificação
 * mas ainda não tem motor: aparece desabilitado, com a dica do que vai fazer.
 */

import { t } from '../idiomas/index.ts';
import type { MenuItem } from './menu.ts';

export interface Comando {
  /** O texto já traduzido. */
  rotulo: string;
  icone: string;
  atalho?: string;
  executar?: () => void;
  ativo?: () => boolean;
  desativado?: () => boolean;
  /** Presente = ainda sem motor: desabilitado, com esta dica. */
  emBreve?: string;
  /** O comando abre um menu (na faixa, um botão com seta; nos menus, um submenu). */
  menu?: () => MenuItem[];
}

export type Registro = Map<string, Comando>;

/** A dica (o `title`) do botão: rótulo, atalho e, se for o caso, "Em breve: …". */
export function dicaDe(c: Comando): string {
  const base = c.atalho ? `${c.rotulo} (${c.atalho})` : c.rotulo;
  return c.emBreve ? `${base} — ${t('faixa.em_breve')}: ${c.emBreve}` : base;
}

/** O comando `id`; um id que não existe é erro de programação (o teste do registro pega). */
export function comando(r: Registro, id: string): Comando {
  const c = r.get(id);
  if (!c) throw new Error(`Comando desconhecido na interface: ${id}`);
  return c;
}

/** Um item de menu suspenso a partir do registro (`'|'` é separador). */
export function itemDeMenu(r: Registro, id: string): MenuItem {
  if (id === '|') return { separator: true };
  const c = comando(r, id);
  const item: MenuItem = { label: c.rotulo, icon: c.icone, shortcut: c.atalho, title: c.emBreve ? dicaDe(c) : undefined };
  if (c.emBreve) return { ...item, disabled: () => true };
  if (c.menu) return { ...item, submenu: c.menu };
  return { ...item, action: c.executar, checked: c.ativo, disabled: c.desativado };
}

/** Um item avulso "em breve" dentro de um menu (uma opção de um comando que já funciona em parte). */
export function itemEmBreve(rotulo: string): MenuItem {
  return { label: rotulo, title: t('faixa.em_breve'), disabled: () => true };
}
