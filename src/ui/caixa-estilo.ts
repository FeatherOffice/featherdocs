/**
 * A caixa "Novo estilo" / "Modificar estilo": nome, base (Normal ou Título 1–6), letra, cor,
 * alinhamento e espaços, com a prévia ao vivo. Só coleta e mostra; quem valida e guarda é o motor.
 * Também converte entre o que o motor guarda (bits, cor em número) e o que a caixa edita.
 */

import { t } from '../idiomas/index.ts';
import { CODIGO, ITALICO, NEGRITO, SERIFA, SUBLINHADO, type Alinhamento, type DadosDoEstilo, type EstiloComNome, type Formato } from '../ponte/tipos.ts';
import { corEmTexto } from './comandos-lista.ts';
import { Dialogo, rotuloFechar } from './dialogo.ts';
import { FAMILIA_CODIGO, FAMILIA_SERIFA, FAMILIA_TEXTO } from './familias.ts';

export const CAIXA_ESTILO_CSS = `
.caixa-estilo .grade-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 8px 12px; }
.caixa-estilo select, .caixa-estilo input[type=number] { padding: 5px 8px; border: 1px solid var(--borda); border-radius: 6px; background: var(--painel); color: inherit; font-size: 13px; }
.caixa-estilo .marcas { display: flex; gap: 14px; align-items: center; font-size: 13px; }
.caixa-estilo .marcas label { flex-direction: row; align-items: center; gap: 5px; }
.caixa-estilo .cor { display: flex; gap: 8px; align-items: center; }
.caixa-estilo .cor label { flex-direction: row; align-items: center; gap: 5px; }
.caixa-estilo .cor input[type=color] { width: 40px; height: 28px; padding: 0; border: 1px solid var(--borda); border-radius: 4px; background: none; }
.caixa-estilo .previa {
  min-height: 72px; padding: 12px 14px; border: 1px solid var(--borda); border-radius: 6px; background: #fff; color: #1a1a1a;
  overflow: hidden; white-space: nowrap; text-overflow: ellipsis;
}
.caixa-estilo .previa-legenda { font-size: 11.5px; opacity: 0.7; }
`;

/** Os corpos padrão de cada base (iguais aos do motor). */
const TAMANHO_DA_BASE = [11, 24, 18, 15, 13, 12, 11];

/** A base do estilo pelo tipo do bloco (`titulo2` → 2; o resto → 0). */
export function baseDoTipo(tipo: string): number {
  const m = /^titulo([1-6])$/.exec(tipo);
  return m ? Number(m[1]) : 0;
}

/** O estilo guardado no motor, como a caixa edita. */
export function dadosDoEstilo(e: EstiloComNome): DadosDoEstilo {
  return {
    nome: e.nome,
    base: e.base,
    fonte: e.bits & CODIGO ? 'mono' : e.bits & SERIFA ? 'serifa' : 'sans',
    negrito: Boolean(e.bits & NEGRITO),
    italico: Boolean(e.bits & ITALICO),
    sublinhado: Boolean(e.bits & SUBLINHADO),
    tamanho: e.tamanho > 0 ? e.tamanho : TAMANHO_DA_BASE[e.base] ?? 11,
    cor: corEmTexto(e.cor),
    alinhamento: e.alinhamento,
    antes: e.antes,
    depois: e.depois,
    entrelinha: e.entrelinha,
  };
}

/** Um estilo novo a partir do parágrafo do cursor (o que o Word chama de "criar a partir da seleção"). */
export function dadosDoCursor(f: Formato | null): DadosDoEstilo {
  const base = baseDoTipo(f?.estilo ?? '');
  return {
    nome: '',
    base,
    fonte: f?.fonte ?? 'sans',
    // O título já é negrito pelo tipo (não pelo trecho): o estilo novo nasce como ele aparece.
    negrito: base > 0 || Boolean(f?.negrito),
    italico: Boolean(f?.italico),
    sublinhado: Boolean(f?.sublinhado),
    tamanho: f?.tamanho ?? TAMANHO_DA_BASE[base] ?? 11,
    cor: f?.cor ?? '',
    alinhamento: f?.alinhamento ?? 'esquerda',
    antes: f?.antes ?? null,
    depois: f?.depois ?? null,
    entrelinha: f?.entrelinha && Math.abs(f.entrelinha - 1.15) > 0.001 ? f.entrelinha : 0,
  };
}

/** O CSS que mostra o estilo (a prévia da caixa e o bloco da galeria, em `escala`). */
export function cssDoEstilo(d: DadosDoEstilo, escala = 1): string {
  const familia = d.fonte === 'mono' ? FAMILIA_CODIGO : d.fonte === 'serifa' ? FAMILIA_SERIFA : FAMILIA_TEXTO;
  const partes = [`font-family: ${familia}`, `font-size: ${Math.round(d.tamanho * escala * 10) / 10}px`, `font-weight: ${d.negrito ? 600 : 400}`];
  if (d.italico) partes.push('font-style: italic');
  if (d.sublinhado) partes.push('text-decoration: underline');
  if (d.cor) partes.push(`color: ${d.cor}`);
  return partes.join('; ');
}

const ALINHAMENTOS: Alinhamento[] = ['esquerda', 'centro', 'direita', 'justificado'];
const ENTRELINHAS = [0, 1, 1.15, 1.5, 2];

function rotulado(texto: string, el: HTMLElement): HTMLLabelElement {
  const l = document.createElement('label');
  const s = document.createElement('span');
  s.textContent = texto;
  l.append(s, el);
  return l;
}

function seletor(opcoes: Array<[string, string]>, valor: string): HTMLSelectElement {
  const s = document.createElement('select');
  for (const [v, r] of opcoes) s.add(new Option(r, v, false, v === valor));
  return s;
}

function numero(valor: number | null, min: number, max: number, passo: number): HTMLInputElement {
  const i = document.createElement('input');
  i.type = 'number';
  i.min = String(min);
  i.max = String(max);
  i.step = String(passo);
  i.value = valor == null ? '' : String(Math.round(valor * 10) / 10);
  return i;
}

function marcar(texto: string, ligado: boolean): { el: HTMLLabelElement; caixa: HTMLInputElement } {
  const el = document.createElement('label');
  const caixa = document.createElement('input');
  caixa.type = 'checkbox';
  caixa.checked = ligado;
  el.append(caixa, document.createTextNode(texto));
  return { el, caixa };
}

/**
 * Abre a caixa. `aplicar` recebe o que foi preenchido e devolve a mensagem de erro do motor (ou
 * `null` se deu certo): com erro, a caixa fica aberta e mostra a mensagem.
 */
export function caixaDeEstilo(titulo: string, inicial: DadosDoEstilo, aplicar: (d: DadosDoEstilo) => string | null): void {
  const d: Dialogo = new Dialogo(titulo, rotuloFechar(), (corpo) => {
    const form = document.createElement('div');
    form.className = 'caixa-form caixa-estilo';
    const nome = document.createElement('input');
    nome.type = 'text';
    nome.value = inicial.nome;
    nome.placeholder = t('estilo.nome_exemplo');
    const base = seletor(
      [['0', t('tipos.paragrafo')], ...[1, 2, 3, 4, 5, 6].map((n): [string, string] => [String(n), t(`tipos.titulo${n}`)])],
      String(inicial.base),
    );
    const fonte = seletor([['sans', t('faixa.fonte_sans')], ['serifa', t('faixa.fonte_serifa')], ['mono', t('faixa.fonte_mono')]], inicial.fonte);
    const tamanho = numero(inicial.tamanho, 1, 400, 0.5);
    const negrito = marcar(t('cmd.negrito'), inicial.negrito);
    const italico = marcar(t('cmd.italico'), inicial.italico);
    const sublinhado = marcar(t('cmd.sublinhado'), inicial.sublinhado);
    const cor = document.createElement('input');
    cor.type = 'color';
    cor.value = inicial.cor || '#1a1a1a';
    const automatica = marcar(t('estilo.cor_automatica'), !inicial.cor);
    const alinhamento = seletor(ALINHAMENTOS.map((a): [string, string] => [a, t(`cmd.alinhar_${a}`)]), inicial.alinhamento);
    const antes = numero(inicial.antes, 0, 1000, 1);
    const depois = numero(inicial.depois, 0, 1000, 1);
    antes.placeholder = depois.placeholder = t('estilo.padrao');
    const entrelinha = seletor(
      ENTRELINHAS.map((n): [string, string] => [String(n), n === 0 ? t('estilo.padrao') : String(n).replace('.', ',')]),
      String(ENTRELINHAS.includes(inicial.entrelinha) ? inicial.entrelinha : 0),
    );
    const previa = document.createElement('div');
    previa.className = 'previa';
    const legenda = document.createElement('div');
    legenda.className = 'previa-legenda';
    legenda.textContent = t('estilo.previa');
    const erro = document.createElement('div');
    erro.style.cssText = 'color: var(--erro, #c5221f); font-size: 12.5px; min-height: 1em';

    const ler = (): DadosDoEstilo => {
      const vazioOuNumero = (i: HTMLInputElement): number | null => (i.value.trim() === '' ? null : Number(i.value));
      return {
        nome: nome.value,
        base: Number(base.value),
        fonte: fonte.value as DadosDoEstilo['fonte'],
        negrito: negrito.caixa.checked,
        italico: italico.caixa.checked,
        sublinhado: sublinhado.caixa.checked,
        tamanho: Number(tamanho.value) || TAMANHO_DA_BASE[Number(base.value)] || 11,
        cor: automatica.caixa.checked ? '' : cor.value,
        alinhamento: alinhamento.value as Alinhamento,
        antes: vazioOuNumero(antes),
        depois: vazioOuNumero(depois),
        entrelinha: Number(entrelinha.value),
      };
    };
    const mostrar = (): void => {
      const atual = ler();
      previa.textContent = atual.nome.trim() || t('estilo.previa_texto');
      previa.style.cssText = cssDoEstilo(atual);
      previa.style.textAlign = atual.alinhamento === 'centro' ? 'center' : atual.alinhamento === 'direita' ? 'right' : 'left';
      cor.disabled = automatica.caixa.checked;
    };
    // Trocar a base traz o corpo e o negrito dela (quem escolhe "Título 2" espera a letra do Título 2).
    base.addEventListener('change', () => {
      const n = Number(base.value);
      tamanho.value = String(TAMANHO_DA_BASE[n] ?? 11);
      negrito.caixa.checked = n > 0;
    });
    form.addEventListener('input', mostrar);
    form.addEventListener('change', mostrar);

    const marcas = document.createElement('div');
    marcas.className = 'marcas';
    marcas.append(negrito.el, italico.el, sublinhado.el);
    const corLinha = document.createElement('div');
    corLinha.className = 'cor';
    corLinha.append(cor, automatica.el);
    const grade = document.createElement('div');
    grade.className = 'grade-2';
    grade.append(
      rotulado(t('estilo.base'), base),
      rotulado(t('estilo.alinhamento'), alinhamento),
      rotulado(t('estilo.fonte'), fonte),
      rotulado(t('estilo.tamanho'), tamanho),
      rotulado(t('estilo.cor'), corLinha),
      rotulado(t('faixa.entrelinha'), entrelinha),
      rotulado(t('estilo.antes'), antes),
      rotulado(t('estilo.depois'), depois),
    );
    const ok = (): void => {
      const msg = aplicar(ler());
      if (msg) {
        erro.textContent = msg;
        return;
      }
      d.fechar();
    };
    nome.addEventListener('keydown', (e) => e.key === 'Enter' && ok());
    const botoes = document.createElement('div');
    botoes.className = 'botoes';
    const cancelar = document.createElement('button');
    cancelar.type = 'button';
    cancelar.textContent = t('estilo.cancelar');
    cancelar.addEventListener('click', () => d.fechar());
    const aceitar = document.createElement('button');
    aceitar.type = 'button';
    aceitar.className = 'principal';
    aceitar.textContent = t('estilo.ok');
    aceitar.addEventListener('click', ok);
    botoes.append(cancelar, aceitar);
    form.append(rotulado(t('estilo.nome'), nome), grade, marcas, legenda, previa, erro, botoes);
    corpo.append(form);
    mostrar();
    setTimeout(() => nome.focus());
  });
  d.abrir();
}
