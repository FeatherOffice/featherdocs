import { icon } from './icones.ts';
import { Z } from './z.ts';

export const MENU_CSS = `
.menubar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 0 18px 8px 17px;
  background: var(--chrome-bg);
}

.menu-list {
  display: flex;
  gap: 2px;
  align-items: center;
}

.quick-access-bar {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: auto;
}

.quick-tool {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 22px;
  padding: 0;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--chrome-ink-soft);
  cursor: pointer;
}

.quick-tool:hover { background: var(--chrome-hover); color: var(--chrome-ink); }
.quick-tool svg { width: 14px; height: 14px; }

.menu { position: relative; }

.menu-title {
  height: 22px;
  padding: 0 8px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--chrome-ink-soft);
  font-size: 11.5px;
  font-weight: 500;
  letter-spacing: 0.09em;
  text-transform: uppercase;
  cursor: pointer;
}

.menu-title:hover { color: var(--chrome-ink); background: var(--chrome-hover); }
.menu.on .menu-title { color: var(--chrome-ink); background: var(--chrome-active); }

.menu-panel {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  z-index: ${Z.menu};
  min-width: 272px;
  padding: 7px 0;
  border: 1px solid var(--rule-soft);
  border-radius: 8px;
  background: var(--surface);
  box-shadow: var(--shadow);
}

/* Só ganha rolagem o painel que realmente não cabe (ver fitPanel). */
.menu-panel.scrolls, .menu-sub.scrolls {
  overflow-y: auto;
  scrollbar-width: thin;
}

.menu-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 7px 14px 7px 12px;
  border: 0;
  background: transparent;
  color: var(--ink);
  font-size: 13.5px;
  text-align: left;
  cursor: pointer;
}

.menu-item:hover, .menu-item.open { background: var(--hover); }
.menu-item.disabled { color: var(--ink-faint); pointer-events: none; }
.menu-item.disabled .icon { opacity: 0.45; }

.menu-label { flex: 1; white-space: nowrap; }
.menu-item .icon { flex: none; color: var(--ink-soft); }

.menu-slot { display: grid; place-items: center; width: 16px; height: 16px; flex: none; }
.menu-check { color: var(--accent); font-size: 13px; }

.menu-item kbd {
  color: var(--ink-faint);
  font-family: var(--sans);
  font-size: 12px;
  white-space: nowrap;
}

.menu-arrow { color: var(--ink-faint); font-size: 11px; }
.menu-sep { height: 1px; margin: 6px 0; background: var(--rule-soft); }

.menu-sub {
  position: absolute;
  top: -8px;
  left: 100%;
  z-index: ${Z.menu + 1};
  min-width: 236px;
  padding: 7px 0;
  border: 1px solid var(--rule-soft);
  border-radius: 8px;
  background: var(--surface);
  box-shadow: var(--shadow);
}

.menu-sub.flip { left: auto; right: 100%; }
`;

export interface MenuItem {
  label?: string;
  /** Texto de ajuda mostrado ao parar o mouse sobre o item. */
  title?: string;
  /** Nome de um ícone de icons.ts. */
  icon?: string;
  /** Atalho exibido à direita, só informativo. */
  shortcut?: string;
  action?: () => void;
  /** Linha divisória em vez de item. */
  separator?: boolean;
  /** Itens filhos: o item vira submenu. */
  submenu?: MenuItem[] | (() => MenuItem[]);
  disabled?: () => boolean;
  checked?: () => boolean;
}

/**
 * Barra de menus com submenus. Abre no clique e, uma vez aberta, troca de menu
 * ao passar o mouse — é o que se espera de uma barra de menus.
 */
export class MenuBar {
  private readonly el = document.createElement('div');
  private readonly menuList = document.createElement('div');
  private readonly quickBar = document.createElement('div');
  private open: HTMLElement | null = null;
  public onToggleToolbar?: () => void;

  constructor(host: HTMLElement) {
    this.el.className = 'menubar';
    this.el.setAttribute('role', 'menubar');

    this.menuList.className = 'menu-list';
    this.quickBar.className = 'quick-access-bar';

    this.el.append(this.menuList, this.quickBar);
    host.append(this.el);

    document.addEventListener('mousedown', (e) => {
      if (!this.el.contains(e.target as Node)) this.close();
    });
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') this.close();
    });
  }

  add(title: string, items: MenuItem[] | (() => MenuItem[])): void {
    const wrap = document.createElement('div');
    wrap.className = 'menu';

    const button = document.createElement('button');
    button.className = 'menu-title';
    button.textContent = title;
    button.setAttribute('aria-haspopup', 'true');
    button.addEventListener('mousedown', (e) => {
      e.preventDefault();
      if (this.open === wrap) this.close();
      else this.show(wrap, items);
    });
    button.addEventListener('mouseenter', () => {
      if (this.open && this.open !== wrap) this.show(wrap, items);
    });
    button.addEventListener('dblclick', (e) => {
      e.preventDefault();
      this.onToggleToolbar?.();
    });

    wrap.append(button);
    this.menuList.append(wrap);
  }

  addQuickAction(iconName: string, title: string, action: () => void): HTMLButtonElement {
    const btn = document.createElement('button');
    btn.className = 'quick-tool';
    btn.title = title;
    btn.setAttribute('aria-label', title);
    btn.append(icon(iconName));
    btn.addEventListener('click', (e) => {
      e.preventDefault();
      action();
    });
    this.quickBar.append(btn);
    return btn;
  }

  /** Os itens são construídos na hora de abrir: estados sempre atuais. */
  private show(wrap: HTMLElement, items: MenuItem[] | (() => MenuItem[])): void {
    this.close();
    const panel = buildPanel(resolve(items), 'menu-panel', () => this.close());
    wrap.append(panel);
    fitPanel(panel);
    wrap.classList.add('on');
    this.open = wrap;
  }

  close(): void {
    if (!this.open) return;
    this.open.classList.remove('on');
    this.open.querySelector('.menu-panel')?.remove();
    this.open = null;
  }

  clear(): void {
    this.close();
    this.menuList.replaceChildren();
    this.quickBar.replaceChildren();
  }
}


function resolve(items: MenuItem[] | (() => MenuItem[])): MenuItem[] {
  return typeof items === 'function' ? items() : items;
}

function buildPanel(items: MenuItem[], className: string, close: () => void): HTMLElement {
  const panel = document.createElement('div');
  panel.className = className;
  panel.setAttribute('role', 'menu');
  for (const item of items) panel.append(buildItem(item, close));
  return panel;
}

/**
 * Um menu curto não deve ganhar barra de rolagem. Só quando a lista passa da
 * altura da janela é que limitamos e deixamos rolar — e a lista sobe para
 * aproveitar o espaço acima.
 */
function fitPanel(panel: HTMLElement): void {
  const margin = 12;
  const box = panel.getBoundingClientRect();
  const room = window.innerHeight - box.top - margin;
  if (box.height <= room) return;
  const above = box.top - margin;
  if (above > room && box.height <= above) {
    // Cabe inteiro se subir.
    panel.style.top = `${Math.max(margin, box.top - box.height)}px`;
    return;
  }
  panel.classList.add('scrolls');
  panel.style.maxHeight = `${Math.max(160, Math.max(room, above))}px`;
  if (above > room) panel.style.top = `${margin}px`;
}

function buildItem(item: MenuItem, close: () => void): HTMLElement {
  if (item.separator) {
    const hr = document.createElement('div');
    hr.className = 'menu-sep';
    return hr;
  }

  const row = document.createElement('button');
  row.className = 'menu-item';
  row.setAttribute('role', 'menuitem');
  const disabled = Boolean(item.disabled?.());
  if (disabled) row.classList.add('disabled');

  const slot = document.createElement('span');
  slot.className = 'menu-slot';
  if (item.checked?.()) {
    slot.textContent = '✓';
    slot.classList.add('menu-check');
  } else if (item.icon) {
    slot.append(icon(item.icon));
  }
  row.append(slot);

  const label = document.createElement('span');
  label.className = 'menu-label';
  label.textContent = item.label ?? '';
  row.append(label);
  if (item.title) row.title = item.title;

  if (item.shortcut) {
    const key = document.createElement('kbd');
    key.textContent = item.shortcut;
    row.append(key);
  }

  if (item.submenu) {
    const arrow = document.createElement('span');
    arrow.className = 'menu-arrow';
    arrow.textContent = '\u25b8';
    row.append(arrow);
    wireSubmenu(row, item.submenu, close);
    return row;
  }

  row.addEventListener('click', () => {
    if (disabled) return;
    close();
    item.action?.();
  });
  return row;
}

/** Abre o submenu ao passar o mouse e fecha ao sair do par item+painel. */
function wireSubmenu(row: HTMLElement, items: MenuItem[] | (() => MenuItem[]), close: () => void): void {
  let sub: HTMLElement | null = null;
  let timer = 0;

  const openSub = () => {
    window.clearTimeout(timer);
    if (sub) return;
    for (const other of row.parentElement?.querySelectorAll('.menu-sub') ?? []) other.remove();
    for (const other of row.parentElement?.querySelectorAll('.menu-item.open') ?? []) {
      other.classList.remove('open');
    }
    sub = buildPanel(resolve(items), 'menu-sub', close);
    row.classList.add('open');
    row.append(sub);
    if (sub.getBoundingClientRect().right > window.innerWidth - 8) sub.classList.add('flip');
    fitSubmenu(sub);
  };

  const closeSub = () => {
    timer = window.setTimeout(() => {
      sub?.remove();
      sub = null;
      row.classList.remove('open');
    }, 160);
  };

  row.addEventListener('mouseenter', openSub);
  row.addEventListener('mouseleave', closeSub);
  row.addEventListener('click', openSub);
}

/** Submenus são posicionados em relação ao item; a altura é ajustada aqui. */
function fitSubmenu(sub: HTMLElement): void {
  const margin = 12;
  const box = sub.getBoundingClientRect();
  if (box.bottom <= window.innerHeight - margin && box.top >= margin) return;
  const available = window.innerHeight - 2 * margin;
  if (box.height > available) {
    sub.classList.add('scrolls');
    sub.style.maxHeight = `${available}px`;
  }
  // Sobe o quanto for preciso para caber na janela.
  const height = Math.min(box.height, available);
  const overflow = box.top + height - (window.innerHeight - margin);
  if (overflow > 0) sub.style.top = `${parseFloat(getComputedStyle(sub).top) - overflow}px`;
}

/**
 * Menu solto ancorado num botão — usado pelos controles da barra de
 * ferramentas e pelo botão direito nas abas.
 */
export function openPopup(
  anchor: HTMLElement | { x: number; y: number },
  items: MenuItem[] | (() => MenuItem[]),
): void {
  const close = () => {
    panel.remove();
    document.removeEventListener('mousedown', dismiss, true);
    document.removeEventListener('keydown', onKey, true);
  };
  const panel = buildPanel(resolve(items), 'menu-panel', close);
  panel.style.position = 'fixed';
  panel.style.zIndex = String(Z.menu + 1);
  panel.style.visibility = 'hidden';
  document.body.append(panel);

  const box =
    anchor instanceof HTMLElement
      ? anchor.getBoundingClientRect()
      : { left: anchor.x, top: anchor.y, bottom: anchor.y };
  const size = panel.getBoundingClientRect();
  const left = Math.min(box.left, Math.max(8, window.innerWidth - size.width - 8));
  const below = box.bottom + 4;
  // Ancorado num botão, o menu vira para cima se não cabe embaixo; num ponto (botão direito),
  // sobe só o que falta, para continuar junto do clique.
  const top =
    below + size.height <= window.innerHeight - 8
      ? below
      : anchor instanceof HTMLElement
        ? Math.max(8, box.top - size.height - 4)
        : Math.max(8, window.innerHeight - 8 - size.height);
  panel.style.left = `${left}px`;
  panel.style.top = `${top}px`;
  panel.style.visibility = '';
  fitPanel(panel);

  const dismiss = (e: MouseEvent) => {
    if (panel.contains(e.target as Node)) return;
    if (anchor instanceof HTMLElement && anchor.contains(e.target as Node)) return;
    close();
  };
  const onKey = (e: KeyboardEvent) => {
    if (e.key === 'Escape') close();
  };
  // No próximo quadro: o clique que abriu não pode fechar em seguida.
  requestAnimationFrame(() => {
    document.addEventListener('mousedown', dismiss, true);
    document.addEventListener('keydown', onKey, true);
  });
}
