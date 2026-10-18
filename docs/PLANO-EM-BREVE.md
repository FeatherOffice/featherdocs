# Plano: os comandos que ainda estão "Em breve"

**Direção atual — 07/10/2026: motor e interface 100% Rust, com Iced.**
Prioridade: **FeatherSlide → FeatherDocs → FeatherSheet**; FeatherCAD
depois, se houver tempo. O inventário abaixo registra funcionalidades e
decisões anteriores; a migração da interface deste aplicativo ainda deve
ser conferida no código, sem presumir que os recursos já existem em Iced.

As regras que valem para cada item:

- **Quatro formatos**: cada recurso novo precisa ser lido e gravado no
  `.fdoz` (coluna Arrow nova, opcional na leitura) e no `.docx`. Também
  precisa ser convertido para LaTeX e para Markdown. O que o Markdown não
  representar entra em `conversao::perdas`, e quem salva é avisado antes.
- **Pronto** quer dizer: o comando sai de `breve(...)`, tem teste no motor,
  tem teste de ida e volta em cada formato, foi conferido numa captura de
  tela e passa no clippy (`too_many_lines` e `cognitive_complexity` estão
  como *deny*). A linha correspondente em `breve.*` do `pt-BR.json` sai
  junto.
- **Uma fase de cada vez**, cada uma com commit próprio, medindo testes e
  build ao fim.

---

## Fase 0: as fundações (sem comando visível)

Muitos dos comandos dependem das mesmas três peças. Elas vêm antes de
qualquer outra coisa.

| Peça | O que muda | Quem precisa dela |
|---|---|---|
| **0.1 Âncora no texto** | `Trecho` ganha `marca: Option<Marca>`, um objeto preso a um ponto do texto que o cursor atravessa como se fosse um caractere só. `Marca` é um tipo fechado: `Nota{id}`, `Citacao{chaves, pagina}`, `Indice{termo, sub}`, `Campo{tipo}`, `Comentario{id, inicio}`. | notas, citações, índice, campos, comentários |
| **0.2 Revisão no trecho** | `Trecho` ganha `rev: Option<Rev{tipo: Insercao\|Remocao\|Formato, autor, quando}>`. O layout desenha sublinhado ou tachado na cor do autor. | controle de alterações, aceitar, rejeitar, comparar |
| **0.3 Paginação com rodapé reservado** | `paginar` passa a reservar no pé de cada página a altura das notas que caem nela, num ciclo de "medir e repaginar" que para quando estabiliza (no máximo 3 voltas). `Pagina` ganha `notas: Vec<FatiaNota>`. | notas de rodapé e, depois, colunas e seções |

Mais: limpar do `pt-BR.json` os textos `breve.*` dos comandos já feitos na
onda 1 (imagem_url, marcador, legenda, lista_figuras, referencia_cruzada,
numeracao_linhas, alinhar_vertical, sumario_personalizado, modo_continuo,
grade, miniaturas, zoom_varias).

**Formatos**: o `.fdoz` grava `marca`/`rev` em colunas novas.
**Risco**: a âncora mexe no cursor (`navegacao.rs`, `digitacao.rs`).
Apagar com Backspace uma âncora seleciona o objeto primeiro, como o Word
faz.

---

## Fase 1: notas (3 comandos)

| Comando | Motor | .docx | LaTeX | Markdown |
|---|---|---|---|---|
| `nota_rodape` | `Documento.notas: Vec<Nota{id, texto: Vec<Bloco>}>` + `Marca::Nota` no ponto. O número sai da ordem no documento. | `footnotes.xml` + `w:footnoteReference` | `\footnote{…}` | `[^1]` |
| `nota_fim` | o mesmo, com `Nota.fim = true`; as notas vão para uma página no fim | `endnotes.xml` | `\endnote` (pacote `endnotes`) | `[^1]` (perde a distinção) |
| `formato_notas` | `Extras.notas: {formato: 1,a,i,*, reinicia: documento\|pagina\|secao}` | `w:footnotePr` | `\renewcommand\thefootnote` | perde |

**Tela**: clicar na nota leva ao pé da página e o texto dela é editável ali
mesmo (o cursor entra no bloco da nota). A ida e volta entre a marca no
texto e a nota é Ctrl+clique, como na referência cruzada.

---

## Fase 2: objetos no texto (4 comandos)

| Comando | Como |
|---|---|
| `campo_dados` | `Marca::Campo{Autor\|Titulo\|Data(fmt)\|Paginas\|Pagina\|Arquivo}`. O valor é resolvido no layout (reaproveita o `{pagina}{total}` do cabeçalho em `pagina.rs`). F9 atualiza. docx: `w:fldSimple`. LaTeX: `\thepage`, `\today`, `\@author`. MD: texto congelado. |
| `recolhivel` | `Conteudo::Recolhivel{titulo, aberto, blocos}` ou um atributo de título que recolhe até o próximo do mesmo nível. **Recomendo o atributo**: não aninha blocos e reaproveita a hierarquia que o sumário já usa. No papel aparece tudo; recolhe só na tela. MD: `<details>`. |
| `caixa_texto` | `Conteudo::Caixa{blocos, largura, borda, fundo, flutua: Esquerda\|Direita\|Linha}`. A primeira versão fica **na linha do texto** (bloco com moldura); a caixa flutuante, com o texto contornando, fica para depois. docx: `w:txbxContent`. LaTeX: `\fbox`/`tcolorbox`. |
| `direcao_texto` | `Atrib.rtl: bool`. O layout inverte a ordem das linhas e o alinhamento. Texto bidirecional completo (árabe e latim misturados na mesma linha) usa o crate `unicode-bidi`, que é Rust puro. docx: `w:bidi`. LaTeX: `\begin{RTL}` (polyglossia). |

---

## Fase 3: tabela, mesclar e dividir (2 comandos)

- O modelo muda: `Celula` deixa de ser `Vec<Trecho>` e vira
  `struct Celula{texto, colspan, rowspan, oculta}`. A célula coberta por
  uma mescla fica no lugar, marcada `oculta`, para que a grade continue
  retangular e a navegação (`navegacao.rs`) só pule as ocultas.
- `tab_mesclar` usa a seleção de várias células, que já existe
  (`celula: Cel` em `Pos`, âncora numa célula e foco em outra) e forma um
  retângulo. `tab_dividir` desfaz a mescla ou divide em N×M.
- Os formatos: docx `w:gridSpan`/`w:vMerge`; LaTeX `\multicolumn`/
  `\multirow`; Markdown perde (repete o texto e avisa).
- **Atenção**: migrar o `.fdoz` (a coluna de células muda de forma). Os
  arquivos atuais continuam abrindo: a leitura entende a forma antiga.

---

## Fase 4: layout de página (3 comandos)

| Comando | Como |
|---|---|
| `quebra_secao` | `Conteudo::QuebraDeSecao{config: ConfigPagina, colunas, cabecalho próprio?}`. A paginação passa a correr seção por seção. Margens, orientação e cabeçalho valem até a próxima quebra. docx: `w:sectPr` no parágrafo. LaTeX: `\newgeometry`. |
| `colunas` | `Secao.colunas{n, espaco, linha_entre}`. A paginação preenche coluna a coluna (largura útil ÷ n). Depende da fase 0.3 e de `quebra_secao`. docx: `w:cols`. LaTeX: `multicol`. |
| `hifenizacao` | crate **`hyphenation`** (Rust puro, padrões Knuth-Liang com pt embutido). O `layout.rs` tenta hifenizar a palavra que não coube antes de mandá-la para a linha seguinte. `Extras.hifenizar: bool` + zona de hifenização. docx: `w:autoHyphenation`. LaTeX: o padrão dele. |

---

## Fase 5: referências (6 comandos)

**Bibliografia**:

| Comando | Como |
|---|---|
| `fontes` | `Documento.fontes: Vec<Fonte{chave, tipo, autores, titulo, ano, …}>`. Caixa "Gerenciar fontes" para cadastrar e editar. **Importar `.bib`** com o crate `biblatex` (Rust puro). |
| `citacao_inserir` | `Marca::Citacao{chaves, pagina}` → "(SILVA, 2023, p. 4)" formatado pelo estilo. |
| `estilo_citacao` | ABNT, APA, Vancouver, IEEE, Chicago. **Decisão**: usar o `hayagriva` (Rust puro, lê estilos CSL) ou escrever os 5 estilos à mão. Recomendo começar à mão por ABNT e APA (as regras são poucas e testáveis) e avaliar o `hayagriva` se a lista crescer. |
| `bibliografia` | `Conteudo::Bibliografia` gerado como o sumário (F9 atualiza), só com as fontes citadas ou com todas. docx: campo `BIBLIOGRAPHY`. LaTeX: `biblatex` + `\printbibliography` + `.bib` ao lado. MD: lista em texto. |

**Índice remissivo**:

| Comando | Como |
|---|---|
| `marcar_entrada` | `Marca::Indice{termo, sub}` na seleção (ou em todas as ocorrências, opção "marcar todas"). docx: campo `XE`. LaTeX: `\index{}`. |
| `indice` | `Conteudo::IndiceRemissivo{colunas}`: termos em ordem alfabética (com acentos tratados) e as páginas, que vêm da paginação, como no sumário. docx: campo `INDEX`. LaTeX: `\printindex` (makeidx). |

---

## Fase 6: seleção múltipla (2 comandos)

- `Sessao` passa a ter `cursores: Vec<Cursor>` (o principal + extras). Cada
  comando de formatação aplica a todos. A digitação com vários cursores
  fica **de fora** nesta rodada (só formatar e apagar).
- `selecionar_semelhante` seleciona os trechos com o mesmo `estilo`/`Fmt`
  do cursor. `selecionar_titulos` seleciona todos os blocos `Titulo`.
- A tela desenha várias faixas de seleção. Ctrl+clique para somar seleção
  vem de graça.

---

## Fase 7: revisão (8 comandos)

| Comando | Como |
|---|---|
| `comentario_novo` | `Documento.comentarios: Vec<Comentario{id, autor, quando, texto, respostas, resolvido}>` + `Marca::Comentario` no início e no fim do trecho. A margem direita mostra balões ligados por uma linha. docx: `comments.xml` + `w:commentRangeStart/End`. LaTeX: `\todo` (pacote `todonotes`) ou perda. MD: perde. |
| `comentarios_mostrar` | filtro na tela: todos, só os abertos, só os do autor. O autor é o nome em Arquivo → Opções (sem conta, rule 5). |
| `modo_edicao` | `Sessao.modo: Editando\|Sugerindo\|Leitura`. Em "Sugerindo", digitar grava `Rev::Insercao` e apagar marca `Rev::Remocao` em vez de remover (depende da fase 0.2). docx: `w:ins`/`w:del`. |
| `aceitar` / `rejeitar` | a alteração sob o cursor, ou todas: o motor aplica ou descarta a `Rev`. |
| `historico` | a **janela Versões…** que está pendente: lista `documento_versoes` (já existe na ponte nativa), mostra uma versão em modo leitura, "Restaurar" e "Comparar com a atual". |
| `comparar` | diff de duas versões (do histórico ou de outro arquivo): alinha blocos pelo `id` e, dentro de cada bloco, compara palavra a palavra (crate `similar`, Rust puro). O resultado é um documento com `Rev` marcadas, que se aceita ou rejeita com os comandos acima. |
| `proteger` | **Decisão**: (a) "só leitura" por trecho ou documento, sem segredo, como o Word; ou (b) **criptografar o `.fdoz`** com senha (`aes-gcm` + `argon2`, Rust puro). Recomendo fazer (a) agora e (b) como opção separada em Salvar como…. |

---

## Fase 8: ortografia e idioma (2 comandos)

- O crate **`spellbook`** (Rust puro, lê dicionários Hunspell). Dicionário
  pt-BR do VERO (LibreOffice, licença LGPL/MPL) **embutido no instalador**,
  em torno de 1 a 2 MB compactado. Nada vai para a rede.
- `ortografia`: o motor devolve os intervalos com erro só dos blocos
  visíveis (nada por quadro pela ponte, a regra do FeatherSheet). A tela
  desenha o sublinhado ondulado, e o botão direito mostra as sugestões e
  "Adicionar ao dicionário" (lista pessoal no `localStorage`/config).
- `idioma`: `Fmt.idioma` por trecho (padrão pt-BR). Os outros dicionários
  (en, es) são baixáveis à parte, só por pedido explícito. docx: `w:lang`.
  LaTeX: `\foreignlanguage`.

---

## Fase 9: objetos desenhados (3 comandos)

| Comando | Como |
|---|---|
| `desenho` | `Conteudo::Desenho{tracos: Vec<Vec<(x,y)>>, …}`, que o motor guarda e exporta como **SVG**. A tela captura o ponteiro e manda o traço inteiro ao soltar o botão, não ponto a ponto pela ponte. |

---

## Fase 10: o que está fora da faixa

1. **Exportar PDF**: o PDF de verdade, a partir das mesmas páginas que o
   motor já monta (`quadro.rs`). Crate `pdf-writer` (Rust puro) + subset
   das fontes. Confirmar com o usuário se entra mais algum formato além
   de PDF.
2. **Edição no papel → código**: ao editar no papel, regravar o
   LaTeX/Markdown aberto no painel. O caminho é regerar o fonte pelo
   `markdown_com_mapa`/LaTeX equivalente e preservar o que o usuário
   escreveu à mão nos blocos que não mudaram (o mapa bloco→linhas já
   existe).
3. **Documentos recentes no Início**, iguais aos do FeatherSheet.

---

## Ordem sugerida e tamanho

| # | Fase | Comandos | Tamanho |
|---|---|---|---|
| 0 | Fundações | — | médio (mexe no cursor) |
| 1 | Notas | 3 | médio |
| 2 | Objetos no texto | 4 | médio |
| 3 | Mesclar/dividir | 2 | médio (migra o .fdoz) |
| 4 | Seção, colunas, hifenização | 3 | grande (paginação) |
| 5 | Bibliografia e índice | 6 | grande |
| 6 | Seleção múltipla | 2 | pequeno-médio |
| 7 | Revisão + Versões… | 8 | grande |
| 8 | Ortografia | 2 | médio |
| 9 | Formas, desenho, gráfico | 3 | grande |
| 10 | PDF, papel→código, recentes | — | grande |

O que a fase 1 do **seu** pedido de 06/10 pede primeiro (bibliografia,
índice, notas, revisão) cai nas fases 1, 5 e 7. Se quiser chegar a elas mais
cedo, a ordem pode ser **0 → 1 → 5 → 7**, deixando 2, 3, 4, 6, 8 e 9 para
depois. Nada nessas três depende das outras, só da fase 0.

## Decisões em aberto

1. Ordem: a de cima (0→10) ou começar pelos seus pedidos (0→1→5→7)?
2. Estilos de citação: à mão (ABNT/APA primeiro) ou `hayagriva` (CSL)?
3. Proteger: só leitura sem segredo, criptografia com senha, ou os dois?
4. Gráfico: dados próprios agora e `.fshz` depois?
5. Dicionário pt-BR embutido no instalador (+1 a 2 MB): ok?
6. Recolhível: atributo do título (recomendado) ou bloco que contém outros?
