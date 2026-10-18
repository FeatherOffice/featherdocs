import { defineConfig } from 'vite';

// `vite dev` serve o index.html com HMR; `vite build` gera o `dist/` que o Tauri
// empacota. A porta é a do `devUrl` do `src-tauri/tauri.conf.json` (5175: o
// FeatherSheet usa 5173 e o FeatherCAD 5174, e os três abrem ao mesmo tempo).
export default defineConfig({
  base: './',
  build: { outDir: 'dist', target: 'es2022', rollupOptions: { input: { main: 'index.html', ajuda: 'ajuda.html' } } },
  server: {
    port: 5175,
    strictPort: true,
    watch: { ignored: ['**/crates/**', '**/src-tauri/target/**'] },
  },
});
