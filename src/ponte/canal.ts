/**
 * O canal até o motor Rust: um comando e um corpo vão, bytes voltam — **na hora**,
 * sem `await`. Sem estado do documento aqui: o TypeScript não guarda cópia dele.
 *
 * Dois canais:
 *
 * - `canalTauri`, no aplicativo: o protocolo `ponte://` atendido pelo `src-tauri`
 *   (XHR síncrono até ele, como no FeatherSheet e no FeatherCAD);
 * - `canalWasm`, nos testes e no `npm run dev` do navegador: o mesmo motor
 *   (`crates/featherdocs-ponte`) compilado para WebAssembly.
 */

export interface Canal {
  /**
   * Atende um comando; lança `Error` com a mensagem do Rust se ele recusar. `args`
   * vai em JSON — a não ser que seja um `Uint8Array`, que vai cru (é como um arquivo
   * inteiro atravessa: em JSON, cada byte viraria um número decimal).
   */
  chamar(comando: string, args?: unknown): Uint8Array;
}

const utf8 = new TextDecoder();
const codificar = new TextEncoder();

function corpoDe(args: unknown): Uint8Array | string {
  if (args instanceof Uint8Array) return args;
  return args === undefined ? '' : JSON.stringify(args);
}

/** A resposta JSON de um comando. */
export function chamarJson<T>(canal: Canal, comando: string, args?: unknown): T {
  return JSON.parse(utf8.decode(canal.chamar(comando, args))) as T;
}

/**
 * O `ponte://` do aplicativo. No Linux o WebKitGTK atende esquema próprio pelo nome
 * (`ponte://localhost/`); no Windows o WebView2 só conhece `http://ponte.localhost/`.
 */
export function canalTauri(): Canal {
  let base: string | null = null;

  function descobrirBase(): string {
    if (base) return base;
    const windows = typeof navigator !== 'undefined' && /Windows/i.test(navigator.userAgent);
    const preferida = windows ? 'http://ponte.localhost/' : 'ponte://localhost/';
    for (const url of [preferida, 'ponte://localhost/', 'http://ponte.localhost/']) {
      try {
        const x = new XMLHttpRequest();
        x.open('POST', url + 'estado', false);
        x.send();
        if (x.status === 200) return (base = url);
      } catch {
        // tenta a próxima
      }
    }
    return (base = preferida);
  }

  return {
    chamar(comando, args) {
      const x = new XMLHttpRequest();
      x.open('POST', descobrirBase() + comando, false);
      // Resposta binária num XHR síncrono: os bytes vêm como texto de um "charset" que
      // mapeia um byte num caractere, sem conversão.
      x.overrideMimeType('text/plain; charset=x-user-defined');
      x.send(corpoDe(args) as XMLHttpRequestBodyInit);
      const texto = x.responseText;
      const bytes = new Uint8Array(texto.length);
      for (let i = 0; i < texto.length; i++) bytes[i] = texto.charCodeAt(i) & 0xff;
      if (x.status !== 200) throw new Error(utf8.decode(bytes) || `ponte://${comando} falhou (${x.status})`);
      return bytes;
    },
  };
}

/** O que o `featherdocs_ponte.wasm` exporta. Ver `crates/featherdocs-ponte/src/wasm.rs`. */
interface PonteWasm {
  memory: WebAssembly.Memory;
  ponte_reiniciar(): void;
  ponte_entrada(n: number): number;
  ponte_chamar(nome: number, corpo: number): number;
  ponte_saida(): number;
  ponte_saida_tamanho(): number;
}

/** O motor Rust dentro do processo, como WebAssembly. `modulo` é o `WebAssembly.Module` já compilado. */
export function canalWasm(modulo: WebAssembly.Module): Canal {
  const instancia = new WebAssembly.Instance(modulo, {});
  const w = instancia.exports as unknown as PonteWasm;
  w.ponte_reiniciar();
  return {
    chamar(comando, args) {
      const nome = codificar.encode(comando);
      const bruto = corpoDe(args);
      const corpo = typeof bruto === 'string' ? codificar.encode(bruto) : bruto;
      const ptr = w.ponte_entrada(nome.length + corpo.length);
      // A memória pode ter crescido na chamada acima: a vista é refeita.
      const mem = new Uint8Array(w.memory.buffer);
      mem.set(nome, ptr);
      mem.set(corpo, ptr + nome.length);
      const status = w.ponte_chamar(nome.length, corpo.length);
      const saida = new Uint8Array(w.memory.buffer, w.ponte_saida(), w.ponte_saida_tamanho()).slice();
      if (status !== 0) throw new Error(utf8.decode(saida));
      return saida;
    },
  };
}
