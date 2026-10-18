/**
 * A largada: escolhe o canal até o motor (o `ponte://` do aplicativo, ou o mesmo
 * motor em WebAssembly no navegador) e monta a janela.
 */

import { codigoAtual, t, usarIdioma } from './idiomas/index.ts';
import { canalTauri, canalWasm, type Canal } from './ponte/canal.ts';
import { Motor } from './ponte/index.ts';
import { iniciarApp } from './ui/app.ts';
import { carregarFontes } from './ui/fontes.ts';
import { injetarEstilos } from './ui/styles.ts';
import { iniciarTema } from './ui/tema.ts';

async function escolherCanal(): Promise<Canal> {
  if ('__TAURI_INTERNALS__' in window) return canalTauri();
  // No navegador (`npm run dev`) o motor Rust vem como WebAssembly.
  const url = (await import('../crates/featherdocs-ponte/target/wasm32-unknown-unknown/release/featherdocs_ponte.wasm?url')).default;
  const modulo = await WebAssembly.compileStreaming(fetch(url));
  return canalWasm(modulo);
}

usarIdioma(codigoAtual());
// O tema antes de tudo, para a janela não piscar na cor errada.
injetarEstilos();
iniciarTema();

await carregarFontes();
const raiz = document.getElementById('app');
if (!raiz) throw new Error('faltou o #app no index.html');
const motor = new Motor(await escolherCanal());
const faltando = motor.comandosFaltando();
if (faltando.length > 0) {
  raiz.textContent = t('erros.motor_desatualizado', { lista: faltando.join(', ') });
} else {
  iniciarApp(raiz, motor);
}
