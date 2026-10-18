# OpenDoc — o contrato

**Direção atual — 07/10/2026: motor e interface 100% Rust, com Iced.**
Prioridade: **FeatherSlide → FeatherDocs → FeatherSheet**; FeatherCAD
depois, se houver tempo. O inventário abaixo registra funcionalidades e
decisões anteriores; a migração da interface deste aplicativo ainda deve
ser conferida no código, sem presumir que os recursos já existem em Iced.

Decidido em 24/09/2026, pelo usuário. Ainda não existe código — este
documento cumpre o mesmo papel que `feathersheet/docs/arquitetura/ARQUITETURA.md` cumpriu
antes de o motor existir, e que `openpdf/ARQUITETURA.md` cumpre hoje: **o
motor é Rust, a interface desenha** (regra 2 do `CLAUDE.md` da raiz).

## O problema que o OpenDoc resolve

Word/Google Docs/LibreOffice Writer quebram layout a cada Enter ou imagem
colada porque o documento não tem paginação determinística — o motor
"adivinha" a cada redesenho. LaTeX resolve isso com rigor, mas com sintaxe
punitiva e uma instalação de vários gigabytes. O OpenDoc não imita a
arquitetura do Word (XML solto, estilo herdado que corrompe) nem exige
compilação do LaTeX puro: escrita em blocos/Markdown por cima, com a
mesma precisão de paginação de Typst/InDesign calculada em Rust.

## O contêiner

```
documento.odoz (ZIP/zstd — contrato geral em FORMATO.md)
├── manifest.json          # metadados, autor, contagem de palavras, versão
├── .git/                  # cada revisão relevante é um commit (ver FORMATO.md)
├── document/
│   ├── tree.json           # AST: a árvore de blocos, schema estrito
│   ├── metadata.json       # variáveis dinâmicas ({{cliente}}, {{data}}…)
│   ├── styles.json         # folha de estilo semântica (página, margem, título)
│   └── equations.cache     # SVG pré-renderizado das fórmulas (evita recompilar)
├── assets/                 # imagem/vetor, deduplicados por hash
└── settings/
    └── dictionary.json     # palavras aceitas só neste documento
```

## Modelo de dados: AST tipada, não HTML solto

`document/tree.json` não guarda marcação livre — é uma árvore de nós com
tipo fechado: `heading`, `paragraph`, `table`, `image`, `code_block`,
`callout`, `page_break`. Um *span* de texto carrega a formatação como
anotação estrutural (`bold: true`, `color: "#333"`), não como tag que se
acumula e corrompe (a doença do Word). O mesmo princípio de `feathersheet-core`:
tipo fechado, nada de "quase qualquer coisa" guardado como texto opaco.

## O motor de paginação — a maior peça do projeto

Isto é, em dificuldade, comparável ao motor de fórmulas do `feathersheet-core`:
não existe atalho, é um layout engine de verdade, e é o que faz o OpenDoc
valer a pena escrever.

## Integração com o resto da família

- **Tabela viva do FeatherSheet.** Um bloco `table` pode ser estático ou
  espelhar uma consulta do OpenQuery num `.fshz` — "Atualizar" relê a
  fonte e recalcula valor, total e formatação. Isso exige que o OpenDoc
  saiba ler `RecordBatch` Arrow e o `pipeline.json` de outro pacote **sem
  importar código do FeatherSheet por caminho relativo** (regra 3 do
  `CLAUDE.md` da raiz) — nasce como crate comum (leitor de pasta Arrow +
  pipeline), com dono definido, do qual os dois aplicativos dependem.
- **Vetor nativo do OpenDraw.** Um diagrama inserido não é PNG — é o bloco
  `.odrz` embutido de verdade, editável com dois cliques dentro do texto e
  exportado em curva vetorial nítida, não rasterizado. Mesma exigência de
  crate comum acima (leitor de cena do OpenDraw), não import relativo.
- **Variáveis globais.** `{{empresa_cliente}}`, `{{valor_total}}` em
  `metadata.json`, substituídas em toda ocorrência do texto ao editar o
  painel — mecanismo de find/replace estrutural na AST, não regex sobre
  string solta.

## Produtividade

- **Atalhos estilo Markdown** (`# `, `- `, `> `, `| col |`) convertidos em
  nó da AST na hora — o usuário nunca vê marcação crua.
- **Numeração multinível** (1, 1.1, 1.1.1) recalculada ao mover bloco;
  **legenda e referência cruzada automáticas** ("Figura 3", "Tabela 2 na
  página 14") atualizam sozinhas ao mover o alvo.
- **Sumário dinâmico**, gerado da árvore, com pontilhado até o número da
  folha.
- **Revisão**: comentário ancorado ao bloco na margem; modo
  sugestão/rastreamento (inserção/exclusão marcada, aceitar ou recusar);
  cada etapa de revisão relevante é commit no Git embutido — é o "time
  travel" de `FORMATO.md` aplicado à revisão de documento, não só ao
  arquivo inteiro.

## Fórmulas matemáticas (LaTeX inline e em bloco)

`$E = mc^2$` inline, `$$ \int_a^b f(x)\,dx $$` em bloco, com numeração
automática e referência cruzada ("vide Equação 3") — sem exigir TeX Live
instalado (5 GB). O SVG renderizado fica em cache
(`document/equations.cache`) para não recompilar a cada abertura. **Isto é
o ponto que mais precisa de decisão explícita** (ver abaixo) — não tem
resposta tão limpa quanto o shaping de texto teve.

**Decidido em 15/10/2026:** a montagem é do crate comum
`crates/feather-math` (Rust puro, `ttf-parser`): o algoritmo do
apêndice G do TeXbook com as constantes da tabela OpenType `MATH` da Latin
Modern Math, embutida no crate (GUST Font License, ver `assets/LICENCA.md`).
O documento guarda o LaTeX (`Conteudo::Equacao` com o ambiente inteiro, ou o
trecho com `MATEMATICA`); o layout pede a fórmula composta (glifos + retângulos
+ largura/altura/profundidade), a tela desenha o contorno de cada glifo e o
`.docx` sai em OMML (equação nativa do Word), que também é lido de volta. Não
há cache em arquivo: compor é rápido e o resultado fica guardado na memória.

## Formatação de página rígida

- Presets de norma (ABNT/IEEE/ISO) com margem, entrelinha e recuo prontos
  num clique; papel parametrizável (A4/A3/Carta/customizado).
- **Orientação mista na mesma árvore**: o documento segue em retrato, uma
  página específica (tabela larga do FeatherSheet, esquema do OpenDraw) vira
  paisagem sem quebrar a numeração — é decisão de layout por seção, não
  por documento inteiro.
- Cabeçalho/rodapé com escopo: capa sem numeração, par/ímpar espelhado
  para frente-e-verso, variável de página atual/total e título de seção.

## Imagem e vetor

- `assets/` deduplicado por hash de conteúdo — duas imagens idênticas
  ocupam espaço uma vez só.
- SVG e `.odrz` nativos: zoom sem pixelizar.
- Reamostragem automática (foto de 48 MP → 300 DPI impressão / 150 DPI
  tela) sem perda perceptível.
- Legenda obrigatória estruturada (`Figura X — título`, `Fonte: …`), não
  texto solto do lado da imagem.

## Correção ortográfica — local, sem telemetria

Resolvido para Rust puro: em vez de vincular `libhunspell` (C/C++),
**`zspell`** lê o mesmo formato de dicionário Hunspell (`.dic`/`.aff`, o
que o LibreOffice e os navegadores já usam) reimplementado em Rust — evita
a mesma classe de dependência de sistema que `gix`/`cosmic-text` já
evitam. Dicionário técnico pessoal (`settings/dictionary.json`) e
correção tipográfica automática (aspas retas → curvas, `--` → em-dash,
`...` → reticências) rodam na mesma thread de fundo.

## Exportação

- **PDF/A instantâneo**: o crate do `openpdf` entra como dependência (não
  código duplicado, mesma regra 3) — fonte embutida, metadado higienizado,
  pronto para assinatura.
- **Markdown puro / Typst**: a AST sai em texto legível para quem não quer
  depender de nenhum software da suíte para ler o conteúdo depois.

## Pontos que precisam de decisão antes de alguém começar a implementar

3. **O crate comum de leitura entre apps** (tabela Arrow do FeatherSheet,
   cena do OpenDraw) ainda não tem nome nem dono. Precisa nascer antes da
   primeira tabela viva funcionar — sem ele, a tentação de importar
   `feathersheet` por caminho relativo (proibido pela regra 3) é grande.
