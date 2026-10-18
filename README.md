# FeatherDocs

**Editor de documentos da suíte FeatherOffice, escrito em Rust com interface
nativa Iced.** Escreve, abre e grava `.docx`, LaTeX e Markdown, com o histórico
de versões guardado dentro do próprio arquivo, sem conta e sem nuvem.

> **Estado:** em desenvolvimento ativo. Versão alpha prevista para **30/10/2026**.

---

## O que já faz

| Área | Destaques |
|---|---|
| **Edição** | editor contínuo (cursor e seleção atravessam parágrafos), estilos com nome e numeração, menu do botão direito, desfazer/refazer no motor |
| **Word** | lê e grava `.docx` nativo em Rust, sem conversores externos |
| **LaTeX** | lê `.tex` com corpus de testes de paridade, fórmulas pelo `feather-math`, painel de código ao lado do papel e aba de configuração (classe, pacotes, preâmbulo) |
| **Markdown** | editar e ver lado a lado |
| **Referências** | bibliografia, campos e revisão de texto |
| **Imagens e gráficos** | efeitos de imagem e gráficos pelo motor comum [`feather-charts`](https://github.com/FeatherOffice/feather-charts) |

O que falta para o uso diário: [`docs/PARIDADE-USO.md`](docs/PARIDADE-USO.md).
Correções da auditoria de 18/10: [`docs/PLANO-CORRECOES.md`](docs/PLANO-CORRECOES.md).
Plano das funções ainda marcadas "Em breve": [`docs/PLANO-EM-BREVE.md`](docs/PLANO-EM-BREVE.md).
Paridade com o LaTeX: [`docs/PARIDADE-LATEX.md`](docs/PARIDADE-LATEX.md).

## Formato `.fdoz`

Contêiner ZIP com os dados em Apache Arrow e um **repositório Git embutido**
(feito com [`gix`](https://github.com/Byron/gitoxide), Rust puro): as versões do
documento ficam dentro do arquivo, sem servidor.

## Estrutura

| Crate | Papel |
|---|---|
| [`featherdocs-core`](crates/featherdocs-core) | o documento (árvore de blocos), a paginação determinística e a edição com desfazer |
| [`featherdocs-io`](crates/featherdocs-io) | `.fdoz`, `.docx`, LaTeX e Markdown |
| [`featherdocs-historico`](crates/featherdocs-historico) | o histórico Git dentro do `.fdoz` |
| [`featherdocs-tabelas`](crates/featherdocs-tabelas) | inserir/excluir linha e coluna, mesclar células, pelo motor comum `feather-tables` |
| [`feather-math`](crates/feather-math) | composição de fórmulas LaTeX com a Latin Modern Math embutida |
| [`featherdocs-gui`](crates/featherdocs-gui) | a interface nativa em Iced |
| [`featherdocs-ponte`](crates/featherdocs-ponte) | camada de comandos da interface antiga (referência de paridade) |

A pasta `src/` guarda a interface antiga em TypeScript/Tauri, mantida como
referência enquanto a migração para Iced termina.

## Compilar

O repositório compila sozinho: as bibliotecas comuns da suíte
([`feather-ui`](https://github.com/FeatherOffice/feather-ui),
[`feather-draw`](https://github.com/FeatherOffice/feather-draw),
[`feather-charts`](https://github.com/FeatherOffice/feather-charts) e
[`feather-io`](https://github.com/FeatherOffice/feather-io)) entram por tag git.

```sh
cd crates/featherdocs-gui
cargo run --release
```

## Privacidade

Tudo roda localmente. Nenhum documento sai da máquina; não há conta, login,
telemetria nem servidor.

## Documentação

- [Arquitetura](ARQUITETURA.md)
- [Especificação do menu](docs/MENU-ESPECIFICACAO.md)
- [Registro de tarefas](docs/REGISTRO-DE-TAREFAS.md) — histórico das entregas até a separação do repositório

## Licença

[GPL-3.0](LICENSE).
