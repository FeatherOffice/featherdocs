/**
 * As cores e medidas do tema, como variáveis CSS. Todo componente usa só estas
 * variáveis, nunca uma cor solta (a folha de papel é a exceção: papel é branco nos
 * dois temas, como no Google Docs — o que se vê é o que se imprime).
 */

export const BASE_CSS = `
:root {
  --fundo: #f1f3f4;
  --painel: #ffffff;
  --painel-2: #f8f9fa;
  --borda: #dadce0;
  --texto: #202124;
  --texto-2: #5f6368;
  --acento: #1a73e8;
  --acento-fundo: #e8f0fe;
  --perigo: #d93025;
  --sombra: 0 1px 3px rgba(60, 64, 67, 0.3), 0 4px 8px 3px rgba(60, 64, 67, 0.15);
  --papel: #ffffff;
  --raio: 6px;
  --fonte-ui: system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
}
:root[data-theme='dark'] {
  --fundo: #1b1c1f;
  --painel: #2a2c30;
  --painel-2: #232529;
  --borda: #44474d;
  --texto: #e8eaed;
  --texto-2: #9aa0a6;
  --acento: #8ab4f8;
  --acento-fundo: #2d3a52;
  --perigo: #f28b82;
  --sombra: 0 1px 3px rgba(0, 0, 0, 0.6), 0 4px 10px 3px rgba(0, 0, 0, 0.4);
}
* { box-sizing: border-box; }
[hidden] { display: none !important; }
html, body { margin: 0; height: 100%; background: var(--fundo); color: var(--texto); font: 13px/1.4 var(--fonte-ui); overflow: hidden; }
#app { display: flex; flex-direction: column; height: 100%; }
button, select { font: inherit; color: inherit; }
button {
  background: transparent; border: 1px solid transparent; border-radius: var(--raio);
  padding: 4px 9px; cursor: pointer; min-height: 28px;
}
button:hover:not(:disabled) { background: var(--acento-fundo); }
button:disabled { opacity: 0.4; cursor: default; }
button:focus-visible, select:focus-visible { outline: 2px solid var(--acento); outline-offset: 1px; }
select {
  background: var(--painel); border: 1px solid var(--borda); border-radius: var(--raio);
  padding: 3px 6px; min-height: 28px;
}
`;
