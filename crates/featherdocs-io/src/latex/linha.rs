//! Comandos em linha dos pacotes mais usados (`siunitx`, `\text`, `\textsc`, `\fontsize`, caixas…).

use featherdocs_core::modelo::{Marca, Trecho};

use super::{grupo, opcional, Inline};

impl Inline<'_> {
    /// Comandos em linha além dos básicos; devolve quantos bytes consumiu depois do nome.
    pub(super) fn extra(&mut self, nome: &str, depois: &str) -> Option<usize> {
        if let Some((t, k)) = super::unidades::ler(nome, depois) {
            self.texto(&t, 0);
            return Some(k);
        }
        match nome {
            // A data de hoje e o total de páginas são campos: o FeatherDocs os preenche ao compor.
            "today" => {
                self.campo("data");
                Some(0)
            }
            "pageref" => {
                let (g, k) = grupo(depois)?;
                if g.trim() != "LastPage" {
                    return None;
                }
                self.campo("paginas");
                Some(k)
            }
            // `csquotes`: as aspas do idioma (português: “…”, e ‘…’ dentro delas).
            "enquote" | "textquote" => {
                let k0 = opcional(depois).map_or(0, |(_, n)| n);
                let (arg, k) = grupo(&depois[k0..])?;
                let (abre, fecha) = if self.aspas > 0 { ("‘", "’") } else { ("“", "”") };
                self.texto(abre, 0);
                self.aspas += 1;
                self.grupo(arg);
                self.aspas -= 1;
                self.texto(fecha, 0);
                Some(k0 + k)
            }
            // `\hyperref[rótulo]{texto}`: link para o rótulo, com o texto que a pessoa escreveu.
            "hyperref" => {
                let (alvo, k0) = opcional(depois)?;
                let (texto, k) = grupo(&depois[k0..])?;
                self.com_link(&format!("#rotulo:{}", alvo.trim()), texto);
                Some(k0 + k)
            }
            // `\parbox[pos]{largura}{texto}` no meio do texto: só o texto.
            "parbox" => {
                let mut k = 0;
                while let Some((_, n)) = opcional(&depois[k..]) {
                    k += n;
                }
                let (_, a) = grupo(&depois[k..])?;
                let (arg, b) = grupo(&depois[k + a..])?;
                self.grupo(arg);
                Some(k + a + b)
            }
            // `\fontsize{12}{14}\selectfont`: o corpo vale até o fim do grupo (a entrelinha não).
            "fontsize" => {
                let (corpo, a) = grupo(depois)?;
                let (_, b) = grupo(&depois[a..])?;
                if let Ok(pt) = corpo.trim().trim_end_matches("pt").trim().parse::<f32>() {
                    self.e.tamanho = pt;
                }
                Some(a + b)
            }
            // Texto como está (`amsmath`, `\mbox`): o argumento sai no estilo em volta.
            "text" | "mbox" | "textrm" | "textnormal" | "textup" | "makebox" => {
                let (arg, k) = grupo(depois)?;
                self.grupo(arg);
                Some(k)
            }
            // `soul`: `\sethlcolor{cor}` vale para os `\hl{…}` seguintes do grupo.
            "sethlcolor" => {
                let (nome, k) = grupo(depois)?;
                self.e.cor_do_realce = super::resolver_cor(&self.ctx.cores, nome.trim()).unwrap_or(0);
                Some(k)
            }
            "hl" => {
                let (arg, k) = grupo(depois)?;
                let antes = self.e;
                self.e.realce = if self.e.cor_do_realce != 0 { self.e.cor_do_realce } else { 0xFFFF00FF };
                self.corre(arg);
                self.e = antes;
                Some(k)
            }
            "textsuperscript" | "textsubscript" => {
                let (arg, k) = grupo(depois)?;
                let antes = self.e;
                self.e.posicao = if nome == "textsuperscript" { 1 } else { -1 };
                self.corre(arg);
                self.e = antes;
                Some(k)
            }
            _ => None,
        }
    }

    fn campo(&mut self, campo: &str) {
        self.saida.push(Trecho { estilo: self.e.estilo | self.ctx.classe.familia, ..Trecho::objeto(Marca::Campo { campo: campo.into() }) });
    }
}
