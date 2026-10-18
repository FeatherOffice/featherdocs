> **Decisão do usuário (05/10/2026):** a parte de **macros** (aba Automação → Macros, gravar macro, editor de scripts, botão de macro, exportação em lote) fica **para o futuro**. O resto da especificação segue a ordem da seção 11.

# FeatherDocs — Especificação detalhada do menu

Oct 5, 2026 · @Maragon

## 1. Visão geral das abas

O menu do FeatherDocs fica em oito abas, cada uma dividida em grupos. As abas Página inicial, Inserir e Layout já existem na sua base visual; Referências, Revisão e Automação são as que o Word e o Google Docs têm de forma limitada e onde o FeatherDocs pode se diferenciar.

| Aba | Grupos | Para que serve |
| --- | --- | --- |
| Página inicial | Área de transferência, Fonte, Parágrafo, Estilos, Edição | Formatar texto no dia a dia |
| Inserir | Tabela, Ilustrações, Links, Equação e símbolos, Quebras, Cabeçalho e rodapé, Código | Colocar objetos no documento |
| Layout | Configurar página, Colunas, Plano de fundo, Espaçamento | Controlar a página física |
| Referências | Sumário, Notas, Legendas, Citações e bibliografia | Documentos longos e acadêmicos |
| Revisão | Ortografia, Comentários, Alterações, Comparar | Trabalho em equipe |
| Exibir | Modos, Mostrar, Zoom, Janela | Como o documento aparece na tela |
| Automação | Macros, Modelos, Integração FeatherSheet | Vantagem sobre Docs e Word Online |
| Ajuda | Atalhos, Pesquisar comando, Sobre | Descoberta de recursos |

Uma caixa **"Pesquisar comando"** no topo do menu (atalho Ctrl+/) vale a pena desde o início: com tantos controles, ela evita que o usuário precise saber em qual aba está cada função.

## 2. Aba Página inicial

Esta aba concentra a formatação de uso diário. A tabela lista cada controle, o tipo de componente e o comportamento esperado.

| Grupo | Controle | Tipo | Comportamento |
| --- | --- | --- | --- |
| Área de transferência | Colar | Botão com menu | Opções: manter formatação, só texto, colar especial |
| Área de transferência | Copiar, Recortar | Botão | Atalhos Ctrl+C e Ctrl+X |
| Área de transferência | Pincel de formatação | Botão | Um clique aplica uma vez; duplo clique fica ativo até Esc |
| Fonte | Família da fonte | Lista suspensa | Mostra cada nome na própria fonte; fontes recentes no topo |
| Fonte | Tamanho | Lista editável | 8 a 72 pt; aceita digitar valor |
| Fonte | Aumentar / diminuir fonte | Dois botões | Ctrl+Shift+> e Ctrl+Shift+<; passo de 1 pt até 12, depois 2 pt |
| Fonte | Negrito, Itálico, Sublinhado, Tachado | Botões de alternância | Ctrl+B, Ctrl+I, Ctrl+U |
| Fonte | Sobrescrito, Subscrito | Botões de alternância | Ctrl+Shift+= e Ctrl+= |
| Fonte | Cor da fonte | Botão dividido | Paleta de temas, cores padrão, "Mais cores" com hexadecimal e conta-gotas |
| Fonte | Realce (marca-texto) | Botão dividido | Paleta de 15 cores e "Sem cor" |
| Fonte | Maiúsculas e minúsculas | Menu | Frase, minúsculas, MAIÚSCULAS, Cada Palavra Maiúscula, alternar |
| Fonte | Limpar formatação | Botão | Ctrl+barra de espaço; volta ao estilo base do parágrafo |
| Parágrafo | Lista com marcadores | Botão dividido | Galeria de estilos de marcador (círculo, quadrado, seta) |
| Parágrafo | Lista numerada | Botão dividido | 1. 2. 3., a) b) c), I. II. III. e reiniciar numeração |
| Parágrafo | Lista de tarefas | Botão | Caixas de seleção clicáveis |
| Parágrafo | Aumentar / diminuir recuo | Dois botões | Tab e Shift+Tab em listas; passo de 1,25 cm |
| Parágrafo | Alinhamento | Quatro botões | Esquerda, centro, direita, justificado (Ctrl+L, E, R, J) |
| Parágrafo | Espaçamento entre linhas | Menu | 1,0 · 1,15 · 1,5 · 2,0 · personalizado; adicionar ou remover espaço antes e depois |
| Parágrafo | Sombreamento e bordas | Dois botões | Cor de fundo do parágrafo; borda em um ou mais lados |
| Parágrafo | Direção do texto | Alternância | Esquerda para direita ou direita para esquerda |
| Estilos | Galeria de estilos | Galeria | Normal, Título 1 a 6, Subtítulo, Citação, Código; ver seção 5 |
| Edição | Localizar | Painel | Ctrl+F com contador "3 de 12", diferenciar maiúsculas, palavra inteira |
| Edição | Substituir | Painel | Ctrl+H com substituir um ou todos e expressões regulares opcionais |
| Edição | Selecionar | Menu | Tudo, texto com formatação semelhante, todos os títulos |

A lista de tarefas e a opção de expressões regulares no Localizar custam pouco para implementar e agradam usuários técnicos.

## 3. Aba Inserir

Cada item abaixo abre um menu, uma caixa de diálogo ou um seletor visual. A equação tem seção própria (4) por ser o recurso mais complexo.

| Grupo | Controle | O que faz | Detalhes de implementação |
| --- | --- | --- | --- |
| Tabela | Inserir tabela | Matriz visual de até 10 x 8; passar o mouse destaca, clicar insere | Menu tem também "Inserir tabela..." com linhas e colunas numéricas |
| Tabela | Contexto da tabela | Aparece ao clicar dentro da tabela | Inserir linha acima e abaixo, coluna à esquerda e à direita, excluir, mesclar, dividir, linha de cabeçalho, estilos de tabela, alinhamento por célula, largura automática |
| Ilustrações | Imagem | Do computador, de URL ou colada com Ctrl+V | Alças de redimensionar com proporção travada; texto alternativo obrigatório de preencher |
| Ilustrações | Quebra de texto da imagem | Em linha, ao redor, acima e abaixo, atrás, na frente | Em linha é o padrão; as demais exigem posicionamento absoluto |
| Ilustrações | Formas | Retângulo, círculo, seta, linha, balão | Preenchimento, contorno e texto dentro da forma |
| Ilustrações | Gráfico | Escolher tipo (coluna, linha, pizza) e origem | Origem pode ser dados digitados ou intervalo do FeatherSheet (ver seção 10) |
| Ilustrações | Desenho livre | Quadro para rabiscar com caneta, borracha e cores | Salva como imagem vetorial (SVG) |
| Links | Link | Caixa com "Texto a exibir" e "Endereço" | Ctrl+K; opções: site, e-mail, título do documento, marcador |
| Links | Marcador (âncora) | Nomeia um ponto do documento | Destino para links internos e referências cruzadas |
| Equação | Equação | Abre o editor matemático | Seção 4 |
| Equação | Símbolo | Grade de caracteres especiais | Categorias: setas, gregas, moedas, matemáticos; busca por nome |
| Quebras | Quebra de página | Força nova página | Ctrl+Enter |
| Quebras | Quebra de seção | Próxima página, contínua, página par ou ímpar | Permite margens, orientação e cabeçalho diferentes por seção |
| Quebras | Linha horizontal | Divisor visual | Atalho Markdown: três hifens |
| Cabeçalho e rodapé | Cabeçalho e Rodapé | Editam o topo e a base de cada página | Opções: primeira página diferente, páginas pares e ímpares diferentes |
| Cabeçalho e rodapé | Número de página | Campo automático | Formatos: 1, "Página 1", "Página 1 de 4", romano; posição no topo, base ou margem |
| Cabeçalho e rodapé | Data e hora | Campo atualizável | Formato configurável (dd/mm/aaaa) |
| Código | Bloco de código | Caixa monoespaçada com destaque | Seletor de linguagem, numeração de linhas, botão Copiar, tema claro e escuro |
| Código | Código em linha | Trecho monoespaçado dentro do parágrafo | Atalho Markdown: acento grave |
| Elementos | Caixa de texto | Texto flutuante | Mover, girar, borda e fundo |
| Elementos | Chamada (callout) | Bloco colorido de aviso, dica, atenção | Bom para documentação técnica |
| Elementos | Bloco recolhível | Seção que abre e fecha | Útil em manuais e FAQs |
| Elementos | Campo de dados | Insere valor dinâmico: autor, data, título, nº de páginas | Atualiza sozinho |

As duas colunas à direita são o que torna a lista executável: com elas você já sabe qual componente criar para cada controle.

## 4. Fórmulas matemáticas (Inserir > Equação)

O editor de equações deve funcionar de três jeitos ao mesmo tempo: clicando em modelos prontos, digitando LaTeX e editando visualmente a fórmula já renderizada. Quem não conhece LaTeX usa os botões; quem conhece digita direto.

**Como a equação aparece no texto**

| Modo | Como inserir | Comportamento |
| --- | --- | --- |
| Em linha | Botão "Equação" ou digitar um cifrão, a fórmula e outro cifrão | Fica dentro do parágrafo, com altura reduzida |
| Em bloco | Botão "Equação em bloco" ou dois cifrões | Centralizada em linha própria, com número opcional à direita |
| Edição | Clicar na fórmula | Abre uma barra flutuante acima dela com os controles abaixo |

**Barra do editor: modelos de estrutura**

| Modelo | Exemplo em LaTeX |
| --- | --- |
| Fração | `\frac{a}{b}` |
| Expoente e índice | `x^{2}` e `x_{i}` |
| Raiz quadrada e enésima | `\sqrt{x}` e `\sqrt[n]{x}` |
| Somatório | `\sum_{i=1}^{n} x_i` |
| Produtório | `\prod_{i=1}^{n} x_i` |
| Integral | `\int_{a}^{b} f(x)\,dx` |
| Limite | `\lim_{x \to 0} f(x)` |
| Derivada | `\frac{dy}{dx}` e `\frac{\partial f}{\partial x}` |
| Matriz | `\begin{pmatrix} a & b \\ c & d \end{pmatrix}` |
| Sistema de equações | `\begin{cases} x + y = 1 \\ x - y = 3 \end{cases}` |
| Parênteses que se ajustam | `\left( \frac{a}{b} \right)` |
| Acentos e vetores | `\vec{v}`, `\hat{x}`, `\bar{x}`, `\dot{x}` |
| Binomial | `\binom{n}{k}` |
| Texto dentro da fórmula | `\text{se } x > 0` |

**Barra do editor: categorias de símbolos**

| Categoria | Exemplos |
| --- | --- |
| Letras gregas | α β γ δ ε θ λ μ π σ φ ω Δ Σ Ω |
| Operadores | ± × ÷ · ∘ ∙ ⊕ ⊗ |
| Relações | ≤ ≥ ≠ ≈ ≡ ∝ ∼ ≪ ≫ |
| Setas | → ← ↔ ⇒ ⇔ ↦ ↑ ↓ |
| Conjuntos e lógica | ∈ ∉ ⊂ ⊆ ∪ ∩ ∅ ∀ ∃ ¬ ∧ ∨ |
| Cálculo | ∂ ∇ ∞ ∫ ∮ ′ ″ |
| Conjuntos numéricos | ℕ ℤ ℚ ℝ ℂ |
| Reticências | … ⋯ ⋮ ⋱ |

**Entrada rápida pelo teclado**

- Digitar `\alpha` e espaço troca por α; vale para toda a lista de comandos LaTeX.
- Digitar `/` depois de um termo cria uma fração com o termo no numerador (como no Google Docs).
- Digitar `^` abre o expoente e `_` abre o índice; a tecla de seta para a direita sai dele.
- Ctrl+Shift+E abre a equação em bloco no ponto do cursor.
- Um campo "Código LaTeX" mostra o texto editável da fórmula; alterar ali atualiza a visualização na hora.

**Numeração e referência cruzada**

| Recurso | Como funciona |
| --- | --- |
| Numerar equação | Caixa "Numerar" na barra; o número (1), (2)... fica à direita e atualiza sozinho se uma equação for inserida antes |
| Rótulo | Campo "Nome" (por exemplo, `eq-energia`) para poder apontar para a equação |
| Referência | Em Referências > Referência cruzada, escolher "Equação" e o nome; o texto vira "(3)" e é um link |
| Estilos | Numeração simples (1), com capítulo (2.1) ou personalizada |

**Sugestão técnica**

Para desenhar a fórmula na tela, a biblioteca KaTeX é rápida e cobre a maior parte do LaTeX. Para o editor visual (clicar e digitar dentro da fórmula), existe a biblioteca MathLive, de código aberto; confira a licença atual antes de adotar. Na exportação, a fórmula vai como LaTeX para `.tex`, como imagem vetorial para PDF e como OMML (formato de equação do Word) para `.docx`.

## 5. Títulos e estilos

Os títulos são a base do sumário, do painel de estrutura e da numeração automática. Por isso o estilo de cada título precisa existir como objeto próprio, e não só como uma fonte maior.

**Estilos prontos na galeria**

| Estilo | Uso | Padrão sugerido |
| --- | --- | --- |
| Título do documento | Nome do arquivo, uma vez no topo | 26 pt, negrito, não entra no sumário |
| Subtítulo | Linha abaixo do título | 15 pt, cinza |
| Título 1 | Capítulo ou seção principal | 20 pt, negrito, espaço antes de 18 pt |
| Título 2 | Subseção | 16 pt, negrito |
| Título 3 | Sub-subseção | 13 pt, negrito |
| Título 4 a 6 | Níveis mais profundos | 12 pt, negrito ou itálico |
| Normal | Texto corrido | 11 pt, espaçamento 1,15 |
| Citação | Trecho citado | Recuo, barra lateral, itálico |
| Código | Texto técnico | Monoespaçada, fundo cinza claro |
| Legenda | Texto sob figuras e tabelas | 9 pt, centralizado |

**Como criar um título**

| Forma | Passos |
| --- | --- |
| Pela galeria | Clicar no parágrafo e escolher "Título 1" na galeria de estilos |
| Pelo teclado | Ctrl+Alt+1 até Ctrl+Alt+6 |
| Pelo Markdown | Digitar ` #  ` no início da linha para Título 1, ` ##  ` para Título 2, até seis hashtags |
| Pelo menu de barra | Digitar `/` e escolher "Título 1" |

**Criar e editar estilos personalizados**

| Ação | Onde | Comportamento |
| --- | --- | --- |
| Novo estilo a partir da seleção | Galeria > "Criar estilo..." | Pega a formatação do trecho selecionado e pede um nome |
| Modificar estilo | Clique direito no estilo > "Modificar" | Caixa com fonte, cor, tamanho, espaçamento, recuo, borda e fundo; muda todos os parágrafos que usam o estilo |
| Estilo baseado em outro | Campo "Baseado em" | Se o Título 1 mudar, o "Título de capítulo" derivado acompanha |
| Estilo do próximo parágrafo | Campo "Próximo estilo" | Depois de um título, o Enter já volta para "Normal" |
| Atualizar para combinar com a seleção | Clique direito > "Atualizar" | Reaplica o estilo ao documento |
| Importar estilos | Menu da galeria > "Importar de outro documento" | Copia estilos de um modelo |
| Conjuntos de estilos | Menu da galeria > "Temas" | Troca fonte e cores de todos os estilos de uma vez (Clássico, Moderno, Acadêmico, Compacto) |

**Numeração automática de títulos**

| Opção | Resultado |
| --- | --- |
| Numerar títulos | 1, 1.1, 1.1.1 conforme o nível |
| Formato por nível | Nível 1 como "Capítulo 1", nível 2 como "1.1", nível 3 como "1.1.1" |
| Reiniciar por nível | O número do nível 2 volta a 1 a cada novo nível 1 |
| Recuo e separador | Espaço, tabulação ou nada entre o número e o texto |

**Painel de estrutura (Exibir > Estrutura de tópicos)**

Painel lateral montado em tempo real a partir dos títulos 1 a 3. Clicar leva o cursor ao ponto; arrastar um título move a seção inteira; setas de expandir e recolher agrupam subseções; o título da seção atual fica destacado conforme o usuário rola o texto.

## 6. Sumário personalizado (Referências > Sumário)

O sumário é montado a partir dos títulos e atualizado sob demanda. O diferencial é deixar o usuário controlar o visual e o conteúdo, não só escolher entre três modelos fixos.

**Inserção**

| Opção | Resultado |
| --- | --- |
| Sumário com números de página | Para documentos paginados e impressos |
| Sumário com links | Para leitura na tela e em modo contínuo; cada linha é um link |
| Sumário personalizado... | Abre a caixa de configuração abaixo |
| Remover sumário | Apaga o bloco do documento |
| Atualizar sumário | Atualiza números e títulos; também pode ser automático ao salvar ou imprimir |

**Caixa de configuração**

| Campo | Opções |
| --- | --- |
| Níveis exibidos | De 1 a 6; padrão 3 |
| Mostrar números de página | Sim ou não |
| Alinhar números à direita | Sim ou não |
| Caractere de preenchimento (líder) | Pontilhado, tracejado, linha contínua ou nenhum |
| Usar hiperlinks | Sim ou não |
| Incluir numeração dos títulos | Sim ou não |
| Título do sumário | Texto editável ("Sumário", "Índice", "Conteúdo") |
| Estilo de cada nível | Fonte, tamanho, cor, negrito e recuo por nível (Sumário 1, Sumário 2, Sumário 3) |
| Incluir estilos personalizados | Escolher quais outros estilos entram no sumário, por exemplo "Título de anexo" |
| Excluir da lista | Marcar um título para ficar fora do sumário |
| Fundo e borda | Sem borda, linha lateral ou caixa |

**Variações que valem implementar**

| Tipo | Para que serve |
| --- | --- |
| Sumário lateral fixo | Painel que acompanha a rolagem, como o do Google Docs |
| Sumário de seção | Lista só os títulos dentro de um capítulo |
| Lista de figuras | Reúne todas as legendas de imagem |
| Lista de tabelas | Reúne todas as legendas de tabela |
| Lista de equações | Reúne as equações numeradas |
| Sumário em colunas | Dois blocos lado a lado para listas muito longas |

**Regras de funcionamento**

- O sumário nunca guarda texto digitado à mão: ele é gerado dos títulos, então renomear um título e atualizar já corrige tudo.
- Editar o texto de uma linha do sumário não altera o título original; o campo "Texto alternativo no sumário" nas propriedades do título resolve esse caso.
- Em modo sem páginas (pageless), os números somem e os links permanecem.
- Ao exportar para PDF, os links do sumário continuam clicáveis e viram marcadores no painel do leitor de PDF.

## 7. Aba Layout

Controla a página física. Tudo aqui deve refletir na régua e no modo de páginas em tempo real.

| Grupo | Controle | Opções e comportamento |
| --- | --- | --- |
| Configurar página | Margens | Normal (2,54 cm), Estreita (1,27 cm), Moderada, Larga, Espelhada; "Margens personalizadas..." com superior, inferior, esquerda, direita e medianiz (encadernação) em centímetros |
| Configurar página | Orientação | Retrato ou paisagem; por seção, não só no documento todo |
| Configurar página | Tamanho do papel | A4 (21 x 29,7 cm), Carta, Ofício, A3, A5 e personalizado |
| Configurar página | Colunas | Uma, duas, três, esquerda estreita, direita estreita; linha entre colunas opcional; largura e espaço por coluna |
| Configurar página | Quebras | Página, coluna, seção contínua, seção em nova página |
| Configurar página | Numeração de linhas | Número em cada linha, comum em textos jurídicos; reinicia por página ou contínuo |
| Configurar página | Hifenização | Automática, manual ou desligada |
| Plano de fundo | Cor da página | Paleta; ignorada na impressão por padrão |
| Plano de fundo | Marca d'água | Texto ("RASCUNHO", "CONFIDENCIAL") ou imagem; fonte, cor, transparência e ângulo |
| Plano de fundo | Bordas da página | Estilo, cor, espessura e distância da borda |
| Espaçamento | Recuo à esquerda e à direita | Valores em centímetros, ligados aos marcadores da régua |
| Espaçamento | Antes e depois | Valores em pontos por parágrafo |
| Disposição | Alinhar página verticalmente | Topo, centro, justificado ou base, útil para capas |
| Disposição | Orientação do texto | Horizontal ou vertical para caixas de texto |

**Ligação com a régua**

| Elemento da régua | Efeito |
| --- | --- |
| Áreas cinzas nas pontas | Arrastar muda as margens esquerda e direita; um rótulo mostra o valor em centímetros enquanto arrasta |
| Triângulo superior | Recuo da primeira linha |
| Triângulo inferior esquerdo | Recuo do bloco inteiro |
| Triângulo direito | Recuo à direita |
| Clique na régua | Cria uma parada de tabulação (esquerda, centro, direita ou decimal) |
| Régua vertical | Arrastar muda as margens superior e inferior |

Todas as alterações de margem e recuo valem para a seção onde está o cursor, e uma caixa discreta pergunta "Aplicar a: esta seção, daqui em diante, documento inteiro".

## 8. Aba Referências

É a aba que faz o editor servir para TCC, artigos, relatórios técnicos e manuais. O Google Docs cobre pouco dela, então é uma área de ganho real.

| Grupo | Controle | Comportamento |
| --- | --- | --- |
| Sumário | Sumário e listas | Ver seção 6 |
| Notas | Nota de rodapé | Insere o número sobrescrito no texto e abre o campo no pé da página; renumera sozinha; Ctrl+Alt+F |
| Notas | Nota de fim | Reúne todas as notas no final do documento ou da seção |
| Notas | Formato de numeração | 1, 2, 3 · a, b, c · símbolos; reinicia por página, seção ou contínuo |
| Legendas | Inserir legenda | Rótulo (Figura, Tabela, Equação, Quadro) mais número automático e texto; acima ou abaixo do objeto |
| Legendas | Novo rótulo | Cria categorias como "Gráfico" ou "Algoritmo" |
| Legendas | Lista de figuras e tabelas | Gera a partir das legendas |
| Referência cruzada | Inserir referência cruzada | Escolher o tipo (título, figura, tabela, equação, nota, marcador) e o destino; insere "Figura 3" ou "página 12" como link que se atualiza |
| Citações | Inserir citação | Busca por autor ou título na lista de fontes do documento; insere "(SILVA, 2023)" |
| Citações | Gerenciar fontes | Tabela com autor, título, ano, editora, URL; importar de arquivo `.bib` |
| Citações | Estilo | ABNT, APA, Vancouver, IEEE, Chicago; troca o formato de todas as citações de uma vez |
| Citações | Bibliografia | Gera a lista de referências no estilo escolhido, em ordem alfabética ou de citação |
| Índice remissivo | Marcar entrada | Seleciona uma palavra e a inclui no índice |
| Índice remissivo | Inserir índice | Lista as palavras com os números das páginas |

Se o público incluir estudantes brasileiros, o estilo **ABNT** merece ficar como padrão de fábrica e como primeiro modelo pronto.

## 9. Abas Revisão e Exibir, e barra de status

**Revisão**

| Grupo | Controle | Comportamento |
| --- | --- | --- |
| Ortografia | Verificar ortografia e gramática | Sublinhado ondulado vermelho e azul; clique direito mostra sugestões e "Adicionar ao dicionário" |
| Ortografia | Idioma do texto | Define o idioma por trecho; padrão pt-BR |
| Ortografia | Contagem de palavras | Caixa com palavras, caracteres com e sem espaços, parágrafos, linhas |
| Comentários | Novo comentário | Balão na margem direita ligado ao trecho selecionado; responder, resolver, mencionar |
| Comentários | Mostrar comentários | Todos, só abertos, só meus |
| Alterações | Modo de edição | Editando, Sugerindo (alterações coloridas por autor) e Somente leitura |
| Alterações | Aceitar e rejeitar | Uma a uma, todas de um autor ou todas |
| Comparar | Comparar versões | Mostra o que mudou entre duas versões do histórico |
| Histórico | Histórico de versões | Lista por data e autor; restaurar ou nomear uma versão |
| Proteção | Restringir edição | Senha para trechos ou o documento todo |

**Exibir**

| Grupo | Controle | Comportamento |
| --- | --- | --- |
| Modos | Páginas | Folhas A4 separadas com sombra, quebras reais |
| Modos | Contínuo (sem páginas) | Texto em fluxo único, largura ajustável |
| Modos | Foco | Esconde menu e painéis; só o texto, com opção de máquina de escrever (linha atual centralizada) |
| Mostrar | Régua | Horizontal e vertical, ligar ou desligar |
| Mostrar | Estrutura de tópicos | Painel lateral de títulos |
| Mostrar | Marcas de formatação | Exibe pilcrow, espaços, tabulações e quebras (Ctrl+Shift+8) |
| Mostrar | Linhas de grade | Para alinhar objetos |
| Mostrar | Miniaturas de página | Painel com páginas em miniatura |
| Zoom | Controle deslizante | 50% a 200%, botão "100%", "Largura da página", "Página inteira", "Várias páginas" |
| Janela | Tema | Claro, escuro, automático |

**Barra de status inferior**

| Item | Detalhe |
| --- | --- |
| Página | "Página 1 de 4"; clicar abre "Ir para..." (página, título, marcador) |
| Palavras | "1.240 palavras"; clicar abre a contagem completa; mostra a contagem da seleção quando há texto selecionado |
| Tempo de leitura | Estimado a partir das palavras; um valor aproximado de 200 palavras por minuto é suficiente |
| Idioma | "Português (Brasil)" |
| Salvamento | "Todas as alterações salvas", "Salvando..." ou "Sem conexão" |
| Zoom | Controle deslizante e porcentagem |

## 10. Aba Automação e integração com o FeatherSheet

É a aba que o Google Docs e o Word Online não têm de forma completa. Ela aproveita o motor de módulos e macros que você já tem no FeatherSheet.

| Grupo | Controle | Comportamento |
| --- | --- | --- |
| Macros | Executar macro | Lista as macros do documento; atalho configurável |
| Macros | Gravar macro | Grava as ações de formatação e gera o script em JavaScript |
| Macros | Editor de scripts | Painel com código, console e lista de módulos; mesmo motor do FeatherSheet |
| Macros | Botão de macro | Insere um botão clicável no texto ou na barra de ferramentas |
| FeatherSheet | Inserir tabela vinculada | Escolher uma planilha e um intervalo; a tabela aparece no texto com os valores |
| FeatherSheet | Inserir gráfico vinculado | Escolher o gráfico existente na planilha; atualiza quando os dados mudam |
| FeatherSheet | Inserir valor de célula | Campo em linha ligado a uma célula, por exemplo "Receita: R$ 128.400" |
| FeatherSheet | Atualizar vínculos | Atualiza tudo agora; ou automático ao abrir |
| FeatherSheet | Desfazer vínculo | Converte em tabela ou imagem fixa |
| Modelos | Salvar como modelo | Guarda estilos, margens, cabeçalho e texto-base |
| Modelos | Galeria de modelos | Relatório, currículo, carta, ata, artigo ABNT, proposta |
| Modelos | Campos de mesclagem | Marcadores como `{{cliente}}` preenchidos a partir de uma linha da planilha, para gerar vários documentos |
| Exportar | PDF | Margens e quebras iguais às da tela; sumário com links; opção PDF/A |
| Exportar | Word (.docx) | Estilos e equações preservados |
| Exportar | LaTeX (.tex) | Títulos, equações, tabelas, citações e imagens convertidos |
| Exportar | Markdown (.md) | Para blogs e repositórios |
| Exportar | HTML e texto simples | Para web e uso rápido |
| Exportar | Exportação em lote | Roda uma macro que exporta vários documentos de uma vez |

**Exemplo de macro que o editor poderia executar**

O exemplo abaixo usa o mesmo estilo de API do documento anterior, aplicado ao texto.

```javascript
// Padroniza um relatório: título, sumário e exportação
async function prepararRelatorio(doc) {
  doc.setStyle(doc.find("Introdução"), "Título 1");
  doc.insertTOC({ levels: 3, leader: "dots", pageNumbers: true });
  doc.setMargins({ top: 2.5, bottom: 2.5, left: 3, right: 2 }); // cm
  doc.insertPageNumber({ format: "Página {n} de {total}", position: "footer" });
  await doc.exportPDF("relatorio.pdf");
}
```

Os nomes `insertTOC`, `setMargins` e `exportPDF` são sugestões para a sua API; ajuste ao que já existir no motor de scripts.

## 11. Ordem de implementação sugerida

Comece pelo que torna o editor utilizável e pelo que você pediu (equações, títulos, sumário), e deixe para o fim o que depende de tudo o resto.

| Fase | Entrega | Por que nesta ordem |
| --- | --- | --- |
| 1 | Fonte, Parágrafo, alinhamento, listas, Pincel, Limpar formatação, Localizar e Substituir | Sem isso o editor não serve para escrever |
| 2 | Estilos de título 1 a 6, galeria de estilos, atalhos Markdown, painel de estrutura | Os títulos sustentam sumário, numeração e navegação |
| 3 | Sumário (níveis, líderes, links), numeração de páginas, cabeçalho e rodapé | Pedido direto; já fecha o ciclo de documento longo |
| 4 | Equações: editor, modelos, símbolos, entrada LaTeX, numeração | Pedido direto; maior risco técnico, melhor começar cedo |
| 5 | Tabelas completas, imagens com quebra de texto, links, bloco de código | Conteúdo rico do dia a dia |
| 6 | Layout: margens ligadas à régua, orientação, papel, colunas, seções | Exige o modo de páginas funcionando |
| 7 | Referências: notas de rodapé, legendas, referência cruzada, citações ABNT | Depende de títulos, legendas e numeração prontos |
| 8 | Revisão: comentários, sugestões, histórico de versões | Depende do modelo de dados estável |
| 9 | Automação: macros, tabelas e gráficos vinculados ao FeatherSheet, modelos | Diferencial competitivo; aproveita o motor existente |
| 10 | Exportação fiel para PDF, Word, LaTeX e Markdown | Validar contra o documento final de cada fase |

Um cuidado desde a fase 1: guarde o documento como uma árvore de blocos (parágrafo, título, tabela, equação, imagem) com estilos nomeados, e não como HTML solto. Sumário, numeração, referência cruzada e exportação ficam muito mais simples assim.
