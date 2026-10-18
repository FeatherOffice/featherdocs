/** Uma janelinha com campos numéricos (margens personalizadas, papel personalizado): só coleta, o motor valida. */

import { t } from '../idiomas/index.ts';
import { Dialogo, rotuloFechar } from './dialogo.ts';

export interface CampoNumerico {
  rotulo: string;
  valor: number;
}

/** Abre a janela; `aoAplicar` recebe os valores na ordem dos campos. */
export function pedirNumeros(titulo: string, campos: CampoNumerico[], aoAplicar: (valores: number[]) => void): void {
  const d: Dialogo = new Dialogo(titulo, rotuloFechar(), (corpo) => {
    const entradas = campos.map((c) => {
      const linha = document.createElement('label');
      linha.style.cssText = 'display:flex;align-items:center;justify-content:space-between;gap:12px;margin:0 0 8px';
      const rotulo = document.createElement('span');
      rotulo.textContent = c.rotulo;
      const campo = document.createElement('input');
      campo.type = 'text';
      campo.inputMode = 'decimal';
      campo.value = String(Math.round(c.valor * 100) / 100).replace('.', ',');
      campo.style.cssText = 'width:96px;padding:4px 8px;text-align:right';
      linha.append(rotulo, campo);
      corpo.append(linha);
      return campo;
    });
    const aplicar = (): void => {
      const valores = entradas.map((e) => Number.parseFloat(e.value.replace(',', '.')));
      if (valores.some((v) => !Number.isFinite(v))) return;
      d.fechar();
      aoAplicar(valores);
    };
    const ok = document.createElement('button');
    ok.type = 'button';
    ok.textContent = t('faixa.aplicar');
    ok.addEventListener('click', aplicar);
    for (const e of entradas) {
      e.addEventListener('keydown', (ev) => {
        if (ev.key === 'Enter') aplicar();
      });
    }
    corpo.append(ok);
    setTimeout(() => entradas[0]?.select());
  });
  d.abrir();
}

/**
 * O aviso antes de converter LaTeX em Markdown: a lista do que se perde (o motor diz o quê,
 * em chaves `perda.*`) e os botões Converter e Cancelar. `true` se o usuário aceitou.
 */
export function confirmarConversao(perdas: string[]): Promise<boolean> {
  return new Promise((resolver) => {
    let resposta = false;
    const d: Dialogo = new Dialogo(
      t('conversao.titulo'),
      rotuloFechar(),
      (corpo) => {
        const intro = document.createElement('p');
        intro.style.cssText = 'margin:0 0 8px';
        intro.textContent = t('conversao.intro');
        const lista = document.createElement('ul');
        lista.style.cssText = 'margin:0 0 12px;padding-left:20px;line-height:1.5';
        for (const p of perdas) {
          const li = document.createElement('li');
          li.textContent = t(`perda.${p}`);
          lista.append(li);
        }
        const nota = document.createElement('p');
        nota.style.cssText = 'margin:0 0 12px;opacity:.8';
        nota.textContent = t('conversao.nota');
        const botoes = document.createElement('div');
        botoes.style.cssText = 'display:flex;justify-content:flex-end;gap:8px';
        const cancelar = document.createElement('button');
        cancelar.type = 'button';
        cancelar.textContent = t('conversao.cancelar');
        cancelar.addEventListener('click', () => d.fechar());
        const converter = document.createElement('button');
        converter.type = 'button';
        converter.textContent = t('conversao.converter');
        converter.addEventListener('click', () => {
          resposta = true;
          d.fechar();
        });
        botoes.append(cancelar, converter);
        corpo.append(intro, lista, nota, botoes);
        setTimeout(() => cancelar.focus());
      },
      () => resolver(resposta),
    );
    d.abrir();
  });
}
