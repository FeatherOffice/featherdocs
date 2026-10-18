# Paridade de abrangência do leitor LaTeX (FeatherDocs)

Levantamento de 18/10/2026. Trabalho só de leitura de código: nada foi compilado, testado nem executado. Nenhum arquivo existente foi alterado.

**Relação com o inventário anterior.** Já existe `docs/PARIDADE-LATEX.md` (medição de 15/10/2026, gerada por exemplo `paridade_latex` que roda o leitor). Este documento é outra coisa: uma leitura do código, comando por comando, para responder "o leitor trata isto ou não". Não substitui a medição e não a corrige. Onde os dois se cruzam, a medição real vale mais que a leitura.

## Método e legenda

- Caminhos de código são relativos a `crates/featherdocs-io/src/` (ex.: `latex.rs:1145`, `latex/avisos.rs:182`). Fórmulas vão ao crate `feather-math` sem auditoria da matemática.
- Um item só recebe `faz` quando achei o tratamento no código. Quando não achei, a linha diz "não achei".
- Itens marcados "(inferido)" são comportamentos que deduzi lendo o código, sem executar. Devem ser confirmados com um trecho mínimo no exemplo de paridade.
- Estados, definidos pelo que aparece no documento lido:
  - `faz`: tratado e com efeito igual ao do LaTeX dentro do escopo do item.
  - `faz em parte`: tratado, mas falta algo (o que falta vem depois do traço).
  - `ignora em silêncio`: some do documento sem aviso e sem efeito.
  - `avisa`: não tratado; o leitor registra aviso (`avisos.rs`). O texto pode sumir ou vazar, o que vem na descrição.
  - `não faz`: não tratado e sem aviso, ou tratado de jeito que produz saída errada sem aviso (ex.: `\pageref` sai `[chave]`).
- Regra geral do código, verificada em `latex.rs:989` (função `barra`, fallback no fim da função): um comando desconhecido com argumento `{…}` tem o argumento impresso como texto e gera aviso. Por isso `\cref{fig:a}` sai `fig:a` e `\parbox{5cm}{x}` sai `5cm x`.
- Regra do preâmbulo: o leitor só analisa o preâmbulo para classe, pacotes (`pacotes.rs:7`), `\title`, `\author`, geometria, cabeçalho, cores, macros e `\setlist`. Outros comandos do preâmbulo somem sem aviso (ex.: `\newtcolorbox`, `\titlespacing`, `\lstdefinestyle`).

---

## 1. Núcleo LaTeX

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\documentclass[opções]{classe}` | Define a classe, o corpo (10/11/12 pt) e o papel | faz em parte - aceita article, report, book, letter, scrartcl, scrreprt, scrbook, extarticle, extreport, memoir, abntex2, minimal; corpo 10/11/12 pt; papel só a4paper e letterpaper; a5paper e b5paper somem sem aviso | `latex.rs:773` (Classe::do_preambulo), `latex.rs:301` (configurar_pagina), `latex/avisos.rs:216` |
| Classe desconhecida | Classe qualquer | avisa - sai como article e registra aviso de classe | `latex/avisos.rs:216`, `latex/pacotes.rs:8` |
| Texto corrido e parágrafos (linha em branco) | Parágrafo | faz | `latex.rs:1865` (paragrafo), `latex.rs:1876` (fim_do_paragrafo) |
| `\\` e `\newline` | Quebra de linha | faz | `latex.rs:989` (barra, ramo `\\`) |
| `\linebreak` | Quebra de linha | faz | `latex.rs:989` |
| `\par` | Fim de parágrafo | faz | `latex.rs:1876` (fim_do_paragrafo), `latex.rs:1453` (IGNORADOS) |
| `\\[2mm]` (com medida) | Espaço vertical entre parágrafos | faz em parte - a medida vira espaço no fim do parágrafo; dentro de tabela o `[2mm]` vaza como texto na célula seguinte (inferido) | `latex.rs:1876`, `latex.rs:2253` |
| `\noindent`, `\indent` | Recuo da primeira linha | ignora em silêncio - o leitor não tem recuo de primeira linha; `\parindent` fica em zero | `latex.rs:1453` (IGNORADOS), `latex.rs:401` (exportação) |
| `\bigskip`, `\medskip`, `\smallskip` | Espaço vertical padrão | ignora em silêncio - sem espaço | `latex.rs:1453` (IGNORADOS) |
| `\vspace{medida}` | Espaço vertical | faz em parte - vale entre blocos e no fim de parágrafo; dentro de parágrafo na linha é consumido e some | `latex.rs:1685` (declaracao_de_bloco), `latex.rs:1024` |
| `\vspace*{\fill}` no fim do corpo | Texto centralizado ou na base da página | faz | `latex.rs:366` (extras_da_pagina) |
| `\hspace{medida}`, `\hspace*` | Espaço horizontal | faz em parte - vira espaços inquebráveis com a largura da fonte (aproximação) | `latex.rs:1011` |
| `\hfill`, `\hfil` | Espaço que empurra o texto | faz em parte - vira tabulação no texto; o efeito de empurrar depende do fim da linha | `latex.rs:1143` (simbolo), `latex.rs:1011` |
| `\quad`, `\qquad`, `\,`, `\;`, `\!`, `\:`, `\enspace`, `\thinspace` | Espaços curtos | faz - viram um espaço (todos iguais, sem medida) | `latex.rs:1129` (simbolo) |
| `\ ` (contrabarra e espaço) | Espaço | faz | `latex.rs:1129` |
| `~` | Espaço inquebrável | faz | `latex.rs:888` (corre) |
| `\setlength{\comprimento}{medida}` no corpo | Muda medidas | ignora em silêncio - consome os dois argumentos e não aplica nada | `latex.rs:1024` |
| `\setlength{\parskip}{…}` no preâmbulo | Espaço entre parágrafos | faz | `latex.rs:265` (comprimento_definido) |
| `\setlength{\parindent}{…}` no preâmbulo | Recuo | ignora em silêncio | `latex.rs:265` (só parskip é lido) |
| `\onehalfspacing`, `\doublespacing`, `\singlespacing`, `\linespread` (setspace) | Entrelinha 1,5 ou 2 (ABNT pede 1,5) | ignora em silêncio - não aplicado | `latex/avisos.rs:184` (comando_conhecido) |
| `\baselinestretch` via `\renewcommand` | Entrelinha | avisa - registra SemEfeito | `latex.rs:1450` (SEM_EFEITO), `latex.rs:1741` |
| `\centering`, `\raggedright`, `\raggedleft` entre blocos | Alinhamento do bloco seguinte | faz | `latex.rs:1685` (declaracao_de_bloco) |
| `\centering`, `\raggedright` dentro de parágrafo | Alinhamento | ignora em silêncio - some na linha; o alinhamento só vale entre blocos | `latex.rs:1143` (simbolo, lista de vazios) |
| `\raggedright` no preâmbulo (ou `ragged2e`) | Texto sem justificar por padrão | faz | `latex.rs:77` (ler_texto, campo alinhamento do Ctx) |
| `\begin{center}`, `flushleft`, `flushright`, `justify`, `raggedright` | Alinhamento de trecho | faz | `latex.rs:2060` (ambiente) |
| `\tiny` a `\Huge` (declarações) | Tamanho do texto | faz | `latex.rs:970` (declaracao), `latex.rs:790` (tamanho_de) |
| `\fontsize{t}{b}\selectfont` | Tamanho exato | faz | `latex/linha.rs:10` (extra); `\selectfont` em `latex.rs:1453` |
| `\bfseries`, `\bf`, `\mdseries`, `\itshape`, `\it`, `\em`, `\slshape`, `\sl`, `\upshape`, `\ttfamily`, `\tt`, `\normalfont`, `\rm` | Declarações de série, forma e família | faz | `latex.rs:970` (declaracao) |
| `\rmfamily`, `\sffamily` | Família serifada ou sem serifa | avisa - não achei tratamento como declaração; `\sfdefault` no preâmbulo é lido | `latex.rs:970`, `latex.rs:773` (Classe::do_preambulo, `\sfdefault`) |
| `\textbf`, `\mathbf` (fora de `$`) | Negrito | faz | `latex.rs:1157` (com_argumento) |
| `\textit`, `\textsl`, `\emph`, `\mathit` | Itálico | faz | `latex.rs:1157` |
| `\textup`, `\textrm`, `\textnormal`, `\text`, `\mbox` | Texto normal dentro de fórmula ou caixa | faz | `latex/linha.rs:19` |
| `\textsf` | Sem serifa | faz | `latex.rs:1157` |
| `\texttt`, `\code`, `\verb`, `\lstinline` (com chaves) | Código | faz em parte - com chaves sim; `\verb` com delimitador de barra não (ver linha seguinte) | `latex.rs:1169` |
| `\verb` com delimitador de barra (não é chave) | Código em linha | avisa - sai o delimitador e o texto como texto comum (o argumento não é `{…}`) | `latex.rs:1169`, `latex.rs:1187` (fallback de barra) |
| `\underline`, `\uline` | Sublinhado | faz | `latex.rs:1157` |
| `\sout`, `\st` | Tachado | faz | `latex.rs:1157` |
| `\ul`, `\xout`, `\dashuline` (ulem) | Sublinhados e tachados extras | avisa | sem tratamento (fallback de `latex.rs:1187`) |
| `\hl`, `\sethlcolor` (soul) | Marca-texto | faz | `latex/linha.rs:25` (sethlcolor), `latex/linha.rs:30` (hl) |
| `\textsuperscript`, `\textsubscript` | Expoente e índice | faz | `latex/linha.rs:38` |
| `\MakeUppercase`, `\uppercase`, `\MakeLowercase`, `\lowercase` | Caixa | faz | `latex.rs:1206` (trocar_caixa) |
| `\textsc`, `\scshape` | Versalete | avisa - sai em caixa normal | `latex/linha.rs:1` (só menciona no comentário); sem tratamento |
| `\LaTeX`, `\TeX` | Logotipos | faz | `latex.rs:1129` (simbolo) |
| `\LaTeXe` | Logotipo | avisa - não achei | sem tratamento |
| `\today` | Data atual | ignora em silêncio - vira texto vazio (a exportação grava `\today{}` no cabeçalho, mas ao reler some) | `latex.rs:1145` (simbolo, ramo `"today"` vira ""); `exportar.rs:212` |
| `\date{…}` | Data no título | ignora em silêncio - o argumento é descartado e a data não entra no `\maketitle` | `latex.rs:1453` (IGNORADOS), `latex.rs:1788` (titulo_do_documento) |
| `\title{…}`, `\author{…}` | Título e autor | faz | `latex.rs:1666` (ramo), `latex.rs:1776` (guardar_meta) |
| `\maketitle` | Bloco de título | faz em parte - título vira Titulo nível 1 e autor vira parágrafo em itálico; data, `\thanks` e a capa de abntex2 não entram | `latex.rs:1632`, `latex.rs:1788` |
| `\thanks{…}` | Nota no título | avisa - sem tratamento; o texto da nota entra no nome do autor (inferido pelo fallback de barra) | `latex.rs:1187` (fallback) |
| `\and` (entre autores) | Separa autores | avisa - não achei tratamento | sem tratamento |
| `\part{…}` | Divisão de nível acima de capítulo | avisa - sai como texto comum, sem título de divisão | `latex.rs:1443` (DIVISOES não tem part) |
| `\chapter{…}`, `\section{…}`, `\subsection{…}`, `\subsubsection{…}`, `\paragraph{…}`, `\subparagraph{…}` | Divisões | faz | `latex.rs:1443` (DIVISOES), `latex.rs:1797` (divisao) |
| `\section*{…}` (sem número) | Título sem numeração | faz | `latex.rs:226` (numerar_como_o_fonte) |
| `\section[curto]{longo}` | Título com versão curta para sumário | faz em parte - usa o título longo; a versão curta é descartada | `latex.rs:1797` (divisao, opcional pulado) |
| `\appendix` | Apêndices com letras | ignora em silêncio - sem tratamento; o título seguinte continua numerado | `latex/avisos.rs:182` (comando_conhecido) |
| `\frontmatter`, `\mainmatter`, `\backmatter` | Divisão do livro | ignora em silêncio | `latex/avisos.rs:182` |
| `\tableofcontents` | Sumário | faz | `latex.rs:1639` (bloco) |
| `\listoffigures`, `\listoftables` | Lista de figuras e tabelas | faz | `latex.rs:1570` (bloco, ramos de `\listof…`) |
| `\listof…` (outras listas, ex.: algorithm) | Outras listas | avisa - não achei | sem tratamento |
| `\label{rótulo}` | Marca um ponto para referência | faz | `latex.rs:1497` (rotular), `latex.rs:1392` (numeros_dos_rotulos) |
| `\ref{rótulo}` | Número do objeto marcado | faz em parte - link interno com o número; com `\chapter`, o número de seção sai sem o capítulo (a contagem em `numeros_dos_rotulos` não conta `\chapter`) (inferido); rótulo inexistente cai em `[chave]` | `latex.rs:1177`, `latex.rs:1187`, `latex.rs:1392` |
| `\autoref{rótulo}` | Referência com nome | faz em parte - igual ao `\ref`, sem o nome do tipo ("Figura") | `latex.rs:1177` |
| `\eqref{rótulo}` | Número de equação entre parênteses | faz | `latex.rs:1178` |
| `\pageref{rótulo}` | Número da página | não faz - sai `[rótulo]` sem aviso; `\pageref{LastPage}` só funciona no cabeçalho | `latex.rs:1187`, `latex.rs:429` |
| `\footnote{…}` | Nota de rodapé | faz | `latex.rs:1188` |
| `\endnote{…}` | Nota final | faz | `latex.rs:1188` |
| `\footnotemark`, `\footnotetext` | Nota com marca separada | avisa - não achei | sem tratamento |
| `\nocite{*}` | Lista todas as fontes | faz | `latex.rs:1720` (referencias), `latex.rs:77` (ler_texto, `nocite{*}`) |
| `\nocite{chave}` | Cita sem marcar | ignora em silêncio - a chave específica é descartada; só `*` tem efeito | `latex.rs:1720` |
| `\hrule` | Linha horizontal | faz | `latex.rs:1642` (bloco) |
| `\rule[elev]{larg}{alt}` | Régua de tamanho livre | avisa - não achei | sem tratamento |
| `\newpage` | Quebra de página | faz | `latex.rs:1637` (bloco) |
| `\clearpage`, `\pagebreak` | Quebra de página | ignora em silêncio - nenhuma quebra | `latex.rs:1453` (IGNORADOS) |
| `\cleardoublepage` | Quebra até página ímpar | avisa - sem tratamento | sem tratamento |
| `\vfill` | Espaço que empurra o resto | ignora em silêncio | `latex.rs:1453` (IGNORADOS) |
| `\hyphenation`, `\sloppy`, `\fussy`, `\frenchspacing`, `\microtype` | Ajustes finos do TeX | ignora em silêncio (não afetam o leitor) | `latex/avisos.rs:180`, `latex.rs:1453` |
| `\%`, `\&`, `\$`, `\#`, `\_`, `\{`, `\}` | Caracteres especiais | faz | `latex.rs:1129` (simbolo) |
| `\textbackslash` | Contrabarra | faz | `latex.rs:1129` |
| `\textasciitilde`, `\textasciicircum`, `\textless`, `\textgreater` | Caracteres | avisa - não achei | sem tratamento |
| `\textbar`, `\textbullet`, `\textendash`, `\textemdash` | Caracteres | faz | `latex.rs:1129` |
| `\S`, `\copyright`, `\ldots`, `\dots`, `\textellipsis` | Símbolos | faz | `latex.rs:1129` |
| `\P`, `\dag`, `\ddag`, `\pounds`, `\texteuro`, `\euro`, `\textregistered`, `\texttrademark`, `\textdegree` | Símbolos | avisa - não achei | sem tratamento |
| `\ss`, `\o`, `\O`, `\ae`, `\AE`, `\oe`, `\OE`, `\aa`, `\AA`, `\i`, `\j` | Letras especiais | avisa - saem sem o caractere | sem tratamento |
| `\'`, `` \` ``, `\^`, `\"`, `\~`, `\c` (com ou sem chaves) | Acentos | faz em parte - só para vogais com a tabela de `compor` (`latex.rs:1259`) e ç/Ç e ñ/Ñ | `latex.rs:1241` (acento), `latex.rs:1259` (compor) |
| `\'{\i}` (acento sobre i sem pingo) | Í | não faz - `\i` não existe, e a base vira `\`: sai barra invertida solta (inferido) | `latex.rs:1241`, `latex.rs:1259` |
| `\=`, `\.`, `\u`, `\v`, `\H`, `\k`, `\r`, `\d`, `\b`, `\t` | Outros acentos | avisa - não achei | sem tratamento |
| `\`` `` ` `` e `''`, `--`, `---` (aspas e travessões) | Aspas e travessões | faz | `latex.rs:925` (pontuacao) |
| `\hypersetup{…}` no preâmbulo | Opções de links | ignora em silêncio (no preâmbulo) | `latex/avisos.rs:182` |
| `\hypersetup{…}` no corpo | Opções de links | não faz - as opções vazam como texto (fallback de barra) | `latex.rs:1187` |
| `\input{arquivo}`, `\include{arquivo}` | Inclui outro arquivo | faz em parte - com o projeto (zip ou pasta) inclui; `\include` vira `\input` e não quebra a página (inferido); sem o arquivo, avisa | `latex/projeto.rs:128` (incluir), `latex/projeto.rs:153` |
| `\usepackage[…]{pacote}`, `\RequirePackage` | Carrega pacotes | faz em parte - pacote fora da lista conhecida gera aviso; as opções são lidas só por pacote (geometry, por exemplo) | `latex/avisos.rs:197` (pacote_conhecido), `latex/avisos.rs:221` (pacotes_do_preambulo), `latex/pacotes.rs:7` |
| `\title`, `\author` no preâmbulo | Dados do documento | faz | `latex.rs:77` (ler_texto) |
| `\newcommand{\nome}[n]{corpo}` | Macro com argumentos (`#1…#n`) | faz em parte - ver seção 8 | `latex.rs:622` (extrair_definicoes), `latex.rs:681` (expandir_macros) |
| `\renewcommand{\arraystretch}{x}` | Altura das linhas da tabela | faz | `latex.rs:1741` (ignorado), `latex.rs:1335` (estica_do) |
| `\renewcommand{\bibname}`, `\refname` (vazio) | Tira o título da bibliografia | faz | `latex.rs:1741` |
| `\renewcommand{\figurename}`, `\tablename`, `\contentsname` e outros nomes | Nome dos rótulos | avisa - SemEfeito (só bibname, refname e arraystretch são lidos) | `latex.rs:1450` (SEM_EFEITO), `latex.rs:1741` |

---

## 2. Floats: figure, table, \caption, posições

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\begin{figure}` (e `figure*`) | Flutuante com imagem e legenda | faz em parte - uma imagem por figura: só o primeiro `\includegraphics` vira bloco; os demais são apagados do texto sem aviso | `latex.rs:2053` (ambiente), `latex.rs:2110` (figura), `latex.rs:2137-2151` (remoção de todos os `\includegraphics`) |
| Segunda e demais imagens dentro de uma figura | Imagens extras | ignora em silêncio - são apagadas; o código faz `while … find(cmd)` e remove todas | `latex.rs:2146-2151` |
| `\begin{table}` (e `table*`) com tabular | Tabela flutuante | faz em parte - a tabela sai no fluxo; a posição flutuante é perdida | `latex.rs:2055-2058` |
| `\begin{table}` com imagem dentro | Figura | faz (vira figura) | `latex.rs:2055` |
| Posições `[h]`, `[!htpb]`, `[H]`, `[t]` | Onde o flutuante vai | ignora em silêncio - o bloco fica onde está no texto (comentário no código) | `latex.rs:2016` (ambiente, pula `[…]`) |
| `\caption{…}` dentro de figura ou tabela | Legenda numerada ("Figura 1 – …") | faz | `latex.rs:2123` (legenda na figura), `latex.rs:1833` (legenda_solta), `latex.rs:1855` (push_legenda) |
| `\caption[curta]{longa}` | Legenda com versão curta para lista | faz em parte - usa a longa; a curta é descartada | `latex.rs:1833` (legenda_solta, opcional pulado) |
| `\captionof{figure}{…}`, `\captionof{table}{…}` | Legenda fora de flutuante | faz | `latex.rs:1833` |
| `\caption` antes da imagem (ordem da ABNT) | Legenda acima da figura | faz | `latex.rs:2110` (figura, `em_cima`) |
| `\label` dentro de figura ou tabela | Rótulo do flutuante | faz | `latex.rs:2137` (figura) |
| `\centering` dentro de figura | Figura centralizada | faz | `latex.rs:2110` (figura, `resto.contains("\\centering")`) |
| `\listoffigures`, `\listoftables` | Listas de flutuantes | faz | `latex.rs:1570` (bloco) |
| `\FloatBarrier`, `\floatplacement`, `\restylefloat` (placeins, float) | Controle de flutuantes | ignora em silêncio (FloatBarrier, `latex/avisos.rs:180`) ; `\restylefloat` avisa - não achei | `latex/avisos.rs:180` |
| `\begin{wrapfigure}` | Figura com texto em volta | avisa - vira bloco comum, sem o texto em volta | `latex/pacotes.rs:22` (ambiente_extra) |

---

## 3. graphicx

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\includegraphics{arquivo}` | Insere imagem | faz - desde que o arquivo esteja no projeto; sem o arquivo, avisa | `latex.rs:1656` (ramo), `latex.rs:1819` (imagem_solta), `latex/projeto.rs:181` (embutir_imagens) |
| `\includegraphics{arquivo}` sem extensão | Acha png, jpg, jpeg, svg, gif, webp | faz | `latex/projeto.rs:181` |
| `[width=3cm]`, `[width=8pt]`, `[width=12mm]` | Largura absoluta | faz | `latex.rs:1342` (largura_da_imagem), `latex.rs:245` (dimensao) |
| `[width=0.5\linewidth]`, `\textwidth`, `\columnwidth`, `\hsize` | Largura relativa | faz | `latex.rs:1342` |
| `[width=0.8\textwidth]` | Largura padrão do FeatherDocs | faz | `latex.rs:1342` (trata como natural) |
| `[height=…]` | Altura | ignora em silêncio - a altura sai pela proporção da imagem | `latex.rs:1342`; `latex/projeto.rs:181` (comentário) |
| `[scale=…]` | Escala | ignora em silêncio | `latex.rs:1342` (só `width` é lido) |
| `[angle=…]` | Rotação | ignora em silêncio | `latex.rs:1342` |
| `[trim=…]`, `[clip]`, `[keepaspectratio]`, `[page=…]` | Recorte e opções | ignora em silêncio | `latex.rs:1342` |
| PNG e JPEG (tamanho natural pelo cabeçalho) | Tamanho natural | faz | `latex/projeto.rs:212` (pixels), `latex/projeto.rs:262` (tipo) |
| GIF e WEBP | Imagem | faz em parte - entram como data URL; o tamanho natural não é lido (pixels só aceita PNG e JPEG) (inferido) | `latex/projeto.rs:212` |
| SVG | Vetor | faz em parte - entra como data URL; sem tamanho natural (inferido) | `latex/projeto.rs:181` |
| PDF e EPS | Vetor | não faz - o arquivo é embutido, mas não achei renderização nem tamanho natural | `latex/projeto.rs:181`, `latex/projeto.rs:262` |
| `\graphicspath{…}` | Pastas de imagens | ignora em silêncio - a busca é só na raiz do projeto | `latex/avisos.rs:182` |
| `\includegraphics` no meio de parágrafo (não no início do bloco) | Imagem em linha | avisa - o nome do arquivo vaza como texto (inferido) | `latex.rs:1187` (fallback) |
| `\rotatebox`, `\scalebox`, `\resizebox` | Girar ou escalar | avisa - os argumentos saem como texto | `latex.rs:1187` (fallback) |

---

## 4. subcaption e subfig (subfigure)

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\begin{subfigure}{larg}` (subcaption) | Subfigura lado a lado | avisa - ambiente desconhecido: o conteúdo sai em sequência, sem grade e sem largura | `latex/pacotes.rs:22` (ambiente_extra) |
| `\subcaption{…}` | Legenda da subfigura | avisa - o texto da legenda sai como texto comum | `latex.rs:1187` (fallback) |
| `\subref{…}` | Referência à subfigura | avisa - sai `[chave]` ou o texto do argumento (inferido) | `latex.rs:1187` |
| `\subfloat[legenda]{…}` (subfig) | Subfigura | não faz - a legenda entre colchetes sai como texto e as imagens seguintes são apagadas (inferido) | `latex.rs:2146-2151` |
| `\begin{subfigure}` dentro de `figure` | Grade de imagens | não faz - só a primeira imagem da figura é mantida (ver seção 2) | `latex.rs:2110`, `latex.rs:2146` |

---

## 5. caption

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\captionsetup{font=bf}` | Legenda em negrito | faz | `latex.rs:77` (ler_texto, `legenda_negrito`) |
| `\captionsetup{…}` com outras opções (labelsep, justification, position, labelfont, font=it) | Ajustes de legenda | avisa - só `font=bf` é lido; o resto vai para SemEfeito | `latex.rs:77` (ler_texto, aviso SemEfeito), `latex.rs:1450` |
| `\captionof` | Legenda solta | faz (ver seção 2) | `latex.rs:1833` |
| `\caption*{…}` (sem número) | Legenda sem número | avisa - não achei tratamento | sem tratamento |
| `\captionsetup` no corpo | Ajustes de legenda | ignora em silêncio no caso `font=bf`; avisa nos outros (SemEfeito) | `latex.rs:1450` (SEM_EFEITO) |
| `\listoffigures` com legenda curta | Lista de figuras | faz em parte - usa a legenda longa | `latex.rs:1833` |

---

## 6. tabular, tabularx e longtable

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\begin{tabular}{colunas}` | Tabela simples | faz em parte - linhas e células sim; alinhamento por coluna (l, c, r) não aplicado; a especificação de colunas só serve para detectar réguas verticais e a última coluna `r` em linha única | `latex.rs:2164` (tabular), `latex.rs:2178` (tem_regua) |
| `\begin{tabular}[t]{…}` | Alinhamento vertical | ignora em silêncio - o `[t]` é pulado | `latex.rs:2164` |
| `&` (separa células) e `\\` (separa linhas) | Células e linhas | faz | `latex.rs:2247` (linhas_de_tabela), `latex.rs:2275` (dividir_celulas) |
| `\hline` | Régua horizontal | faz em parte - some; serve só para decidir se a primeira linha é cabeçalho; o modelo não guarda régua por linha | `latex.rs:2249` (linhas_de_tabela), `latex.rs:2180` (cabecalho) |
| `\cline{i-j}` | Régua em parte das colunas | avisa - não é removido da célula; o argumento `2-3` sai como texto (inferido) | `latex.rs:2249` (lista de réguas sem `\cline`), `latex.rs:2179` |
| `\vline` e barra vertical nas colunas | Régua vertical | ignora em silêncio - só usado para decidir réguas; o modelo não tem réguas verticais | `latex.rs:2178` |
| `@{…}`, `>{…}`, `<{…}`, `p{larg}`, `m{larg}`, `b{larg}`, `X` (tabularx) | Tipos e larguras de coluna | ignora em silêncio - a especificação não é lida, só o `r` final e as réguas | `latex.rs:2164` |
| `\begin{tabularx}{larg}{…}` | Tabela de largura fixa | faz em parte - a largura é pulada; as colunas `X` não são distribuídas | `latex.rs:2032` (ambiente, pula `{larg}`), `latex.rs:2052` |
| `\multicolumn{n}{align}{texto}` | Célula que ocupa n colunas | não faz - a saída é `2cx` (n, alinhamento e texto viram três pedaços de texto); o aviso é do fallback | `latex.rs:1187` (fallback de barra), `latex/tabelas.rs:1` (módulo só com documentação) |
| `\multirow{n}{larg}{texto}` | Célula que ocupa n linhas | avisa - a saída é `3*texto` ou similar, com o número e o asterisco no texto | `latex.rs:1187` (fallback) |
| `\cmidrule{i-j}`, `\addlinespace`, `\specialrule` (booktabs) | Réguas parciais e espaçamento | avisa - sem tratamento; o argumento sai como texto | `latex.rs:1187` (fallback) |
| `\toprule`, `\midrule`, `\bottomrule` (booktabs) | Réguas do booktabs | faz em parte - some; `\midrule` conta como cabeçalho; sem réguas no modelo | `latex.rs:2249` (linhas_de_tabela), `latex.rs:2178` |
| `\\[2pt]` (com medida) | Espaço entre linhas | não faz - a medida vaza para o início da célula seguinte como texto (inferido: a divisão é só por `\\`) | `latex.rs:2253` |
| Tabela com uma só linha e sem réguas | Arrumação de texto | faz em parte - vira parágrafo (a última coluna `r` vai para a direita com tabulação) | `latex.rs:2183` |
| `\renewcommand{\arraystretch}{x}` | Altura das linhas | faz | `latex.rs:1741` (ignorado), `latex.rs:1335` (estica_do) |
| `\setlength{\tabcolsep}{…}` | Espaço entre colunas | ignora em silêncio | `latex.rs:1024` |
| `\begin{tabular*}{larg}{…}` | Tabela com largura exata | avisa - ambiente desconhecido; as células saem como texto corrido (inferido) | `latex/pacotes.rs:22` |
| `\begin{longtable}` | Tabela que quebra de página | faz em parte - as linhas entram; o cabeçalho repetido aparece duplicado (inferido) | `latex.rs:2052` |
| `\endfirsthead`, `\endhead`, `\endfoot`, `\endlastfoot` (longtable) | Marcas de cabeçalho e rodapé | avisa - sem tratamento; o cabeçalho que se repete fica duplicado no texto (inferido) | `latex.rs:1187` (fallback) |
| `\caption` dentro de longtable | Legenda de tabela longa | avisa - não é legenda numerada; o texto entra como linha ou célula (inferido) | `latex.rs:1833` (só no nível de bloco) |
| `\caption` dentro de `table` (com tabular) | Legenda numerada | faz | `latex.rs:2056` (table executa os filhos), `latex.rs:1833` |
| `\begin{array}` fora de fórmula | Matriz no texto | avisa - ambiente desconhecido; conteúdo como texto | `latex/pacotes.rs:22` |
| `\tabularnewline` | Quebra de linha | avisa - sem tratamento | `latex.rs:1187` |

---

## 7. booktabs, multirow, multicolumn, colortbl e array

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| booktabs: `\toprule`, `\midrule`, `\bottomrule` | Réguas de tabela | faz em parte - ver seção 6 | `latex.rs:2249` |
| booktabs: `\cmidrule`, `\addlinespace`, `\specialrule`, `\heavyrulewidth` | Réguas e espaços | avisa | `latex.rs:1187` |
| multirow: `\multirow` | Célula em várias linhas | avisa - ver seção 6 | `latex/tabelas.rs:1` (só documentação) |
| `\multicolumn` | Célula em várias colunas | não faz - ver seção 6 | `latex/tabelas.rs:1` (só documentação) |
| `\cline` | Régua parcial | avisa - ver seção 6 | `latex.rs:2179` |
| colortbl: `\rowcolors{i}{cor1}{cor2}` | Faixas alternadas | avisa - os argumentos saem como texto na célula | `latex.rs:1187` (fallback) |
| colortbl: `\cellcolor{cor}` | Cor da célula | avisa - o nome da cor sai como texto | `latex.rs:1187` (fallback) |
| colortbl: `\rowcolor{cor}`, `\columncolor{cor}` | Cor da linha ou coluna | avisa - o nome da cor sai como texto | `latex.rs:1187` (fallback) |
| `\arrayrulecolor{cor}` | Cor das réguas | avisa (SemEfeito) | `latex.rs:1450` (SEM_EFEITO) |
| `array` no texto (fora de equação) | Matriz | avisa - ambiente desconhecido | `latex/pacotes.rs:22` |
| `array` dentro de equação | Matriz em fórmula | faz (repassa ao feather-math; matemática não auditada) | `latex.rs:2048` |

---

## 8. xcolor e cores

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\definecolor{nome}{HTML}{RRGGBB}` | Cor por hexadecimal | faz | `latex.rs:622` (extrair_definicoes), `latex.rs:567` (parsear_cor) |
| `\definecolor{nome}{RGB}{r,g,b}` com 0 a 255 | Cor RGB inteira | faz | `latex.rs:567` |
| `\definecolor{nome}{rgb}{r,g,b}` com 0 a 1 (padrão do xcolor) | Cor RGB fracionária | ignora em silêncio - `u8` não lê `0.5`; a cor não é registrada e nenhum aviso aparece | `latex.rs:567` (parsear_cor, parse::<u8>) |
| `\definecolor{nome}{gray}{x}` (0 a 1) | Tom de cinza | faz | `latex.rs:567` (GRAY) |
| `\definecolor{nome}{cmyk}{c,m,y,k}` | Cor CMYK | ignora em silêncio - modelo não suportado | `latex.rs:567` |
| `\definecolor` com Hsb, tHsb, wave | Outros modelos | ignora em silêncio | `latex.rs:567` |
| Nomes padrão: white, black, red, green, blue, gray, lightgray, darkblue, midblue, lightblue, darktext, graytext | Cores prontas | faz em parte - só estes 12 nomes têm valor fixo | `latex.rs:592` (resolver_cor) |
| Nomes padrão: cyan, magenta, yellow, orange, purple, brown, olive, teal, violet, pink, lime, darkgray | Cores prontas do xcolor | ignora em silêncio - não reconhecidas; a cor não é aplicada | `latex.rs:592` |
| Mistura `red!80`, `blue!50!white`, `red!20!black` | Cor misturada | ignora em silêncio - `resolver_cor` não entende `!`; a cor não é aplicada | `latex.rs:592` |
| `\color{nome}` | Cor até o fim do grupo | faz em parte - só com os nomes lidos acima | `latex.rs:1061` (barra, ramo color) |
| `\textcolor{nome}{texto}`, `\textcolor[rgb]{…}{texto}` | Texto colorido | faz em parte - só com os nomes lidos acima | `latex.rs:1069`, `latex.rs:1111` (textcolor) |
| `\colorbox{cor}{texto}` no início de bloco | Caixa colorida de fundo | faz em parte - vira fundo do parágrafo; o padrão `[model]` é lido; funciona só no início de bloco | `latex.rs:1581` (colorbox de bloco) |
| `\colorbox{cor}{texto}` no meio de parágrafo | Caixa colorida em linha | avisa - sai o nome da cor como texto, depois o conteúdo | `latex.rs:1187` (fallback) |
| `\fcolorbox{borda}{fundo}{texto}` | Caixa com borda colorida | avisa - sai o nome da cor da borda como texto (inferido) | `latex.rs:1187` (fallback) |
| `\pagecolor{cor}` | Cor do papel | faz | `latex.rs:366` (extras_da_pagina) |
| `\sethlcolor{cor}` | Cor do marca-texto | faz | `latex/linha.rs:25` |
| `\xcolorbox`, `\colorlet` | Outros | avisa - não achei | sem tratamento |

---

## 9. parbox, minipage e caixas

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\parbox[pos]{larg}{texto}` | Caixa de largura fixa | avisa - sem tratamento; a largura sai como texto (inferido) | `latex.rs:1187` (fallback) |
| `\begin{minipage}[pos]{larg}` | Bloco de largura fixa | faz em parte - o conteúdo sai empilhado em blocos; largura e lado a lado não são aplicados (duas minipages ficam uma sobre a outra) | `latex.rs:2071` (minipage), `latex.rs:2016` (pula `[pos]{larg}`) |
| `\makebox{texto}` (sem opções) | Texto em caixa | faz em parte - só o texto; sem caixa e sem alinhamento | `latex/linha.rs:19` |
| `\makebox[larg][pos]{texto}` | Caixa com largura | avisa - as opções saem como texto `[larg][pos]` (inferido) | `latex/linha.rs:19`, `latex.rs:1187` |
| `\framebox{texto}`, `\fbox{texto}` | Texto com moldura | avisa - sai só o texto, sem moldura | `latex.rs:1187` |
| `\fboxsep`, `\fboxrule` | Medidas da moldura | ignora em silêncio (no preâmbulo) ou avisa (no corpo) | `latex/avisos.rs` (não achei lista própria) |
| `\hfill` entre minipages | Espaço para colunas | faz em parte - vira tabulação no parágrafo, sem colunas | `latex.rs:1143` |

---

## 10. amsmath

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `$…$` | Fórmula em linha | faz (repassa ao feather-math) | `latex.rs:937` (matematica) |
| `$$…$$` e `\[…\]` | Fórmula em destaque | faz (repassa ao feather-math) | `latex.rs:1811` (equacao_de), `latex.rs:1622` |
| `\(…\)` | Fórmula em linha | faz (repassa) | `latex.rs:959` (matematica_parenteses) |
| `equation`, `equation*` | Fórmula numerada | faz (repassa; numeração no feather-math) | `latex.rs:2048` |
| `align`, `align*`, `gather`, `multline`, `flalign`, `alignat`, `eqnarray`, `split` | Fórmulas alinhadas | faz (repassa) | `latex.rs:2048`, `latex.rs:2047` (split vira aligned) |
| `displaymath`, `math` | Fórmula | faz (repassa) | `latex.rs:2046` |
| `dmath`, `dgroup`, `dseries`, `empheq` | Pacotes de fórmula | faz (repassa) | `latex.rs:2048` |
| `cases`, `pmatrix`, `bmatrix`, `matrix`, `array` dentro de equação | Matrizes e casos | faz (repassa ao feather-math; não auditado) | `latex.rs:2048` |
| `\text{…}`, `\mbox{…}` dentro de fórmula | Texto na fórmula | faz (repassa) | feather-math |
| `\tag{…}`, `\nonumber` | Rótulo ou sem número | faz (repassa) | feather-math |
| `\intertext{…}` | Texto entre linhas de align | avisa - não achei tratamento | `latex.rs:1187` |
| `\eqref{…}` | Referência de equação | faz | `latex.rs:1178` |
| `\begin{subequations}` | Numeração em subgrupo | avisa - ambiente desconhecido; o conteúdo é lido, sem o subgrupo | `latex/pacotes.rs:22` |
| `\numberwithin{equation}{section}` | Numeração por seção | avisa - não achei | sem tratamento |

---

## 11. amsthm

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\newtheorem{teorema}{Teorema}` no preâmbulo | Define o ambiente | ignora em silêncio - o preâmbulo não é lido para isso | `latex/avisos.rs:186` (só vale no corpo) |
| `\begin{teorema}…\end{teorema}` | Teorema numerado, em negrito | avisa - ambiente desconhecido: o conteúdo sai como texto corrido, sem título nem número | `latex/pacotes.rs:22` |
| `\begin{proof}…\end{proof}` | Prova com QED | avisa - texto corrido | `latex/pacotes.rs:22` |
| `\qed`, `\qedhere` | Quadradinho de fim de prova | avisa - não achei | sem tratamento |
| `\theoremstyle{…}` | Estilo do teorema | ignora em silêncio | `latex/avisos.rs:186` |
| Pacote amsthm | Reconhecido | faz em parte - está na lista de pacotes conhecidos, mas nenhum comando é tratado | `latex/avisos.rs:197-210` |

---

## 12. siunitx

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\SI{valor}{unidade}` | Número com unidade e espaço | avisa - sai só o valor; a unidade some (o comando não é tratado) | `latex.rs:1187` (fallback), `latex.rs:1157` |
| `\si{unidade}` | Unidade | avisa - sai vazio ou os nomes das unidades | `latex.rs:1187` |
| `\num{1234.5}` | Número com separador e vírgula decimal | avisa - sai `1234.5` cru, com ponto decimal | `latex.rs:1187` |
| `\SIrange`, `\numrange`, `\ang`, `\qty`, `\unit` | Intervalos e unidades | avisa | `latex.rs:1187` |
| `\sisetup{…}` | Opções do pacote | ignora em silêncio (no preâmbulo) | `latex/avisos.rs:183` |
| Prefixos e nomes de unidades (kilo, metre, second…) | Vocabulário do siunitx | não faz - não achei tabela | sem tratamento |

---

## 13. hyperref

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\href{url}{texto}` | Link | faz | `latex.rs:1173` (ramo href), `latex.rs:1224` (com_link) |
| `\url{endereço}` | Link com o próprio endereço | faz | `latex.rs:1171` |
| `\hyperref[rótulo]{texto}` | Link interno | não faz - não achei; a chave e o texto saem como texto | `latex.rs:1187` (fallback) |
| `\hyperlink{alvo}{texto}`, `\hypertarget{alvo}{texto}` | Link interno e alvo | avisa - sai o alvo e o texto (inferido) | `latex.rs:1187` (fallback) |
| `\autoref{rótulo}` | Referência | faz (ver seção 1) | `latex.rs:1177` |
| `\nameref{rótulo}` | Nome da seção | avisa - não achei | sem tratamento |
| `\texorpdfstring{a}{b}` | Texto diferente no PDF | avisa - sai `a` e `b` juntos (inferido) | `latex.rs:1187` |
| `\hypersetup{…}` no preâmbulo | Opções | ignora em silêncio | `latex/avisos.rs:182` |
| `\phantomsection`, `\pdfbookmark` | Marcadores | ignora em silêncio | `latex/avisos.rs:182`, `latex/avisos.rs:187` |
| Links internos (`\label` e `\ref`) | Ligação ao bloco | faz | `latex.rs:77` (ler_texto, `#rotulo:` vira `#bloco:`) |

---

## 14. cleveref

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\cref{rótulo}` | "figura 1" com o tipo | avisa - sai a chave do rótulo como texto (ex.: `fig:a`) | `latex.rs:1187` (fallback) |
| `\Cref{rótulo}` | "Figura 1" com maiúscula | avisa - mesma saída | `latex.rs:1187` |
| `\crefrange{a}{b}`, `\labelcref`, `\cpageref` | Intervalos e variantes | avisa - sem tratamento | `latex.rs:1187` |
| `\crefname{tipo}{…}`, `\Crefname` | Nomes dos tipos | ignora em silêncio (no preâmbulo) | `latex/avisos.rs` (não lido) |
| Pacote cleveref | Reconhecido | faz em parte - pacote aceito sem aviso, mas `\cref` não funciona | `latex/avisos.rs:197-210` |

---

## 15. csquotes

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\enquote{texto}` | Aspas de citação | avisa - sai só o texto, sem aspas | `latex.rs:1187` (fallback) |
| `\textquote{…}`, `\foreignquote{…}` | Aspas de citação | avisa | `latex.rs:1187` |
| `\blockquote{…}` | Citação em bloco | avisa | `latex.rs:1187` |
| `\MakeOuterQuote`, `\MakeAutoQuote` | Configuração de aspas | ignora em silêncio (no preâmbulo) | `latex/avisos.rs` (não lido) |
| Pacote csquotes | Reconhecido | faz em parte - aceito sem aviso, mas `\enquote` não tem aspas | `latex/avisos.rs:197-210` |

---

## 16. babel

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\usepackage[brazil]{babel}` | Idioma | faz em parte - aceito sem aviso; os rótulos ("Figura", "Tabela") são fixos e a hifenização não é aplicada | `latex/avisos.rs:200` |
| `\selectlanguage{idioma}` | Troca de idioma | avisa | `latex.rs:1187` |
| `\foreignlanguage{idioma}{texto}` | Trecho em outro idioma | avisa - sai o nome do idioma e o texto | `latex.rs:1187` |
| `\hyphenation{…}` | Hifenização | ignora em silêncio | `latex/avisos.rs:180` |
| `\babelprovide`, `\languagename` | Outros | avisa - não achei | sem tratamento |

---

## 17. biblatex e natbib

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\cite{chaves}`, `\citep`, `\citet`, `\parencite`, `\textcite`, `\autocite`, `\footcite`, `\citeonline`, `\apud` | Citação | faz | `latex.rs:1074` (citacao), `latex.rs:1096` |
| `\cite[p.~4]{chave}` (página) | Citação com página | faz | `latex.rs:1096` (citacao, a última opção é a página) |
| `\citeauthor{chave}`, `\citeyear{chave}` | Autor ou ano | faz em parte - só se a chave existe na lista de fontes; senão sai `[chave]` | `latex.rs:1180` (ramo) |
| `\citealp`, `\citealt`, `\citeyearpar`, `\citetitle`, `\citeurl`, `\fullcite`, `\cites`, `\citeauthoronline` | Variantes | avisa - sai a chave ou o texto do argumento | `latex.rs:1187` (fallback) |
| `\nocite{*}` | Lista todas as fontes | faz | `latex.rs:1720`, `latex.rs:77` |
| `\nocite{chave}` | Lista uma fonte | ignora em silêncio (só `*` tem efeito) | `latex.rs:1720` |
| `\printbibliography` | Lista de referências | faz | `latex.rs:1720` (referencias) |
| `\printbibliography[título=…]` | Lista com título próprio | faz em parte - o título e as opções são descartados em silêncio | `latex.rs:1720` (opções puladas) |
| `\bibliography{arquivo}` | Lista (BibTeX) | faz em parte - aceita o arquivo do projeto; a lista sai com o título padrão | `latex.rs:1720`, `latex/projeto.rs:153` |
| `\bibliographystyle{estilo}` | Estilo | ignora em silêncio - o estilo sai do preâmbulo, não do comando | `latex.rs:1461` (IGNORADOS), `latex.rs:196` (ler_fontes) |
| `\addbibresource{arquivo.bib}` | Fonte (biblatex) | faz - lido do projeto (zip ou pasta) | `latex/projeto.rs:153` |
| Estilos ABNT, APA, IEEE, Vancouver, Chicago (por `style=` ou pelo nome do pacote) | Formato da lista e da citação | faz em parte - a escolha é lida do preâmbulo; a formatação em si fica em outro módulo (não auditado) | `latex.rs:196` (ler_fontes) |
| Arquivo `.bib` (entradas) | Dados | faz (lido por `crate::bib`) | `latex.rs:196` |
| `\printindex`, `\index{termo}`, `\index{termo!sub}` | Índice remissivo | faz | `latex.rs:1157` (com_argumento, ramo index), `latex.rs:1637` |
| `\makeindex` | Liga o índice | ignora em silêncio | `latex/avisos.rs:186` |
| `\printglossaries`, `\newglossaryentry`, `\gls` | Glossário | avisa - não achei | sem tratamento |

---

## 18. listings e minted

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\begin{lstlisting}` | Código literal | faz | `latex.rs:2043`, `latex.rs:2093` (codigo) |
| `\begin{lstlisting}[language=Python]` | Código com linguagem | faz | `latex.rs:2101` |
| `\begin{lstlisting}[caption=…, label=…]` | Legenda e rótulo do código | ignora em silêncio - a legenda e o rótulo são descartados; só `language` é lido | `latex.rs:2101` |
| `\begin{lstlisting}[numbers=left, frame=…, basicstyle=…, breaklines]` | Aparência | ignora em silêncio | `latex.rs:2101` |
| `\lstinputlisting{arquivo}` | Código de arquivo externo | avisa - o nome do arquivo sai como texto | `latex.rs:1187` (fallback) |
| `\lstinline{texto}` (com chaves) | Código em linha | faz | `latex.rs:1169` |
| `\lstinline` com delimitador de barra | Código em linha | avisa - sai com as barras | `latex.rs:1169` |
| `\lstset{…}` | Padrões de código | ignora em silêncio | `latex/avisos.rs:183` |
| `\lstdefinestyle{…}{…}` | Estilo de código | ignora em silêncio no preâmbulo; avisa no corpo | `latex/avisos.rs:182` |
| `\begin{minted}{python}` | Código com linguagem | faz | `latex.rs:2096` |
| `\begin{minted}[linenos]{python}` (com opções) | Código com opções | não faz - as opções `[…]` entram no código como texto (inferido: `ambiente` não pula `[…]` para minted) | `latex.rs:2016` (ambiente), `latex.rs:2096` (codigo) |
| `\inputminted{python}{arquivo}` | Código de arquivo | avisa | `latex.rs:1187` |
| `\begin{verbatim}`, `Verbatim` (fancyvrb) | Texto literal | faz | `latex.rs:2043` |
| `\verb` (ver seção 1) | Texto literal em linha | avisa com delimitador | `latex.rs:1169` |

---

## 19. fancyhdr

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\pagestyle{fancy}` | Liga o cabeçalho e rodapé | faz | `latex.rs:408` (cabecalho_e_rodape) |
| `\pagestyle{empty}` | Sem cabeçalho nem rodapé | faz | `latex.rs:301`, `latex.rs:408` |
| `\pagestyle{plain}`, `headings` | Outros estilos | ignora em silêncio | `latex.rs:1453` (IGNORADOS) |
| `\fancyhf{}` | Limpa o cabeçalho e rodapé | faz | `latex.rs:408` |
| `\fancyhead[L/C/R]{…}`, `\fancyfoot[…]{…}` | Texto de cabeçalho e rodapé | faz em parte - lados L, C e R; `[LE,RO]` (página par e ímpar) usa só o lado indicado (o documento não tem página par diferente) | `latex.rs:408` (cabecalho_e_rodape) |
| `\lhead`, `\chead`, `\rhead`, `\lfoot`, `\cfoot`, `\rfoot` | Atalhos | faz | `latex.rs:408` |
| `\thepage` | Número da página | faz | `latex.rs:429` (`{pagina}`) |
| `\pageref{LastPage}` (lastpage) no cabeçalho ou rodapé | Total de páginas | faz | `latex.rs:429` (`{total}`) |
| `\headrulewidth`, `\footrulewidth` com `\renewcommand` | Régua do cabeçalho | avisa - SemEfeito; a régua não é controlada | `latex.rs:1450` (SEM_EFEITO), `latex.rs:1741` |
| `\fancypagestyle{nome}{…}` | Estilo de página nomeado | avisa - não achei | sem tratamento |
| `\leftmark`, `\rightmark` | Marcas de seção no cabeçalho | avisa - não achei; saem vazios | `latex.rs:1187` |
| `\fancyheadoffset`, `\fancyhfoffset` | Ajuste de medida | avisa | `latex.rs:1187` |

---

## 20. titlesec

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\titleformat{\section}{…\MakeUppercase…}` | Título em maiúsculas | faz | `latex.rs:1358` (titulos_em_maiusculas) |
| `\titleformat{\section}{\bfseries\large\centering}` (negrito, tamanho, alinhamento) | Formato do título | ignora em silêncio - só `\MakeUppercase` é lido; o resto do preâmbulo some sem aviso | `latex.rs:1358` |
| `\titleformat` nos níveis chapter, section, subsection, subsubsection | Por nível | faz em parte - só capítulo a subseção no `\MakeUppercase`; paragraph e subparagraph não são lidos | `latex.rs:1358` |
| `\titlespacing{…}` | Espaços do título | ignora em silêncio (no preâmbulo) | `latex/avisos.rs:184` |
| `\titleclass`, `\titlecontents` | Outros | ignora em silêncio (no preâmbulo) | sem tratamento |
| `\titleformat` com `[display]` | Forma do título | ignora em silêncio | `latex.rs:1358` |

---

## 21. geometry

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\usepackage[a4paper]{geometry}` | Papel A4 | faz | `latex.rs:301` (configurar_pagina) |
| `letterpaper` | Papel carta | faz | `latex.rs:301` |
| `a5paper`, `b5paper`, `legalpaper`, `paperwidth=`, `paperheight=` | Outros papéis | faz em parte - só paperwidth e paperheight são lidos; a5paper, b5paper e legalpaper somem sem aviso | `latex.rs:301` |
| `landscape` | Paisagem | faz | `latex.rs:301` |
| `top`, `bottom`, `left`, `right`, `inner`, `outer` | Margens | faz em parte - `inner` e `outer` viram esquerda e direita, sem página par e ímpar | `latex.rs:301` |
| `margin=`, `hmargin=`, `vmargin=` | Margens | faz | `latex.rs:301` |
| `textwidth=`, `textheight=`, `headheight=`, `headsep=`, `footskip=`, `includehead`, `twoside` | Área do texto e cabeçalho | ignora em silêncio | `latex.rs:301` |
| `\geometry{…}` no corpo ou no preâmbulo | Mesmas opções | faz | `latex.rs:301` |
| Margens da abntex2 (3 cm em cima e à esquerda; 2 cm embaixo e à direita) | Margens ABNT | faz | `latex.rs:301` |

---

## 22. lastpage

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\pageref{LastPage}` no cabeçalho ou rodapé | Número total de páginas | faz | `latex.rs:429` |
| `\pageref{LastPage}` no corpo | Número total | não faz - sai `[LastPage]` | `latex.rs:1187` |
| Pacote lastpage | Reconhecido | faz em parte - aceito sem aviso; só funciona em cabeçalho ou rodapé | `latex/avisos.rs:203` |

---

## 23. tcolorbox

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\usepackage{tcolorbox}` | Carrega o pacote | avisa - pacote não está na lista de conhecidos | `latex/avisos.rs:197` |
| `\newtcolorbox{nome}[…]{opções}` | Define caixa | ignora em silêncio (no preâmbulo, não é lido) | `latex/avisos.rs:186` (só vale no corpo, e não é tratado) |
| `\begin{tcolorbox}[opções]…\end{tcolorbox}` | Caixa com título e cores | avisa - o conteúdo sai como texto corrido, sem caixa | `latex/pacotes.rs:22` |
| `\tcbset`, `\tcbuselibrary` | Ajustes | ignora em silêncio (no preâmbulo) | sem tratamento |

---

## 24. TikZ

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\usepackage{tikz}` | Carrega o pacote | avisa - não está na lista de conhecidos | `latex/avisos.rs:197` |
| `\usetikzlibrary{…}`, `\tikzset{…}` | Bibliotecas e estilos | ignora em silêncio (no preâmbulo) | sem tratamento |
| `\begin{tikzpicture}…\end{tikzpicture}` | Desenho | avisa - o conteúdo sai como texto corrido (comandos, coordenadas e rótulos viram texto) | `latex/pacotes.rs:22`, `latex/avisos.rs:253` (teste de aviso de tikz) |
| `\node`, `\draw`, `\fill`, `\path`, `\coordinate`, `\foreach`, `\begin{scope}` | Elementos do desenho | avisa - sem tratamento; o texto dos argumentos sai como texto | `latex.rs:1187` (fallback) |
| Desenho TikZ como imagem | Renderizar o desenho | não faz - não há renderizador de TikZ no leitor | sem tratamento |

---

## 25. enumitem

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\usepackage[shortlabels]{enumitem}` | Carrega o pacote | faz em parte - pacote aceito sem aviso; as opções do pacote não são lidas | `latex/avisos.rs:206` |
| `\setlist[itemize]{itemsep=…}` (preâmbulo) | Espaço entre itens | faz | `latex.rs:465` (listas_do_preambulo) |
| `\setlist[itemize]{label=\textcolor{cor}{…}}` | Cor do marcador | faz em parte - só a cor; o formato do marcador não é lido | `latex.rs:465` |
| `\setlist[enumerate]{label=\alph*)}` | Marcador em letras | ignora em silêncio - `listas_do_preambulo` lê só o primeiro `\setlist` do preâmbulo, e só itemsep e cor | `latex.rs:465-470` |
| `\setlist` no corpo | Ajuste de lista | avisa (SemEfeito) | `latex.rs:1450` (SEM_EFEITO) |
| `\begin{enumerate}[label=\alph*)]` | Marcador em letras | ignora em silêncio - o argumento antes do primeiro `\item` é descartado (a lista sai com 1., 2., 3.) | `latex.rs:2042` (lista), `latex.rs:2196` (resultado de inline_em descartado) |
| `\begin{enumerate}[label=\Roman*.]`, `\arabic*`, `[resume]`, `[start=5]` | Outros marcadores e recomeços | ignora em silêncio | `latex.rs:2196` |
| `\begin{itemize}[label=--]` | Marcador próprio | ignora em silêncio - o marcador é o padrão | `latex.rs:2196` |
| `\begin{itemize}[itemsep=…]` local | Espaço entre itens | ignora em silêncio | `latex.rs:2196` |
| `\item[rótulo]` em `itemize` ou `enumerate` | Rótulo próprio | ignora em silêncio - o rótulo entre colchetes é pulado e descartado | `latex.rs:2229` (itens_de) |
| `\begin{description}` com `\item[Termo]` | Lista de termos | faz em parte - vira lista com marcador; o termo é descartado em silêncio | `latex.rs:2042`, `latex.rs:2229` |
| Listas aninhadas (itemize dentro de enumerate, etc.) | Níveis | faz - até o nível 5 | `latex.rs:2191` (lista) |
| `\begin{compactitem}`, `\begin{inparaenum}` | Listas compactas | avisa - ambiente desconhecido | `latex/pacotes.rs:22` |

---

## 26. \newcommand, \renewcommand e outras definições

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\newcommand{\nome}{corpo}` | Macro sem argumentos | faz | `latex.rs:622` (extrair_definicoes), `latex.rs:681` (expandir_macros) |
| `\newcommand{\nome}[n]{corpo}` com `#1…#n` | Macro com argumentos | faz - expansão em até 10 passadas | `latex.rs:681` |
| `\newcommand{\nome}[n][padrão]{corpo}` | Argumento opcional com valor padrão | ignora em silêncio - a macro não é registrada (o `[padrão]` não é lido) | `latex.rs:622` |
| `\renewcommand{\nome}{corpo}` | Troca a macro | faz | `latex.rs:622` |
| `\providecommand{\nome}{…}` | Define se não existe | avisa - a macro não é registrada; o uso de `\nome` avisa | `latex.rs:622` |
| `\def\nome{corpo}`, `\let` | Definição do TeX puro | não faz - no corpo, o corpo vaza como texto sem aviso; no preâmbulo, some | `latex/avisos.rs:186` (comando_conhecido: `def`, `let`) |
| `\newenvironment{nome}{início}{fim}` | Ambiente novo | avisa - não é registrado; o uso avisa e o conteúdo sai como texto | `latex.rs:2016` (ambiente), `latex/pacotes.rs:22` |
| `\renewenvironment` | Troca de ambiente | avisa | `latex/pacotes.rs:22` |
| `\newlength{\x}`, `\addtolength{\x}{…}`, `\settowidth{\x}{…}` | Comprimentos | não faz - no corpo o argumento vaza como texto (fallback; `newlength` e `addtolength` estão em comando_conhecido, então sem aviso) (inferido) | `latex/avisos.rs:184`, `latex.rs:1187` |
| `\DeclareMathOperator`, `\DeclareRobustCommand` | Comandos de matemática e robustos | ignora em silêncio no preâmbulo; avisa no corpo | sem tratamento |

---

## 27. abntex2 (classe e comandos da ABNT)

| Comando/ambiente | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| `\documentclass[12pt,a4paper,…]{abntex2}` | Classe da ABNT | faz em parte - classe aceita; 12 pt, margens ABNT e `section=TITLE` (títulos em maiúsculas) são lidos; as outras opções somem sem aviso | `latex/avisos.rs:216`, `latex.rs:301`, `latex.rs:1358` |
| `\usepackage[alf]{abntex2cite}` | Citação ABNT | faz em parte - o estilo ABNT é detectado; a formatação das citações fica fora deste arquivo (não auditado) | `latex/avisos.rs:210`, `latex.rs:196` |
| `\citeonline{chave}` | Citação ABNT | faz (ver seção 17) | `latex.rs:1074` |
| `\citeauthoronline{…}` | Citação com autor | avisa | `latex.rs:1187` |
| `\titulo`, `\autor`, `\local`, `\data`, `\instituicao`, `\orientador`, `\coorientador`, `\preambulo`, `\tipotrabalho`, `\data` | Dados da capa | não faz - só `\title` e `\author` são lidos (no preâmbulo, sem aviso) | `latex.rs:77` (ler_texto) |
| `\imprimircapa`, `\imprimirfolhaderosto`, `\imprimirfolhadeaprovacao` | Capa e folhas | avisa - sem tratamento | `latex.rs:1187` |
| `\begin{resumo}…\end{resumo}` (e `\imprimirresumo`) | Resumo em português | avisa - o conteúdo sai como texto corrido, sem título "RESUMO" | `latex/pacotes.rs:22` |
| `\begin{abstract}…\end{abstract}` | Resumo em inglês | faz em parte - vira citação (bloco recuado), sem título | `latex.rs:2051` |
| `\imprimirsumario`, `\imprimirlistadefiguras`, `\imprimirlistadetabelas`, `\imprimirindice`, `\imprimirbibliografia` | Listas e índices da ABNT | avisa - sem tratamento (o `\tableofcontents` e o `\printbibliography` funcionam) | `latex.rs:1187` |
| `\apendice`, `\anexo`, `\begin{apendice}`, `\begin{anexo}` | Apêndice e anexo | avisa - sem tratamento | `latex/pacotes.rs:22` |
| `\abntexchapterfont`, `\abntexsectionfont` | Fonte dos títulos | ignora em silêncio (no preâmbulo) | sem tratamento |
| Maiúsculas no sobrenome das citações (ABNT) | Autor em caixa alta | faz | `latex.rs:77` (ler_texto, `maiusculas`) |

---

## 28. Outros pacotes encontrados na lista de conhecidos (sem tratamento próprio)

Estes pacotes estão em `latex/avisos.rs:197-210` (`pacote_conhecido`). Ser reconhecido significa só que não gera aviso de pacote. Os comandos deles seguem as regras das seções acima.

| Pacote | O que faz no LaTeX | Estado | Onde no código |
|---|---|---|---|
| fontspec, times, mathptmx, helvet, palatino, lmodern, newtxtext, etc. | Trocam a fonte | ignora em silêncio - a fonte é a do FeatherDocs (comentário em `avisos.rs:198`) | `latex/avisos.rs:197-210` |
| ulem, soul | Sublinhado e marca-texto | faz em parte - só `\uline`, `\sout`, `\hl`; `\ul` e `\xout` avisam | `latex.rs:1157`, `latex/linha.rs:30` |
| footmisc, endnotes | Notas | faz em parte - `\footnote` e `\endnote` sim; `\theendnotes` e opções do footmisc não | `latex.rs:1188` |
| makeidx, imakeidx | Índice | faz em parte - `\index`, `\printindex`; `\makeindex` silencioso | `latex.rs:1157`, `latex/avisos.rs:186` |
| varioref | `\vref`, `\vpageref` | avisa - sem tratamento | `latex.rs:1187` |
| multicol | `\begin{multicols}{2}` | avisa - ambiente desconhecido; colunas não são feitas | `latex/pacotes.rs:22` |
| fancyvrb, verbatim | Código | faz | `latex.rs:2043` |
| natbib, biblatex, cite | Citações | faz - ver seção 17 | `latex.rs:1074` |
| endnotes, longtable, array, colortbl, multirow, tabularx, booktabs | Tabelas | faz em parte - ver seções 6 e 7 | `latex.rs:2052` |
| placeins, afterpage | Flutuantes | ignora em silêncio (`\FloatBarrier`) ; `\afterpage` avisa | `latex/avisos.rs:203` |
| etoolbox, ifthen, calc, xparse | Programação do TeX | ignora em silêncio no preâmbulo; no corpo, `\ifthenelse` avisa | `latex/avisos.rs:200` |
| lipsum, blindtext | Texto de teste | avisa - `\lipsum` sai como texto `[…]` ou nada (inferido) | `latex.rs:1187` |
| ragged2e | `\RaggedRight` | faz em parte - lido no preâmbulo (alinhamento); `\justifying` avisa | `latex.rs:77` |
| setspace | Espaçamento | não faz - ver seção 1 (`\onehalfspacing` ignorado) | `latex/avisos.rs:184` |
| parskip | Espaço entre parágrafos | faz em parte - só `\setlength{\parskip}` no preâmbulo | `latex.rs:265` |

---

## Resumo

Contagem por estado (linhas de tabela das seções 1 a 28, uma linha por comando ou ambiente). Os números abaixo são preenchidos a partir da própria tabela deste arquivo.

| Estado | Linhas | Parcela |
|---|---|---|
| faz | 119 | 33% |
| faz em parte | 55 | 15% |
| ignora em silêncio | 70 | 19% |
| avisa | 99 | 27% |
| não faz | 18 | 5% |

Total de 361 linhas nas tabelas das seções 1 a 28. Um mesmo comando pode aparecer em mais de uma seção, e cada linha tem um único estado (o primeiro da coluna).

Os problemas de maior dano são os que deixam o documento errado sem aviso, ou com aviso que não diz o que se perdeu. Todos vêm da leitura do código e pedem confirmação com um trecho mínimo:

- **Cor do xcolor com escala de 0 a 1** (`\definecolor{x}{rgb}{0.5,0.2,0.1}`): a cor some sem aviso, porque `parsear_cor` só lê inteiros de 0 a 255 (`latex.rs:567`). É o padrão do xcolor.
- **Figura com mais de uma imagem**: só a primeira vai para o documento; as outras são apagadas sem aviso (`latex.rs:2137-2151`).
- **`\item[termo]` em description e `enumerate[label=\alph*]`**: o termo e o formato do marcador são descartados sem aviso (`latex.rs:2229`, `latex.rs:2196`).
- **`\onehalfspacing` e `\doublespacing`**: silenciosos e sem efeito; o espaçamento de 1,5 da ABNT não é aplicado (`latex/avisos.rs:184`).
- **`\\[medida]` e `\cline` dentro de tabela**: a medida e o argumento de `\cline` entram como texto na célula (inferido, `latex.rs:2249-2253`).
- **`\pageref` e `\ref` sem alvo**: saem `[chave]` sem aviso (`latex.rs:1187`).
- **`minted` com opções** (`\begin{minted}[linenos]{python}`): as opções entram no código como texto (inferido, `latex.rs:2096`).

### Lista priorizada do que falta (do mais usado em relatórios e TCCs para o menos)

Cada item traz como caberia no modelo atual (`crates/featherdocs-core/src/modelo.rs`: `Conteudo` com Paragrafo, Titulo, Item, Citacao, Codigo, Equacao, Imagem, Tabela, Bibliografia, Sumario etc.; `Atrib` com fundo, borda, cor_borda, legenda, lista, entrelinha, recuo_esq, recuo_dir; `Bloco.mesclagens` com `MesclagemTabela` só horizontal).

1. **Tabelas com células mescladas (`\multicolumn`, `\multirow`).** Muito usadas em cabeçalhos de TCC. Caberia como `MesclagemTabela` para o horizontal (o tipo já existe); o vertical (`\multirow`) pede um campo novo, porque `MesclagemTabela` só tem `colunas`.
2. **Régua e borda de tabela (`\hline`, `\cline`, `\toprule`, `\midrule`, `\cmidrule`).** Caberia como régua por linha no modelo de `Tabela` (hoje as réguas são descartadas). Antes disso, `\cline` precisa de tratamento para não vazar no texto.
3. **Subfiguras (`subfigure`, `\subcaption`) e figura com mais de uma imagem.** Hoje a segunda imagem é apagada sem aviso. Caberia como uma `Tabela` sem borda com imagens nas células, ou como uma sequência de `Imagem` com legenda de subfigura.
4. **Referências cruzadas (`\cref`, `\Cref`, `\pageref`, `\hyperref`).** `\cref` sai a chave como texto. Caberia como `Trecho.link` com `#rotulo:` (o mecanismo já existe) e o tipo do rótulo tirado do prefixo; `\pageref` pede paginação, que o modelo não tem.
5. **Data `\today` e `\date`.** Caberia como `Paragrafo` depois do `Titulo` no `\maketitle` (`titulo_do_documento` já monta o título e o autor desse jeito).
6. **Caixas com moldura e fundo (`\fbox`, `\framebox`, `\fcolorbox`, `tcolorbox`).** Caberia como `Paragrafo` com `Atrib.fundo`, `Atrib.borda` e `Atrib.cor_borda` (campos existentes); o título da tcolorbox seria um segundo parágrafo na mesma caixa.
7. **Cores do xcolor: `rgb` com 0 a 1, cmyk, nomes completos e mistura `red!80`.** Não precisa de modelo novo. É conversão em `parsear_cor` e `resolver_cor`. O `rgb` 0 a 1 é o caso mais grave, porque falha em silêncio.
8. **Cor de célula e de linha (`\cellcolor`, `\rowcolor`, `\rowcolors`, colortbl).** Caberia como fundo por célula; `Celula` hoje é só lista de `Trecho`, sem fundo próprio.
9. **Opções de listas: `\item[termo]` (description) e `enumitem label=\alph*`.** Caberia como um rótulo em `Item` e o formato de marcador em `Atrib.lista` (confirmar se o enum `Lista` aceita letras e romanos).
10. **Espaçamento de 1,5 linha (`\onehalfspacing`, `\doublespacing`).** Relevante para ABNT. Caberia como `Atrib.entrelinha` lido do preâmbulo (o campo existe).
11. **Apêndices (`\appendix`, `\apendice`, `\anexo`).** Pede um modo de numeração por letra em `Numeracao`, mais o título de apêndice.
12. **Legenda de código e marcador de linguagem (`lstlisting [caption, label]`, `\lstinputlisting`, `minted [opções]`).** Caberia como campos novos em `Conteudo::Codigo` (legenda e nome). O bug de `minted` com opções vazando para o código deve ser corrigido antes.
13. **Siunitx (`\SI`, `\si`, `\num`).** Caberia como conversão para `Trecho` de texto (valor, espaço, unidade, vírgula decimal). Não precisa de modelo novo.
14. **Acentos e símbolos restantes (`\o`, `\ss`, `\i`, `\euro`, `\textregistered`, `\=`, `\.`, `\u`, `\v`, `\H`, `\k`, `\d`).** Caberia na tabela de caracteres em `compor` e `simbolo`. Também precisa corrigir `\'{\i}`, que sai com barra.
15. **Macros com argumento opcional (`\newcommand[n][padrão]`), `\providecommand`, `\def`, `\newenvironment`.** Caberia como expansão em `extrair_definicoes` e `expandir_macros`. O `\def` no corpo vaza texto hoje.
16. **Teoremas (`amsthm`: `\newtheorem`, `theorem`, `proof`, `\qed`).** Caberia como `Paragrafo` com trecho em negrito e um contador próprio por ambiente; o contador é a parte nova.
17. **Caixa de texto com largura (`\parbox`, `minipage` lado a lado, `\makebox[larg]`).** Caberia com `Atrib.recuo_esq` e `Atrib.recuo_dir` (já existem) para a largura; colunas lado a lado pedem um bloco novo.
18. **Geometria (`textwidth`, `a5paper`, `headheight`, `twoside`).** Caberia em `configurar_pagina` e `ConfigPagina`.
19. **Títulos com formato próprio (`\titleformat` com negrito, tamanho, cor, centralizado).** Caberia no `Titulo` com `Atrib.alinhamento` e `Atrib.estilo`. Hoje só `\MakeUppercase` é lido.
20. **Cabeçalho de `\fancyhead` com régua (`\headrulewidth`) e `\fancypagestyle`.** Caberia em `Faixas` e num campo de régua.
21. **Imagens com `height`, `scale` e `angle`.** `Imagem` já tem `altura`; `scale` vira largura proporcional e `angle` pede campo novo.
22. **PDF e EPS como imagem.** Precisa de renderização (fora do leitor), mais um tamanho natural. Não cabe só no modelo.
23. **Versalete (`\textsc`) e família sem serifa (`\sffamily`, `\rmfamily`).** Caberia como bit de `estilo` (`SERIFA` já existe; versalete pede bit novo).
24. **abntex2: capa, folha de rosto, resumo, listas e índices (`\imprimir…`, `\begin{resumo}`).** Caberia como `Documento.meta` e blocos `Titulo` e `Paragrafo` centralizados; o resumo seria `Citacao` com título.
25. **Desenho (`tikzpicture`) e `tcolorbox` como ambiente.** TikZ precisa de renderizador, fora do leitor. Não cabe no modelo como texto.

---

## Coluna de volta: o que `exportar.rs` grava

Arquivo: `crates/featherdocs-io/src/exportar.rs`. Resposta curta por área.

| Área | Grava? | Observação |
|---|---|---|
| Texto, negrito, itálico, sublinhado, tachado, código, cor, tamanho, sobrescrito, subscrito, marca-texto | sim | `exportar.rs:218` (trecho_tex) |
| Títulos, divisões e numeração (`\section*` quando sem número) | sim | `exportar.rs:290` (bloco_tex) |
| `\title`, `\author`, `\maketitle` | sim | `exportar.rs:613-620` |
| `\date`, `\today` no corpo | não | `\today{}` só aparece no cabeçalho (`exportar.rs:212`) e não é lido de volta |
| Listas (itemize, enumerate) | sim | `exportar.rs:627-628`; description não |
| Itens com rótulo próprio | não | |
| Citação (`quote`) | sim | `exportar.rs:306` |
| Código | parcial | `verbatim` sempre; linguagem e legenda não são gravadas (`exportar.rs:307`) |
| Tabela | parcial | `tabular` só com colunas `l`, `\hline` em todas as linhas e `\arraystretch`; sem mesclagem, sem cabeçalho marcado, sem cor de célula |
| Imagem e figura com legenda | sim | `exportar.rs:315-318` |
| Legenda solta (`\captionof`) | sim | `exportar.rs:302` |
| Equação | sim | `exportar.rs:321` (`\[ … \]`) |
| Sumário, lista de figuras, lista de tabelas | sim | `exportar.rs:322-324` |
| Régua, quebra de página, quebra de seção | sim | `exportar.rs:325-329` |
| Bibliografia, citações (`\cite`) | sim | `exportar.rs:204-205`, `exportar.rs:331-332`, estilos em `exportar.rs:657-663` |
| Índice remissivo | sim | `exportar.rs:206-207`, `exportar.rs:333` |
| Notas de rodapé e finais | sim | `exportar.rs:202-203` |
| Link (`\href`) | sim | `exportar.rs:252` (`\url` não é gravado como tal) |
| `\label`, `\ref`, `\pageref` (fora do cabeçalho) | não | `exportar.rs` não grava `\label`; os links internos não voltam |
| Cores de texto (`\textcolor[HTML]`) | sim | `exportar.rs:240` |
| Fundo do bloco (`\colorbox`) | sim | `exportar.rs:337` |
| Cor de página, marca d'água, números de linha | sim | `exportar.rs:419-422`, `exportar.rs:440` |
| Geometria (papel, margens, paisagem) | sim | `exportar.rs:572-584` (`geometry`) |
| Cabeçalho e rodapé (fancyhdr) | sim | `exportar.rs:409-413` |
| Espaçamentos e recuos (`\setlength`: leftskip, rightskip, parindent, baselineskip, `\vspace`) | sim | `exportar.rs:354-388` |
| `\onehalfspacing` e entrelinha de 1,5 | não | grava `\setlength{\baselineskip}` em vez disso (`exportar.rs:367`) |
| Fontes e pacotes de fonte (times etc.) | não | sempre `T1`/`utf8`/`babel brazil` (`exportar.rs:595`) |
| Macros (`\newcommand`) | não | não grava definições |
| `\input`/`\include` | não | o `.tex` sai como arquivo único |
| abntex2 | não | sai como `article` ou `report` com geometria |
| Figuras com subfigura, `subcaption`, `wrapfigure`, TikZ, tcolorbox, siunitx, amsthm, cleveref, csquotes, listings (com opções), multirow, multicolumn, booktabs | não | o que não existe no modelo não é gravado |

---

**Arquivo deste levantamento:** `/run/media/maragon/Arquivos/GitHub/featherdocs/docs/PARIDADE-LATEX-ABRANGENCIA.md`
