/**
 * Os menus suspensos (quando a faixa está minimizada), montados do mesmo registro de comandos
 * que a faixa: um id por item, `'|'` é separador. Comando "em breve" aparece desabilitado.
 */

import { t } from '../idiomas/index.ts';
import { itemDeMenu, type Registro } from './comandos.ts';
import type { MenuItem } from './menu.ts';

const MENUS: Array<[string, string[]]> = [
  ['menus.arquivo', ['inicio', '|', 'novo', 'abrir', '|', 'salvar', 'salvar_como', 'exportar']],
  ['menus.editar', ['desfazer', 'refazer', '|', 'recortar', 'copiar', 'colar', 'colar_sem', '|', 'selecionar', '|', 'localizar', 'substituir']],
  ['menus.ver', ['modo_paginas', 'modo_continuo', 'codigo_latex', 'codigo_markdown', 'modo_foco', '|', 'regua', 'estrutura', 'marcas', 'grade', 'miniaturas', '|', 'zoom_mais', 'zoom_menos', 'zoom_100', 'zoom_pagina', 'zoom_largura', '|', 'tema']],
  [
    'menus.inserir',
    ['tabela_dialogo', 'imagem', 'formas', 'grafico', '|', 'link', 'marcador', '|', 'quebra_pagina', 'quebra_secao', 'linha_horizontal', '|', 'cabecalho', 'rodape', 'numero_pagina', 'data_hora', '|', 'chamada', 'caixa_texto', 'recolhivel', 'campo_dados', '|', 'equacao', 'simbolo', 'bloco_codigo'],
  ],
  [
    'menus.formatar',
    [
      'fonte_familia', 'negrito', 'italico', 'sublinhado', 'tachado', 'codigo_linha', 'sobrescrito', 'subscrito', '|',
      'aumentar_fonte', 'diminuir_fonte', 'maiusculas', 'limpar', 'pincel', '|',
      'estilos', 'lista_marcadores', 'lista_numerada', 'lista_tarefas', 'citacao', '|',
      'alinhar_esquerda', 'alinhar_centro', 'alinhar_direita', 'alinhar_justificado', 'recuo_menos', 'recuo_mais', 'entrelinha', 'direcao_texto',
    ],
  ],
  ['menus.layout', ['margens', 'orientacao', 'tamanho_papel', 'colunas', 'quebras', 'numeracao_linhas', 'hifenizacao', '|', 'marca_dagua', 'cor_pagina', 'bordas_pagina', '|', 'alinhar_vertical']],
  ['menus.referencias', ['sumario', 'adicionar_texto', 'sumario_personalizado', '|', 'nota_rodape', 'nota_fim', 'formato_notas', '|', 'citacao_inserir', 'fontes', 'estilo_citacao', 'bibliografia', '|', 'legenda', 'lista_figuras', 'referencia_cruzada', '|', 'marcar_entrada', 'indice']],
  ['menus.revisao', ['ortografia', 'idioma', 'contagem', '|', 'comentario_novo', 'comentarios_mostrar', '|', 'modo_edicao', 'aceitar', 'rejeitar', '|', 'comparar', 'historico', 'proteger']],
  ['menus.ferramentas', ['configuracoes', '|', 'tab_linha_acima', 'tab_linha_abaixo', 'tab_coluna_esq', 'tab_coluna_dir', 'tab_excluir', 'tab_cabecalho', 'tab_mesclar', 'tab_dividir', 'tab_feathersheet']],
  ['menus.ajuda', ['ajuda']],
];

/** Os menus como `[título, itens]`, para a `MenuBar`. */
export function menusSuspensos(r: Registro): Array<[string, () => MenuItem[]]> {
  return MENUS.map(([chave, ids]) => [t(chave), () => ids.map((id) => itemDeMenu(r, id))]);
}

/** Todos os ids citados nos menus (o teste confere que existem no registro). */
export const IDS_DOS_MENUS: string[] = MENUS.flatMap(([, ids]) => ids.filter((id) => id !== '|'));
