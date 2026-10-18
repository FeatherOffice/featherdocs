# Registro de tarefas

> Era o README de `featherdocs/` enquanto o produto morava no repositório de desenvolvimento da suíte (até 15/10/2026). Fica como histórico das entregas e validações; links para `featheroffice/` e outros produtos apontam para pastas que não existem neste repositório.


> **Produto independente — repositório planejado:** `FeatherOffice/featherdocs`.
> O projeto ainda está nesta pasta do repositório de desenvolvimento; a separação Git não foi realizada.

**Documentos:** formatos `.fdoz`, `.docx`, `.tex` e `.md`; desenvolvimento e versões próprios.

**Arquitetura compartilhada:** utiliza a base de motores Rust em [`featheroffice/`](../../featheroffice/) e os componentes de interface Rust/Iced de [`feather-ui/`](../../feather-ui/) quando aplicáveis. As dependências compartilhadas serão versionadas sem acoplar um produto ao código interno de outro.

**Publicação prevista:** versão **alpha em 30/10/2026**. O produto terá distribuição própria e também poderá ser selecionado isoladamente ou junto aos demais em um **instalador geral planejado**. Isso não significa que a funcionalidade de instalação já esteja pronta.


## UI-013 — Ajuda F1 completa em Rust/Iced (validação pendente)

A janela F1 do FeatherDocs agora usa o componente `feather-ui::ajuda`
com **Como usar** (sete assuntos), **Atalhos** (30 combinações em cinco
grupos), busca global sem acentos, árvore lateral, introdução por seção,
navegação anterior/próximo e **Ctrl+F** no campo de pesquisa.
Os textos originais pt-BR/en-US são incorporados no binário; para testar
inglês, inicie com `FEATHEROFFICE_LANG=en-US`. Esc/F1 fecham o painel
sem alterar o documento.

**Cargo/smoke ainda não executados**, e a seleção de idioma pela
preferência nativa precisa ser ligada.
[Arquitetura e comandos de validação](../../featheroffice/UI-013-AJUDA-RUST.md).

## IO-014 — Base64 consolidado em Image (compilação pendente)

O antigo `featherdocs-io::base64` mantém os quatro símbolos públicos,
agora como reexportações de `featheroffice-image::base64`. O algoritmo duplicado
foi removido; FDOZ, DOCX e os gráficos SVG utilizam os mesmos pontos de
entrada. **Testes de compilação e round-trip externos continuam pendentes.**
[Guia e comandos](../../featheroffice/IO-014-DEDUP-BASE64.md).

## IMG-014 — Efeitos de imagem compartilhados (em validação)

Selecione uma imagem PNG/JPEG embutida no documento e escolha **Inserir →
Efeitos imagem**. O painel `feather-ui` oferece Original, Cinza, Desfocar e
Nitidez; os filtros são implementados apenas em `featheroffice-image`.
O motor Docs atualiza o conteúdo da imagem numa única transação, preservando
ID, legenda, dimensões, undo/redo e `.fdoz`. Gráficos SVG não são
rasterizados. Limite interativo: 4 milhões de pixels. Testes escritos,
Cargo/smoke ainda não aprovados. [Detalhes e comandos](../../featheroffice/IMG-014-EFEITOS-DOCS-SHEET.md).


## APP-007 — Gráficos Charts editáveis em FDOZ e DOCX (validação pendente)

Na aba **Inserir**, com o cursor dentro de uma tabela com cabeçalho e dados
numéricos, é possível inserir **Colunas, Barras, Linhas, Área, Pizza,
Dispersão ou Degraus** usando o motor compartilhado `featheroffice-charts`.
Clique no gráfico e escolha **Atualizar selecionado** para recalcular seus
dados. A ação fica desabilitada em uma imagem ou SVG não reconhecido.

O gráfico permanece como SVG com vínculo embutido à tabela; novas referências
guardam também ordem e cabeçalhos para resistir à renumeração de IDs após
DOCX. O gráfico pode ser reimportado/atualizado no FeatherDocs, mas não vira
um gráfico OOXML editável no Word. O FDOZ nativo não mudou. **Testes Cargo,
GUI e abertura em Word/LibreOffice pendentes; APP-007 fica `[/]`.**
[Guia, limitações, testes e exemplo DOCX/SVG/FDOZ](../../featheroffice/APP-007-GRAFICOS-DOCS.md).

## Gráficos no FeatherDocs (CHT-016, em validação)

Na guia **Inserir**, após preparar uma tabela com cabeçalho, use
**Gráfico colunas** ou **Gráfico linhas**. A primeira coluna é categoria,
as demais contêm séries numéricas; o SVG é gerado pelo motor Charts comum,
não pelo Iced. Para refletir alterações, clique no gráfico e pressione
**Atualizar gráfico**. O documento mantém a tabela original e o SVG com
vínculo de origem; FDOZ reabre ambos. Undo/redo usa a transação do motor Docs.
**Cargo, round-trip DOCX externo e inspeção visual ainda pendentes**.
[Guia de arquitetura e testes](../../featheroffice/CHT-016-CHARTS-FEATHERDOCS.md).


## APP-006 — Edição de tabelas na interface nativa (validação pendente)

Na aba **Inserir**, o FeatherDocs oferece os comandos de tabela:
**Linha +/−**, **Coluna +/−**, **Mesclar →**, **Desmesclar**,
**Cabeçalho** e **Excluir tabela**. Os comandos de edição ficam
desativados enquanto o cursor não estiver em uma célula; selecione
uma célula da tabela para habilitá-los.

- A inserção/exclusão de linhas e colunas, mesclagem e desmesclagem
  passam por `featheroffice_table_adapters::docs_editar_estrutura`,
  `docs_mesclar_celulas` e `docs_desmesclar_celulas`.
- O adaptador manipula índices pelo motor `featheroffice-tables`,
  preservando os `Trecho` originais (formatação, links e conteúdo).
  O `featherdocs-core::Sessao` mantém a transação, seleção, paginação
  e histórico; não há duplicação de modelo nem mudança em `.fdoz`.
- A alternância de **Cabeçalho** e **Excluir tabela** usam o motor
  Docs, pois o cabeçalho semântico e a substituição do bloco são
  operações específicas do documento.
- Mesclar não descarta conteúdo de células preenchidas. Se a operação
  for inválida, o editor apresenta um erro e mantém a tabela original.
- Quatro linhas de controles foram distribuídas em grupos mais baixos
  para não ficarem cortadas pela altura fixa da Ribbon.

**APP-006 = `[/]` (em validação).** Testes Rust foram escritos na GUI
para cabeçalho/exclusão com undo, conteúdo rico após mudanças estruturais,
recusa de mesclagem destrutiva, repaginação e round-trip `.fdoz`.
Não houve execução do Cargo neste ambiente; antes de encerrar é preciso
obter `cargo check/test` e validar visualmente, inclusive tabelas
que se estendem por várias páginas.

Execute na raiz do repositório:

```bash
cargo run --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml -j 2
cargo test --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml app006_ -j 2
cargo run --manifest-path featheroffice/Cargo.toml -p featheroffice-table-adapters --example validacao_docs -j 2
cargo run --manifest-path featheroffice/Cargo.toml -p featheroffice-table-adapters --example validacao_mesclagem_docs -j 2
```

## APP-008 — Controles compartilhados do FeatherDocs (em validação)

A interface nativa Iced do FeatherDocs consome o `feather-ui` para barra de
arquivos, menu principal, abas Ribbon, dropdowns tipados de fonte/tamanho,
configurações e seletor de cores HSV/RGBA. A aba **Início** oferece
**Cor do texto** e **Marca-texto**. O painel reutiliza
`feather_ui::seletor_cor::painel` e entrega os eventos à sessão do
`featherdocs-core` via `cor_definir` e `realce_definir`.

Para experimentar, selecione texto, abra **Início → Cores**, escolha uma cor
pré-definida ou edite Hex/RGBA/HSV; a alteração aparece na página.
**Sem cor** restaura o padrão do texto ou remove o marca-texto.
**Fechar** e **Esc** fecham o painel sem apagar formatação existente.
Alterações válidas entram no histórico do documento, preservadas no `.fdoz`
e no fluxo de desfazer/refazer. Hex incompleto/inválido não modifica o
documento; não há necessidade de duplicar conversões de cores na GUI.

A lógica de cores permanece no motor Docs; `feather-ui` não conhece
`Documento`, sessões ou formatos. O painel é exibido numa faixa adicional
abaixo da Ribbon (não em uma janela flutuante). Gestos de arrasto do
HSV podem registrar mais de um passo no histórico enquanto a cor muda.

**Status APP-008 = `[/]`:** código e testes escritos, porém a conclusão
depende de Cargo/CI e inspeção visual em Linux e Windows.

```bash
cargo run --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml -j 2
cargo test --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml app008_ -j 2
cargo test --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml cor_ui::testes -j 2
```

## APP-005 — Imagens pelo Drawing/Image compartilhado

O fluxo real de **Inserir imagem** da GUI agora utiliza
`featherdocs-gui::imagem_compartilhada` para validar PNG/JPEG e dimensões
antes de embutir os **bytes originais** no documento. A prévia Iced usa
`featheroffice-image::ImagemRgba` e o redimensionamento bilinear com
alfa pré-multiplicado. As caixas são validadas em pontos pelo Drawing.

`fdoz`, DOCX, paginação, cursor e undo/redo não foram modificados;
a serialização continua recebendo a imagem original, e não a miniatura.
Esta etapa cobre inserção e exibição; impressão, exportações adicionais
e edição avançada ainda não estão migradas. **APP-005 = `[/]`.**

[Detalhes, limites e testes](../../featheroffice/APP005-DOCS-IMAGEM.md).

```bash
cargo test --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml imagem_compartilhada::testes -j 2
cargo run --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml --example validacao_imagem_docs -j 2 -- /tmp/docs-imagem
cargo check --all-targets --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml -j 2
```



## Persistência nativa e regressão (IO-009 / IO-013)

O codec `.fdoz` usa ZIP/Arrow e preserva a estrutura do documento.
**IO-009** testa leitura e regravação de textos Unicode e tabelas;
**IO-013** amplia para uma tabela extensa e para revisões, comentários
e respostas. Esses testes de dados **não** demonstram histórico completo
de desfazer/refazer. Os dois itens estão `[/]` até Cargo e inspeção.

```bash
cargo run --manifest-path featherdocs/crates/featherdocs-io/Cargo.toml --example io009_roundtrip -j 2 -- /tmp/io009-docs.fdoz
cargo run --manifest-path featherdocs/crates/featherdocs-io/Cargo.toml --example io013_extremos -j 2 -- /tmp/io013-docs.fdoz
```

[Corpus IO-009](../../featheroffice/fixtures/io009/README.md) ·
[Estresse IO-013](../../featheroffice/fixtures/io013/README.md)


> **Interface nativa (08/10/2026):** `crates/featherdocs-gui/` usa Rust/Iced,
> o tema e os componentes de `../feather-ui/`, com os motores existentes.
> A interface Tauri permanece disponível durante a migração dos demais recursos.


Editor de documentos da suíte FeatherOffice, formato próprio `.fdoz`.
Abre e grava também `.docx`, `.tex` e `.md`.

**Direção: motor e interface 100% Rust, com Iced.** Segundo foco de trabalho,
depois do FeatherSlide e antes do FeatherSheet. FeatherCAD vem depois, se houver tempo.

O modelo e as regras existentes devem ser reaproveitados na migração da
interface nativa. O contrato está em [`ARQUITETURA.md`](../ARQUITETURA.md);
o plano de funcionalidades está em [`docs/PLANO-EM-BREVE.md`](../docs/PLANO-EM-BREVE.md).

## Executar a interface nativa

Na raiz do repositório:

```bash
cargo run --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml
# Build otimizado:
cargo run --release --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml
# Abrir um documento:
cargo run --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml -- documento.fdoz
```

Clique na página para escrever. A faixa oferece formatação, títulos, listas,
tabelas 3 × 3, imagens PNG/JPEG, sumário, papel e orientação. A seleção, o cursor,
a paginação e o desfazer são do `featherdocs-core`. Ctrl+O abre, Ctrl+S salva,
Ctrl+Shift+S salva como, Ctrl+F localiza; os atalhos de edição do motor continuam
disponíveis. Use Revisão para sugerir alterações ou entrar em leitura.

Abre `.fdoz`, `.docx`, `.md` e `.tex`; salva o documento em `.fdoz` e exporta
cópias em Word, Markdown ou LaTeX. Importar preserva o arquivo de origem.
Novo, Abrir e Fechar pedem uma decisão de salvamento quando há alterações.
Preferências de tema, acento, estrutura e zoom ficam em
`~/.config/featherdocs/config.json` (ou `$XDG_CONFIG_HOME/featherdocs/`).

Esta etapa ainda não oferece na interface nativa PDF, impressão, comentários,
histórico Git, bibliografia, edição de fórmulas, configuração avançada de tabelas
e a totalidade dos controles da interface anterior. Esses dados são mantidos
no modelo e no `.fdoz`; o acesso aos respectivos editores será migrado por etapas.
O canvas ainda desenha todas as páginas; virtualização e IME precisam de validação.

## Verificar

```bash
cargo check --all-targets --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml
cargo test --manifest-path featherdocs/crates/featherdocs-gui/Cargo.toml
cargo test --manifest-path featherdocs/crates/featherdocs-core/Cargo.toml
cargo test --manifest-path featherdocs/crates/featherdocs-io/Cargo.toml
```
