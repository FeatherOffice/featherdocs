/**
 * Um motor novo por teste, rodando o `featherdocs_ponte.wasm` — o mesmo motor Rust
 * do aplicativo, compilado por `npm run ponte:wasm`. Teste de comportamento passa
 * por aqui, nunca por um dublê em TypeScript.
 */

import { readFileSync } from 'node:fs';
import { canalWasm } from '../src/ponte/canal.ts';
import { Motor } from '../src/ponte/index.ts';

const caminho = new URL('../crates/featherdocs-ponte/target/wasm32-unknown-unknown/release/featherdocs_ponte.wasm', import.meta.url);
const modulo = new WebAssembly.Module(readFileSync(caminho));

/** Um motor limpo (documento novo). */
export function motorNovo(): Motor {
  return new Motor(canalWasm(modulo));
}
