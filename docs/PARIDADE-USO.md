# FeatherDocs — o que falta para usar no dia a dia

Pedido do usuário em 18/10/2026: lista das demandas que faltam para o
FeatherDocs (interface Iced) ser usável, feita em ordem, marcando `[X]` ao
terminar. Levantamento: os métodos do motor (`Sessao`) que a GUI não chama.
O motor já desenha cabeçalho, rodapé, marca d'água, notas e comentários;
quase tudo abaixo é ligar o comando na faixa e, quando pede texto, num
diálogo interno (`feather_ui::dialogo`).

Legenda: `[X]` feito e compilado · `[ ]` falta.

## 0. Base

- [X] `feather-ui` do repositório próprio (v0.2.9, igual ao FeatherSheet), para ter o diálogo interno padrão

## 1. Início — texto e parágrafo

- [X] Tachado, sobrescrito, subscrito, código na faixa (o motor já tem, só teclado)
- [X] Aumentar/diminuir fonte (A+ / A−) e limpar formatação
- [X] Maiúsculas/minúsculas (MAIÚSCULAS, minúsculas, Primeira Maiúscula)
- [X] Aumentar/diminuir recuo; reiniciar numeração da lista; lista de tarefas
- [X] Entrelinha (1, 1,15, 1,5, 2) e espaço antes/depois do parágrafo
- [X] Sombreamento e bordas do parágrafo
- [X] Galeria de estilos completa (Título 3, Citação, Código) e estilos com nome do documento
- [X] Colar sem formatação (Ctrl+Shift+V)

## 2. Inserir

- [X] Link (Ctrl+K): inserir, editar, remover
- [X] Nota de rodapé e nota de fim
- [X] Comentário novo
- [X] Cabeçalho e rodapé (esquerda/centro/direita, com campos de página)
- [X] Número de página (posição e modelo) e remover
- [X] Legenda (figura/tabela/equação) e referência cruzada
- [X] Marcador (indicador)
- [X] Marca d'água
- [X] Índice remissivo: marcar entrada e inserir índice
- [X] Remover sumário e configurar níveis

## 3. Layout

- [X] Margens (Normal, Estreita, Larga e personalizadas)
- [X] Tamanho de papel personalizado
- [X] Cor da página e borda da página
- [X] Numeração de linhas e primeira página diferente
- [X] Alinhamento vertical da página

## 4. Revisão

- [X] Painel de comentários (ir, responder, resolver, excluir)
- [X] Lista de alterações sugeridas (aceitar/rejeitar uma a uma)
- [X] Comparar com outro arquivo
- [X] Proteger o documento (só leitura)
- [X] Estatísticas (contagem detalhada de palavras, caracteres, parágrafos)

## 5. Arquivo

- [X] Propriedades (título e autor)
- [X] Arquivos recentes no menu
- [X] Abrir arquivo arrastado para a janela

## 6. Interação na página

- [X] Menu do botão direito (copiar, colar, link, comentário, tabela)
- [X] Clicar na caixa da lista de tarefas marca/desmarca
- [X] Ctrl+clique em link abre no navegador do sistema (só por pedido explícito)

## 7. Segunda rodada (18/10, depois da lista acima)

- [X] Exportar PDF (texto em Helvetica/Times/Courier com as mesmas métricas do motor, formas, tabelas, cabeçalho e rodapé); `feather-draw` local ganhou Times e Courier, falta publicar a tag
- [X] Imagens, gráficos e marca d'água no PDF (imagem RGB sem compressão; fórmulas ainda não saem e a barra avisa)
- [X] Tabela com o tamanho escolhido (hoje sempre 3 × 3)
- [X] Equação e linha horizontal na faixa
- [X] Localizar com opções (maiúsculas/minúsculas, palavra inteira)
- [X] Citações e bibliografia: cadastrar fonte, importar .bib, inserir citação, inserir bibliografia, estilo
- [X] Editar nota de rodapé já inserida
- [X] Novo estilo a partir da seleção e modificar estilo

## 8. Bugs encontrados no caminho

- [X] Teste `app006_cabecalho_exclusao_undo_e_round_trip_fdoz` falhava antes de 18/10: o conteúdo volta exato no desfazer, só o contador de ids não volta (de propósito); o teste passou a comparar os blocos
