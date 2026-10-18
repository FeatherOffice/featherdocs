/**
 * A janela inteira: liga a barra, a estrutura, as folhas, a entrada de teclado e o
 * rodapé ao motor. Só fiação — nenhuma regra de documento mora aqui. O cursor, a
 * seleção e o que cada tecla faz são do motor; aqui só se traduz mouse e teclado
 * em pedido e se desenha o que ele devolve.
 */

import { formatar, t, aoMudarIdioma } from '../idiomas/index.ts';
import type { Motor } from '../ponte/index.ts';
import { medirAcao } from '../ponte/metricas.ts';
import { noApp } from '../ponte/nativo.ts';
import type { Estado, Formato, FormatoDoCodigo, Mudanca, Papel, Ponto, Titulo } from '../ponte/tipos.ts';
import { abrirAjuda } from './ajuda.ts';
import { painelDeConfiguracoes, type SettingsPanel } from './configuracoes.ts';
import { PaginaInicial } from './inicio.ts';
import { Barra, MARGENS_PRONTAS } from './barra.ts';
import { PainelDeBusca } from './busca.ts';
import { Dialogo, rotuloFechar } from './dialogo.ts';
import { confirmarConversao, pedirNumeros } from './pergunta.ts';
import { caixaDeEstilo, dadosDoCursor, dadosDoEstilo } from './caixa-estilo.ts';
import { caixaDeCabecalho, caixaDeContagem, caixaDeLink, dataFormatada, escolherImagem, pedirTexto } from './caixas.ts';
import { Regua } from './regua.ts';
import { Entrada } from './entrada.ts';
import { Folhas } from './folhas.ts';
import { PainelEstrutura } from './painel-estrutura.ts';
import { PainelDeCodigo } from './painel-codigo.ts';
import { PainelDeMiniaturas } from './painel-miniaturas.ts';
import { caixaDeMarcador, caixaDeReferencia, caixaDeSumario } from './caixas-referencias.ts';
import { caixaDeFormatoDasNotas, caixaDeNota } from './notas.ts';
import { caixaDeCitacao, caixaDeEntrada, caixaDeFonte, caixaDeFontes } from './citacoes.ts';
import { Arquivos, abrirTexto, escolherDocumento, salvarTexto } from './arquivos.ts';
import { PainelDeRevisao } from './painel-revisao.ts';
import { caixaDeVersoes } from './versoes.ts';
import { abrirVersao, compararVersao, versoes } from '../ponte/nativo.ts';
import { Rodape } from './rodape.ts';
import { injetarEstilos } from './styles.ts';
import { LigacaoComFeatherSheet } from './tabela-feathersheet.ts';
import { openPopup } from './menu.ts';
import { menuDeContexto } from './menu-contexto.ts';
import { definirModo } from './tema.ts';
import { avisar } from './toast.ts';

const PT_POR_CM = 72 / 2.54;
/** Pixels CSS por ponto com zoom 1 (o mesmo das folhas). */
const PX_POR_PT = 96 / 72;
const PASSOS_DE_ZOOM = [0.5, 0.75, 0.9, 1, 1.25, 1.5, 2, 3];

/** O passo de zoom seguinte (ou anterior) ao atual. */
function passoDeZoom(atual: number, sentido: number): number {
  const prox = sentido > 0 ? PASSOS_DE_ZOOM.find((z) => z > atual + 0.001) : [...PASSOS_DE_ZOOM].reverse().find((z) => z < atual - 0.001);
  return prox ?? atual;
}

/** Onde fica guardado o nome de quem edita (autor das sugestões e dos comentários). */
const CHAVE_AUTOR = 'featherdocs.autor';

function autorGuardado(): string {
  try {
    return localStorage.getItem(CHAVE_AUTOR) ?? '';
  } catch {
    return '';
  }
}

class App {
  private readonly arquivos: Arquivos;
  private readonly barra: Barra;
  private readonly regua: Regua;
  private readonly estrutura: PainelEstrutura;
  private readonly folhas: Folhas;
  /** O modo dividido: o código à esquerda das páginas. */
  private readonly codigo: PainelDeCodigo;
  private readonly miniaturas: PainelDeMiniaturas;
  private relogioCodigo = 0;
  private editandoCodigo = false;
  private readonly entrada: Entrada;
  private readonly rodape = new Rodape();
  private readonly inicio: PaginaInicial;
  private readonly configuracoes: SettingsPanel;
  private jaViuODocumento = false;
  private readonly feathersheet: LigacaoComFeatherSheet;
  private estado: Estado;
  private formato: Formato | null = null;
  private relogioEstrutura = 0;
  private readonly busca: PainelDeBusca;
  private cliqueNaCaixa = false;
  /** O nome do arquivo que o motor conhece (o campo `arquivo`). */
  private ultimoArquivo = '';
  private readonly revisao: PainelDeRevisao;
  /** O pincel de formatação: `uma` aplica na próxima seleção e desarma; `sempre` fica até Esc. */
  private pincel: 'uma' | 'sempre' | null = null;
  /** Modo foco: só o texto (sem barra, régua, estrutura e rodapé). */
  private foco = false;
  private sairDoFoco: HTMLButtonElement | null = null;
  private readonly raiz: HTMLElement;

  constructor(
    raiz: HTMLElement,
    private readonly motor: Motor,
  ) {
    this.raiz = raiz;
    injetarEstilos();
    this.arquivos = new Arquivos(motor);
    motor.rotuloDoSumario(t('sumario.rotulo'));
    motor.rotuloDoCapitulo(t('capitulo.rotulo'));
    this.estado = motor.estado();
    this.configuracoes = painelDeConfiguracoes({
      aoMudarTema: () => this.atualizarBarras(),
      arquivoAberto: () => (this.arquivos.nome !== t('app.sem_titulo') ? this.arquivos.nome : ''),
      noDisco: () => noApp(),
      salvarAgora: () => void this.salvar(false),
      salvarComo: () => void this.salvar(true),
      papel: () => (this.estado.largura < 600 ? 'a4' : 'carta'),
      setPapel: (p) => this.trocarPapel(p),
      margens: () => {
        const [esq] = MARGENS_PRONTAS.normal;
        return Math.abs(this.estado.margem_esq - esq) < 1 ? 'normal' : this.estado.margem_esq < esq ? 'estreita' : 'larga';
      },
      setMargens: (pronta) => {
        const [esq, dir, sup, inf] = MARGENS_PRONTAS[pronta];
        this.mudarMargens(esq, dir, sup, inf);
      },
      regua: () => !this.regua.el.hidden,
      setRegua: (mostrar) => {
        this.regua.el.hidden = !mostrar;
      },
      estrutura: () => this.estrutura.visivel,
      setEstrutura: (mostrar) => {
        this.estrutura.definirVisivel(mostrar);
        this.barra.estruturaVisivel(mostrar);
      },
      zoom: () => this.folhas.zoom,
      setZoom: (z) => this.folhas.definirZoom(z),
      aoFechar: () => this.entrada.focar(),
    }, raiz);
    this.barra = new Barra({
      inicio: () => this.mostrarInicio(),
      configurar: () => this.configuracoes.abrir(),
      ajuda: () => this.pedirAjuda(),
      novo: () => void this.novo(),
      abrir: () => void this.abrir(),
      salvar: () => void this.salvar(false),
      salvarComo: () => void this.salvar(true),
      exportar: (f) => void this.exportar(f),
      desfazer: () => this.desfazerRefazer(false),
      refazer: () => this.desfazerRefazer(true),
      recortar: () => void this.recortarCopiarColar('recortar'),
      copiar: () => void this.recortarCopiarColar('copiar'),
      colar: (sem) => void this.recortarCopiarColar(sem ? 'colar_sem' : 'colar'),
      selecionarTudo: () => this.acao('selecionar tudo', () => (this.motor.selecionarTudo(), null)),
      estilo: (tipo) => this.acao('estilo', () => this.motor.paragrafoEstilo(tipo)),
      estiloComNome: (id) => this.acaoDeEstilo('estilo', () => this.motor.estiloAplicar(id)),
      estiloNovo: () => this.abrirCaixaDeEstilo(0),
      estiloModificar: (id) => this.abrirCaixaDeEstilo(id),
      estiloExcluir: (id) => this.excluirEstilo(id),
      numeracao: (n) => this.acaoDeEstilo('numeração', () => this.motor.numeracao(n)),
      tituloSemNumero: () => this.acao('numeração', () => this.motor.tituloSemNumero()),
      modoDividido: (f) => this.modoDividido(f),
      codigoAtivo: () => (this.codigo?.visivel ? this.codigo.formato : null),
      continuoAtivo: () => Boolean(this.folhas?.modoContinuo),
      variasAtivo: () => Boolean(this.folhas?.variasPaginas),
      miniaturasAtivas: () => Boolean(this.miniaturas?.visivel),
      sumarioPersonalizado: () => caixaDeSumario((n, numeros, pontilhado) => this.acao('sumário', () => this.motor.sumarioConfigurar(n, numeros, pontilhado))),
      numerarLinhas: (n) => this.aplicarPagina(() => this.motor.numerarLinhas(n)),
      vertical: (v) => this.aplicarPagina(() => this.motor.vertical(v)),
      alternarGrade: () => this.aplicarPagina(() => this.motor.grade(!this.estado.grade)),
      legenda: (tp) => this.acao('legenda', () => this.motor.legenda(tp)),
      listaDeLegendas: (k) => this.acao('lista', () => this.motor.listaDeLegendas(k)),
      marcador: () => this.abrirMarcadores(),
      referenciaCruzada: () => this.abrirReferencia(),
      referenciasAtualizar: () => this.acao('referências', () => this.motor.referenciasAtualizar()),
      notaInserir: (fim) => caixaDeNota(t(fim ? 'cmd.nota_fim' : 'cmd.nota_rodape'), { fonte: '', fim }, (d) => this.tentar('nota', () => this.motor.notaInserir(d.fonte, d.fim))),
      formatoDasNotas: () => caixaDeFormatoDasNotas(this.estado.extras.formato_notas, this.estado.extras.formato_notas_fim, (r, f) => this.aplicarPagina(() => this.motor.notasFormato(r, f))),
      campo: (nome) => this.acao('campo', () => this.motor.campoInserir(nome)),
      citacaoInserir: () => this.abrirCitacao(null),
      fontes: () => this.gerenciarFontes(),
      estiloCitacao: (nome) => this.acao('estilo de citação', () => this.motor.estiloCitacao(nome)),
      bibliografiaInserir: (todas) => this.acao('bibliografia', () => this.motor.bibliografiaInserir(todas)),
      marcarEntrada: () =>
        caixaDeEntrada(this.motor.selecaoTexto().split('\n')[0]!.trim(), (termo, sub, todas) =>
          this.tentar('índice', () => {
            const r = this.motor.indiceMarcar(termo, sub, todas);
            if (todas) avisar(t('entrada.marcadas', { n: r.n }));
            return r.mudanca;
          }),
        ),
      indiceInserir: () => this.acao('índice', () => this.motor.indiceInserir()),
      comentarioNovo: () =>
        this.comAutor(() =>
          pedirTexto(t('revisao.novo'), t('revisao.novo_texto'), '', (texto) => {
            this.acao('comentário', () => this.motor.comentarioNovo(texto));
            this.revisao.definirVisivel(true);
            this.atualizarRevisao();
          }),
        ),
      comentariosMostrar: (filtro) => {
        this.revisao.definirVisivel(filtro !== null);
        if (filtro) this.revisao.filtro = filtro;
        this.atualizarRevisao();
        this.atualizarBarras();
      },
      comentariosFiltro: () => (this.revisao.visivel ? this.revisao.filtro : null),
      modoEdicao: (m) => (m === 'sugerindo' ? this.comAutor(() => this.aplicarPagina(() => this.motor.modoEdicao(m))) : this.aplicarPagina(() => this.motor.modoEdicao(m))),
      revisoesResolver: (aceitar, tudo) => {
        this.acao('revisão', () => this.motor.revisoesResolver(aceitar, tudo));
        this.atualizarRevisao();
      },
      comparar: () => void this.comparar(),
      historico: () => void this.historico(),
      proteger: (sim) => {
        this.aplicarPagina(() => this.motor.proteger(sim));
        avisar(t(sim ? 'revisao.protegido' : 'revisao.desprotegido'));
      },
      imagemUrl: () => this.inserirImagemDeUrl(),
      modoContinuo: () => this.arrumarFolhas(!this.folhas.modoContinuo, false),
      zoomVarias: () => this.arrumarFolhas(false, !this.folhas.variasPaginas),
      alternarMiniaturas: () => {
        this.miniaturas.alternar();
        this.atualizarMiniaturas();
        this.atualizarBarras();
      },
      formatar: (e) => this.acao('formatar', () => this.motor.formatar(e)),
      inserir: (tipo) => this.acao('inserir', () => this.motor.inserirBloco(tipo)),
      nivel: (d) => this.acao('recuo', () => this.motor.recuar(d)),
      alinhar: (como) => this.acao('alinhar', () => this.motor.alinhar(como)),
      fonte: (f) => this.acao('fonte', () => this.motor.fonte(f)),
      tamanho: (pt) => this.acao('tamanho', () => this.motor.tamanho(pt)),
      tamanhoPasso: (d) => this.acao('tamanho', () => this.motor.tamanhoPasso(d)),
      cor: (c) => this.acao('cor', () => this.motor.cor(c)),
      realce: (c) => this.acao('realce', () => this.motor.realce(c)),
      posicao: (p) => this.acao('posição', () => this.motor.posicao(p)),
      maiusculas: (modo) => this.acao('maiúsculas', () => this.motor.maiusculas(modo)),
      limparFormatacao: () => this.acao('limpar', () => this.motor.limparFormatacao()),
      pincel: (persistente) => this.armarPincel(persistente),
      entrelinha: (fator) => this.acao('entrelinha', () => this.motor.entrelinha(fator)),
      entrelinhaPersonalizada: () => this.pedirEntrelinha(),
      espaco: (antes, depois) => this.acao('espaço', () => this.motor.espaco(antes, depois)),
      fundo: (c) => this.acao('fundo', () => this.motor.fundo(c)),
      borda: (lados) => this.acao('borda', () => this.motor.borda(lados)),
      lista: (nome) => this.acao('lista', () => this.motor.lista(nome)),
      listaReiniciar: () => this.acao('lista', () => this.motor.listaReiniciar()),
      localizar: () => this.abrirBusca(false),
      substituir: () => this.abrirBusca(true),
      papel: (p) => this.trocarPapel(p),
      papelPersonalizado: () => this.pedirPapel(),
      orientacao: (paisagem) => this.aplicarPagina(() => this.motor.orientacao(paisagem)),
      margensPersonalizadas: () => this.pedirMargens(),
      recuos: (esq, dir) => this.acao('recuo', () => this.motor.recuosDefinir(esq, dir, this.formato?.primeira ?? 0)),
      margens: (pronta) => {
        const [esq, dir, sup, inf] = MARGENS_PRONTAS[pronta];
        this.mudarMargens(esq, dir, sup, inf);
      },
      zoom: (d) => this.folhas.definirZoom(passoDeZoom(this.folhas.zoom, d)),
      zoomPara: (z) => this.folhas.definirZoom(z),
      zoomLargura: () => this.folhas.definirZoom((this.folhas.rolagem.clientWidth - 48) / (this.estado.largura * PX_POR_PT)),
      zoomPagina: () => this.folhas.definirZoom((this.folhas.rolagem.clientHeight - 48) / (this.estado.altura * PX_POR_PT)),
      zoomAtual: () => this.folhas.zoom,
      alternarEstrutura: () => this.alternarEstrutura(),
      alternarRegua: () => {
        this.regua.el.hidden = !this.regua.el.hidden;
        this.atualizarBarras();
      },
      reguaVisivel: () => !this.regua.el.hidden,
      alternarMarcas: () => this.aplicarPagina(() => this.motor.marcas(!this.estado.marcas)),
      modoFoco: () => this.alternarFoco(),
      focoAtivo: () => this.foco,
      tema: (modo) => {
        definirModo(modo);
        this.atualizarBarras();
      },
      inserirTabela: (l, c) => this.acao('tabela', () => this.motor.inserirTabela(l, c)),
      tabelaDialogo: () =>
        pedirNumeros(t('cmd.tabela_dialogo'), [
          { rotulo: t('faixa.linhas'), valor: 3 },
          { rotulo: t('faixa.colunas_n'), valor: 3 },
        ], ([l, c]) => this.acao('tabela', () => this.motor.inserirTabela(Math.round(l!), Math.round(c!)))),
      inserirImagem: () => void this.inserirImagem(),
      link: () => caixaDeLink(this.formato?.link ?? null, Boolean(this.formato?.tem_selecao), (url, texto) => this.acao('link', () => this.motor.linkDefinir(url, texto)), () => this.acao('link', () => this.motor.linkRemover())),
      linkRemover: () => this.acao('link', () => this.motor.linkRemover()),
      simbolo: (s) => (s ? this.acao('símbolo', () => this.motor.inserirTexto(s)) : this.barra.abrirSimbolos()),
      cabecalhoRodape: (foco) =>
        caixaDeCabecalho(this.estado.extras, foco, (cab, rod, primeira) =>
          this.aplicarPagina(() => {
            this.motor.cabecalho(cab.esquerda, cab.centro, cab.direita);
            this.motor.primeiraDiferente(primeira);
            return this.motor.rodape(rod.esquerda, rod.centro, rod.direita);
          }),
        ),
      numeroDePagina: (modelo, onde) => this.aplicarPagina(() => this.motor.numeroDePagina(modelo, onde)),
      numeroRemover: () => this.aplicarPagina(() => this.motor.numeroRemover()),
      dataHora: (fm) => this.acao('data', () => this.motor.inserirTexto(dataFormatada(fm))),
      chamada: (tipo) => this.acao('chamada', () => this.motor.chamada(tipo)),
      corPagina: (c) => this.aplicarPagina(() => this.motor.corPagina(c)),
      marcaDagua: (texto) => this.aplicarPagina(() => this.motor.marcaDagua(texto)),
      marcaDaguaPersonalizada: () => pedirTexto(t('faixa.marca_personalizada'), t('faixa.marca_texto'), this.estado.extras.marca, (s) => this.aplicarPagina(() => this.motor.marcaDagua(s))),
      bordaPagina: (w) => this.aplicarPagina(() => (w > 0 ? this.motor.bordaPagina('#5f6368', w, 24) : this.motor.bordaPagina('', 0, 0))),
      inserirSumario: (n) => this.acao('sumário', () => this.motor.inserirSumario(n)),
      removerSumario: () => this.acao('sumário', () => this.motor.removerSumario()),
      contagem: () => caixaDeContagem(this.motor.contagem()),
      tabelaLinha: (abaixo) => this.acao('tabela', () => this.motor.tabelaLinha(abaixo)),
      tabelaColuna: (direita) => this.acao('tabela', () => this.motor.tabelaColuna(direita)),
      tabelaExcluir: (o) => this.acao('tabela', () => this.motor.tabelaExcluir(o)),
      tabelaCabecalho: () => this.acao('tabela', () => this.motor.tabelaCabecalho()),
      tabelaNoFeatherSheet: noApp() ? () => this.editarTabelaNoFeatherSheet() : null,
    });
    this.regua = new Regua({
      margens: (esq, dir) => this.mudarMargens(esq, dir, this.estado.margem_sup, this.estado.margem_inf),
    });
    this.estrutura = new PainelEstrutura((ti) => this.irParaTitulo(ti));
    this.revisao = new PainelDeRevisao({
      irComentario: (id) => this.mostrarComentario(id),
      responder: (id, texto) => this.comAutor(() => (this.acao('comentário', () => this.motor.comentarioMudar(id, { resposta: texto })), this.atualizarRevisao(id))),
      resolver: (id, sim) => (this.acao('comentário', () => this.motor.comentarioMudar(id, { resolvido: sim })), this.atualizarRevisao(id)),
      excluir: (id) => (this.acao('comentário', () => this.motor.comentarioExcluir(id)), this.atualizarRevisao()),
      irBloco: (id) => {
        this.motor.irParaBloco(id);
        this.depois(null, true);
      },
      resolverTodas: (aceitar) => (this.acao('revisão', () => this.motor.revisoesResolver(aceitar, true)), this.atualizarRevisao()),
      fechar: () => {
        this.revisao.definirVisivel(false);
        this.atualizarBarras();
      },
    });
    this.miniaturas = new PainelDeMiniaturas(motor, (p) => this.folhas.irParaPagina(p));
    this.codigo = new PainelDeCodigo({
      aplicar: (f, texto) => this.aplicarCodigo(f, texto),
      irParaBloco: (id) => {
        this.motor.irParaBloco(id);
        this.depois(null, true);
      },
      recarregar: () => this.recarregarCodigo(),
      cores: (f, texto) => this.motor.codigoCores(f, texto),
      fechar: () => this.modoDividido(null),
    });
    this.folhas = new Folhas(motor, {
      aoPressionar: (p, x, y, estender, contagem, ctrl) =>
        medirAcao('clique', () => {
          this.cliqueNaCaixa = false;
          if (ctrl && this.seguirLink(p, x, y)) return;
          const balao = this.motor.comentarioNoPonto(p, x, y);
          if (balao !== null) return this.mostrarComentario(balao);
          if (!estender && contagem === 2) {
            const obj = this.motor.objetoNoPonto(p, x, y);
            if (obj?.marca.tipo === 'nota') return this.editarNota(obj.ponto);
            if (obj?.marca.tipo === 'citacao') return this.abrirCitacao({ ponto: obj.ponto, chaves: obj.marca.chaves, pagina: obj.marca.pagina ?? '' });
            if (obj?.marca.tipo === 'comentario') return this.mostrarComentario(obj.marca.id);
          }
          if (!estender && contagem === 1) {
            const m = this.motor.cliqueNaTarefa(p, x, y);
            if (m) {
              this.cliqueNaCaixa = true;
              this.depois(m, false);
              this.entrada.focar();
              return;
            }
          }
          this.motor.clicar(p, x, y, estender, contagem);
          this.entrada.focar();
          this.depois(null, false);
        }),
      aoArrastar: (p, x, y) => {
        if (this.cliqueNaCaixa) return;
        this.motor.clicar(p, x, y, true, 1);
        this.depois(null, false);
      },
      aoSoltar: () => this.aplicarPincel(),
      aoMenu: (p, x, y, janela) => this.abrirMenuDeContexto(p, x, y, janela),
      aoMudarVisao: () => this.depois(null, false),
      aoMudarPagina: (p) => {
        this.miniaturas.marcar(p);
        this.atualizarBarras();
      },
    });
    this.entrada = new Entrada({
      tecla: (k, ctrl, shift, alt) =>
        medirAcao(`tecla ${k}`, () => {
          const r = this.motor.tecla(k, ctrl, shift, alt);
          if (r.tratada) this.depois(r.mudanca);
          return r.tratada;
        }),
      texto: (x) => medirAcao('texto', () => this.depois(this.motor.inserirTexto(x))),
      copiar: () => this.motor.copiar(),
      recortar: () => {
        const r = this.motor.recortar();
        this.depois(r.mudanca);
        return r.texto;
      },
      colar: (x, sem) => medirAcao('colar', () => this.depois(this.motor.colar(x, sem))),
      colarImagem: (url) => this.acao('imagem', () => this.motor.inserirImagem(url)),
      composicao: (x) => this.folhas.rascunho(x),
      foco: (tem) => this.folhas.definirFoco(tem),
    });
    this.folhas.miolo.append(this.entrada.el);
    this.feathersheet = new LigacaoComFeatherSheet(motor, (m) => this.depois(m), avisar);
    const corpo = document.createElement('div');
    corpo.style.cssText = 'display:flex;flex:1;min-height:0';
    corpo.append(this.estrutura.el, this.folhas.rolagem);
    this.inicio = new PaginaInicial({
      novo: () => void this.novo().then((feito) => feito && this.esconderInicio()),
      abrir: () => void this.abrir().then((feito) => feito && this.esconderInicio()),
      continuar: () => this.esconderInicio(),
      podeContinuar: () => this.jaViuODocumento || this.estado.sujo,
      configurar: () => this.configuracoes.abrir(),
    });
    raiz.style.position = 'relative';
    const miolo = document.createElement('div');
    miolo.style.cssText = 'display:flex;flex-direction:column;flex:1;min-height:0;min-width:0';
    miolo.append(this.regua.el, this.folhas.rolagem);
    corpo.replaceChildren(this.miniaturas.el, this.estrutura.el, this.codigo.el, miolo, this.revisao.el);
    this.busca = new PainelDeBusca({
      definir: (q, o) => this.motor.buscaDefinir(q, o),
      ir: (d) => this.motor.buscaIr(d),
      substituir: (por) => this.motor.substituir(por),
      substituirTudo: (por) => this.motor.substituirTudo(por),
      encerrar: () => {
        this.motor.buscaLimpar();
        this.depois(null, false);
        this.entrada.focar();
      },
      repintar: (m) => this.depois(m),
    });
    miolo.style.position = 'relative';
    miolo.append(this.busca.el);
    raiz.append(this.barra.el, corpo, this.rodape.el, this.inicio.el);
    this.estrutura.definirVisivel(false);
    this.barra.estruturaVisivel(false);
    this.ligarAtalhos();
    this.carregar(this.estado);
    this.contextoDosObjetos();
    aoMudarIdioma(() => {
      this.motor.rotuloDoSumario(t('sumario.rotulo'));
      this.motor.rotuloDoCapitulo(t('capitulo.rotulo'));
      this.contextoDosObjetos();
      this.barra.atualizarIdioma();
      this.inicio.atualizarIdioma();
      this.estrutura.atualizarIdioma();
      this.atualizarBarras();
    });
    this.inicio.mostrar();
  }

  /** O botão direito no papel: o motor acerta o cursor (a seleção fica se o clique caiu nela) e o menu abre ali. */
  private abrirMenuDeContexto(p: number, x: number, y: number, janela: { x: number; y: number }): void {
    medirAcao('botão direito', () => {
      this.motor.clicarDireito(p, x, y);
      this.depois(null, false);
    });
    this.entrada.focar();
    openPopup(janela, menuDeContexto(this.barra.registro, this.formato, noApp()));
  }

  /** O F1: a Ajuda, já no tópico do que se está fazendo (hoje: a tabela sob o cursor). */
  private pedirAjuda(): void {
    abrirAjuda('uso', this.formato?.tabela != null ? 'tabelas' : undefined);
  }

  private mostrarInicio(): void {
    this.inicio.mostrar();
  }

  /** Sai da página inicial só se o documento mudou de fato (cancelar o Abrir deixa tudo como estava). */
  private esconderInicio(): void {
    this.jaViuODocumento = true;
    this.inicio.esconder();
    this.entrada.focar();
  }

  private carregar(e: Estado): void {
    this.estado = e;
    this.folhas.carregar(e.largura, e.altura, e.paginas, e.versao);
    this.recarregarCodigo();
    this.agendarEstrutura();
    this.depois(null, true);
    this.entrada.focar();
  }

  /**
   * Depois de qualquer ação: se o documento mudou, repinta o que mudou; sempre, pergunta ao motor onde está o
   * cursor e a seleção (um comando só) e atualiza a entrada de teclado, a barra e o rodapé.
   */
  private depois(m: Mudanca | null, rolar = true): void {
    if (m && m.afetados.length > 0) {
      this.folhas.invalidar(m);
      this.estado = this.motor.estado();
      this.agendarEstrutura();
      this.atualizarMiniaturas();
      this.atualizarRevisao();
    }
    const { de, ate } = this.folhas.paginasVisiveis();
    const v = this.motor.visao(de, ate);
    this.folhas.definirVisao(v);
    this.formato = v.formato;
    if (this.codigo.visivel) {
      // Mudou pelo papel: o código se refaz logo depois (não enquanto a pessoa digita nele).
      if (m && m.afetados.length > 0 && !this.codigo.editando && !this.editandoCodigo) {
        clearTimeout(this.relogioCodigo);
        this.relogioCodigo = window.setTimeout(() => this.recarregarCodigo(), 120);
      }
      // O código acompanha o bloco do cursor (clique ou digitação no papel); só rola se ele estiver fora de vista.
      this.codigo.marcarBloco(v.formato.bloco, true);
    }
    if (v.cursor) {
      const { x, y } = this.folhas.posicaoEmPx(v.cursor);
      this.entrada.posicionar(x, y, v.cursor.h * this.folhas.escala);
      if (rolar) this.folhas.mostrarCursor(v.cursor);
    }
    this.atualizarBarras();
  }

  /** Uma ação de menu ou botão: o motor muda o documento, o foco volta ao texto. */
  /** Como `acao`, mas o erro volta para a caixa que pediu (que o mostra no lugar). */
  private tentar(nome: string, f: () => Mudanca | null): string | null {
    try {
      medirAcao(nome, () => this.depois(f()));
    } catch (e) {
      return e instanceof Error ? e.message : String(e);
    }
    this.entrada.focar();
    return null;
  }

  /** Duplo clique numa chamada de nota (ou na nota, no pé da página): a caixa com o texto dela. */
  private editarNota(p: Ponto): void {
    const info = this.motor.notaInfo(p);
    caixaDeNota(t('nota.editar', { numero: info.numero }), info, (d) => this.tentar('nota', () => this.motor.notaEditar(p, d.fonte, d.fim)));
  }

  /** A data de hoje, o nome do arquivo e os títulos das listas geradas, que o motor mostra no documento. */
  private contextoDosObjetos(): void {
    const arquivo = this.arquivos.nome !== t('app.sem_titulo') ? this.arquivos.nome : '';
    this.ultimoArquivo = arquivo;
    this.depois(this.motor.objetosContexto({ hoje: dataFormatada('curta'), arquivo, notas: t('nota.titulo_fim'), referencias: t('citacao.titulo_bib'), indice: t('citacao.titulo_indice'), autor: autorGuardado(), comparacao: t('revisao.comparacao') }), false);
  }

  /** Roda `f` depois de saber o nome de quem edita (pergunta uma vez e guarda). */
  private comAutor(f: () => void): void {
    if (autorGuardado()) return f();
    pedirTexto(t('revisao.seu_nome'), t('revisao.nome'), '', (nome) => {
      try {
        localStorage.setItem(CHAVE_AUTOR, nome.trim());
      } catch {
        /* sem armazenamento: vale só nesta sessão, pelo contexto abaixo */
      }
      this.motor.objetosContexto({ hoje: dataFormatada('curta'), arquivo: this.ultimoArquivo, notas: t('nota.titulo_fim'), referencias: t('citacao.titulo_bib'), indice: t('citacao.titulo_indice'), autor: nome.trim(), comparacao: t('revisao.comparacao') });
      f();
    });
  }

  /** O painel de revisão aberto, com o comentário `id` à vista e o trecho dele selecionado. */
  private mostrarComentario(id: number): void {
    this.revisao.definirVisivel(true);
    this.motor.comentarioIr(id);
    this.depois(null, true);
    this.atualizarRevisao(id);
  }

  /** Redesenha o painel de revisão (se estiver aberto). */
  private atualizarRevisao(destaque?: number): void {
    if (this.revisao.visivel) this.revisao.atualizar(this.motor.comentarios(), this.motor.revisoes(), destaque);
  }

  /** Revisão → Comparar: escolhe o outro arquivo e marca as diferenças no documento aberto. */
  private async comparar(): Promise<void> {
    try {
      const doc = await escolherDocumento();
      if (!doc) return;
      this.acao('comparar', () => this.motor.comparar(doc.formato, doc.bytes));
      avisar(t('revisao.comparado'));
      this.revisao.definirVisivel(true);
      this.atualizarRevisao();
    } catch (e) {
      avisar(e instanceof Error ? e.message : String(e), true);
    }
  }

  /** Revisão → Histórico: as versões salvas no .fdoz (só no aplicativo, que tem o arquivo no disco). */
  private async historico(): Promise<void> {
    if (!noApp()) return avisar(t('revisao.so_no_app'));
    try {
      const lista = await versoes();
      if (!lista.length) return avisar(t('revisao.sem_versoes'));
      caixaDeVersoes(
        lista,
        (id) => void abrirVersao(id).then((a) => this.carregar(a.estado), (e: unknown) => avisar(String(e), true)),
        (id) =>
          void compararVersao(id).then(
            (m) => {
              this.depois(m);
              avisar(t('revisao.comparado'));
              this.revisao.definirVisivel(true);
              this.atualizarRevisao();
            },
            (e: unknown) => avisar(String(e), true),
          ),
      );
    } catch (e) {
      avisar(e instanceof Error ? e.message : String(e), true);
    }
  }

  /**
   * Inserir citação (ou corrigir a de `edicao`). Sem fonte cadastrada, abre o cadastro primeiro; a
   * fonte nova volta já marcada na caixa da citação.
   */
  private abrirCitacao(edicao: { ponto: Ponto; chaves: string[]; pagina: string } | null, marcadas: string[] = []): void {
    const fontes = this.motor.bibliografia().fontes;
    const nova = (): void =>
      caixaDeFonte(null, (f) => {
        const erro = this.tentar('fonte', () => this.motor.fonteSalvar(f, ''));
        if (!erro) setTimeout(() => this.abrirCitacao(edicao, [...(edicao?.chaves ?? marcadas), f.chave]));
        return erro;
      });
    if (!fontes.length && !edicao) return nova();
    const inicial = { chaves: marcadas.length ? marcadas : (edicao?.chaves ?? []), pagina: edicao?.pagina ?? '' };
    caixaDeCitacao(t(edicao ? 'citacao.editar' : 'citacao.inserir'), fontes, inicial, (chaves, pagina) => this.tentar('citação', () => (edicao ? this.motor.citacaoEditar(edicao.ponto, chaves, pagina) : this.motor.citacaoInserir(chaves, pagina))), nova);
  }

  /** Referências → Gerenciar fontes. */
  private gerenciarFontes(): void {
    caixaDeFontes({
      lista: () => this.motor.bibliografia(),
      salvar: (f, antiga) => this.tentar('fonte', () => this.motor.fonteSalvar(f, antiga)),
      remover: (chave) => this.acao('fonte', () => this.motor.fonteRemover(chave)),
      importar: async () => {
        const texto = await abrirTexto('BibTeX (.bib)', 'bib');
        if (texto === null) return null;
        try {
          const r = this.motor.fontesImportarBib(texto);
          this.depois(r.mudanca);
          return t('fonte.importadas', { n: r.n });
        } catch (e) {
          return e instanceof Error ? e.message : String(e);
        }
      },
      exportar: () => void salvarTexto(`${this.arquivos.nome}.bib`, 'BibTeX (.bib)', 'bib', this.motor.fontesExportarBib()).catch((e: unknown) => avisar(String(e), true)),
    });
  }

  private acao(nome: string, f: () => Mudanca | null): void {
    try {
      medirAcao(nome, () => this.depois(f()));
    } catch (e) {
      avisar(e instanceof Error ? e.message : String(e), true);
    }
    this.entrada.focar();
  }

  /** As miniaturas acompanham o documento (só pintam se estiverem abertas). */
  private atualizarMiniaturas(): void {
    this.miniaturas?.atualizar(this.estado.paginas, this.estado.versao, this.estado.largura, this.estado.altura);
  }

  /** Exibir → Contínuo / Várias páginas (um desliga o outro). */
  private arrumarFolhas(continuo: boolean, varias: boolean): void {
    this.folhas.arrumar(continuo, varias);
    this.atualizarBarras();
    this.entrada.focar();
  }

  /** Ctrl+clique num link interno (referência cruzada) leva ao alvo. `false` se ali não há link interno. */
  private seguirLink(p: number, x: number, y: number): boolean {
    this.motor.clicar(p, x, y, false, 1);
    const link = this.motor.visao(p, p).formato.link;
    const m = link ? /^#(?:ref:(\d+):|bloco:(\d+))/.exec(link) : null;
    if (!m) return false;
    this.motor.irParaBloco(Number(m[1] ?? m[2]));
    this.depois(null, true);
    this.entrada.focar();
    return true;
  }

  private abrirMarcadores(): void {
    const marcadores = this.motor.alvosDeReferencia().filter((a) => a.tipo === 'marcador');
    caixaDeMarcador(
      marcadores,
      (nome) => {
        try {
          medirAcao('marcador', () => this.depois(this.motor.marcadorAdicionar(nome)));
        } catch (e) {
          return e instanceof Error ? e.message : String(e);
        }
        this.entrada.focar();
        return null;
      },
      (id) => {
        this.motor.irParaBloco(id);
        this.depois(null, true);
        this.entrada.focar();
      },
      (nome) => this.acao('marcador', () => this.motor.marcadorRemover(nome)),
    );
  }

  private abrirReferencia(): void {
    caixaDeReferencia(this.motor.alvosDeReferencia(), (id, modo) => {
      try {
        medirAcao('referência', () => this.depois(this.motor.referenciaInserir(id, modo)));
      } catch (e) {
        return e instanceof Error ? e.message : String(e);
      }
      this.entrada.focar();
      return null;
    });
  }

  /** Inserir → Imagem → Da web: o endereço fica como origem; o tamanho vem da própria imagem. */
  private inserirImagemDeUrl(): void {
    pedirTexto(t('cmd.imagem_url'), t('faixa.imagem_url_pergunta'), 'https://', (url) => {
      const img = new Image();
      img.onload = () => this.acao('imagem', () => this.motor.inserirImagem(url.trim(), img.naturalWidth * 0.75, img.naturalHeight * 0.75));
      img.onerror = () => avisar(t('faixa.imagem_url_falhou'), true);
      img.src = url.trim();
    });
  }

  /** Liga o modo dividido (o código em Markdown ou LaTeX à esquerda) ou, com `null`, desliga. */
  private modoDividido(f: FormatoDoCodigo | null): void {
    if (f === null) this.codigo.esconder();
    else this.codigo.mostrar(f, this.motor.codigoExportar(f));
    this.depois(null, false);
    this.entrada.focar();
  }

  /** O código editado vai ao motor: só os blocos que mudaram são trocados. O foco fica no código. */
  private aplicarCodigo(f: FormatoDoCodigo, texto: string): void {
    try {
      this.editandoCodigo = true;
      medirAcao('código', () => this.depois(this.motor.codigoAplicar(f, texto), false));
    } catch (e) {
      avisar(e instanceof Error ? e.message : String(e), true);
    } finally {
      this.editandoCodigo = false;
    }
  }

  /** O código de novo, na forma do motor (o documento mudou pelo papel, ou o código saiu de foco). */
  private recarregarCodigo(): void {
    clearTimeout(this.relogioCodigo);
    if (!this.codigo.visivel) return;
    medirAcao('código (refazer)', () => this.codigo.carregar(this.motor.codigoExportar(this.codigo.formato)));
    if (this.formato) this.codigo.marcarBloco(this.formato.bloco, false);
  }

  /** Uma ação que mexe nos estilos com nome ou na numeração: mesmo sem bloco tocado, a galeria e os menus mudam. */
  private acaoDeEstilo(nome: string, f: () => Mudanca): void {
    this.acao(nome, f);
    this.estado = this.motor.estado();
    this.atualizarBarras();
  }

  /** "Novo estilo" (`id` 0, preenchido com o parágrafo do cursor) ou "Modificar estilo". O erro do motor aparece na caixa. */
  private abrirCaixaDeEstilo(id: number): void {
    const e = id ? this.estado.estilos.lista.find((x) => x.id === id) : undefined;
    if (id && !e) return;
    const titulo = t(e ? 'estilo.modificar_titulo' : 'estilo.novo_titulo');
    caixaDeEstilo(titulo, e ? dadosDoEstilo(e) : dadosDoCursor(this.formato), (d) => {
      try {
        medirAcao(e ? 'modificar estilo' : 'novo estilo', () => this.depois(e ? this.motor.estiloModificar(e.id, d) : this.motor.estiloCriar(d)));
      } catch (err) {
        return err instanceof Error ? err.message : String(err);
      }
      this.estado = this.motor.estado();
      this.atualizarBarras();
      this.entrada.focar();
      return null;
    });
  }

  private excluirEstilo(id: number): void {
    const e = this.estado.estilos.lista.find((x) => x.id === id);
    if (!e) return;
    let sim = true;
    try {
      sim = confirm(t('estilo.excluir_confirmar', { nome: e.nome }));
    } catch {
      // sem caixa de confirmação (testes): segue
    }
    if (sim) this.acaoDeEstilo('excluir estilo', () => this.motor.estiloExcluir(id));
    else this.entrada.focar();
  }

  /** Liga ou desliga o modo foco; um botão discreto (e o Esc) volta. */
  private alternarFoco(): void {
    this.foco = !this.foco;
    this.raiz.classList.toggle('foco-ligado', this.foco);
    if (this.foco) {
      const b = document.createElement('button');
      b.type = 'button';
      b.className = 'sair-foco';
      b.textContent = t('faixa.sair_foco');
      b.addEventListener('click', () => this.alternarFoco());
      document.body.append(b);
      this.sairDoFoco = b;
    } else {
      this.sairDoFoco?.remove();
      this.sairDoFoco = null;
    }
    this.atualizarBarras();
    this.entrada.focar();
  }

  /** Inserir → Imagem: escolhe o arquivo e manda ao motor com o tamanho natural. */
  private async inserirImagem(): Promise<void> {
    const img = await escolherImagem();
    if (img) this.acao('imagem', () => this.motor.inserirImagem(img.origem, img.largura, img.altura));
    else this.entrada.focar();
  }

  /** O botão Pincel: guarda a formatação do cursor e espera a próxima seleção. */
  private armarPincel(persistente: boolean): void {
    this.motor.pincelCopiar();
    this.pincel = persistente ? 'sempre' : 'uma';
    this.folhas.rolagem.style.cursor = 'copy';
    this.depois(null, false);
    this.entrada.focar();
  }

  private desarmarPincel(): void {
    this.pincel = null;
    this.folhas.rolagem.style.cursor = '';
    this.depois(null, false);
  }

  /** Soltou o mouse: com o pincel armado e algo selecionado, pinta. */
  private aplicarPincel(): void {
    this.cliqueNaCaixa = false;
    if (!this.pincel || !this.formato?.tem_selecao) return;
    this.depois(this.motor.pincelAplicar());
    if (this.pincel === 'uma') this.desarmarPincel();
  }

  /** Abre o Localizar (ou o Localizar e Substituir) com o texto selecionado já na consulta. */
  private abrirBusca(comSubstituir: boolean): void {
    this.busca.abrir(comSubstituir, this.formato?.tem_selecao ? this.motor.copiar() : '');
  }

  /** A entrelinha personalizada: uma janelinha com o número. */
  private pedirEntrelinha(): void {
    const d: Dialogo = new Dialogo(t('faixa.entrelinha'), rotuloFechar(), (corpo) => {
      const rotulo = document.createElement('label');
      rotulo.textContent = t('faixa.entrelinha_pergunta');
      rotulo.style.display = 'block';
      const campo = document.createElement('input');
      campo.type = 'text';
      campo.value = String(this.formato?.entrelinha ?? 1.15).replace('.', ',');
      campo.style.cssText = 'margin:8px 8px 0 0;padding:4px 8px';
      const ok = document.createElement('button');
      ok.type = 'button';
      ok.textContent = t('faixa.aplicar');
      const aplicar = (): void => {
        const n = Number.parseFloat(campo.value.replace(',', '.'));
        d.fechar();
        if (Number.isFinite(n)) this.acao('entrelinha', () => this.motor.entrelinha(n));
        else this.entrada.focar();
      };
      ok.addEventListener('click', aplicar);
      campo.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') aplicar();
      });
      corpo.append(rotulo, campo, ok);
      setTimeout(() => campo.select());
    });
    d.abrir();
  }

  private atualizarBarras(): void {
    this.barra.atualizar(this.estado, this.formato);
    this.rodape.atualizar(this.estado, this.folhas.paginaAtual);
    this.atualizarRegua();
    document.title = `${this.estado.sujo ? '• ' : ''}${this.arquivos.nome} — ${t('app.nome')}`;
    const arquivo = this.arquivos.nome !== t('app.sem_titulo') ? this.arquivos.nome : '';
    if (arquivo !== this.ultimoArquivo) queueMicrotask(() => this.contextoDosObjetos());
  }

  private atualizarRegua(): void {
    const e = this.estado;
    const rol = this.folhas.rolagem.getBoundingClientRect();
    const mio = this.folhas.miolo.getBoundingClientRect();
    this.regua.atualizar({
      xDaPagina: mio.left - rol.left,
      escala: this.folhas.escala,
      larguraPt: e.largura,
      margemEsq: e.margem_esq,
      margemDir: e.margem_dir,
      larguraVisivel: this.folhas.rolagem.clientWidth,
    });
  }

  /** Margens novas (pontos); o motor recusa as que não deixam espaço, e o aviso diz por quê. */
  private mudarMargens(esq: number, dir: number, sup: number, inf: number): void {
    try {
      const m = this.motor.margens(esq, dir, sup, inf);
      this.estado = this.motor.estado();
      this.folhas.papel(this.estado.largura, this.estado.altura, m);
      this.depois(null, true);
      this.agendarEstrutura();
    } catch (err) {
      avisar(t('regua.margens_falhou', { erro: err instanceof Error ? err.message : String(err) }), true);
      this.atualizarRegua();
    }
    this.entrada.focar();
  }

  /** Recortar, copiar e colar pelos menus e botões (as teclas passam pela entrada de teclado). */
  private async recortarCopiarColar(que: 'recortar' | 'copiar' | 'colar' | 'colar_sem'): Promise<void> {
    try {
      if (que === 'copiar') await navigator.clipboard.writeText(this.motor.copiar());
      else if (que === 'recortar') {
        const r = this.motor.recortar();
        await navigator.clipboard.writeText(r.texto);
        this.depois(r.mudanca);
      } else this.depois(this.motor.colar(await navigator.clipboard.readText(), que === 'colar_sem'));
    } catch {
      avisar(t('faixa.area_falhou'), true);
    }
    this.entrada.focar();
  }

  private agendarEstrutura(): void {
    if (!this.estrutura.visivel) return;
    clearTimeout(this.relogioEstrutura);
    this.relogioEstrutura = window.setTimeout(() => this.estrutura.atualizar(this.motor.estrutura()), 250);
  }

  private alternarEstrutura(): void {
    const v = !this.estrutura.visivel;
    this.estrutura.definirVisivel(v);
    this.barra.estruturaVisivel(v);
    if (v) this.estrutura.atualizar(this.motor.estrutura());
  }

  private irParaTitulo(ti: Titulo): void {
    this.motor.irParaBloco(ti.id);
    this.depois(null, true);
    this.entrada.focar();
  }

  /** A tabela onde está o cursor, aberta no FeatherSheet (volta ao motor quando salvarem lá). */
  private editarTabelaNoFeatherSheet(): void {
    const id = this.formato?.tabela;
    if (id != null) void this.feathersheet.editar(id);
  }

  private desfazerRefazer(refazer: boolean): void {
    this.acao(refazer ? 'refazer' : 'desfazer', () => (refazer ? this.motor.refazer() : this.motor.desfazer()));
  }

  private trocarPapel(p: Papel): void {
    this.aplicarPagina(() => this.motor.papel(p));
  }

  /** Uma mudança de página (papel, orientação): o motor repagina e tudo se repinta; o erro vira aviso. */
  private aplicarPagina(f: () => Mudanca): void {
    try {
      const m = f();
      this.estado = this.motor.estado();
      this.folhas.papel(this.estado.largura, this.estado.altura, m);
      this.depois(null, true);
      this.agendarEstrutura();
    } catch (err) {
      avisar(err instanceof Error ? err.message : String(err), true);
    }
    this.entrada.focar();
  }

  private pedirPapel(): void {
    const cm = (pt: number): number => pt / PT_POR_CM;
    pedirNumeros(t('faixa.papel_personalizado'), [
      { rotulo: t('faixa.largura_cm'), valor: cm(this.estado.largura) },
      { rotulo: t('faixa.altura_cm'), valor: cm(this.estado.altura) },
    ], ([l, a]) => this.aplicarPagina(() => this.motor.papelPersonalizado(l! * PT_POR_CM, a! * PT_POR_CM)));
  }

  private pedirMargens(): void {
    const e = this.estado;
    const cm = (pt: number): number => pt / PT_POR_CM;
    pedirNumeros(t('faixa.margens_personalizadas'), [
      { rotulo: t('faixa.margem_sup'), valor: cm(e.margem_sup) },
      { rotulo: t('faixa.margem_inf'), valor: cm(e.margem_inf) },
      { rotulo: t('faixa.margem_esq'), valor: cm(e.margem_esq) },
      { rotulo: t('faixa.margem_dir'), valor: cm(e.margem_dir) },
    ], ([sup, inf, esq, dir]) => this.mudarMargens(esq! * PT_POR_CM, dir! * PT_POR_CM, sup! * PT_POR_CM, inf! * PT_POR_CM));
  }

  private async confirmarDescarte(): Promise<boolean> {
    if (!this.estado.sujo) return true;
    try {
      return confirm(t('arquivo.confirmar_descartar'));
    } catch {
      return true;
    }
  }

  /** Roda uma ação de arquivo; erro vira aviso em português (o motor já escreve a mensagem). */
  private async comAviso(chave: string, f: () => Promise<void>): Promise<void> {
    try {
      await f();
    } catch (e) {
      avisar(formatar(t(chave), { erro: e instanceof Error ? e.message : String(e) }), true);
    }
  }

  /** `true` se criou o documento novo (`false` se o usuário desistiu de descartar o atual). */
  private async novo(modo: 'convencional' | 'markdown' | 'latex' = 'convencional'): Promise<boolean> {
    if (!(await this.confirmarDescarte())) return false;
    await this.arquivos.novo();
    if (modo === 'latex') {
      const template = `\\documentclass{article}
\\usepackage[utf8]{inputenc}
\\title{Documento}
\\author{Autor}
\\begin{document}
\\maketitle

\\section{Introdução}
Texto inicial.

\\end{document}
`;
      this.carregar(this.motor.abrir('tex', new TextEncoder().encode(template)));
      this.modoDividido('tex');
    } else if (modo === 'markdown') {
      const template = `# Documento\n\nTexto inicial.\n`;
      this.carregar(this.motor.abrir('md', new TextEncoder().encode(template)));
      this.modoDividido('md');
    } else {
      this.modoDividido(null);
      this.carregar(this.motor.novo());
    }
    return true;
  }

  /** `true` se abriu um arquivo (`false` se cancelou ou falhou). */
  private async abrir(): Promise<boolean> {
    if (!(await this.confirmarDescarte())) return false;
    let feito = false;
    await this.comAviso('arquivo.abrir_falhou', async () => {
      const aberto = await this.arquivos.abrir();
      if (!aberto) return;
      this.carregar(aberto.estado);
      const ext = aberto.nome.split('.').pop()?.toLowerCase();
      // O motor guardou o texto do arquivo como está e devolve com o mapa de blocos → linhas (o clique no papel acha a linha).
      if (ext === 'tex' || ext === 'latex') {
        this.modoDividido('tex');
      } else if (ext === 'md' || ext === 'markdown') {
        this.modoDividido('md');
      } else {
        this.modoDividido(null);
      }
      feito = true;
    });
    return feito;
  }

  /**
   * Salva nos formatos da vertente: convencional em `.fdoz`/`.docx`, código em `.tex`/`.md`.
   * LaTeX salvo como Markdown passa pelo aviso do que se perde e, aceito, o documento vira Markdown.
   */
  private async salvar(como: boolean): Promise<void> {
    await this.comAviso('arquivo.salvar_falhou', async () => {
      let texto: string | undefined;
      let formato: 'md' | 'tex' | undefined;
      if (this.codigo.visivel) {
        texto = this.codigo.textoAtual;
        formato = this.codigo.formato;
        this.aplicarCodigo(this.codigo.formato, texto);
      }
      let salvo = await this.arquivos.salvar(como, texto, formato);
      if (salvo?.perdas?.length) {
        if (!(await confirmarConversao(salvo.perdas))) return;
        salvo = await this.arquivos.confirmar(salvo, texto, formato);
      }
      if (!salvo) return;
      this.estado = this.motor.marcarGravado();
      if ((salvo.formato === 'md' || salvo.formato === 'tex') && this.codigo.visivel && this.codigo.formato !== salvo.formato) {
        this.modoDividido(salvo.formato);
      }
      this.atualizarBarras();
      avisar(t('arquivo.salvo'));
    });
  }

  private async exportar(formato: 'md' | 'tex' | 'docx'): Promise<void> {
    await this.comAviso('arquivo.salvar_falhou', async () => {
      let texto: string | undefined;
      if (formato !== 'docx' && this.codigo.visivel && this.codigo.formato === formato) {
        texto = this.codigo.textoAtual;
        this.aplicarCodigo(formato, texto);
      }
      const perdas = this.motor.perdas(formato);
      if (perdas.length && !(await confirmarConversao(perdas))) return;
      const nome = await this.arquivos.exportar(formato, texto);
      if (nome) avisar(t('arquivo.exportado', { nome }));
    });
  }

  /** Ctrl+K (link), Ctrl+Shift+8 / Ctrl+* (marcas), Ctrl+Shift+G (contagem): comandos da faixa com atalho. */
  private atalhoDaFaixa(k: string, shift: boolean, alt: boolean): (() => void) | null {
    if (alt) return null;
    const id = k === 'k' && !shift ? 'link' : k === '*' || (k === '8' && shift) ? 'marcas' : k === 'g' && shift ? 'contagem' : null;
    const c = id ? this.barra.registro.get(id) : undefined;
    return c?.executar ?? null;
  }

  private ligarAtalhos(): void {
    document.addEventListener('keydown', (e) => {
      if (e.key === 'F1' && e.ctrlKey && !e.altKey && !e.shiftKey) {
        e.preventDefault();
        this.barra.alternarFaixa();
        return;
      }
      if (e.key === 'F9' && !e.ctrlKey && !e.altKey && !e.shiftKey) {
        e.preventDefault();
        this.acao('referências', () => this.motor.referenciasAtualizar());
        return;
      }
      if (e.key === 'F1' && !e.ctrlKey && !e.altKey && !e.shiftKey) {
        e.preventDefault();
        this.pedirAjuda();
        return;
      }
      if (e.key === 'Escape') {
        if (this.foco) this.alternarFoco();
        if (this.pincel) this.desarmarPincel();
        if (this.configuracoes.aberto) this.configuracoes.fechar();
        else if (this.inicio.visivel && this.jaViuODocumento) this.esconderInicio();
        return;
      }
      const ctrl = e.ctrlKey || e.metaKey;
      if (!ctrl) return;
      const k = e.key.toLowerCase();
      if ((k === 'f' || k === 'h') && !e.altKey && !e.shiftKey && !this.inicio.visivel) {
        e.preventDefault();
        this.abrirBusca(k === 'h');
        return;
      }
      const atalho = this.atalhoDaFaixa(k, e.shiftKey, e.altKey);
      if (atalho && !this.inicio.visivel) {
        e.preventDefault();
        atalho();
        return;
      }
      const acao: (() => void) | undefined = k === 's' ? () => void this.salvar(e.shiftKey) : k === 'o' ? () => void this.abrir() : k === 'n' ? () => void this.novo() : undefined;
      if (acao) {
        e.preventDefault();
        acao();
      }
    });
    window.addEventListener('beforeunload', (e) => {
      if (this.estado.sujo) e.preventDefault();
    });
  }
}

/** Monta a janela do FeatherDocs em `raiz`. */
export function iniciarApp(raiz: HTMLElement, motor: Motor): void {
  new App(raiz, motor);
}
