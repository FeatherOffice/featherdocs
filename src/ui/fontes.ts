/**
 * As fontes do papel, embutidas no aplicativo (Liberation Sans e Mono, licença OFL).
 * O motor mede o texto com as larguras dessas fontes; se o canvas desenhasse com
 * outra, o cursor não cairia entre as letras. Por isso elas vão junto, em vez de
 * depender do que o sistema tem.
 *
 * O endereço sai de `new URL(…, import.meta.url)` e não de `import … from '…ttf?url'`: o import
 * faz o navegador pedir o `.ttf` como módulo em alguns casos (o WebKit do `app:dev`), e ele recusa
 * ("'font/ttf' is not a valid JavaScript MIME type"). O Vite trata os dois iguais no build.
 */

const italico = new URL('../assets/fontes/LiberationSans-Italic.ttf', import.meta.url).href;
const negritoItalico = new URL('../assets/fontes/LiberationSans-BoldItalic.ttf', import.meta.url).href;
const negrito = new URL('../assets/fontes/LiberationSans-Bold.ttf', import.meta.url).href;
const regular = new URL('../assets/fontes/LiberationSans-Regular.ttf', import.meta.url).href;
const monoNegrito = new URL('../assets/fontes/LiberationMono-Bold.ttf', import.meta.url).href;
const mono = new URL('../assets/fontes/LiberationMono-Regular.ttf', import.meta.url).href;
const serifaRegular = new URL('../assets/fontes/LiberationSerif-Regular.ttf', import.meta.url).href;
const serifaNegrito = new URL('../assets/fontes/LiberationSerif-Bold.ttf', import.meta.url).href;
const serifaItalico = new URL('../assets/fontes/LiberationSerif-Italic.ttf', import.meta.url).href;
const serifaNegritoItalico = new URL('../assets/fontes/LiberationSerif-BoldItalic.ttf', import.meta.url).href;

const FACES: Array<[string, string, FontFaceDescriptors]> = [
  ['FeatherDocs Sans', regular, { weight: '400', style: 'normal' }],
  ['FeatherDocs Sans', negrito, { weight: '700', style: 'normal' }],
  ['FeatherDocs Sans', italico, { weight: '400', style: 'italic' }],
  ['FeatherDocs Sans', negritoItalico, { weight: '700', style: 'italic' }],
  ['FeatherDocs Serif', serifaRegular, { weight: '400', style: 'normal' }],
  ['FeatherDocs Serif', serifaNegrito, { weight: '700', style: 'normal' }],
  ['FeatherDocs Serif', serifaItalico, { weight: '400', style: 'italic' }],
  ['FeatherDocs Serif', serifaNegritoItalico, { weight: '700', style: 'italic' }],
  ['FeatherDocs Mono', mono, { weight: '400', style: 'normal' }],
  ['FeatherDocs Mono', monoNegrito, { weight: '700', style: 'normal' }],
];

/** Carrega as fontes antes do primeiro desenho (senão a primeira página sairia com a fonte do sistema). */
export async function carregarFontes(): Promise<void> {
  if (typeof FontFace === 'undefined') return;
  await Promise.all(
    FACES.map(async ([familia, url, d]) => {
      try {
        const f = new FontFace(familia, `url(${url})`, d);
        document.fonts.add(await f.load());
      } catch (e) {
        console.warn(`[fonte] ${familia}: ${String(e)}`);
      }
    }),
  );
}
