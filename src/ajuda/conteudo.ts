/**
 * O que a Ajuda tem: a ordem dos tópicos do "Como usar" e os atalhos, com a tecla de cada um. O nome de cada
 * tópico e de cada atalho, e as introduções, estão nos arquivos de língua (`src/idiomas/*.json`, em `ajuda`).
 * Ao mudar uma tecla no motor (`teclado.rs`), mude aqui — o teste confere que todo nome tem texto.
 */

export const TOPICOS = ['comecar', 'formatar', 'estrutura', 'tabelas', 'pagina', 'arquivos', 'configuracoes'] as const;

export interface Atalho {
  /** A chave do nome em `ajuda.teclas`. */
  id: string;
  tecla: string;
}

export interface GrupoDeAtalhos {
  /** A chave do título em `ajuda.rotulos` (`g_…`) e o id do subitem no menu. */
  id: 'movimento' | 'edicao' | 'formato' | 'arquivo' | 'janela';
  atalhos: Atalho[];
}

export const GRUPOS_DE_ATALHOS: GrupoDeAtalhos[] = [
  {
    id: 'movimento',
    atalhos: [
      { id: 'setas', tecla: '← ↑ → ↓' },
      { id: 'palavra', tecla: 'Ctrl + ← →' },
      { id: 'linha', tecla: 'Home / End' },
      { id: 'documento', tecla: 'Ctrl+Home / Ctrl+End' },
      { id: 'pagina', tecla: 'PgUp / PgDn' },
      { id: 'selecionar', tecla: 'Shift + qualquer um acima' },
    ],
  },
  {
    id: 'edicao',
    atalhos: [
      { id: 'desfazer', tecla: 'Ctrl+Z' },
      { id: 'refazer', tecla: 'Ctrl+Y / Ctrl+Shift+Z' },
      { id: 'selTudo', tecla: 'Ctrl+A' },
      { id: 'area', tecla: 'Ctrl+C / Ctrl+X / Ctrl+V' },
      { id: 'colarSem', tecla: 'Ctrl+Shift+V' },
      { id: 'apagar', tecla: 'Backspace / Delete' },
      { id: 'enter', tecla: 'Enter' },
      { id: 'tab', tecla: 'Tab / Shift+Tab' },
      { id: 'quebra', tecla: 'Ctrl+Enter' },
      { id: 'esc', tecla: 'Esc' },
    ],
  },
  {
    id: 'formato',
    atalhos: [
      { id: 'negrito', tecla: 'Ctrl+B' },
      { id: 'italico', tecla: 'Ctrl+I' },
      { id: 'sublinhado', tecla: 'Ctrl+U' },
      { id: 'codigo', tecla: 'Ctrl+E' },
      { id: 'tachado', tecla: 'Ctrl+Shift+X' },
      { id: 'lista', tecla: 'Ctrl+Shift+8' },
      { id: 'listaNum', tecla: 'Ctrl+Shift+7' },
      { id: 'titulos', tecla: 'Ctrl+Alt+0 … 6' },
    ],
  },
  {
    id: 'arquivo',
    atalhos: [
      { id: 'novo', tecla: 'Ctrl+N' },
      { id: 'abrir', tecla: 'Ctrl+O' },
      { id: 'salvar', tecla: 'Ctrl+S' },
      { id: 'salvarComo', tecla: 'Ctrl+Shift+S' },
    ],
  },
  {
    id: 'janela',
    atalhos: [
      { id: 'ajuda', tecla: 'F1' },
      { id: 'fechar', tecla: 'Esc' },
    ],
  },
];
