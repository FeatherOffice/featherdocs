/** O estilo da janela da Ajuda (`src/ajuda/pagina.ts`), sobre as variáveis de cor do app (copiado do FeatherSheet). */
export const AJUDA_CSS = String.raw`
body { margin: 0; background: var(--surface); color: var(--ink); font-family: var(--fonte-ui); font-size: 13px; }
.aj { display: flex; flex-direction: column; height: 100vh; }
.aj-topo { display: flex; align-items: center; gap: 16px; padding: 10px 16px; border-bottom: 1px solid var(--rule); }
.aj-topo h1 { font-size: 15px; margin: 0; white-space: nowrap; }
.aj-busca { flex: 1; max-width: 520px; padding: 6px 10px; border: 1px solid var(--rule); border-radius: 6px; background: var(--surface); color: var(--ink); font: inherit; }
.aj-busca:focus { outline: none; border-color: var(--accent); }
.aj-meio { flex: 1; min-height: 0; display: flex; }
.aj-nav { width: 200px; flex: none; padding: 10px 8px; border-right: 1px solid var(--rule); overflow-y: auto; display: flex; flex-direction: column; gap: 2px; }
.aj-nav button { text-align: left; border: 0; border-radius: 6px; background: none; color: var(--ink-soft); padding: 7px 10px; font: inherit; cursor: pointer; }
.aj-nav button:hover { background: var(--pill); }
.aj-nav button[aria-current="true"] { background: var(--pill); color: var(--accent); font-weight: 600; }
.aj-nav .aj-secao { font-weight: 600; color: var(--ink); }
.aj-nav .aj-secao::before { content: '▸'; display: inline-block; width: 14px; color: var(--ink-faint); }
.aj-nav .aj-secao[aria-expanded="true"]::before { content: '▾'; }
.aj-nav .aj-sub { padding: 5px 10px 5px 30px; font-size: 12.5px; }
.aj-intro { margin: 0 0 18px; max-width: 760px; line-height: 1.6; color: var(--ink-soft); font-size: 13.5px; }
.aj-indice { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 8px; max-width: 820px; }
.aj-cartao { text-align: left; padding: 10px 12px; border: 1px solid var(--rule); border-radius: 8px; background: var(--surface); color: var(--ink); font: inherit; cursor: pointer; }
.aj-cartao:hover { border-color: var(--accent); color: var(--accent); }
.aj-migalha { border: 0; background: none; padding: 0; margin: 0 0 4px; color: var(--accent); font: inherit; font-size: 12px; cursor: pointer; }
.aj-migalha:hover { text-decoration: underline; }
.aj-passos { display: flex; justify-content: space-between; gap: 12px; margin-top: 28px; padding-top: 14px; border-top: 1px solid var(--rule); max-width: 820px; }
.aj-passos button { padding: 7px 12px; border: 1px solid var(--rule); border-radius: 6px; background: var(--surface); color: var(--ink); font: inherit; cursor: pointer; max-width: 48%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.aj-passos button:hover { border-color: var(--accent); color: var(--accent); }
.aj-passos .aj-proximo { margin-left: auto; border-color: var(--accent); color: var(--accent); font-weight: 600; }
.aj-corpo { flex: 1; min-width: 0; overflow-y: auto; padding: 16px 24px 40px; }
.aj-corpo h2 { font-size: 18px; margin: 0 0 12px; font-weight: 600; }
.aj-corpo h3 { font-size: 12px; margin: 22px 0 8px; font-weight: 600; letter-spacing: .04em; text-transform: uppercase; color: var(--ink-faint); }
.aj-corpo h3:first-of-type { margin-top: 8px; }
.aj-topico { margin: 0 0 20px; max-width: 760px; }
.aj-topico h4 { font-size: 14px; margin: 0 0 6px; }
.aj-topico p { margin: 0 0 6px; line-height: 1.55; color: var(--ink-soft); }
.aj-topico ul { margin: 0 0 6px; padding-left: 20px; line-height: 1.55; color: var(--ink-soft); }
.aj-item { padding: 9px 0; border-bottom: 1px solid var(--rule-soft, var(--rule)); max-width: 820px; scroll-margin-top: 12px; }
.aj-item.aj-alvo { background: color-mix(in srgb, var(--accent) 10%, transparent); border-radius: 6px; padding-left: 8px; padding-right: 8px; }
.aj-assinatura { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12.5px; color: var(--ink); }
.aj-descricao { margin: 3px 0 0; color: var(--ink-soft); line-height: 1.45; }
.aj-apelidos { margin: 3px 0 0; color: var(--ink-faint); font-size: 12px; }
.aj-exemplo { margin: 6px 0 0; padding: 5px 8px; border: 1px solid var(--rule); border-radius: 6px; background: var(--pill); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 11.5px; white-space: pre-wrap; word-break: break-word; }
.aj-nota { color: var(--ink-faint); margin: 0 0 12px; line-height: 1.5; max-width: 760px; }
.aj-atalhos { border-collapse: collapse; width: 100%; max-width: 720px; }
.aj-atalhos td { padding: 6px 8px; border-bottom: 1px solid var(--rule-soft, var(--rule)); }
.aj-atalhos td:last-child { text-align: right; white-space: nowrap; }
.aj-atalhos code { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12px; padding: 1px 6px; border: 1px solid var(--rule); border-radius: 4px; background: var(--pill); }
.aj-vazio { color: var(--ink-faint); padding: 20px 0; }
`;
