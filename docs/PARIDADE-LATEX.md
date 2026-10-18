> Inventário de paridade com o LaTeX (levantamento de 15/10/2026, status MEDIDO, não deduzido).
>
> Cada item abaixo é lido por um trecho mínimo de LaTeX no leitor próprio (`crates/featherdocs-io/src/latex.rs`) e a marca sai
> do modelo resultante (`featherdocs-core/src/modelo.rs`), não de olhar o código. Fonte dos trechos e dos critérios:
> `featherdocs/crates/featherdocs-io/dados/paridade_latex.txt`.
>
> Legenda: `[x]` o efeito do LaTeX aparece no modelo (estilo, bloco, cor, alinhamento…) · `[~]` parcial (reconhecido mas
> perde parte; ou reconhecido sem efeito visível; fórmula guardada como texto LaTeX limpo, sem montagem) · `[ ]` falta (o comando
> some, vira texto cru, o nome aparece no texto ou o modelo fica igual ao de um comando desconhecido).
> Referência de "certo": o PDF do `pdflatex`/Overleaf. Mede **amplitude**; fidelidade fina fica no fim.
>
> **Como refazer a medição** (a partir de `featherdocs/crates/featherdocs-io`, sob limite de memória):
> `systemd-run --user --scope -p MemoryMax=6G cargo run --example paridade_latex -- --saida ../../docs/PARIDADE-LATEX.md`
> (`--detalhe` explica cada marca; `--comparar docs/PARIDADE-LATEX.md` lista o que subiu ou desceu). Para marcar um item:
> melhore o leitor, rode de novo, e se o critério do item no arquivo de dados for fraco, acerte-o lá. Itens que o leitor não
> consegue medir (desenho, hifenização) levam marca fixa `=x`/`=~`/`= ` no arquivo de dados.
>
> Última medição: 15/10/2026. **Total: 439 / 1069 (41%)** pronto · 420 parcial · 210 falta
>

| Seção | Pronto | Parcial | Falta | Total | Paridade |
|---|---|---|---|---|---|
| 1. Classes e opções do documento | 18 | 22 | 8 | 48 | 37% |
| 2. Divisões, título e sumário | 20 | 24 | 5 | 49 | 40% |
| 3. Referências cruzadas, notas e índices | 22 | 25 | 19 | 66 | 33% |
| 4. Texto, fontes e estilos | 45 | 19 | 19 | 83 | 54% |
| 5. Símbolos de texto, acentos e caracteres | 13 | 18 | 28 | 59 | 22% |
| 6. Espaços, quebras, alinhamento e comprimentos | 34 | 33 | 19 | 86 | 39% |
| 7. Ambientes de texto, listas e citações | 27 | 22 | 12 | 61 | 44% |
| 8. Tabelas | 33 | 33 | 4 | 70 | 47% |
| 9. Figuras, imagens e flutuantes | 22 | 13 | 11 | 46 | 47% |
| 10. Caixas, cor e decoração | 11 | 26 | 29 | 66 | 16% |
| 11. Macros, contadores e programação | 16 | 27 | 11 | 54 | 29% |
| 12. Matemática (LaTeX, amsmath, amssymb, mathtools) | 117 | 13 | 1 | 131 | 89% |
| 13. Pacotes — texto, página e cabeçalhos | 30 | 61 | 9 | 100 | 30% |
| 14. Bibliografia e citações (BibTeX, natbib, biblatex) | 21 | 28 | 3 | 52 | 40% |
| 15. Pacotes — código, algoritmos, anotações e desenho | 8 | 23 | 15 | 46 | 17% |
| 16. Classes de documento e modelos (abntex2, beamer, currículos, KOMA) | 2 | 33 | 17 | 52 | 3% |
| **Total** | **439** | **420** | **210** | **1069** | **41%** |


## 1. Classes e opções do documento (18 / 48)

### Classes
- [x] `article` — numeração de seção 1, 1.1
- [x] `report` — `\chapter` com número de capítulo
- [x] `book` — `\chapter` e matter
- [~] `book` — `\frontmatter` (numeração romana, sem número de capítulo)
- [x] `book` — `\mainmatter` (reinicia em arábico)
- [~] `book` — `\backmatter`
- [~] `letter` — `\begin{letter}{destinatário}`
- [~] `letter` — `\opening` `\closing` `\signature` `\address` `\ps` `\cc` `\encl` `\name`
- [ ] `slides` (classe de slides do LaTeX2e)
- [ ] `beamer` — `\begin{frame}` → slide (rota para FeatherSlide)
- [~] `proc`, `IEEEtran`, `revtex`, `llncs`, `svjour`, `elsarticle`, `acmart` (classes de periódico)
- [x] `memoir` (classe de livro/artigo configurável)
- [x] `scrartcl` `scrreprt` `scrbook` (KOMA-Script)
- [~] `extarticle` `extreport` (corpo 8–20 pt, pacote `extsizes`)
- [x] `abntex2` (classe ABNT)
- [~] `standalone` (recorte de um trecho)
- [ ] `moderncv` / `altacv` / `awesome-cv` (currículo)

### Opções de classe
- [x] `10pt` `11pt` `12pt` (corpo base)
- [x] `a4paper` `letterpaper` `a5paper` `b5paper` `legalpaper` `executivepaper`
- [~] `a5paper`
- [~] `landscape` (paisagem)
- [~] `oneside` / `twoside` (margens de frente e verso)
- [~] `onecolumn` / `twocolumn` (duas colunas)
- [~] `titlepage` / `notitlepage` (capa separada)
- [x] `openright` / `openany` (capítulo abre em página ímpar)
- [x] `draft` / `final`
- [~] `fleqn` (equações à esquerda)
- [~] `leqno` (número da equação à esquerda)
- [x] `openbib` (bibliografia em formato aberto)
- [x] `@cls` opções globais passadas aos pacotes (`english,brazil`)

### Preâmbulo e arquivo
- [x] `\documentclass[opções]{classe}`
- [x] `\usepackage[opções]{pacote}` (lista com vírgula)
- [~] `\RequirePackage`
- [~] `\PassOptionsToPackage`
- [x] `\begin{document}` … `\end{document}`
- [ ] `\AtBeginDocument` / `\AtEndDocument`
- [ ] `\input{arquivo}` — conteúdo de outro `.tex`
- [ ] `\include{arquivo}` — capítulo em outro `.tex`
- [~] `\includeonly{lista}`
- [ ] `\endinput` (para a leitura do arquivo)
- [~] `\stop` / `\end{document}` antes do fim (ignora o resto)
- [ ] `\newif` `\iftrue` `\iffalse` `\fi` (condicionais de preâmbulo)
- [~] `\listfiles` `\nofiles` `\batchmode` `\scrollmode` `\errorstopmode` `\nonstopmode`
- [~] `\typeout{msg}` `\message{msg}` (só no terminal)
- [~] `\write` `\immediate\write` (grava arquivo auxiliar)
- [~] `\jobname`
- [x] `filecontents` / `filecontents*` (arquivo embutido)
- [x] `\newcommand{\texto}{…}` no preâmbulo e uso no corpo

## 2. Divisões, título e sumário (20 / 49)

### Divisões
- [ ] `\part{}` / `\part*{}`
- [x] `\chapter{}` (report/book)
- [~] `\chapter*{}` — sem número, mas aparece no corpo
- [x] `\section{}`
- [x] `\subsection{}`
- [x] `\subsubsection{}`
- [x] `\paragraph{}` (título de run-in)
- [x] `\subparagraph{}`
- [~] `\section*{}` (sem número)
- [x] `\section[curto]{longo}` (título curto no sumário)
- [x] `\appendix` — seções viram A, B…
- [x] `\appendix` em `report`/`book` — capítulos viram Apêndice A
- [~] `\secnumdepth` / `\setcounter{secnumdepth}{n}` (profundidade de numeração)
- [~] `\thesection` `\thesubsection` `\thechapter` (número como texto)
- [~] `\addcontentsline{toc}{section}{}` (entrada manual no sumário)
- [~] `\addtocontents{toc}{}`
- [~] `\markboth` `\markright` (cabeçalho corrente)

### Título e resumo
- [x] `\title{}` `\author{}` `\date{}`
- [x] `\maketitle` — bloco de título
- [x] `\and` entre autores
- [~] `\thanks{}` (nota do título)
- [~] `\today` (data de hoje em pt-BR)
- [~] `\date{\today}` / `\date{}` (sem data)
- [ ] `titlepage` (ambiente de capa)
- [x] `abstract`
- [x] `\tableofcontents`
- [x] `\listoffigures`
- [x] `\listoftables`
- [ ] `\contentsname` `\listfigurename` `\listtablename` `\refname` `\bibname` `\abstractname` `\partname` `\chaptername` `\appendixname` `\figurename` `\tablename` `\indexname`
- [x] `\setcounter{tocdepth}{n}` (níveis do sumário)
- [x] `\newpage`
- [~] `\clearpage`
- [ ] `\cleardoublepage` (para página ímpar)
- [~] `\pagebreak[n]` / `\nopagebreak[n]`
- [x] `\FloatBarrier` (placeins)

### Frente e verso, numeração
- [~] `\pagenumbering{arabic}`
- [~] `\pagenumbering{roman}` / `Roman`
- [~] `\pagenumbering{alph}` / `Alph` / `gobble`
- [~] `\setcounter{page}{n}`
- [~] `\pagestyle{empty}`
- [~] `\pagestyle{plain}` `headings` `myheadings`
- [~] `\thispagestyle{empty}`
- [~] `\pagename` `\thepage` (número da página como texto)
- [~] `\twocolumn[topo]` / `\onecolumn` (muda colunas no meio)
- [~] `\flushbottom` / `\raggedbottom`
- [x] `\newpage` + `\null` página em branco
- [~] `\mbox{}` vazio / `\leavevmode`
- [~] `\hrulefill` / `\dotfill` / `\hfill`
- [ ] `\clearemptydoublepage` (comum em livros)

## 3. Referências cruzadas, notas e índices (22 / 66)

### Referências
- [~] `\label{}` + `\ref{}` (seção)
- [~] `\label{}` + `\ref{}` (figura, tabela, equação, item)
- [~] `\pageref{}` — número da página
- [~] `\eqref{}` (hyperref/amsmath)
- [~] `\autoref{}` (hyperref) — "Seção 2"
- [x] `\nameref{}` (título da seção)
- [x] `\hyperref[label]{texto}`
- [ ] `\hypertarget{}{}` / `\hyperlink{}{}`
- [x] `\url{}`
- [x] `\href{url}{texto}`
- [x] `\href{mailto:}{}`
- [x] `\hyperbaseurl` `\urlstyle`
- [x] `\phantomsection`
- [~] `\texorpdfstring{}{}`
- [x] `\footnote{}` (hyperref `\footnote` clicável)
- [x] `\pdfbookmark` `\bookmark`
- [~] `\ref*` / `\autoref*` (sem link colorido)
- [~] `\label` em `\caption`
- [~] `\label` em `enumerate` (`\item\label{}`)
- [~] `\label` em `theorem` / `proof`
- [~] `\ref` para referência não definida ("??")
- [~] `\vref{}` — "seção 2 na página 3"
- [~] `\vpageref{}`
- [~] `\vrefrange{}{}`
- [~] `\cref{}` — "seção 2"
- [~] `\Cref{}` (maiúscula)
- [~] `\cref{a,b}` / `\crefrange{a}{b}` (lista)
- [~] `\cpageref{}`
- [~] `\labelcref{}`
- [~] `\crefname{}{}{}` (nome do tipo em pt-BR)

### Notas
- [x] `\footnote{}` — nota de rodapé numerada
- [~] `\footnote[n]{}` — número forçado
- [ ] `\footnotemark` + `\footnotetext{}`
- [ ] `\thanks{}` fora do título
- [x] `\footnote` dentro de tabela, `minipage`, título (`\protect`)
- [x] `\footnote` dentro de `\caption` / `\fbox`
- [x] `\endnote{}` + `\theendnotes` (`endnotes`)
- [x] `\thefootnote` / `\renewcommand{\thefootnote}{\fnsymbol{footnote}}`
- [ ] `\fnsymbol{}` `\arabic{}` `\roman{}` `\Roman{}` `\alph{}` `\Alph{}` (numeração de contador)
- [~] `\footnoterule` `\footnotesep` `\skip\footins`
- [ ] `\marginpar{}` (nota na margem)
- [ ] `\marginparwidth` `\marginparsep` `\reversemarginpar`
- [ ] `\sidenote` / `\marginnote` (`sidenotes`/`marginnote`)
- [x] `\mpfootnote` (nota em `minipage`)
- [x] `footmisc` — `bottom` `perpage` `symbol` `stable` `hang` `multiple`
- [x] `\footnote` repetida com `\footref{}`

### Índices e glossários
- [ ] `\makeindex` + `\index{termo}`
- [ ] `\index{termo!subtermo}`
- [ ] `\index{termo@forma|textbf}` (ordem e formato)
- [ ] `\index{termo|see{outro}}`
- [ ] `\index{termo|(}` … `\index{termo|)}` (intervalo)
- [x] `\printindex`
- [x] `\printindex[nome]` (`imakeidx`, vários índices)
- [~] `\indexspace` `\indexentry` `\see` `\seealso`
- [~] `\makeglossaries` + `\newglossaryentry{}{name,description}`
- [ ] `\gls{}` `\Gls{}` `\glspl{}` `\glsdesc{}` `\glssymbol{}`
- [x] `\printglossary` / `\printglossaries`
- [ ] `\newacronym{}{}{}` + `\gls` (primeira vez por extenso)
- [ ] `\glsresetall` `\glsunset` `\acrshort` `\acrlong` `\acrfull`
- [ ] `acro`: `\DeclareAcronym{}{short=,long=}` + `\ac{}`
- [ ] `\printacronyms` / `\acs` `\acl` `\acf` `\acp`
- [x] `acronym`: `\acro{}[]{}` `\ac{}` `\acf{}` + ambiente `acronym`
- [ ] `\nomenclature{}{}` + `\printnomenclature`
- [ ] `\listofalgorithms` / `\listoflistings` (lista de códigos)
- [x] `\tableofcontents*` (sem entrada de si mesmo)
- [~] `\addcontentsline` p/ bibliografia e lista (ABNT)

## 4. Texto, fontes e estilos (45 / 83)

### Estilos de fonte (comandos com argumento)
- [x] `\textbf{}`
- [x] `\textit{}`
- [x] `\textsl{}` (inclinado)
- [x] `\emph{}`
- [x] `\emph` dentro de itálico (volta a romano)
- [x] `\texttt{}`
- [x] `\textsf{}`
- [ ] `\textrm{}`
- [x] `\textmd{}` / `\textup{}` / `\textnormal{}`
- [ ] `\textsc{}` (versalete)
- [ ] `\textsuperscript{}`
- [ ] `\textsubscript{}`
- [x] `\underline{}`
- [~] `\mbox{}` / `\hbox{}`
- [ ] `\texorpdfstring{}{}`
- [x] `\textcolor{cor}{texto}`
- [x] `\textsf{\bfseries …}` combinados

### Estilos de fonte (declarações)
- [x] `\bfseries`
- [x] `\mdseries`
- [x] `\itshape`
- [x] `\slshape`
- [ ] `\scshape` (versalete)
- [x] `\upshape`
- [x] `\ttfamily`
- [x] `\sffamily`
- [ ] `\rmfamily`
- [x] `\normalfont`
- [~] `\bf` `\it` `\tt` `\sf` `\rm` `\sc` `\sl` (antigos, em grupo)
- [x] `\em` (declaração de ênfase)
- [x] `\emph` e `\em` em ambientes (`theorem` em itálico)
- [~] `\DeclareTextFontCommand` / `\newcommand{\negrito}[1]{\textbf{#1}}`

### Tamanhos
- [x] `\tiny`
- [x] `\scriptsize`
- [x] `\footnotesize`
- [x] `\small`
- [x] `\normalsize`
- [x] `\large`
- [x] `\Large`
- [x] `\LARGE`
- [x] `\huge`
- [x] `\Huge`
- [~] `\fontsize{}{}\selectfont`
- [~] `\fontseries` `\fontshape` `\fontfamily` `\fontencoding` + `\selectfont`
- [ ] `\relsize{}` / `\larger` `\smaller` (`relsize`)
- [ ] `\resizebox{}{}{}` (texto)
- [ ] `\scalebox{}{}`
- [ ] `\rotatebox{}{}`
- [~] `\mathversion` / `\boldmath` (texto em fórmula)

### Famílias e pacotes de fonte
- [x] `\usepackage{lmodern}` (Latin Modern)
- [x] `\usepackage[T1]{fontenc}`
- [x] `\usepackage[utf8]{inputenc}`
- [x] `\usepackage[latin1]{inputenc}` `\inputencoding`
- [x] `times` / `mathptmx` / `newtxtext` (Times)
- [~] `helvet` + `\renewcommand{\familydefault}{\sfdefault}`
- [~] `palatino` / `mathpazo` / `newpxtext`
- [~] `bookman` `charter` `utopia` `fourier` `libertine` `kpfonts` `concrete` `cmbright`
- [~] `avant` `courier` `tgtermes` `tgheros` `tgcursor` `tgpagella`
- [~] `fontspec` — `\setmainfont{}` `\setsansfont{}` `\setmonofont{}` (XeLaTeX/LuaLaTeX)
- [~] `fontspec` — `\fontspec{}` / `\newfontfamily`
- [ ] `unicode-math` (`\setmathfont`)
- [x] `\usepackage{microtype}` (ajuste micro-tipográfico)
- [x] `\usepackage{setspace}` — ver seção de espaçamento
- [~] `\DeclareMathAlphabet` `\DeclareFontFamily` `\DeclareFontShape`
- [~] `\usepackage{textcomp}` (símbolos de texto)
- [~] `\usepackage{babel}` e fontes de idioma
- [~] `\usepackage[scaled]{helvet}` (escala)
- [x] `\usepackage{anyfontsize}` / `type1ec` / `cm-super`

### Ênfase estendida (ulem, soul, outros)
- [x] `\uline{}` `\uuline{}` `\uwave{}`
- [x] `\sout{}` (tachado)
- [ ] `\xout{}` (rasurado)
- [ ] `\dashuline{}` `\dotuline{}`
- [x] `\normalem` (restaura `\emph`)
- [ ] `\hl{}` (realce)
- [ ] `\ul{}` (sublinhado do soul)
- [x] `\st{}` (tachado do soul)
- [ ] `\so{}` (espaçado) / `\caps{}` / `\sethlcolor{}`
- [~] `\colorbox{cor}{texto}`
- [ ] `\fcolorbox{borda}{fundo}{texto}`
- [x] `\textbf` com acentos e `\'{}` dentro
- [x] `\textit{\textbf{}}` aninhados
- [~] `\texttt{}` com `\_` `\#` `\%` `\&` escapados
- [~] `\verb|…|` e `\verb*|…|`
- [ ] `\url{}` com `%` e `#` dentro

## 5. Símbolos de texto, acentos e caracteres (13 / 59)

### Acentos e letras especiais
- [~] `\'` `\`` `\^` `\"` `\~` `\=` `\.` (acentos agudo, grave, circunflexo, trema, til, macro, ponto)
- [ ] `\u{}` `\v{}` `\H{}` `\r{}` `\k{}` (breve, caron, trema húngaro, anel, ogonek)
- [x] `\c{}` (cedilha) `\d{}` `\b{}` `\t{}` (ponto embaixo, barra, tie)
- [ ] `\'{a}` com chaves e `\'\i` (i sem ponto)
- [ ] `\i` `\j` (i e j sem ponto)
- [~] `\ae` `\AE` `\oe` `\OE` `\aa` `\AA`
- [ ] `\o` `\O` `\l` `\L` `\ss` `\SS`
- [ ] `\dh` `\DH` `\th` `\TH` `\ng` `\NG` `\dj` `\DJ` (T1)
- [~] `\~{n}` ñ e `\"{u}` ü (compostos)
- [x] Letras acentuadas em UTF-8 direto (`ção`, `é`, `ü`)

### Pontuação, aspas e travessões
- [x] `--` (en dash) e `---` (em dash)
- [x] `` `` `` e `''` (aspas duplas inglesas)
- [~] `` ` `` e `'` (aspas simples)
- [ ] `\glqq` `\grqq` `\flqq` `\frqq` `\guillemotleft` `\guillemotright` `\quotedblbase` (babel)
- [ ] `\enquote{}` / `\textquote` (csquotes)
- [ ] `\mkbibquote` `\blockquote` `\foreignquote`
- [~] `\ldots` `\dots` `\textellipsis` `\cdots`
- [~] `\!` `!`` ``?`` (pontos invertidos `¡ ¿`)
- [x] `\-` (ponto de hifenização) e `\hyphenation{}`
- [~] `\nobreakdash-` `\textendash` `\textemdash`
- [ ] `\textquoteleft` `\textquoteright` `\textquotedblleft` `\textquotedblright`
- [ ] `\textquotesingle` `\textasciigrave` `\textquotedbl`
- [ ] `\guilsinglleft` `\guilsinglright` `\quotesinglbase`

### Caracteres reservados
- [x] `\%` `\$` `\&` `\#` `\_` `\{` `\}` (escapados)
- [~] `\textbackslash` `\textasciitilde` `\textasciicircum` `\textbar` `\textless` `\textgreater`
- [ ] `\slash` (barra que permite quebra)
- [~] `~` (espaço que não quebra)
- [x] `\ ` (espaço depois de comando) e `\@` (fim de frase)
- [x] `\frenchspacing` / `\nonfrenchspacing`
- [x] `\obeylines` `\obeyspaces`

### Símbolos de texto (textcomp / padrão)
- [~] `\textcopyright` `\copyright` `\textregistered` `\texttrademark` `\textcircledP`
- [~] `\S` `\P` (seção e parágrafo) `\dag` `\ddag` `\textdagger` `\textdaggerdbl`
- [ ] `\pounds` `\textsterling` `\texteuro` `\yen` `\textyen` `\textdollar` `\textcent` `\textcurrency`
- [~] `\euro` / `eurosym`: `\EUR{}` `\euro`
- [ ] `\textdegree` `\textcelsius` `\textohm` `\textmu` `\textperthousand` `\textpertenthousand`
- [ ] `\textpm` `\texttimes` `\textdiv` `\textonehalf` `\textonequarter` `\textthreequarters` `\textminus`
- [ ] `\textordfeminine` `\textordmasculine` (ª º)
- [~] `\textbullet` `\textperiodcentered` `\textasteriskcentered` `\textparagraph` `\textsection`
- [ ] `\textless` `\textgreater` `\textleftarrow` `\textrightarrow` `\textuparrow` `\textdownarrow`
- [ ] `\textnumero` `\textreferencemark` `\textcolonmonetary` `\textlira` `\textnaira` `\textwon` `\textbaht`
- [ ] `\textflorin` `\textlnot` `\textbrokenbar` `\textlozenge` `\textinterrobang` `\textborn` `\textdied`
- [ ] `\checkmark` `\ding{}` (`pifont`) `\cmark` `\xmark`
- [x] `\ldots` em modo texto e `\textellipsis`
- [~] `\ensuremath{}` (símbolo que funciona em texto e fórmula)
- [ ] `\ss{}` `\i{}` e outros "com chaves vazias" para colar texto
- [ ] `\char"41` `\symbol{65}` `\textunicode`/`\unichar` (código de caractere)
- [~] `\today` com `babel` (data por extenso pt-BR)
- [ ] `\datebrazil` `\dateenglish` `\selectlanguage{}` `\foreignlanguage{}{}` (babel)
- [ ] `\textgreek{}` `\textcyrillic{}` `\begin{otherlanguage}` (babel)
- [~] `\begin{hyphenrules}{}`, `\babelhyphenation{}`
- [x] `\selectlanguage{brazil}` e textos fixos ("Sumário", "Figura")
- [~] `\captionsbrazil` / `\addto\captionsbrazil{\renewcommand{\figurename}{Fig.}}`

### Símbolos diversos
- [x] `\LaTeX` `\TeX` `\LaTeXe` `\XeTeX` `\BibTeX` (logos)
- [ ] `\ding{}` / `pifont` (dingbats)
- [~] `\faIcon{}` (`fontawesome5`)
- [ ] `\texttildelow` `\textbardbl` `\textcircled{}` `\textcircled{a}` (a em círculo)
- [ ] `\ding{172}` números em círculo `\textcircled{1}`
- [ ] `\perthousand` (mille) `\micro` `\celsius` `\ohm` `\degree` (gensymb)
- [x] `\SI{}{}` `\num{}` (siunitx) como símbolo de texto

## 6. Espaços, quebras, alinhamento e comprimentos (34 / 86)

### Quebras de linha e parágrafo
- [x] `\\` (quebra de linha)
- [x] `\\[2pt]` / `\\*` (com espaço extra, sem quebra de página)
- [x] `\newline`
- [x] `\linebreak[n]` `\nolinebreak[n]`
- [x] `\par` e linha em branco (parágrafo novo)
- [~] `\noindent`
- [~] `\indent` (força recuo)
- [~] `\parindent` — `\setlength{\parindent}{1cm}`
- [~] `\parskip` — `\setlength{\parskip}{6pt}`
- [x] `\sloppy` / `\fussy` / `\emergencystretch`
- [~] `\hyphenpenalty` `\tolerance` `\exhyphenpenalty` `\hbadness` `\vbadness` `\clubpenalty` `\widowpenalty`
- [~] `\mbox{}` impede quebra · `\nobreak` · `\nolinebreak`
- [~] `\unskip` `\ignorespaces` `\relax` `\null` `\protect`
- [x] `\obeylines`, `\\` em `\parbox` e células

### Alinhamento
- [x] `\centering`
- [x] `center`
- [x] `flushleft` / `\raggedright`
- [x] `flushright` / `\raggedleft`
- [x] `\raggedright` (declaração)
- [x] `\raggedleft` (declaração)
- [~] `\justifying` (`ragged2e`) / `\Centering` `\RaggedRight` `\RaggedLeft`
- [x] `justify` (ambiente `ragged2e`)
- [x] `\hfil` `\hfill` `\hfilll` (cola elástica horizontal)
- [~] `\hss` `\hskip` `\kern` `\mkern` `\mskip`
- [x] `\leftskip` `\rightskip` `\parfillskip` `\@rightskip`
- [ ] `\leftmargin`/`\rightmargin` em listas e `\leftskip` em parágrafo

### Espaço horizontal e vertical
- [x] `\hspace{}` `\hspace*{}`
- [x] `\vspace{}` `\vspace*{}` (inclusive negativo)
- [~] `\quad` `\qquad` `\enspace` `\thinspace` `\negthinspace`
- [x] `\,` `\:` `\;` `\!` `\ ` (espaços finos)
- [~] `\smallskip` `\medskip` `\bigskip`
- [~] `\vfill` `\vfil` `\vss`
- [~] `\addvspace` `\addpenalty` `\vskip` `\vphantom`
- [x] `\phantom{}` `\hphantom{}` `\vphantom{}`
- [~] `\strut` `\rule{}{}` (régua cheia com largura/altura)
- [ ] `\rule[raise]{w}{h}` / `\hrulefill` / `\dotfill`
- [ ] `\hrule` / `\vrule`
- [~] `\noindent\rule{\textwidth}{0.4pt}` (linha cheia)
- [~] `\leaders\hrule\hfill` `\xleaders` `\cleaders`
- [ ] `\raisebox{}{}`
- [ ] `\lower` `\raise` `\moveleft` `\moveright`
- [x] `\enlargethispage{}`

### Comprimentos e unidades
- [x] `\setlength{\len}{valor}`
- [~] `\addtolength` `\settowidth` `\settoheight` `\settodepth`
- [x] `\newlength` `\the\len`
- [x] Unidades `pt` `bp` `mm` `cm` `in` `pc` `dd` `cc` `sp` `em` `ex` `mu`
- [x] Comprimentos elásticos `plus` / `minus` (`1em plus 0.5em minus 0.2em`)
- [~] `\textwidth` `\linewidth` `\columnwidth` `\paperwidth` `\textheight` `\paperheight`
- [ ] `\baselineskip` `\baselinestretch` `\lineskip` `\lineskiplimit`
- [~] `\parsep` `\itemsep` `\topsep` `\partopsep` `\labelsep` `\labelwidth` `\leftmargin`
- [ ] `\evensidemargin` `\oddsidemargin` `\topmargin` `\headheight` `\headsep` `\footskip` `\marginparwidth`
- [~] `\textwidth` alterado com `\setlength{\textwidth}{16cm}`
- [ ] `\hoffset` `\voffset` `\columnsep` `\columnseprule` `\arrayrulewidth` `\tabcolsep` `\arraycolsep` `\fboxsep` `\fboxrule`
- [~] `\dimexpr` / `\glueexpr` / `\numexpr` `\fpeval`
- [ ] `\totalheight` `\height` `\width` `\depth` (em `\makebox`/`\raisebox`)
- [~] `\@` e `\ensuremath` em unidades de comprimento do `calc` (`\widthof` `\heightof` `\depthof` `\totalheightof`)

### Espaçamento entre linhas
- [x] `\linespread{1.5}`
- [x] `\onehalfspacing`
- [x] `\doublespacing`
- [x] `\singlespacing`
- [x] `\setstretch{1.2}`
- [x] `spacing` (ambiente) / `singlespace` / `onehalfspace` / `doublespace`
- [~] `\setSingleSpace` `\SetSinglespace` `\vspace{\baselineskip}`
- [~] `parskip` — parágrafos sem recuo e com espaço entre eles
- [~] `\setlength{\parskip}{\baselineskip}`
- [~] `indentfirst` — recuo também no primeiro parágrafo

### Página e margens
- [~] `\usepackage[a4paper,margin=2.5cm]{geometry}`
- [x] `geometry`: `left=` `right=` `top=` `bottom=` `margin=` `inner=` `outer=`
- [~] `geometry`: `landscape` `paperwidth=` `paperheight=` `papersize=`
- [x] `geometry`: `letterpaper`
- [~] `geometry`: `a5paper`
- [~] `geometry`: `twoside` `bindingoffset` `headheight` `footskip` `includehead` `includefoot` `heightrounded` `showframe`
- [ ] `\newgeometry{}` / `\restoregeometry` (muda no meio)
- [~] `\setmarginsrb` (`vmargin`)
- [~] `\paperwidth` / `\setlength{\paperwidth}{…}`
- [ ] `\pdfpagewidth` `\pdfpageheight` `\pdfminorversion` `\pdfcompresslevel`
- [ ] `lscape` — `landscape` (ambiente, página girada)
- [ ] `pdflscape` — página em paisagem no PDF
- [ ] `rotating` — `sidewaystable` `sidewaysfigure` `\rotatebox` `turn`
- [ ] `\textheight` + `\topmargin` clássicos

### Colunas
- [ ] `multicols` — `\begin{multicols}{2}`
- [ ] `multicols*` (balanceamento) · `\columnbreak` · `\columnsep` · `\columnseprule`
- [ ] `\multicolsep` `\premulticols` `\postmulticols` `\raggedcolumns`
- [~] `\twocolumn` / `\onecolumn` / `\columnwidth`
- [x] `\begin{multicols}` com título largo `\begin{multicols}{2}[\section{T}]`
- [ ] `paracol` — colunas paralelas (`\switchcolumn`)

## 7. Ambientes de texto, listas e citações (27 / 61)

### Listas
- [x] `itemize`
- [x] `enumerate`
- [~] `description` (`\item[termo]`)
- [x] `\item[rótulo]` personalizado
- [x] Listas aninhadas (4 níveis)
- [~] Marcadores por nível: `\labelitemi` … `\labelitemiv`
- [~] Numeração por nível: `\labelenumi` … `\theenumi` (a., i., A.)
- [x] `\setcounter{enumi}{n}` (continuar numeração)
- [x] `\item` com `\label` e `\ref`
- [~] `\par` / linha em branco dentro de item
- [ ] `list` / `trivlist` (listas genéricas)
- [~] `\listparindent` `\itemindent` `\leftmargin` em `list`
- [~] `enumitem`: `label=` (`\alph*)`, `\roman*.`, `(\arabic*)`)
- [~] `enumitem`: `label=` com romano maiúsculo `\Roman*`
- [x] `enumitem`: `start=`
- [x] `enumitem`: `resume` / `resume*`
- [x] `enumitem`: `noitemsep` `nosep` `itemsep=` `parsep=` `topsep=` `partopsep=`
- [~] `enumitem`: `leftmargin=` `labelindent=` `labelwidth=` `labelsep=` `itemindent=` `align=`
- [x] `enumitem`: `\setlist{}` / `\setlist[itemize]{}` / `\setlist[enumerate,2]{}`
- [~] `enumitem`: `\newlist{}{}{}` + `\setlist` (listas novas)
- [~] `enumitem`: `label=\textbullet` / `label={\color{red}\textbullet}`
- [x] `enumitem`: `before=` `after=` `font=` `wide` `ref=`
- [x] `enumitem`: descrição `style=nextline` / `style=multiline` / `style=sameline`
- [~] `enumitem`: `\SetLabelAlign` `\AddEnumerateCounter` `\setenumerate` `\setitemize` `\setdescription`
- [ ] `paralist`: `compactitem` `compactenum` `compactdesc`
- [ ] `paralist`: `inparaenum` (lista no texto) `asparaenum` `inparaitem`
- [x] Lista de tarefas com caixa (`\item[$\square$]`, `\item[\checkmark]`)
- [x] `enumerate` com `\item[(a)]` rótulo manual
- [x] `itemize` dentro de tabela e de `minipage`

### Citações e blocos de texto
- [x] `quote`
- [x] `quotation` (com recuo de parágrafo)
- [ ] `verse` (poema, `\\` e `\\!`)
- [x] `center` / `flushleft` / `flushright`
- [x] `abstract` (resumo)
- [ ] `titlepage`
- [x] `minipage` — `\begin{minipage}[pos]{largura}`
- [x] `minipage` lado a lado (`\hfill`, `\hspace`)
- [ ] `\parbox[pos]{larg}{texto}`
- [ ] `\parbox` com `[t]`, `[b]`, `[c]` e altura `[c][3cm]`
- [ ] `tabbing` (tabulação) com `\=` `\>` `\\` `\kill`
- [~] `\raggedright` dentro de `parbox`, `\begin{flushleft}` aninhado
- [~] `\begin{samepage}` / `\nopagebreak`
- [ ] `\begin{sloppypar}`
- [ ] `\begin{comment}` (`comment`) e `\iffalse … \fi` — texto oculto
- [~] `\begin{filecontents}` fora do preâmbulo
- [~] `\newenvironment{nome}{início}{fim}` + uso
- [~] `\renewenvironment`
- [~] `\newenvironment` com argumentos `[n]` e `#1`
- [~] `\newenvironment{x}` com contador e `\item` próprio
- [~] `\renewcommand{\labelenumi}` e `\renewcommand{\theenumi}`

### Código literal
- [x] `verbatim`
- [x] `verbatim*` (espaços visíveis)
- [~] `\verb|texto|` e `\verb*`
- [x] `\begin{Verbatim}` (`fancyvrb`) `[frame=single,numbers=left]`
- [~] `\VerbatimInput{}` / `\fvset{}` / `\DefineVerbatimEnvironment`
- [~] `\lstinline{}` / `\lstinline|…|`
- [x] `\begin{lstlisting}`
- [x] `\mintinline{}{}` / `\begin{minted}{linguagem}`
- [ ] `\texttt{}` com caracteres especiais e `\detokenize`
- [ ] `alltt` (`alltt`)
- [x] `\begin{lstlisting}[language=Python]` com linguagem

## 8. Tabelas (33 / 70)

### tabular básico
- [x] `tabular` — colunas `l c r`
- [x] `tabular` — `|` separadores de coluna e `\hline`
- [~] `tabular` — `p{larg}` (parágrafo na célula)
- [~] `tabular` — `@{}` e `!{}` (espaço/separador de coluna)
- [~] `tabular` — `*{3}{c}` (repetição de coluna)
- [x] `tabular` — `[t]` `[b]` `[c]` (alinhamento vertical)
- [x] `\hline` linha horizontal
- [x] `\cline{i-j}` linha parcial
- [~] `\multicolumn{n}{c}{texto}`
- [x] `\\[espaço]` em tabela e `\\*`
- [~] `\tabularnewline`
- [x] Texto de célula com negrito, cor, fórmula, nota
- [x] `\\` dentro de `\parbox`/`\shortstack`/`\makecell` na célula
- [~] `\arraystretch` (altura das linhas)
- [x] `\tabcolsep` / `\arrayrulewidth` / `\extrarowheight`
- [~] `tabular*` com largura e `\extracolsep{\fill}`
- [x] `array` (matriz de texto em modo matemático)
- [x] `tabular` aninhada em célula
- [~] Célula vazia e `&` escapado `\&`
- [~] `array`: `m{larg}` e `b{larg}` (meio e base)
- [~] `array`: `>{\bfseries}c` (formatação por coluna)
- [x] `array`: `<{texto}` (depois da coluna)
- [~] `array`: `\newcolumntype{C}{>{\centering\arraybackslash}p{3cm}}`
- [x] `array`: `\arraybackslash` / `\raggedright` em coluna `p`
- [~] `array`: `\extrarowheight` · `\firsthline` `\lasthline` · `\hdashline` (`arydshln`)
- [x] `booktabs`: `\toprule` `\midrule` `\bottomrule`
- [x] `booktabs`: `\cmidrule(lr){2-3}`
- [x] `booktabs`: `\addlinespace` `\specialrule` `\morecmidrules`
- [~] `booktabs`: `\heavyrulewidth` `\lightrulewidth` `\cmidrulewidth` `\belowrulesep` `\aboverulesep`
- [~] `multirow`: `\multirow{n}{*}{texto}`
- [~] `multirow`: `\multirow` com largura fixa e `\multirowsetup` · `bigstrut` · `\bigstrut`
- [~] `multirow` + `\multicolumn` combinados (célula 2×2)
- [~] `colortbl`: `\rowcolor{cor}`
- [~] `colortbl`: `\cellcolor{cor}`
- [~] `colortbl`: `\columncolor{cor}` (em `>{\columncolor{}}`)
- [~] `colortbl`/`xcolor` `table`: `\rowcolors{1}{white}{gray!10}` (zebra)
- [x] `colortbl`: `\arrayrulecolor{cor}` `\doublerulesep` `\hline` colorida
- [~] `tabularx`: coluna `X` com largura total
- [x] `tabularx`: `\raggedright\arraybackslash` na coluna `X` · `\newcolumntype{Y}{>{\centering\arraybackslash}X}`
- [ ] `tabulary`: colunas `L C R J`
- [x] `longtable` (quebra entre páginas)
- [~] `longtable`: `\endfirsthead` `\endhead` `\endfoot` `\endlastfoot` (cabeçalho repetido)
- [x] `longtable`: `\caption` e `\label` dentro da tabela · `\caption*`
- [~] `longtable`: `\LTleft` `\LTright` `\LTpre` `\LTpost` `\LTcapwidth` · `\setcounter{LTchunksize}`
- [ ] `supertabular` / `xtab` (alternativas)
- [x] `makecell`: `\makecell{A\\B}`
- [~] `makecell`: `\thead{}` `\Gape` `\makecell[l]{}` `\rowcolors`
- [x] `diagbox`: `\diagbox{A}{B}` (célula diagonal)
- [~] `siunitx`: coluna `S` (alinha pelo decimal)
- [x] `threeparttable`: `tablenotes` `\tnote{a}`
- [~] `adjustbox`: `\adjustbox{max width=\textwidth}{tabela}` · `adjustbox` (ambiente)
- [~] `\resizebox{\textwidth}{!}{tabela}`
- [ ] `csvsimple` / `pgfplotstable` / `datatool` (tabela de CSV)
- [~] `ltablex` / `xltabular` (`tabularx` que quebra)
- [x] `hhline`: `\hhline{|=|=|}` (linhas duplas)
- [ ] `tabu` (`X[2c]`) / `nicematrix` / `tabularray` (`tblr`)

### Tabela flutuante
- [~] `table` com `\caption` e `\label`
- [x] `table` — posição `[h]` `[t]` `[b]` `[p]` `[!]` `[H]`
- [x] `table*` (duas colunas)
- [~] `\caption[curta]{longa}` · `\caption*{}` · `\captionof{table}{}`
- [~] Legenda acima ou abaixo (ordem de `\caption` e `tabular`)
- [x] `\centering` dentro de `table`
- [x] Fonte da tabela (nota abaixo, ABNT)
- [x] `\listoftables` com as legendas das tabelas
- [x] `float`: `[H]` posição fixa
- [~] `float`: `\newfloat{nome}{htbp}{ext}` · `\floatstyle{boxed}` · `\restylefloat`
- [x] `placeins`: `\FloatBarrier` · `\usepackage[section]{placeins}`
- [x] `\afterpage{\clearpage}`
- [~] `\floatpagefraction` `\topfraction` `\bottomfraction` `\textfraction` `\floatsep` `\textfloatsep` `\intextsep` `\dbltopfraction` `\dblfloatpagefraction` `\topnumber` `\bottomnumber` `\totalnumber`
- [x] `\suppressfloats` · `\clearpage` para despejar flutuantes

## 9. Figuras, imagens e flutuantes (22 / 46)

### graphicx
- [x] `\includegraphics{arquivo}`
- [~] `\includegraphics[width=…]` (cm, `\textwidth`, `\linewidth`)
- [~] `\includegraphics[height=…]`
- [~] `\includegraphics[scale=…]`
- [x] `\includegraphics[angle=…]` (girar)
- [x] `\includegraphics[keepaspectratio]` · `totalheight` · `origin` · `viewport` · `trim` · `clip` · `page=` (PDF)
- [~] `\includegraphics[width=.5\textwidth]` (fração da largura útil)
- [x] `\includegraphics` de `.pdf` `.eps` `.jpg` `.svg` (`svg`) · `\graphicspath{}` · `\DeclareGraphicsExtensions`
- [x] `\includegraphics[draft]` · `\setkeys{Gin}{width=\linewidth}`
- [ ] `\rotatebox[origin=c]{90}{texto}`
- [ ] `\scalebox{sx}[sy]{texto}` · `\reflectbox{}`
- [ ] `\resizebox{larg}{alt}{texto}` · `\resizebox*`
- [x] `\graphicspath` com várias pastas · `\includegraphics*`
- [x] `\includegraphics` em linha (dentro de parágrafo, `\raisebox`)
- [x] Imagem embutida por `data:`/arquivo de dentro do `.fdoz` (nome com espaços e acentos)

### figure
- [x] `figure` com `\includegraphics` + `\caption`
- [x] `figure` — posição `[h]` `[t]` `[b]` `[p]` `[H]` `[!htbp]`
- [x] `figure*` (duas colunas)
- [ ] `\caption[curta]{longa}`
- [~] `\label` depois da `\caption` + `\ref` ("Figura 1")
- [x] `\centering` em `figure` · `center` em volta de figura
- [x] `\captionof{figure}{}` fora de flutuante · `\captionsetup`
- [ ] Fonte da figura (nota ABNT) abaixo da legenda
- [x] Várias imagens numa `figure` (`\hfill`, `\quad`)
- [x] `\listoffigures` com legendas
- [x] `\figurename` `\tablename` (nomes dos rótulos: "Figura", "Tabela")
- [~] `\setcounter{figure}{n}` · `\numberwithin{figure}{section}`
- [x] `float` `[H]`
- [x] `caption`: `\captionsetup{font=small,labelfont=bf,format=hang,justification=centering}`
- [x] `caption`: `labelsep=period` `textfont=it` `skip=` `margin=` `width=` `singlelinecheck=off` `position=top`
- [~] `caption`: `\captionsetup[figure]{}` `\captionsetup[table]{}` · `\DeclareCaptionFormat` `\DeclareCaptionStyle` · `\caption*`
- [~] `caption`: `\captionof{}` · `\ContinuedFloat` · `\bothIfFirst` · `\clearcaptionsetup`
- [x] `subcaption`: `subfigure` + `\caption` + legenda geral (a) (b)
- [x] `subcaption`: `subtable` · `\subcaptionbox` · `\subref{}`
- [ ] `subfig`: `\subfloat[legenda]{\includegraphics{}}`
- [ ] `subfigure` (antigo): `\subfigure[]{}`
- [~] `wrapfig`: `\begin{wrapfigure}{r}{5cm}` (texto em volta)
- [~] `wrapfig`: `wraptable` · `\wrapoverhang` · `\intextsep` · `[l]` `[r]` `[i]` `[o]`
- [~] `floatflt`/`picinpar`/`sidecap` (`SCfigure` legenda lateral)
- [x] `\begin{center}\includegraphics\end{center}`
- [~] `\fbox{\includegraphics{}}` (imagem com moldura)
- [ ] `\includepdf` (`pdfpages`)
- [ ] `pdfpages`: `pages=` `nup=` `landscape` `scale=` `pagecommand` `\includepdfmerge`
- [ ] `\input{figura.pgf}` / `\includesvg` (`svg`) · `\includestandalone` (`standalone`)
- [ ] `overpic` / `eso-pic` / `textpos` (texto sobre imagem; marca-d'água de página)
- [~] `\setlength{\fboxsep}{0pt}\fbox{…}` borda ao redor da imagem

## 10. Caixas, cor e decoração (11 / 66)

### Caixas do LaTeX2e
- [~] `\mbox{}` · `\makebox[larg][pos]{}`
- [ ] `\fbox{}` (moldura)
- [ ] `\framebox[larg][pos]{}`
- [ ] `\parbox[pos][alt][int]{larg}{}`
- [ ] `\raisebox{alt}[prof][alt]{}`
- [ ] `\sbox` `\savebox` `\usebox` `\newsavebox` (caixa guardada)
- [ ] `lrbox`
- [ ] `\fboxsep` `\fboxrule` (espaço e espessura da moldura)
- [x] `\shortstack[pos]{A\\B}`
- [ ] `picture` / `\put` / `\line` / `\circle` / `\framebox` / `\makebox` (ambiente picture)
- [ ] `\unitlength` · `\linethickness` · `\thicklines` · `\thinlines` · `\oval` · `\vector` · `\qbezier`
- [~] `\footnotesize` dentro de caixa; `\bf` em `\fbox` · `\fbox{\parbox}`
- [ ] `\framebox` com borda e fundo (`\fcolorbox`)
- [~] `framed`: `framed` `shaded` `leftbar` `snugshade` · `\fcolorbox`/`\colorbox` envolvendo texto
- [ ] `framed`: `\FrameCommand` `\MakeFramed` `oframed` `titled-frame`
- [ ] `mdframed`: `\begin{mdframed}[linecolor=,backgroundcolor=]`
- [ ] `mdframed`: `linewidth` `innerleftmargin` `roundcorner` `frametitle` `topline`/`leftline` `\newmdenv` `\surroundwithmdframed`
- [~] `tcolorbox`: `\begin{tcolorbox}[colback=,colframe=,title=]`
- [ ] `tcolorbox`: `\tcbset` · `\newtcolorbox{nome}[n][opts]{opções}` · `\DeclareTColorBox`
- [~] `tcolorbox`: `breakable` (quebra entre páginas) · `enhanced` · `sharp corners` · `arc=` · `boxrule=` · `left=` `right=` `top=` `bottom=`
- [~] `tcolorbox`: `\tcbuselibrary{skins,breakable,listings,theorems,minted,raster}`
- [ ] `tcolorbox`: `\tcbox[opts]{texto}` · `\tcboxmath` (caixa em linha)
- [ ] `tcolorbox`: `tcbtheorem` · `\tcblower` (parte inferior) · `\tcbline` · `fonttitle` `coltitle` `colbacktitle`
- [ ] `tcolorbox`: `tcblisting` (código em caixa) · `tcbraster`
- [ ] `fancybox`: `\shadowbox{}` `\doublebox{}` `\ovalbox{}` `\Ovalbox{}` `Sbox`/`Bbox` · `\fancyoval` · `\cornersize`
- [ ] `boxedminipage`: `\begin{boxedminipage}{larg}`
- [ ] `bclogo` / `awesomebox` / `tcolorbox` blocos de aviso (`\begin{notebox}`)

### Cor (xcolor, color)
- [~] `\usepackage{color}` e `\usepackage{xcolor}`
- [~] `\color{nome}` (declaração)
- [x] `\textcolor{nome}{texto}`
- [x] `\color[rgb]{r,g,b}` · `\color[RGB]{}` · `\color[HTML]{RRGGBB}` · `\color[gray]{0.5}` · `\color[cmyk]{}`
- [~] `\textcolor[rgb]{0.1,0.2,0.3}{}`
- [~] Cores nomeadas básicas (`red` `green` `blue` `black` `white` `cyan` `magenta` `yellow`)
- [~] Cores nomeadas dvips (`xcolor` com `dvipsnames`: `Orange`, `RoyalBlue`, `ForestGreen`…)
- [~] `svgnames` (`DarkOliveGreen`, `Tomato`…) · `x11names` · `[table]` · `[usenames]`
- [x] `\definecolor{nome}{rgb}{r,g,b}` e uso
- [x] `\definecolor{nome}{HTML}{RRGGBB}`
- [x] `\definecolor{nome}{RGB}{r,g,b}` (0–255)
- [~] `\definecolor{nome}{gray}{g}` · `{cmyk}{c,m,y,k}` · `{hsb}` · `{Hsb}` · `{HSB}` · `{wave}`
- [~] `\colorlet{novo}{base}` / `\colorlet{novo}{base!50}`
- [~] Mistura `red!50`, `red!30!blue`, `-red`, `red!50!white!80!black`
- [~] `\colorbox{cor}{texto}`
- [ ] `\fcolorbox{borda}{fundo}{texto}`
- [ ] `\pagecolor{cor}` (cor da página) · `\nopagecolor`
- [~] `\normalcolor` · `\defaultcolor`
- [~] `\rowcolor` `\cellcolor` `\columncolor` · `\rowcolors` (ver Tabelas)
- [x] `\textcolor` dentro de `\textbf`, título, legenda, nota
- [x] `\color` em ambiente (`\begin{color}{red}`) · `\color` em grupo `{…}`
- [x] `\hypersetup{colorlinks,linkcolor=,urlcolor=,citecolor=}`
- [ ] `\setulcolor` `\setstcolor` (soul) · `\sethlcolor`
- [~] `\textcolor[named]{}` e `\selectcolormodel` · `\xcolorsetup` · `\definecolorset` · `\providecolor`
- [~] `xcolor`: funções `\colorlet{c}{red!20}` · `\blendcolors` · `\harmony` · `\xcolor` séries (`\definecolorseries`)

### Linhas, réguas e decoração
- [ ] `\rule{larg}{alt}` (barra preta)
- [ ] `\hrule` · `\hrulefill` · `\dotfill` · `\leaders`
- [x] `\underline{}` · `\overline{}` (texto) · `\uline`
- [~] `\ul` · `\sout` · `\xout` · `\uwave` · `\dashuline`
- [~] `\cancel{}` `\bcancel{}` `\xcancel{}` `\cancelto{}{}` (`cancel`, em texto/fórmula)
- [ ] `\fbox` · `\framebox` (ver Caixas)
- [~] `\hl{}` · `\hlc[cor]{}` (realce com cor) · `\mbox{\colorbox}`
- [ ] `\textsuperscript{}` · `\textsubscript{}` (marca d'água)
- [ ] `background`/`eso-pic`/`draftwatermark`: `\SetWatermarkText{RASCUNHO}`
- [x] `fancyhdr` + `\fancypagestyle` borda ou linha decorativa (`\headrulewidth`)
- [~] `pagecolor` · `\pagecolor` · `afterpage` + `\pagecolor`
- [ ] `\pgfdeclareverticalshading` · `tikz` sombreamento (fundo degradê)
- [~] `ornate`/`fancyvrb` bordas de página: `\usepackage{pgfpages}` `eso-pic` + `tikz` moldura da página
- [~] `\pagecolor` com `\afterpage{}` · `\thispagestyle{empty}` em capa colorida

## 11. Macros, contadores e programação (16 / 54)

### Definição de comandos
- [x] `\newcommand{\cmd}{texto}`
- [~] `\newcommand{\cmd}[n]{… #1 …}`
- [~] `\newcommand{\cmd}[n][padrão]{…}` (argumento opcional)
- [x] `\newcommand*` (versão curta, sem `\par`)
- [x] `\renewcommand{\cmd}{…}`
- [x] `\providecommand{\cmd}{…}`
- [x] `\DeclareRobustCommand`
- [~] `\newenvironment{env}{início}{fim}`
- [~] `\newenvironment{env}[n][padrão]` e `\renewenvironment`
- [~] `\def\cmd#1#2{…}` (primitiva TeX)
- [~] `\def` com delimitadores (`\def\a#1.#2{}`) · `\edef` · `\gdef` · `\xdef`
- [~] `\let\novo\velho`
- [~] `\expandafter` · `\csname…\endcsname` · `\noexpand` · `\the` · `\string` · `\meaning`
- [x] `\@namedef` · `\@nameuse` · `\makeatletter`/`\makeatother` · comandos `\@x`
- [ ] Macro que chama macro (recursão limitada, `#` duplo `##`)
- [~] Macro com `\xspace` (`xspace`)
- [ ] `\NewDocumentCommand{\cmd}{m o}{…}` (xparse)
- [ ] `\NewDocumentEnvironment` · `\RenewDocumentCommand` · `\ProvideDocumentCommand` · `\DeclareDocumentCommand` · tipos `s m o O{} d<> v`
- [ ] `\NewExpandableDocumentCommand` · `\IfBooleanTF` · `\IfValueTF` · `\IfNoValueTF` · `\IfBlankTF`
- [~] `\newrobustcmd` (etoolbox) · `\robustify` · `\preto` `\apptocmd` `\patchcmd` `\pretocmd` `\csdef` `\csuse` `\ifdef` `\ifundef` `\ifstrequal` `\ifblank`
- [ ] `\ifthenelse{cond}{sim}{não}` (ifthen)
- [ ] `\ifthenelse` com `\equal{}{}` `\isodd{}` `\lengthtest{}` `\boolean{}` `\and` `\or` `\not`
- [x] `\newboolean{b}` · `\setboolean{b}{true}` · `\provideboolean`
- [ ] `\newif\ifx` · `\iftrue \iffalse` · `\ifx` · `\ifnum` · `\ifdim` · `\ifodd` · `\else` · `\or` · `\fi`
- [ ] `\ifdefined` · `\ifcsname` · `\ifcat` · `\ifvoid` · `\ifhmode` · `\ifvmode` · `\ifmmode`
- [x] `\@ifstar` · `\@ifnextchar` · `\@ifundefined` · `\@ifpackageloaded` · `\IfFileExists` · `\@ifclassloaded`
- [x] `\@gobble` · `\@firstofone` · `\@firstoftwo` · `\@secondoftwo` · `\@empty` · `\@tempa`
- [x] `\ifpdf` · `\ifxetex` · `\ifluatex` (`ifpdf`/`iftex`)
- [ ] `\pdfstrcmp` · `\strcmp` · `\IfStrEq` (xstring) `\IfSubStr` `\StrLen` `\StrLeft` `\StrBehind` `\StrSubstitute`
- [~] `\fpeval{}` · `\numexpr` · `\dimexpr` · `\FPadd` (`fp`, `xfp`, `pgfmath`: `\pgfmathparse`)
- [~] `\count` · `\dimen` · `\skip` · `\toks` · `\box` (registradores TeX) · `\newcount` `\newdimen` `\newskip` `\newtoks`
- [x] `\loop … \repeat` · `\whiledo` · `\dountil` · `\@whilenum`
- [ ] `\foreach` (pgffor) · `\forloop{}` (forloop) · `\forcsvlist` · `\docsvlist` (etoolbox) · `\dolistloop` · `\forlistloop`
- [~] `\newcommand` com `\ifx#1\empty` testes · `\ifstrempty` · `\ifdefempty`
- [x] `\input{}` de arquivo de macros (`macros.tex`, `.sty` próprio, `\usepackage{meu}`)
- [~] `\ProvidesPackage` · `\ProvidesClass` · `\LoadClass` · `\DeclareOption` · `\ProcessOptions` · `\NeedsTeXFormat` · `\ExecuteOptions` (criar pacote/classe)
- [x] `\newtheorem` · `\theoremstyle` (ver amsthm)
- [~] `expl3`: `\ExplSyntaxOn` · `\cs_new:Npn` · `\tl_set:Nn` · `\int_eval:n` · `\seq_` · `\prop_` · `\clist_` · `\str_` · `\regex_`
- [x] `Lua`: `\directlua{}` · `\luaexec` · `luacode` (LuaLaTeX)
- [~] `\write18` · `\immediate\write18{}` · `\input{|"cmd"}` (shell escape) · `\ShellEscape`
- [~] `\catcode` · `\lccode` · `\uccode` · `\mathcode` · `\sfcode` (códigos de categoria)
- [~] `\afterassignment` · `\aftergroup` · `\begingroup`/`\endgroup` · `\bgroup`/`\egroup` · `\global` · `\long` · `\outer` · `\protected`
- [~] `\csname` · `\string` · `\detokenize` · `\unexpanded` · `\scantokens` · `\everypar` · `\everymath` · `\everydisplay` · `\everyhbox` · `\everyjob`

### Contadores
- [~] `\newcounter{c}` · `\newcounter{c}[pai]` · `\setcounter{c}{n}`
- [~] `\addtocounter{c}{n}` · `\stepcounter{c}` · `\refstepcounter{c}`
- [ ] `\arabic{c}` · `\roman{c}` · `\Roman{c}` · `\alph{c}` · `\Alph{c}` · `\fnsymbol{c}` · `\value{c}`
- [~] `\thec` (apresentação do contador) · `\renewcommand{\thesection}{\Roman{section}}`
- [x] `\numberwithin{equation}{section}` (amsmath) · `\counterwithin{c}{pai}` · `\counterwithout` (chngcntr)
- [~] `\setcounter{section}{n}` · `\setcounter{chapter}{n}` · `\setcounter{page}{n}` · `\setcounter{equation}{n}` · `\setcounter{footnote}{n}`
- [x] `\setcounter{secnumdepth}` · `\setcounter{tocdepth}` · `\setcounter{totalnumber}`
- [~] `\the\value` · `\number` · `\romannumeral` · `\ifnum\value{c}>3`
- [~] `\addtocontents{lof}{}` · `\addcontentsline{lot}{table}{}` · `\newcounter{contador}[section]` zera sozinho
- [~] Contador `enumi` `enumii` `enumiii` `enumiv` · `page` · `equation` · `figure` · `table` · `footnote` · `mpfootnote` · `part` · `chapter` · `section` … `subparagraph`
- [~] `\totalcount` · `\lastpage` · `\pageref{LastPage}` (lastpage) · `zref-totpages`

## 12. Matemática (LaTeX, amsmath, amssymb, mathtools) (117 / 131)
A fórmula é composta pelo motor `featheroffice/crates/feather-math` (algoritmo do TeX, fonte Latin Modern Math com a tabela
OpenType MATH). Critério `math`: o leitor reconhece a fórmula **e** o `feather-math` a monta sem comando desconhecido e sem
caractere que falte na fonte (`feather_math::desconhecidos`); se sobra algum, `[~]` com o nome dele. Desenho conferido por foto.

### Modos e delimitação
- [x] `$…$` (fórmula na linha)
- [x] `\(…\)`
- [x] `$$…$$` (fórmula em destaque, TeX)
- [x] `\[…\]`
- [x] `equation` (numerada)
- [x] `equation*`
- [x] `displaymath` · `math` (ambientes)
- [x] `eqnarray` · `eqnarray*` (antigo)
- [x] `\displaystyle` · `\textstyle` · `\scriptstyle` · `\scriptscriptstyle`
- [x] `\text{}` · `\textrm{}` · `\mbox{}` · `\intertext{}` (texto dentro da fórmula)
- [x] `\ensuremath{}` (comando que funciona em texto e fórmula)
- [~] `\everymath{}` · `\everydisplay{}` · `\abovedisplayskip` · `\belowdisplayskip` · `\abovedisplayshortskip` · `\jot`
- [x] `\(` com `\label` e referência (`\eqref`)
- [x] `\ifmmode` · `\mathchoice` · `\mathsurround`

### Índices, expoentes, frações e raízes
- [x] Expoente `x^2` e índice `x_i` · `x_i^2` · `x^{a^b}` · `x_{i_j}`
- [x] Apóstrofo (derivada) `f'` `f''` e `x^\prime` · `\prime`
- [x] `\frac{a}{b}` · `\dfrac` · `\tfrac` · `\cfrac` (fração contínua)
- [x] `\binom` · `\dbinom` · `\tbinom` · `\genfrac{}{}{}{}{}{}` · `\choose` · `\over` · `\atop`
- [x] `\sqrt{x}` · `\sqrt[n]{x}` · `\surd`
- [x] `\overset{a}{b}` · `\underset{a}{b}` · `\stackrel{a}{b}` · `\substack{a\\b}` · `\overbrace` · `\underbrace` com legenda `^{}` `_{}`
- [x] `\xrightarrow[sob]{sobre}` · `\xleftarrow` · `\xlongequal` · `\xRightarrow` (mathtools) · `\xhookrightarrow`
- [x] `\sideset{_a^b}{_c^d}\sum` · `\tensor` · `\prescript{a}{b}{X}` (mathtools/tensor)
- [x] `\mathclap` · `\mathllap` · `\mathrlap` · `\smashoperator` (mathtools)

### Letras gregas e variantes
- [x] Gregas minúsculas: `\alpha \beta \gamma \delta \epsilon \varepsilon \zeta \eta \theta \vartheta \iota \kappa \lambda \mu \nu \xi \pi \varpi \rho \varrho \sigma \varsigma \tau \upsilon \phi \varphi \chi \psi \omega`
- [x] Gregas maiúsculas: `\Gamma \Delta \Theta \Lambda \Xi \Pi \Sigma \Upsilon \Phi \Psi \Omega`
- [x] Gregas maiúsculas inclinadas (amsmath): `\varGamma \varDelta \varTheta \varLambda \varXi \varPi \varSigma \varUpsilon \varPhi \varPsi \varOmega`
- [~] Gregas extras (amssymb): `\digamma \varkappa \backepsilon` · hebraicas `\aleph \beth \gimel \daleth`
- [x] Gregas em negrito `\boldsymbol{\alpha}` · `\bm{\alpha}` (bm) · `\pmb`
- [x] Letras diversas: `\ell \hbar \imath \jmath \wp \Re \Im \mho \eth \Bbbk \complement \partial \nabla \infty \emptyset \varnothing \angle \measuredangle \sphericalangle`

### Alfabetos de fórmula
- [x] `\mathrm{}` · `\mathbf{}` · `\mathit{}` · `\mathsf{}` · `\mathtt{}` · `\mathnormal{}`
- [x] `\mathbb{R}` (quadro-negro, amssymb) · `\mathbbm` · `\mathds`
- [x] `\mathcal{L}` (caligráfico) · `\mathscr{L}` (mathrsfs) · `\mathfrak{g}` (eufrak)
- [~] `\mathbf` + `\boldsymbol` + `\bm` + `\mathversion{bold}` + `\boldmath`
- [x] `\operatorname{}` · `\operatorname*{}` · `\DeclareMathOperator{\sen}{sen}` · `\DeclareMathOperator*` · `\mathop`
- [x] `\mathord` · `\mathbin` · `\mathrel` · `\mathopen` · `\mathclose` · `\mathpunct` · `\mathinner` (classes)
- [x] `\DeclareMathAlphabet` · `\SetMathAlphabet` · `\setmathfont` (unicode-math)

### Funções e operadores nomeados
- [x] `\sin \cos \tan \cot \sec \csc \arcsin \arccos \arctan \sinh \cosh \tanh \coth`
- [x] `\log \ln \lg \exp \deg \det \dim \gcd \hom \ker \arg \Pr`
- [x] `\lim \limsup \liminf \max \min \sup \inf \varlimsup \varliminf \varinjlim \varprojlim` com limites embaixo
- [x] `\bmod` · `\pmod{n}` · `\mod` · `\pod`
- [x] `\mathop{\rm sen}` · `\mathrm{sen}` (função em português: sen, tg, cossec)

### Operadores grandes
- [x] `\sum` `\prod` `\coprod` com limites `_{}` `^{}` · `\limits` · `\nolimits`
- [x] `\int` `\iint` `\iiint` `\iiiint` `\idotsint` `\oint` `\oiint` `\oiiint` `\intop`
- [x] `\bigcup` `\bigcap` `\bigsqcup` `\bigvee` `\bigwedge` `\bigoplus` `\bigotimes` `\bigodot` `\biguplus`
- [x] `\smallint` · `\smallsum` · `\sumop` · `\intertext` · `\substack` · `\sideset` (limites especiais)
- [x] Limite em baixo vs. ao lado (`\displaystyle` · `\limits` · `\nolimits` · `\intlimits`)

### Relações
- [x] Igualdade e ordem: `= \ne \neq \equiv \not\equiv \sim \simeq \approx \cong \propto \doteq \asymp \le \leq \ge \geq \ll \gg \lll \ggg \leqslant \geqslant \lesssim \gtrsim \lessgtr \prec \succ \preceq \succeq`
- [x] Conjuntos: `\in \notin \ni \subset \supset \subseteq \supseteq \subsetneq \supsetneq \nsubseteq \sqsubset \sqsupset \sqsubseteq \sqsupseteq`
- [~] Perpendicular e outras: `\perp \parallel \nparallel \mid \nmid \vdash \dashv \models \vDash \bowtie \smile \frown \between \pitchfork`
- [x] Negação: `\not=` · `\not\in` · `\nless \ngtr \nleq \ngeq \nsim \ncong \nprec \nsucc \nvdash \nvDash`
- [x] `\because \therefore \triangleq \eqcirc \circeq \doteqdot \fallingdotseq \risingdotseq \bumpeq \Bumpeq \eqsim \backsim \thicksim \thickapprox`
- [x] `\coloneqq` · `\eqqcolon` · `\Coloneqq` · `\vcentcolon` (mathtools) · `\dblcolon`

### Operadores binários
- [x] `+ - \pm \mp \times \div \cdot \ast \star \circ \bullet \oplus \ominus \otimes \oslash \odot \setminus \cup \cap \sqcup \sqcap \vee \wedge \lor \land \uplus \amalg \wr \diamond \triangle \bigtriangleup \bigtriangledown`
- [x] amssymb binários: `\dotplus \ltimes \rtimes \leftthreetimes \rightthreetimes \barwedge \veebar \doublebarwedge \boxplus \boxminus \boxtimes \boxdot \circledast \circledcirc \circleddash \centerdot \intercal \divideontimes \smallsetminus \curlyvee \curlywedge \Cup \Cap`

### Setas
- [x] Simples: `\leftarrow \rightarrow \to \gets \leftrightarrow \uparrow \downarrow \updownarrow \nearrow \searrow \swarrow \nwarrow \mapsto \longmapsto \hookrightarrow \hookleftarrow \longrightarrow \longleftarrow \longleftrightarrow`
- [x] Duplas: `\Leftarrow \Rightarrow \Leftrightarrow \Uparrow \Downarrow \Updownarrow \Longrightarrow \Longleftarrow \Longleftrightarrow \implies \impliedby \iff`
- [x] amssymb: `\leadsto \rightsquigarrow \leftrightsquigarrow \twoheadrightarrow \twoheadleftarrow \rightarrowtail \leftarrowtail \rightleftarrows \leftrightarrows \rightrightarrows \leftleftarrows \upuparrows \downdownarrows \rightleftharpoons \leftrightharpoons \curvearrowright \curvearrowleft \circlearrowright \circlearrowleft \Rrightarrow \Lleftarrow \nrightarrow \nleftarrow \nRightarrow \nLeftarrow \nleftrightarrow \Lsh \Rsh \looparrowright \looparrowleft \multimap \dashrightarrow \dashleftarrow`
- [x] Arpões: `\leftharpoonup \leftharpoondown \rightharpoonup \rightharpoondown \upharpoonleft \upharpoonright \downharpoonleft \downharpoonright` · `\overleftarrow` · `\overrightarrow` · `\overleftrightarrow` · `\underrightarrow`
- [x] Extensíveis: `\xrightarrow` `\xleftarrow` `\xleftrightarrow` `\xmapsto` `\xlongequal` `\xtwoheadrightarrow` com texto em cima e embaixo
- [x] Reações e equilíbrio: `\rightleftharpoons` · `\ce{<=>}` (mhchem) · `\xrightleftharpoons`

### Delimitadores
- [x] Parênteses, colchetes, chaves: `( ) [ ] \{ \} \lbrace \rbrace \langle \rangle \lvert \rvert \lVert \rVert \lfloor \rfloor \lceil \rceil \lbrack \rbrack \vert \Vert | \|`
- [x] `\left … \right` (delimitador que cresce) · `\left.` (invisível) · `\middle`
- [x] Tamanhos manuais: `\big \Big \bigg \Bigg` · `\bigl \bigr \Bigl \Bigr \biggl \biggr \Biggl \Biggr` · `\bigm`
- [x] Delimitadores pareados (mathtools): `\DeclarePairedDelimiter{\abs}{\lvert}{\rvert}` · `\abs*{}` · `\abs[\big]{}`
- [~] Delimitadores extensos: `\lgroup \rgroup \lmoustache \rmoustache \arrowvert \Arrowvert \bracevert \ulcorner \urcorner \llcorner \lrcorner`
- [x] `\delimiter` · `\bigg\{` · `\left\{ … \right.` (casos manuais) · `\left\langle … \right\rangle` · `\langle x,y \rangle`

### Acentos e decoração
- [x] Acentos curtos: `\hat \check \tilde \acute \grave \dot \ddot \dddot \ddddot \breve \bar \vec \mathring`
- [x] Acentos largos: `\widehat \widetilde \overline \underline \overbrace \underbrace \overrightarrow \overleftarrow \underleftarrow \overleftrightarrow`
- [x] `\not` · `\cancel` · `\bcancel` · `\xcancel` · `\cancelto` · `\boxed{}` · `\fbox` em fórmula · `\colorbox` em fórmula
- [x] `\underbrace{…}_{texto}` · `\overbrace{…}^{texto}` · `\underbracket` · `\overbracket` (mathtools)
- [x] `\mathstrut` · `\vphantom` · `\hphantom` · `\phantom` · `\smash` · `\mathllap`

### Reticências e pontos
- [x] `\ldots \dots \cdots \vdots \ddots \iddots \dotsb \dotsc \dotsm \dotsi \dotso \hdotsfor{n}`
- [x] `\cdot \cdotp \centerdot \colon \ldotp \mathellipsis` · `\: \,` · `:` (relação vs. pontuação)

### Símbolos diversos
- [x] Lógica e conjuntos: `\forall \exists \nexists \neg \lnot \top \bot \emptyset \varnothing \in \complement \setminus \cup \cap \mathbb{N} \mathbb{Z} \mathbb{Q} \mathbb{R} \mathbb{C}`
- [x] Cálculo: `\partial \nabla \infty \prime \Delta \int \oint \sum \prod \lim \sqrt \Re \Im \hbar`
- [x] Geometria: `\angle \measuredangle \sphericalangle \triangle \square \blacksquare \Box \Diamond \blacktriangle \blacktriangledown \blacklozenge \lozenge \bigstar \circ \bigcirc \diamond \parallel \perp`
- [~] Cartas e outros: `\spadesuit \heartsuit \diamondsuit \clubsuit \flat \natural \sharp \checkmark \maltese \yen \circledR \circledS \S \P \dag \ddag`
- [x] Teoria dos números: `\mid \nmid \equiv \pmod \gcd \lfloor \rfloor \lceil \rceil \left\lfloor \right\rfloor \sum_{d|n}`
- [x] Unidades em fórmula: `\,\mathrm{m/s}` · `\mathrm{kg}` · `\si{}` (siunitx) · `\text{ m}` · `\qty{}{}`
- [x] Texto em fórmula: `\text{...}` · `\textnormal{}` · `\mbox{}` · `\intertext{}` · `\shortintertext{}`
- [x] Espaços matemáticos: `\, \: \; \! \ \quad \qquad \enspace \hspace{} \mkern \mskip \negthinspace \negmedspace \negthickspace \thinspace \medspace \thickspace`
- [x] Barra de fração e traço: `\vline \hline \vert \| \Vert` em matriz · `\rule` em fórmula · `\dfrac{\,}{\,}`
- [x] Constantes de uso comum com `\mathrm{e}` `\mathrm{i}` `\mathrm{d}x` · diferencial `\,\mathrm{d}x` · `\partial_t` · `\frac{\partial f}{\partial x}`
- [x] Unicode em fórmula (`α β ≤ ∑ ∫ →` direto, `unicode-math`)

### Matrizes e arranjos
- [x] `matrix` (sem delimitador)
- [x] `pmatrix` (parênteses)
- [x] `bmatrix` (colchetes) · `Bmatrix` (chaves)
- [x] `vmatrix` (barras) · `Vmatrix` (barras duplas)
- [x] `smallmatrix` (matriz pequena na linha) · `psmallmatrix` · `bsmallmatrix` (mathtools)
- [x] `array` com `{ccc}` e linhas `\hline` · `|c|` · `@{}` · `\\[2pt]` · `*{3}{c}`
- [x] `cases` (sistema/por partes) · `dcases` · `rcases` · `drcases` · `cases*` (mathtools)
- [x] `matrix*` · `pmatrix*` · `bmatrix*` com alinhamento `[r]` (mathtools)
- [x] `\hdotsfor{n}` · `\vdots` · `\ddots` dentro de matriz · `\cdots`
- [x] `\substack` e `subarray` (condições empilhadas em `\sum`)
- [~] `nicematrix`: `pNiceMatrix` · `\Block` · `\hline` · `\CodeBefore` · `\Vdots` (matriz com anotações)
- [~] `blkarray` / `\begin{blockarray}` (rótulos de linha e coluna) · `\matrix`/`\bordermatrix` (plain TeX)
- [x] Determinantes e sistemas de equações com `aligned` dentro de `\left\{ … \right.`

### Equações em várias linhas (amsmath)
- [x] `align` (alinha em `&`, numera cada linha)
- [x] `align*`
- [x] `alignat` · `alignat*` (espaço entre colunas controlado)
- [x] `flalign` · `flalign*` (alinha com as margens)
- [x] `gather` · `gather*` (centraliza cada linha)
- [x] `multline` · `multline*` (equação longa em várias linhas) · `\shoveleft` · `\shoveright`
- [x] `split` (dentro de `equation`, uma só numeração)
- [x] `aligned` · `gathered` · `alignedat` (dentro de outras fórmulas, com `[t]` `[b]`)
- [x] `subequations` (numeração 1a, 1b…) · `parentequation`
- [x] `\tag{}` · `\tag*{}` · `\notag` · `\nonumber` (numeração manual e supressão)
- [~] `\label{}` + `\eqref{}` · `\ref{}` · `\autoref{}` · `\cref{}` em equação · `\numberwithin{equation}{section}`
- [x] `\intertext{}` · `\shortintertext{}` entre linhas · `\displaybreak` · `\allowdisplaybreaks` · `\\*`
- [~] `\MoveEqLeft` · `\mathllap` · `\mathrlap` · `\ArrowBetweenLines` · `\SwapAboveDisplaySkip` (mathtools)
- [x] `\begin{equation}…\end{equation}` com `\left\{ \begin{array}…` e `\hfill`
- [x] `dmath` (breqn: quebra automática) · `dgroup` · `dseries`
- [x] `\begin{eqnarray}` com `\lefteqn{}` e `&=&` (antigo)
- [x] `empheq` (`\begin{empheq}[box=\fbox]{align}`) · `\boxed` em linha do `align`
- [x] `cases` com `&` e `\text{se}` · numeração de `cases` com `numcases` · `\begin{cases}` aninhado
- [x] Equações numeradas dentro de lista, tabela e `minipage`
- [x] Fórmula dentro de título, legenda, nota e célula de tabela
- [x] Fórmula inline com quebra de linha permitida (`\allowbreak` · `\linebreak` · `\mathbin{}`)
- [x] Fonte e tamanho da fórmula acompanham o corpo do texto (inline, display, script) · `\small` em fórmula · `\mathsurround`

### Símbolos de pacotes de ciência
- [~] `siunitx`: `\SI{valor}{unidade}` · `\si{}` · `\num{}` · `\qty{}{}` · `\unit{}` · `\ang{}` · `\numlist` · `\numrange` · `\SIrange` · `\sisetup{}`
- [~] `siunitx`: `\ohm` `\celsius` `\micro` `\degree` `\kilo` `\milli` `\mega` `\giga` `\per` `\squared` `\cubed` `\tothe{}` `\DeclareSIUnit`
- [x] `physics`: `\abs{}` `\norm{}` `\order{}` `\ket{}` `\bra{}` `\braket{}{}` `\dd` `\dv{}{}` `\pdv{}{}` `\grad` `\div` `\curl` `\laplacian` `\qty()` `\mqty()` `\Tr` `\rank` `\expval{}` `\comm{}{}`
- [x] `braket`: `\bra{}` `\ket{}` `\braket{}{}` `\Set{}` `\Bra{}` `\Ket{}`
- [x] `mhchem`: `\ce{H2O}` · `\ce{A + B -> C}` · `\ce{^{14}C}` · `\ce{Fe^{3+}}` · `\ce{<=>}` · `\cee{}`
- [x] `chemformula`: `\ch{H2O}` · `\chlabel` · `\chlewis` (alternativa ao mhchem)
- [ ] `chemfig`: `\chemfig{*6(-=-=-=)}` (estruturas moleculares, TikZ)
- [x] `tensor`: `\tensor{T}{^a_b}` · `\tensor*` · `\indices` · `\Tensor`
- [x] `cancel`: `\cancel{}` `\bcancel{}` `\xcancel{}` `\cancelto{}{}` `\CancelColor`
- [x] `bm`: `\bm{x}` · `\bm{\alpha}` · `\bmdefine` · `\boldsymbol` (negrito em fórmula)
- [~] `esint`: `\oiint` `\iint` `\ointctrclockwise` `\varoint` `\sqint` `\fint` (integrais especiais)
- [x] `mathrsfs`: `\mathscr{}` · `eucal` · `euscript` · `calrsfs` · `\mathbbm` (bbm) · `dsfont` `\mathds`

## 13. Pacotes — texto, página e cabeçalhos (30 / 100)

### amsmath, amssymb, mathtools (opções e comandos fora de fórmula)
- [~] `amsmath`: opções `leqno` `fleqn` `intlimits` `sumlimits` `namelimits` `nointlimits` `tbtags` `centertags`
- [x] `amsmath`: `\numberwithin{equation}{section}`
- [~] `amsmath`: `\text{}` e `\tag{}` em `equation` · `\eqref` · `\boldsymbol` · `\binom` · `\operatorname`
- [x] `amsmath`: `\allowdisplaybreaks[1]`
- [x] `amsmath`: `\DeclareMathOperator` · `\DeclareMathSymbol` · `\mathbf`
- [~] `amssymb` (inclui `amsfonts`): `\mathbb` `\mathfrak` `\square` `\blacksquare` `\checkmark` `\therefore` `\because`
- [~] `mathtools`: `\mathtoolsset{showonlyrefs}` · `\coloneqq` · `\DeclarePairedDelimiter` · `\prescript` · `\xrightarrow` · `\MoveEqLeft` · `\shortintertext` · `multlined` · `lgathered` · `rgathered`
- [x] `mathtools`: `\begin{multlined}[b][5cm]`

### amsthm — teoremas
- [~] `\newtheorem{teo}{Teorema}` + `\begin{teo}…\end{teo}`
- [~] `\newtheorem{lema}[teo]{Lema}` (contador compartilhado)
- [~] `\newtheorem{def}{Definição}[section]` (numera por seção)
- [~] `\newtheorem*{obs}{Observação}` (sem número)
- [x] `\begin{teo}[Nome]` (título entre colchetes)
- [ ] `proof` — "Demonstração." com o quadradinho final
- [~] `proof` com `[título]` (`\begin{proof}[Prova do lema]`) · `\qedhere` · `\qedsymbol` · `\renewcommand{\proofname}{Prova}`
- [~] `\theoremstyle{plain|definition|remark}`
- [ ] `\newtheoremstyle{}` (estilo próprio) · `\swapnumbers` · `\thmname` `\thmnumber` `\thmnote`
- [ ] `thmtools`: `\declaretheorem[name=]{}` · `\listoftheorems`
- [~] `ntheorem` · `aliascnt` (`\aliascntresetthe`) · `cleveref` + teoremas · `\crefname{teo}{teorema}{teoremas}`
- [ ] `tcolorbox` + teoremas (`tcbtheorem`) · `mdframed` + `\newmdtheoremenv`

### hyperref
- [~] `hyperref` (links clicáveis em sumário, `\ref`, `\cite`, URLs)
- [x] `\hypersetup{colorlinks=true,linkcolor=,urlcolor=,citecolor=}`
- [~] `\hypersetup{pdftitle=,pdfauthor=,pdfsubject=,pdfkeywords=,pdfcreator=}`
- [x] `hyperref` opções de carregamento: `[hidelinks]` `[colorlinks]` `[breaklinks]` `[bookmarks]` `[bookmarksnumbered]` `[pdfborder=0 0 0]` `[unicode]` `[pdfstartview=FitH]` `[linktoc=all]`
- [x] `\href{url}{texto}`
- [x] `\url{}` / `\nolinkurl{}` / `\path{}`
- [~] `\hyperref[rótulo]{texto}`
- [ ] `\hypertarget{}{}` · `\hyperlink{}{}`
- [~] `\autoref{}` · `\nameref{}` · `\pageref{}` · `\autopageref` · `\Autoref` · `\figureautorefname` `\sectionautorefname` (nomes em pt-BR)
- [x] `\texorpdfstring{}{}` · `\pdfstringdefDisableCommands` · `\phantomsection`
- [x] `\pdfbookmark[nível]{texto}{nome}` · `\currentpdfbookmark` · `\belowpdfbookmark`
- [x] `\hypersetup{allcolors=}` · `\urlstyle{same}` · `\hypersetup{bookmarksopen}`
- [x] `\href{run:arquivo}{}` · `\href{mailto:}{}` · `\href{tel:}{}` · anexos `\attachfile` (`attachfile`)
- [~] `\Acrobatmenu` · campos de formulário `\TextField` `\CheckBox` `\ChoiceMenu` `\PushButton` `\begin{Form}`
- [x] `hyperxmp` · `bookmark` (`\bookmarksetup`) · `pdfx` (PDF/A) · `\hypersetup{pdfa}`
- [~] `url`: `\url{}` · `\urlstyle{rm}` · `\UrlBreaks` · `\Urlmuskip` · `\path{}`

### babel e idioma
- [x] `\usepackage[brazil]{babel}` (português do Brasil: "Sumário", "Figura", "Referências", data)
- [~] `\usepackage[portuguese]{babel}` · `[brazilian]` · `[english]` · `[spanish]` · `[french]` · `[german]` · `[main=brazil]`
- [ ] `\selectlanguage{english}` · `\begin{otherlanguage}{}` · `\foreignlanguage{}{}` · `\babeltags` · `\babelprovide`
- [ ] `\languagename` · `\iflanguage{}{}{}` · `\captionsbrazil` · `\datebrazil` · `\brazilhyphenmins` · `\extrasbrazil`
- [x] `babel`: `\shorthandon{"}` · `"-` `"|` `"~` `"=` atalhos de hifenização · `\languageshorthands` · `\defineshorthand`
- [~] `babel`: textos fixos (`\contentsname`, `\listfigurename`, `\listtablename`, `\bibname`, `\refname`, `\abstractname`, `\figurename`, `\tablename`, `\appendixname`, `\partname`, `\chaptername`, `\indexname`) em pt-BR
- [~] `polyglossia` (XeLaTeX): `\setmainlanguage{portuguese}` · `\setotherlanguage{}` · `\textenglish{}`
- [x] `inputenc`: `[utf8]` `[latin1]` `[ansinew]` `\inputencoding` · UTF-8 nativo no LuaLaTeX/XeLaTeX
- [x] `fontenc`: `[T1]` `[OT1]` `[T2A]` `[LGR]` `[EU1]` `[EU2]` · `\DeclareFontEncoding`
- [~] `lmodern` · `cmbright` · `gensymb` · `textcomp` · `upgreek` (`\upalpha`) · `lmodern` + `T1`
- [x] `iftex` · `ifxetex` · `ifluatex` · `ifpdf`

### microtype, fontspec e fontes
- [x] `microtype`: `\microtypesetup{protrusion=true,expansion=true,tracking=true,kerning=true,spacing=true}` · `\textls{}` (espaçamento entre letras) · `\lsstyle`
- [~] `microtype`: `\textls[50]{ZQX}`
- [~] `fontspec`: `\setmainfont{Nome}[Ligatures=TeX]` · `\setsansfont` · `\setmonofont` · `\newfontfamily\x{Nome}` · `\fontspec{}` · `\defaultfontfeatures`
- [~] `fontspec`: `\addfontfeatures{}` · `Scale=` · `Numbers=OldStyle` · `SmallCapsFeatures` · `UprightFont=` · `BoldFont=` · `ItalicFont=` · `Path=`
- [x] `lmodern` (padrão do Overleaf)
- [~] `times` · `mathptmx` · `newtxtext,newtxmath` (Times)
- [~] `helvet` · `[scaled=0.92]` · `\sfdefault` · `\familydefault`
- [~] `palatino` · `mathpazo` · `bookman` · `newcent` · `charter` · `utopia` · `fourier` · `libertine` · `kpfonts` · `gentium` · `tgpagella` · `tgtermes` · `roboto` · `inter` · `sourcesanspro` · `opensans` · `fira`
- [~] `DejaVu` · `FiraMono` · `inconsolata` · `courier` · `beramono` · `txtt` (monoespaçadas)

### fancyhdr — cabeçalho e rodapé
- [~] `\pagestyle{fancy}`
- [~] `\fancyhead[L|C|R]{texto}` · `\lhead` `\chead` `\rhead`
- [~] `\fancyfoot[L|C|R]{texto}` · `\lfoot` `\cfoot` `\rfoot`
- [~] `\fancyhf{}` (limpa tudo) · `\fancyhead[LE,RO]{}` (páginas par/ímpar)
- [~] `\thepage` · `\leftmark` · `\rightmark` · `\nouppercase` · `\thesection` dentro de cabeçalho
- [~] `\headrulewidth` · `\footrulewidth` (réguas) · `\headrule` · `\footrule`
- [~] `\fancypagestyle{nome}{…}` · `\thispagestyle{nome}` · redefinir `plain`
- [~] `\headheight` · `\setlength{\headheight}{}` · `\addtolength{\headheight}{}` · `\fancyheadoffset` · `\fancyfootoffset`
- [~] `\markboth` · `\markright` · `\chaptermark` · `\sectionmark` · `\renewcommand{\sectionmark}[1]{\markright{#1}}`
- [~] `fancyhdr` + `lastpage`: "Página \thepage\ de \pageref{LastPage}"
- [~] `fancyhdr` + `\includegraphics` no cabeçalho (logotipo)
- [~] `scrlayer-scrpage` (KOMA): `\ihead` `\chead` `\ohead` `\ifoot` `\cfoot` `\ofoot` · `\pagestyle{scrheadings}`
- [~] `lastpage`: `\pageref{LastPage}` · `\pageref*{LastPage}` · `zref-totpages` `\ztotpages`
- [~] `\pagenumbering{roman}` com `fancyhdr` · `\pagenumbering{gobble}` (sem número)
- [~] `\fancyhead` com `\hfill` e régua com `\rule` · cabeçalho em duas linhas com `\\` · `\headrulewidth` colorida (`\renewcommand{\headrule}`)

### titlesec — títulos de seção
- [x] `\titleformat{\section}{fonte}{rótulo}{sep}{antes}` · `\titleformat{\chapter}[display]{…}`
- [x] `\titlespacing{\section}{esq}{antes}{depois}` · `\titlespacing*`
- [~] `\titleformat` com `\color` · `\titlerule` · `\hrule` · `\filright` `\filcenter` `\filleft` (alinhamento do título)
- [~] `\titleclass` · `\titleformat{name=\section,numberless}` · `\assignpagestyle` · `\newpagestyle` · `\sethead` · `\setfoot`
- [~] `sectsty`: `\sectionfont{\color{red}}` · `\allsectionsfont{\sffamily}` · `\chapterfont`
- [x] `titletoc`: `\titlecontents{section}[1.5em]{}{}{}{\titlerule*[.5pc]{.}\contentspage}`
- [~] `\@startsection` · `\renewcommand\section{\@startsection…}` (redefinição manual)
- [~] `secdot` · `\renewcommand{\thesection}{\arabic{section}.}` (ponto depois do número)
- [x] `\section` com `\MakeUppercase` (ABNT: seção primária em caixa alta)

### tocloft — sumário e listas
- [~] `\renewcommand{\cftsecfont}{\bfseries}` · `\cftchapfont` · `\cftsubsecfont` · `\cftsecpagefont`
- [~] `\cftsetindents{section}{1em}{2em}` · `\cftsecindent` · `\cftsecnumwidth` · `\cftdotsep` · `\cftsecleader`
- [~] `\setlength{\cftbeforesecskip}{}` · `\cftbeforechapskip` · `\cftafterloftitleskip` · `\cftbeforetoctitleskip`
- [~] `\renewcommand{\contentsname}{Sumário}` · `\cfttoctitlefont` · `\cftaftertoctitle` · `\listfigurename` · `\renewcommand{\cftfigpresnum}{Figura }`
- [ ] `\newlistof{quadros}{loq}{Lista de Quadros}` · `\listofquadros`
- [x] `\tableofcontents` com `\setcounter{tocdepth}{3}` e `\addcontentsline` de seções sem número
- [~] `minitoc` (`\dominitoc` `\minitoc`) · `\localtableofcontents` (`etoc`) · `\etocsettocstyle`
- [x] `\tableofcontents` agrupando apêndices/anexos (`appendix`: `\appendixpage` `\addappheadtotoc`)

### parskip, indentfirst, setspace, ragged2e, microtype
- [x] `parskip`: `[parfill]` `[skip=]` `[indent=]` · `\parskip` · `\parindent`
- [~] `ragged2e`: `\RaggedRight` `\RaggedLeft` `\Centering` `\justifying` · `\setlength{\RaggedRightRightskip}{}` · `\DelayedRaggedRight`
- [~] `ifthen`: `\ifthenelse` · `\whiledo` · `\newboolean` · `\setboolean` · `\equal` · `\lengthtest` · `\isodd`
- [~] `calc`: `\setcounter{c}{2*3+1}` · `\setlength{\a}{(\textwidth-1cm)/2}` · `\widthof` · `\heightof` · `\depthof` · `\ratio` · `\real`
- [x] `lipsum`: `\lipsum` · `\lipsum[1-3]` · `\lipsum[2][1-2]` · `\setlipsum`
- [~] `blindtext`: `\blindtext` · `\Blindtext` · `\blindmathtrue` · `\blinddocument`
- [~] `appendix`: `\appendix` · `\begin{appendices}` · `\appendixpage` · `\addappheadtotoc` · `\appendixname` · `\noappendicestocpagenum`
- [x] `footmisc`: `[bottom]` `[perpage]` `[symbol]` `[para]` `[hang]` `[stable]` `[multiple]` · `\footnotelayout` · `\setfnsymbol`
- [~] `etoolbox`: `\AtEndPreamble` · `\AtBeginEnvironment{}{}` · `\AtEndEnvironment` · `\BeforeBeginEnvironment` · `\apptocmd` · `\patchcmd` · `\csdef` · `\ifdef` · `\ifdefstring` · `\forcsvlist` · `\newtoggle` `\toggletrue` `\iftoggle`
- [~] `xparse`: `\NewDocumentCommand` · `\NewDocumentEnvironment` · `\DeclareDocumentCommand` · `\IfNoValueTF` · `\IfBooleanTF` · `\IfValueT` · `\ProcessKeyOptions` · `\SplitList` · `\TrimSpaces`
- [ ] `xstring`: `\IfSubStr{}{}{}{}` · `\IfBeginWith` · `\IfEndWith` · `\IfEq` · `\IfStrEq` · `\StrBefore` · `\StrBehind` · `\StrBetween` · `\StrChar` · `\StrCut` · `\StrLen` · `\StrMid` · `\StrRemoveBraces` · `\StrSubstitute` · `\StrCount` · `\StrGobbleLeft` · `\StrLeft` · `\StrRight`
- [~] `catchfile` · `readarray` · `datatool` · `csvsimple` · `pgfplotstable` · `filecontents` (arquivo de dados/macros)

## 14. Bibliografia e citações (BibTeX, natbib, biblatex) (21 / 52)

### Ambiente thebibliography e BibTeX
- [x] `thebibliography` + `\bibitem{chave} texto` + `\cite{chave}`
- [x] `\bibitem[rótulo]{chave}` (rótulo manual)
- [~] `\bibliographystyle{plain|unsrt|alpha|abbrv|ieeetr|apalike|abnt-alf|abntex2-num|abntex2-alf}`
- [~] `\bibliography{arquivo}` (`.bib`)
- [x] `\cite{a,b}` · `\cite[p.~10]{k}` (lista e página)
- [x] `\nocite{chave}` · `\nocite{*}`
- [~] `\bibname` / `\refname` ("Referências") · `\renewcommand{\refname}{Referências}`
- [~] `\bibliographystyle` `unsrtnat` `plainnat` `abbrvnat` `apalike` `ieeetr` `IEEEtran` `siam` `acm` (estilos)
- [x] Arquivo `.bib`: `@article` `@book` `@inproceedings` `@incollection` `@phdthesis` `@mastersthesis` `@techreport` `@misc` `@online` `@manual` `@unpublished` `@proceedings` `@booklet` `@conference` `@inbook`
- [x] Arquivo `.bib`: campos `author` `title` `journal` `year` `volume` `number` `pages` `publisher` `address` `doi` `url` `urldate` `note` `edition` `editor` `booktitle` `school` `institution` `month` `isbn` `issn` `series` `chapter` `howpublished` `keywords` `abstract` `file`
- [x] `.bib`: `@string{}` · `@preamble{}` · `@comment{}` · `crossref` · `{\'a}` acentos · `{TeX}` proteção de caixa · `and others` · `author = {Last, First}` e `{First Last}`
- [ ] `bibtex`/`biber` rodando para gerar `.bbl` · ler `.bbl` pronto (`\input{arquivo.bbl}`)
- [~] `\bibliography` com várias bases (`a,b`) · `\bibdata` · `\bibstyle` · `\citation` · `.aux`
- [ ] `chapterbib`/`bibunits`/`multibib`: uma bibliografia por capítulo · `\newcites{}{}` · `\citeA{}`
- [~] `bibtopic`/`splitbib`/`bibtopicprefix` · `\begin{btSect}` · bibliografia por categoria
- [~] `\addcontentsline{toc}{section}{\refname}` · `tocbibind` (inclui bibliografia/índice no sumário)

### natbib
- [~] `natbib`: `\citet{}` (autor (ano)) · `\citep{}` ((autor, ano)) · `\citealt{}` · `\citealp{}` · `\citeauthor{}` · `\citeyear{}` · `\citeyearpar{}`
- [~] `natbib`: `\citet*{}` · `\citep*{}` (lista completa de autores) · `\Citet{}` `\Citep{}` (maiúscula) · `\citet[cap.~2]{}` · `\citep[ver][p.~3]{}`
- [~] `natbib`: `\usepackage[round,colon,authoryear,sort&compress,super,numbers]{natbib}`
- [~] `natbib`: `\bibpunct{(}{)}{;}{a}{,}{,}` · `\setcitestyle{authoryear,open={((},close={))}}`
- [~] `natbib`: `\bibsection` · `\bibhang` · `\bibsep` · `\newblock` · `\bibfont` · `\citenumfont` · `\cite*`
- [x] `natbib`: estilos `plainnat` `abbrvnat` `unsrtnat` `apalike` `agsm` `dcu` `chicago` `named` `abntex2-alf` (autor-ano completo)
- [x] `apacite` / `apalike` / `harvard` / `chicago` · `\citeNP` · `\citeA` · `\shortcite`

### biblatex
- [~] `\usepackage[backend=biber,style=…]{biblatex}`
- [x] `\addbibresource{arquivo.bib}`
- [x] `\printbibliography` · `\printbibliography[title=,heading=,type=,keyword=,category=]`
- [~] `\cite{}` · `\parencite{}` · `\textcite{}` · `\autocite{}` · `\footcite{}` · `\smartcite{}` · `\supercite{}` · `\citeauthor{}` · `\citetitle{}` · `\citeyear{}` · `\citeurl{}` · `\fullcite{}` · `\volcite{}` · `\cites{}{}` · `\parencites` · `\textcites` · `\autocites`
- [~] `\Cite` `\Parencite` `\Textcite` `\Autocite` (maiúscula inicial) · `\parencite[p.~5]{}` · `\parencite[ver][p.~5]{}` · `\nocite{*}`
- [x] `biblatex`: `style=numeric` `numeric-comp` `authoryear` `authoryear-comp` `alphabetic` `verbose` `verbose-ibid` `trad-plain` `reading` `draft`
- [x] `biblatex`: `style=abnt` (ABNT NBR 6023/10520, `biblatex-abnt`)
- [x] `biblatex`: `style=ieee` (IEEE, `biblatex-ieee`)
- [x] `biblatex`: `style=apa` (APA 7, `biblatex-apa`)
- [x] `biblatex`: `style=chicago-authordate` · `chicago-notes` (`biblatex-chicago`)
- [x] `biblatex`: `style=nature` · `science` · `phys` · `mla` · `vancouver` · `numeric-verb` · `gb7714-2015` · `oxyear`
- [~] `biblatex`: `sorting=nty|ynt|none|nyt|anyt` · `maxnames=` · `minnames=` · `maxbibnames=` · `uniquename=` · `giveninits=true` · `dashed=false` · `doi=false` · `url=false` · `isbn=false` · `backref=true` · `citestyle=` · `bibstyle=` · `sortcites` · `autocite=footnote`
- [~] `biblatex`: `\defbibheading{}{}` · `\DeclareFieldFormat` · `\renewbibmacro` · `\DefineBibliographyStrings{brazil}{}` · `\DeclareNameFormat` · `\AtEveryBibitem{}` · `\DeclareSortingTemplate`
- [~] `biblatex`: `\newrefsection` · `\begin{refsection}` · `\printbibliography[section=n]` · `\defbibfilter` · `\addtocategory` · `\DeclareBibliographyCategory` · `\bibbysection`
- [~] `biblatex`: `\mkbibquote` · `\mkbibemph` · `\mkbibbold` · `\bibnamedelima` · `\addcomma` · `\addspace` · `\adddot` · `\newunit` · `\newblock` · `\finentry` · `\printfield{}` · `\printtext{}` · `\iffieldundef{}` · `\ifentrytype{}`
- [~] `biblatex`: `\printbibheading` · `\bibbysegment` · `\printbiblist` · `\printshorthands` · `\printbibliography[nottype=online]`
- [x] `biblatex`: campos de entrada `date` `urldate` `eprint` `eprinttype` `langid` `shorthand` `related` `entrysubtype` `pubstate` `origdate` `location` `organization` `usera`-`userf` · tipos `@online` `@thesis` `@report` `@software` `@dataset` `@patent` `@legislation` `@jurisdiction` `@video` `@audio` `@image` `@standard`
- [~] `biblatex`: `\autocite` nota de rodapé (ABNT/chicago-notes: `ibid.` `op. cit.`) · `\ibidtracker` · `\iffootnote`
- [x] `biblatex-abnt`: `\citeonline{}` · `\citeauthoronline{}` · `\apud{}` · `\cite[p. 4]{}` · `[abnt-emphasize=bf]` · `[abnt-repeated-author-omit]` · `[abnt-url-accessdate]` · `[abnt-doi=true]` · `[abnt-etal-cite=3]`
- [~] `biblatex`: `biber` (backend; `.bcf` → `.bbl`) e `bibtex8`/`bibtex` (backends antigos) · `\usepackage[backend=bibtex]{biblatex}`
- [x] `biblatex`: `\usepackage{csquotes}` junto · `\DeclareQuoteStyle` · `\MakeOuterQuote{"}`
- [x] `biblatex`: lista de abreviaturas `\printglossary` / `\nocite{*}` + `\printbibliography[heading=subbibliography]`
- [~] `abntex2cite`: `\citeonline` · `\cite` · `\citeauthoronline` · `\apud` · `\citeyear` · `\bibliographystyle{abntex2-alf}` · `\bibliographystyle{abntex2-num}` (estilos BibTeX ABNT)
- [~] `jurabib`/`bibtopic`/`babelbib`/`bibleref`/`bibunits`/`multibib` (alternativas antigas)

### csquotes
- [~] `csquotes`: `\enquote{}` · `\enquote*{}` · `\textquote{}` · `\foreignquote{}{}` · `\hyphenquote{}{}` · `\blockquote[fonte]{}` · `\foreignblockquote` · `\hyphenblockquote` · `\displayquote`
- [~] `csquotes`: `\MakeOuterQuote{"}` · `\SetCiteCommand{\parencite}` · `\DeclareQuoteStyle` · `\ExecuteQuoteOptions{style=}` · `\setquotestyle` · `\csq@`
- [~] `csquotes`: `\begin{displayquote}[fonte]` · `quote`/`quotation` com fonte · citação longa ABNT (recuo 4 cm, corpo menor)
- [~] `csquotes`: `\mkbegdispquote` · `\mkccitation` · `\mkcitation` · `\textcquote` · `\blockcquote` (citação com `\cite`)
- [ ] `quoting` · `epigraph` (`\epigraph{texto}{fonte}`) · `dedication` (`\dedication{}`)

## 15. Pacotes — código, algoritmos, anotações e desenho (8 / 46)

### listings
- [x] `lstlisting` — `\begin{lstlisting}[language=Python]`
- [~] `\lstinline[language=C]{código}` · `\lstinline|…|`
- [ ] `\lstinputlisting[language=,firstline=,lastline=]{arquivo}`
- [~] `\lstset{basicstyle=\ttfamily\small,keywordstyle=\color{blue},commentstyle=,stringstyle=,numbers=left,numberstyle=,frame=single,breaklines=true,showstringspaces=false,tabsize=4,backgroundcolor=}`
- [x] `\lstdefinestyle{nome}{…}` + `style=nome` · `\lstdefinelanguage{}{}` (linguagem própria) · `morekeywords` · `sensitive` · `morecomment` · `morestring`
- [x] `listings`: `caption=` `label=` (legenda "Listagem 1") · `captionpos=b` · `\lstlistoflistings` · `\lstlistingname`
- [x] `listings`: `escapeinside={(*@}{@*)}` · `mathescape` · `literate={á}{{\'a}}1` (acentos) · `inputencoding=utf8` · `extendedchars` · `frame=lines` · `rulecolor=` · `xleftmargin=` · `framexleftmargin=` · `firstnumber=` · `stepnumber=` · `numbersep=` · `linewidth=` · `float` · `gobble=` · `columns=flexible`
- [x] `listings`: realce de linguagens (C, C++, Python, Java, SQL, Bash, Rust, JavaScript, Matlab, R, Haskell, HTML, XML, TeX, Fortran, Pascal, Lisp)

### minted, fancyvrb
- [x] `minted`: `\begin{minted}{python}` · `\begin{minted}[linenos,frame=lines,fontsize=\small,bgcolor=]{c}`
- [~] `\mint{linguagem}|código|` · `\mintinline{linguagem}{código}` · `\inputminted{linguagem}{arquivo}` · `\newminted{}{}` · `\setminted{}` · `\usemintedstyle{}`
- [~] `minted`: `-shell-escape` (Pygments) · `\usepackage[cache=false]{minted}` · estilos `monokai` `friendly` `default` `tango` `colorful` · `breaklines` · `autogobble` · `tabsize` · `escapeinside` · `mathescape`
- [~] `tcblisting` · `\newtcblisting` · `tcolorbox` + `listings`/`minted`
- [~] `fancyvrb`: `Verbatim` `BVerbatim` `LVerbatim` `SaveVerbatim` `\UseVerb` `\Verb` `\VerbatimInput` `\fvset{frame=single,numbers=left,fontsize=\small}` `\DefineVerbatimEnvironment` `\RecustomVerbatimCommand` `commandchars=` `\CustomVerbatimCommand` `codes={}` `gobble=` `label=` `tabsize=`
- [~] `verbatim` (pacote): `\verbatiminput{arquivo}` · `\begin{comment}…\end{comment}`

### algorithm2e, algorithmicx, algorithms
- [~] `algorithm2e`: `\begin{algorithm}[H]` · `\caption{}` · `\KwIn{}` · `\KwOut{}` · `\KwResult{}` · `\SetKwInOut{}{}` · `\SetKwFunction` · `\SetKw{}{}` · `\SetAlgoLined` · `\DontPrintSemicolon` · `\LinesNumbered`
- [~] `algorithm2e`: `\If{cond}{…}` · `\uIf` · `\ElseIf` · `\Else` · `\For{}{}` · `\ForEach{}{}` · `\While{}{}` · `\Repeat{}{}` · `\Return` · `\tcp{comentário}` · `\tcc{}` · `\;` · `\nl` · `\eIf{}{}{}` · `\Switch` · `\Case` · `\Fn` · `\Proc` · `\lIf` · `\lFor` · `\emph{}`
- [x] `algorithm` (float) + `algpseudocode`/`algorithmic`: `\begin{algorithm}\caption{}\begin{algorithmic}[1]`
- [ ] `algpseudocode`: `\State` · `\If{}`…`\ElsIf{}`…`\Else`…`\EndIf` · `\For{}`…`\EndFor` · `\ForAll{}` · `\While{}`…`\EndWhile` · `\Repeat`…`\Until{}` · `\Function{Nome}{args}`…`\EndFunction` · `\Procedure`…`\EndProcedure` · `\Return` · `\Require` · `\Ensure` · `\Comment{}` · `\Call{}{}` · `\algnewcommand` · `\algrenewcommand` · `\algorithmicrequire`
- [ ] `algorithmic`/`algorithmicx` antigo: `\STATE` `\IF{}` `\ENDIF` `\FOR` `\ENDFOR` `\WHILE` `\ENDWHILE` `\REQUIRE` `\ENSURE` `\RETURN` `\COMMENT`
- [ ] `algorithmicx`: `\begin{algorithmic}[1]` numeração · `\algsetup{linenodelimiter=.}` · `\algdef{SE}` · `\algloopdefx` · `\algorithmiccomment` · `\algtext*`
- [~] `algorithmicx`: `\floatname{algorithm}{Algoritmo}` · `\renewcommand{\listalgorithmcfname}` · `\listofalgorithms` · `\algorithmicrequire` renomear (pt-BR)

### todonotes, comentários e revisão
- [~] `todonotes`: `\todo{}` (nota na margem) · `\todo[inline]{}` · `\todo[color=,author=,size=,caption=,noline,fancyline,prepend]{}` · `\missingfigure{}` · `\listoftodos` · `\todototoc` · `\setuptodonotes{}` · `[disable]` `[colorinlistoftodos]`
- [~] `changes`/`trackchanges`/`xcolor`: `\added{}` `\deleted{}` `\replaced{}{}` `\highlight{}` `\comment{}` · `\listofchanges` · `\definechangesauthor`
- [ ] `comment`: `\begin{comment}…\end{comment}` · `\excludecomment{nome}` · `\includecomment` · `\specialcomment` · `\comment`
- [~] `fixme`/`easy-todo`/`pdfcomment` (`\pdfcomment{}` · `\pdfmargincomment{}`) · `marginnote` · `soul` para realçar · `\hl{}`

### ulem, soul, outros
- [~] `ulem`: `\uline` `\uuline` `\uwave` `\sout` `\xout` `\dashuline` `\dotuline` `\normalem` `\ULdepth` `\ULthickness` `\markoverwith` `\useunder`
- [~] `soul`: `\so{}` (espaçamento) · `\caps{}` · `\ul{}` · `\st{}` · `\hl{}` · `\sethlcolor` · `\setul{}{}` · `\setulcolor` · `\setstcolor` · `\soulaccent` · `\soulregister`
- [~] `textcomp`: `\texteuro` `\textcelsius` `\textdegree` `\textcopyright` `\textregistered` `\texttrademark` `\textperthousand` `\textbullet` `\textmu` `\textohm` `\textonesuperior` `\texttwosuperior` `\textthreesuperior` `\textnormal{}`
- [~] `eurosym`: `\euro` · `\EUR{}` · `\EURdig` · `\EURhv` · `\EURcr` · `\EURtm`
- [ ] `gensymb`: `\degree` `\celsius` `\perthousand` `\micro` `\ohm`
- [~] `url`: `\url{}` · `\urldef` · `\path{}` · `\UrlFont` · `\UrlBreaks` · `\UrlBigBreaks` · `\UrlOrds` · `\urlstyle{same|tt|rm|sf}`
- [~] `pdfpages`: `\includepdf[pages={1,3-5},nup=2x2,landscape,angle=90,scale=0.9,frame,offset=,pagecommand={\thispagestyle{plain}},noautoscale,fitpaper,delta=,width=,addtotoc={}]{arquivo.pdf}` · `\includepdfmerge`
- [ ] `lscape`/`pdflscape`/`rotating`: página em paisagem para tabela larga

### TikZ e PGF
- [~] `tikzpicture`: `\begin{tikzpicture}…\draw (0,0) -- (1,1);\end{tikzpicture}`
- [~] `tikz`: `\node[draw,circle] (a) at (0,0) {texto};` · `\path` · `\fill` · `\filldraw` · `\clip` · `\shade` · `\coordinate` · `\foreach` · `\pic` · `\matrix` · `\graph` · estilos (`\tikzset`) · `\tikzstyle` · cores · setas (`->`, `stealth`) · `bend left` · `rounded corners` · `dashed` · `thick` · `scale` · `rotate` · `shift`
- [~] `tikz`: bibliotecas (`\usetikzlibrary{arrows.meta,positioning,calc,shapes,decorations.pathmorphing,patterns,fit,backgrounds,matrix,trees,mindmap,automata,intersections,through,shadows,spy,3d,angles,quotes,chains,petri,er,datavisualization,fadings,external}`)
- [ ] `tikz`: `\tikz[baseline]{…}` (inline) · `\tikz \draw (0,0) circle (1ex);` · `remember picture,overlay` (sobre a página) · `\tikzmark`
- [ ] `tikz-cd` (`tikzcd`): diagramas comutativos `\begin{tikzcd}A \arrow[r] & B\end{tikzcd}`
- [ ] `forest` · `tikz-qtree` · `smartdiagram` · `pgf-umlcd` · `tikz-uml` · `bytefield` · `tikz-timing` · `karnaugh-map` · `tkz-euclide` · `tkz-tab` · `tkz-graph` · `pst-*` (PSTricks)
- [~] `pgfplots`: `\begin{axis}[xlabel=,ylabel=,title=,legend pos=,grid=major,xmin=,xmax=,width=,height=]` + `\addplot{x^2};` · `\addplot table {dados.dat};` · `\addplot3` · `\addlegendentry{}` · `ybar` · `\pgfplotstableread` · `\pgfplotsset{compat=1.18}` · `log` · `error bars` · `contour` · `surf`
- [ ] `circuitikz`: `\begin{circuitikz}\draw (0,0) to[R=$R_1$] (2,0) to[C] (4,0);\end{circuitikz}` · `battery` · `ioff` · `short` · `american`/`european` · portas lógicas · amplificadores · `\ctikzset` · transistores · transformadores
- [ ] `chemfig`: `\chemfig{A-B}` · `\chemfig{*6(-=-=-=)}` · `\setchemfig{}` · `\schemestart`/`\schemestop` · `\arrow` · `\chemname` · `\charge` · `\chemmove` · `\lewis` · `\Charge` · `\definesubmol`
- [ ] `pstricks`/`pst-plot`/`pst-node` · `\psframe` · `\psline` · `\pscircle` · `\rput` · `\psaxes`
- [ ] `asymptote` · `\begin{asy}` · `gnuplottex` · `pythontex` · `sagetex` · `knitr`/`Sweave` · `luacode` · `luaplot` · `metapost` (`\begin{mplibcode}`)
- [x] `tikzpicture` dentro de `figure`, de `\fbox`, de `minipage`, de célula · `\input{figura.tikz}` · `externalize` (cache PDF) · `standalone` compilado à parte
- [ ] Imagem vetorial a partir do desenho TikZ (compilador TeX próprio ou externo)

## 16. Classes de documento e modelos (abntex2, beamer, currículos, KOMA) (2 / 52)

### abntex2
- [~] `abntex2` — `\imprimircapa` · `\imprimirfolhaderosto` · `\imprimirfichacatalografica` · `\imprimirerrata` · `\imprimirfolhadeaprovacao` · `\imprimirdedicatoria` · `\imprimiragradecimentos` · `\imprimirepigrafe` · `\imprimirresumo` · `\imprimirlistadefiguras` · `\imprimirsumario`
- [~] `abntex2` — `\titulo{}` · `\autor{}` · `\data{}` · `\local{}` · `\instituicao{}` · `\orientador{}` · `\coorientador{}` · `\tipotrabalho{}` · `\preambulo{}`
- [~] `abntex2` — `\begin{resumo}` · `\begin{resumo}[Abstract]` · `\begin{otherlanguage*}{english}` · `\palavraschave` · `\textbf{Palavras-chave}:`
- [~] `abntex2` — `\frontmatter` · `\mainmatter` · `\backmatter` · `\textual` · `\pretextual` · `\postextual` · `\apendices` · `\anexos` · `\begin{apendicesenv}` · `\partanexos`
- [~] `abntex2` — `\begin{citacao}` (citação longa com recuo de 4 cm) · `\begin{citacao}[docs]`
- [~] `abntex2` — `\chapter*{}` + `\pdfbookmark` · `\cleardoublepage` · `\phantompart` · `\tableofcontents*` · `\printindex` · `\printbibliography` · `\bibliography{}` + `\bibliographystyle{abntex2-alf}`
- [~] `abntex2` — `\ABNTEXchapterfont` · `\ABNTEXsectionfont` · `\ABNTEXsubsectionfont` · `\ABNTEXsubsubsectionfont` · `\ABNTEXchapterfontsize` · `\setlrmarginsandblock{3cm}{2cm}{*}` · `\setulmarginsandblock{3cm}{2cm}{*}` · `\checkandfixthelayout`
- [~] `abntex2` — `\SingleSpacing` · `\OnehalfSpacing` · `\DoubleSpacing` · `\setlength{\parindent}{1.3cm}` · `\setlength{\parskip}{0.2cm}` · `\setlength{\ABNTEXsignthickness}{0.4pt}`
- [~] `abntex2` — `\begin{agradecimentos}` · `\begin{dedicatoria}` · `\begin{epigrafe}` · `\begin{errata}` · `\begin{folhadeaprovacao}` · `\assinatura{}` · `\membrabanca{}` · `\listaabreviaturas` · `\begin{siglas}`/`\begin{simbolos}`
- [ ] `abntex2` — `\citeonline{}` · `\apud{}{}` · notas explicativas `\footnote` · legenda "Fonte:" · `\fonte{}` · figuras ABNT (`\caption` acima, `\legend{Fonte: …}`)
- [~] `abntex2` — numeração: capítulo `1 TÍTULO` em caixa alta e negrito, seção `1.1 TÍTULO` · página no canto superior direito · `\pagestyle{abntheadings}` · `\pagestyle{simple}`
- [~] `abntex2` — `\newpage` + `\thispagestyle{empty}` capas · `\setlength{\ABNTEXsignwidth}{8cm}` · modelos `abntex2-modelo-trabalho-academico.tex` / `abntex2-modelo-artigo.tex` · `[article]` · `[chapter=TITLE,section=TITLE]`
- [~] `abntex2` — `\texto` / `\tableofcontents` com "Sumário", `\listoffigures` "Lista de ilustrações" · `\renewcommand{\listfigurename}{Lista de ilustrações}`

### beamer (slides) — vão para o FeatherSlide
- [ ] `beamer`: `\begin{frame}{Título}…\end{frame}` · `\frametitle{}` · `\framesubtitle{}`
- [~] `beamer`: `\titlepage` · `\title` · `\subtitle` · `\author` · `\institute` · `\date` · `\titlegraphic` · `\logo` · `\frame{\titlepage}`
- [ ] `beamer`: `\usetheme{Madrid|Berlin|Warsaw|CambridgeUS|Singapore|Boadilla|metropolis|default|Frankfurt|Copenhagen|Antibes|Montpellier|Goettingen|Hannover|AnnArbor|Rochester|PaloAlto|Marburg|Ilmenau|JuanLesPins|Malmoe|Luebeck|Szeged|Darmstadt|Dresden|Bergen|Boadilla}`
- [~] `beamer`: `\usecolortheme{}` · `\usefonttheme{}` · `\useinnertheme{}` · `\useoutertheme{}` · `\setbeamercolor{}{fg=,bg=}` · `\setbeamerfont{}{}` · `\setbeamertemplate{navigation symbols}{}` · `\setbeamertemplate{footline}` · `\setbeamersize{text margin left=}`
- [ ] `beamer`: `\pause` · `\onslide<2->{}` · `\only<1>{}` · `\uncover<2->{}` · `\visible<3->{}` · `\invisible` · `\alt<1>{a}{b}` · `\temporal` · `<+->` em listas · `\item<2->` · `\action` · `\transduration` · `\transboxout`
- [x] `beamer`: `[allowframebreaks]` · `[fragile]` · `[plain]` · `[t]` · `[c]` · `[shrink]` · `[noframenumbering]` · `[label=]` · `[containsverbatim]` · `[b]`
- [~] `beamer`: `block` · `alertblock` · `exampleblock` · `theorem` · `definition` · `lemma` · `proof` · `corollary` · `example` · `\alert{}` · `\structure{}` · `\emph`
- [ ] `beamer`: `\begin{columns}\column{.5\textwidth}…\end{columns}` · `columns [T]` · `\column`
- [~] `beamer`: `\section` · `\subsection` · `\tableofcontents[currentsection]` · `\AtBeginSection[]{}` · `\AtBeginSubsection` · `\appendix` · `\insertframenumber` · `\inserttotalframenumber` · `\insertshortauthor` · `\insertsection` · `\inserttitle`
- [~] `beamer`: `\hyperlink` · `\hypertarget` · `\hyperlinkframestartnext` · `\againframe` · `\framezoom` · `\begin{overprint}` · `\includegraphics` · `\movie` · `\animate` (animação) · `\note{}` · `\setbeameroption{show notes}` · `handout` · `[notes=only]` · `\mode<presentation>` · `\mode<handout>`
- [~] `beamer`: `\documentclass[aspectratio=169,10pt,xcolor=dvipsnames,handout,t]{beamer}` · `[unicode]` · `[hyperref={pdfpagelabels=false}]` · `[compress]` · `[mathserif]`

### moderncv, altacv, awesome-cv, res
- [~] `moderncv`: `\moderncvstyle{classic|casual|banking|oldstyle|fancy}` · `\moderncvcolor{blue|orange|green|red|purple|grey|black}` · `\name{}{}` · `\title{}` · `\address{}{}{}` · `\phone[mobile]{}` · `\email{}` · `\homepage{}` · `\social[linkedin]{}` · `\photo[64pt][0.4pt]{}` · `\extrainfo{}` · `\quote{}`
- [~] `moderncv`: `\section{}` · `\cventry{anos}{cargo}{empresa}{cidade}{nota}{descrição}` · `\cvitem{rótulo}{texto}` · `\cvdoubleitem{}{}{}{}` · `\cvlistitem{}` · `\cvlistdoubleitem{}{}` · `\cvitemwithcomment{}{}{}` · `\cvcolumn` · `\makecvtitle` · `\makeletterhead` · `\recipient{}{}` · `\opening{}` · `\closing{}` · `\enclosure[]{}`
- [ ] `altacv`: `\cvsection{}` · `\cvevent{cargo}{empresa}{datas}{local}` · `\cvtag{}` · `\cvskill{nome}{nível}` · `\cvachievement{}{}{}` · `\cvref{}{}{}` · `\divider` · `\begin{paracol}{2}` · `\name{}` · `\tagline{}` · `\photoR{}{}` · `\personalinfo{\email{}\phone{}\mailaddress{}\location{}\homepage{}\linkedin{}\github{}}` · `\makecvheader` · `\columnratio{}`
- [~] `awesome-cv`/`res`/`europecv`/`friggeri-cv`/`deedy-resume`/`twentysecondcv` (currículos) · `\cvsection` · `\cvskills` · `\cventries` · `\cvhonors`

### standalone, import, subfiles
- [~] `standalone`: `\documentclass[tikz,border=5pt]{standalone}` (recorte do conteúdo) · `\standaloneconfig{}` · `\begin{standalone}` · `\includestandalone[]{}` · `class=`
- [ ] `import`: `\import{caminho/}{arquivo}` · `\subimport{}{}` · `\inputfrom{}{}` · `\includefrom{}{}` · `\subincludefrom` · `\subimportfrom`
- [ ] `subfiles`: `\documentclass[principal.tex]{subfiles}` · `\subfile{capitulo}` · `\subfileinclude` · `\subfix{}` · `\subfiles@` (capítulo compilado separado)
- [~] `\input{capitulos/introducao}` (arquivo ao lado, sem `.tex`) · `\include{}` · `\InputIfFileExists{}{}{}` · `\IfFileExists{}{}{}` · caminho relativo e `\graphicspath`
- [~] `\includeonly{cap1,cap3}` · `\clearpage` implícito do `\include` · `\input` aninhado · `\endinput`

### KOMA-Script (scrartcl, scrreprt, scrbook, scrlttr2)
- [~] `KOMA-Script`: `\KOMAoptions{fontsize=12pt,paper=a4,parskip=half,DIV=12,BCOR=5mm,headings=small,numbers=noenddot,toc=bibliography}` · `\addtokomafont{section}{\sffamily}` · `\setkomafont{disposition}{}` · `\recalctypearea` · `\typearea` · `\areaset`
- [ ] `KOMA-Script`: `\subject{}` · `\publishers{}` · `\titlehead{}` · `\uppertitleback{}` · `\lowertitleback{}` · `\dedication{}` · `\extratitle{}` · `\minisec{}` · `\setcapindent{}` · `\captionabove` · `\captionbelow` · `\listofnotes` · `\thefootnote` · `\scrheadings` · `\manualmark` · `\automark` · `\ohead` · `\ifoot` · `\pagemark`
- [~] `scrlttr2`: `\begin{letter}{Destinatário}` · `\opening{}` · `\closing{}` · `\setkomavar{fromname}{}` · `\KOMAoptions{fromalign=center}` · `\ps` · `\cc` · `\encl` · `\signature{}` · `\yourref` · `\yourmail` · `\myref` · `\customer` · `\invoice` · `\place` · `\date`

### memoir
- [~] `memoir`: `\chapterstyle{}` · `\pagestyle{}` · `\makepagestyle{}` · `\makeevenhead` · `\setsecheadstyle{}` · `\setlength{\beforechapskip}` · `\newsubfloat` · `\setsubparaindent` · `\midsloppy` · `\abstractintoc` · `\settocdepth` · `\setsecnumdepth` · `\maxsecnumdepth` · `\openepigraph` · `\epigraph` · `\poemtitle` · `\begin{verse}` · `\Verbatim` · `\tightlists` · `\newlistentry` · `\cftchapterfont`

### Pacotes de layout diversos
- [~] `titling`: `\pretitle{}` · `\posttitle{}` · `\preauthor{}` · `\postauthor{}` · `\predate{}` · `\postdate{}` · `\thetitle` · `\theauthor` · `\thedate` · `\droptitle`
- [ ] `changepage`/`chngpage`: `\begin{adjustwidth}{2cm}{2cm}` · `\strictpagecheck` · `\checkoddpage`
- [~] `lineno`: `\linenumbers` · `\nolinenumbers` · `\pagewiselinenumbers` · `\begin{linenumbers}` · `\modulolinenumbers[5]` · `\setcounter{linenumber}`
- [ ] `lettrine`: `\lettrine[lines=3]{A}{o começo}` (capitular) · `\lhang` · `\loversize` · `\lraise`
- [ ] `hanging`/`hangparas`: `\hangpara{}{}` · `\begin{hangparas}{1cm}{1}` (recuo suspenso) · `\hangindent` · `\hangafter`
- [ ] `tabto`: `\tabto{2cm}` · `\NumTabs{}` · `\tab` · `\tabto*`
- [~] `setspace`: `\setstretch` · `\onehalfspacing` · `\doublespacing` · `\singlespacing` · `\begin{spacing}{1.5}` · `\SetSinglespace` · `\setdisplayskipstretch`
- [~] `datetime`/`datetime2`/`isodate`/`scrdate`: `\today` · `\formatdate{}{}{}` · `\DTMtoday` · `\newdateformat` · `\ddmmyyyydate` · `\currenttime`
- [ ] `qrcode`: `\qrcode[height=2cm]{texto}` · `\barcode` · `\usepackage{pst-barcode}`
- [ ] `pgf-pie`: `\pie{50/A,50/B}` (pizza) · `smartdiagram` · `tikz-3dplot` · `pgfgantt` (`\begin{ganttchart}`) · `bytefield` · `tikz-timing` · `pgf-spectra`
- [~] `multicol`: `\begin{multicols}{3}[\section{T}]` · `\columnbreak` · `\vfill\null` · `\setlength{\columnseprule}{0.4pt}` · `\raggedcolumns` · `multicols*`
- [~] `verse`: `\poemtitle{}` · `\settowidth{\versewidth}{}` · `\begin{verse}[\versewidth]` · `\attrib{}` · `\flagverse{}` · `\poemlines{5}` · `\vin` (versos numerados)
- [ ] `musixtex` / `lilypond` / `abc` (partituras) · `\begin{music}` · `\notes`
- [ ] `bytefield`, `rail` (diagramas de sintaxe), `syntax`, `bnf` · `\begin{syntaxdiagram}` · `\begin{grammar}`
- [x] `inputenc`/`textcomp` com `\usepackage[utf8x]{inputenc}` + `ucs` (legado) · `\DeclareUnicodeCharacter{00A0}{~}`

## Fidelidade (medidas, não amplitude)
Itens que o leitor não mede (marca fixa no arquivo de dados); cada um precisa de comparação visual com o PDF do `pdflatex`.
- [ ] Mesmas quebras de linha e de página do `pdflatex` (Knuth–Plass, viúvas e órfãs)
- [ ] Hifenização TeX em pt-BR (padrões do `hyph-pt`)
- [ ] Métricas de fonte iguais às do LaTeX (Computer/Latin Modern, Helvetica)
- [ ] Espaçamento vertical exato (`\parskip`, `\baselineskip`, `\topsep`, `\abovedisplayskip`)
- [ ] Posição dos flutuantes (`[htbp]`, `\floatpagefraction`) igual à do LaTeX
- [ ] Numeração de páginas e de seções idêntica no sumário (duas passagens)
- [ ] Kerning, ligaduras (`fi`, `fl`) e espaçamento após ponto final
- [ ] Largura de colunas de tabela (`p{}`, `X`, `tabularx` iterativo) igual à do TeX

## Fora do alcance do leitor próprio (precisa de um TeX de verdade)
TikZ/PGF e `pgfplots` (desenho), `\foreach`/loops, pacotes que programam o TeX (LaTeX3 interno, `expl3`), `chemfig`,
`circuitikz`, `musixtex`, `Lua` (`\directlua`). Caminho previsto: compilar só esse trecho com um motor TeX e inserir
o resultado como imagem vetorial; ou um motor TeX próprio em Rust (projeto grande, ainda não decidido). Os itens
correspondentes aparecem nas seções acima como `[ ]` e continuam contando contra a paridade de propósito.
