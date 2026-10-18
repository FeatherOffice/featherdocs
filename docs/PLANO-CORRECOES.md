# FeatherDocs — plano de correções (auditoria de 18/10/2026)

Achados da auditoria da pasta `featherdocs/` (interface Iced e motores), em
ordem de prioridade. Cada item diz o que corrigir, onde, como conferir e o
custo estimado. Marcar `[X]` ao terminar, como em `PARIDADE-USO.md`.

Carga considerada: uma pessoa, um documento aberto por vez, documentos de
até algumas centenas de páginas.

## Primeiro: perda de dados

- [ ] **1. Salvar `.fdoz` pela GUI apaga o histórico de versões.**
  `crates/featherdocs-gui/src/arquivos.rs:40` grava com
  `featherdocs_io::fdoz::gravar`, que não leva o `.git/` embutido. O
  contrato do formato (README, `FORMATO.md` da suíte) diz que as versões
  ficam dentro do arquivo; quem abre um `.fdoz` com histórico e salva pela
  GUI Iced perde tudo.
  - Correção: ler os bytes do arquivo anterior (se existir) e passar por
    `featherdocs_historico::gravar_com_historico(novo, anterior, autor,
    mensagem)`, como a interface antiga fazia em `src-tauri/src/arquivos.rs`.
    A GUI passa a depender de `featherdocs-historico`.
  - Teste: salvar duas vezes o mesmo `.fdoz` e conferir que o segundo tem
    dois commits no `.git/` embutido.
  - Custo: pequeno (uma função e um teste).

## Depois: velocidade em documento grande

- [ ] **2. Toda mensagem refaz os quadros de todas as páginas.**
  `app.rs:373` chama `sessao.quadro(p, u64::MAX)` para cada página a cada
  `atualizar()`, inclusive movimento de mouse e rolagem. Em documento de
  200 páginas isso é trabalho por quadro que cresce com o tamanho.
  - Correção: guardar a versão do documento junto de cada quadro e só
    pedir de novo as páginas cuja versão mudou (o motor já devolve a
    versão; o `u64::MAX` força refazer sempre). Rolagem e mouse não mudam
    a versão.
  - Teste: medir no F12 com o `.docx` de 18 páginas da especificação do
    menu antes e depois; contar quantas páginas foram refeitas por tecla.
  - Custo: médio.

- [ ] **3. Os canvas das páginas não têm `canvas::Cache`.**
  Cada redesenho do Iced refaz a geometria de todas as operações da página.
  - Correção: um `canvas::Cache` por página, limpo só quando o quadro da
    página muda (depende do item 2).
  - Custo: pequeno depois do item 2.

- [ ] **4. O painel de revisão recalcula tudo a cada `view`.**
  `app.rs:1997` (`view_revisao`) conta palavras e percorre comentários e
  revisões em toda chamada de `view`.
  - Correção: calcular em `atualizar()` só quando a versão do documento
    muda e guardar o resultado no `App`.
  - Custo: pequeno.

## Depois: interface

- [ ] **5. O menu do botão direito pode sair da janela.**
  `app.rs:1951` (`view_contexto`) posiciona o menu no ponto do clique sem
  limitar à área visível; clicando perto da borda direita ou de baixo, parte
  do menu fica fora da tela.
  - Correção: limitar `x` e `y` ao tamanho da janela menos o tamanho do
    menu (a janela já chega em `window::Event::Resized`).
  - Custo: pequeno.

- [ ] **6. Recentes guardam arquivos que já não existem.**
  `configuracao.rs:69` (`lembrar`) nunca remove caminhos apagados ou
  movidos; o menu mostra a entrada e abrir dá erro.
  - Correção: filtrar com `Path::exists` ao montar o menu e tirar da lista
    ao falhar a abertura.
  - Custo: pequeno.

## Limpeza (sem efeito para quem usa)

- [ ] **7. Código morto no leitor LaTeX.**
  `crates/featherdocs-io/src/latex/avisos.rs`: `MARCA_DE_ROTULO`, `rotular`,
  `contar` e os campos `rotulos`, `numeros`, `contadores` não são usados
  (o compilador avisa). Apagar, depois de conferir com `grep` que nada os
  chama.

- [ ] **8. Import sem uso no motor.**
  `crates/featherdocs-core/src/sessao.rs:17` importa `compor_em` sem usar.
  Apagar.

## Já resolvido na separação do repositório (18/10/2026)

- [X] Dependências por pasta do monorepo trocadas por tags git
  (`feather-io` v0.1.1, `feather-charts` v0.3.1, `feather-ui` v0.2.9).
  `feather-math` e a parte "docs" do `featheroffice-table-adapters` só
  eram usadas aqui e vieram para `crates/feather-math` e
  `crates/featherdocs-tabelas`.
- [X] Teste `fonte_renumerada_no_docx_e_religada_sem_perder_o_svg` pedia
  pizza com duas séries (o motor de gráficos só aceita uma); passou a
  usar barras, que é o que o teste quer conferir.
- [X] Teste de nota dentro de tabela comparava o rótulo antes da
  numeração automática das notas.
- [X] `feather-draw` v0.1.2 (Times, Courier, imagem e texto girado no
  PDF) publicado; a GUI usa a tag, sem `[patch]`. O `featherdocs-io` fica
  na v0.1.1 porque passa cores ao `feather-charts`, que ainda usa a v0.1.1;
  subir os dois juntos quando sair um `feather-charts` com a v0.1.2.
- [ ] No monorepo, tirar a parte "docs" de `featheroffice-table-adapters`
  (agora duplicada aqui) e a pasta `feather-math`.
