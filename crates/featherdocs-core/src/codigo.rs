//! O modo dividido (código | páginas): o que o código editado mandou vira **uma** troca de blocos,
//! um passo de desfazer. Quem casa o código com os blocos de antes é `featherdocs-io::dividido`;
//! aqui só se aplica a lista nova, trocando o mínimo (o começo e o fim iguais ficam onde estão).

use crate::alvo::primeiro_alvo;
use crate::cursor::{Cursor, Pos};
use crate::modelo::{Bloco, Conteudo, Meta, Numeracao};
use crate::sessao::{Mudanca, Sessao, Splice};

impl Sessao {
    /// Troca os blocos do documento por `novos`. Bloco com `id` 0 é novo (ganha um `id`); os outros
    /// são blocos de antes, que voltam com o mesmo `id`. O cursor fica onde estava se o bloco dele
    /// continua; senão vai para o começo da parte trocada. Também acerta título/autor e a numeração.
    pub fn trocar_blocos(&mut self, mut novos: Vec<Bloco>, meta: Option<Meta>, numeracao: Option<Numeracao>) -> Mudanca {
        if novos.is_empty() {
            novos.push(Bloco::novo(0, Conteudo::Paragrafo { texto: Vec::new() }));
        }
        for b in &mut novos {
            if b.id == 0 {
                b.id = self.doc.proximo_id;
                self.doc.proximo_id += 1;
            }
        }
        if let Some(m) = meta {
            self.doc.meta = m;
        }
        let antes = self.doc.estilos.clone();
        let numeracao_mudou = numeracao.is_some_and(|n| n != self.doc.estilos.numeracao);
        if let Some(n) = numeracao {
            self.doc.estilos.numeracao = n;
        }
        let velhos = &self.doc.blocos;
        let prefixo = velhos.iter().zip(&novos).take_while(|(a, b)| a == b).count();
        let maximo = velhos.len().min(novos.len()) - prefixo;
        let sufixo = velhos.iter().rev().zip(novos.iter().rev()).take(maximo).take_while(|(a, b)| a == b).count();
        if prefixo == velhos.len() && prefixo == novos.len() && !numeracao_mudou {
            return self.sem_mudanca();
        }
        let quantos = velhos.len() - prefixo - sufixo;
        let dentro: Vec<Bloco> = novos[prefixo..novos.len() - sufixo].to_vec();
        let cursor = self.cursor_depois_da_troca(prefixo, quantos, &dentro);
        let s = Splice { pos: prefixo, quantos, dentro };
        if numeracao_mudou {
            self.transacao_de_estilos(s, cursor, antes)
        } else {
            self.transacao(s, cursor, None)
        }
    }

    /// O cursor continua no bloco dele se o bloco sobreviveu; senão, no começo do primeiro bloco novo.
    fn cursor_depois_da_troca(&self, pos: usize, quantos: usize, dentro: &[Bloco]) -> Cursor {
        let sumiu = |id: u64| self.doc.blocos[pos..pos + quantos].iter().any(|b| b.id == id) && !dentro.iter().any(|b| b.id == id);
        if !sumiu(self.cursor.foco.bloco) && !sumiu(self.cursor.ancora.bloco) {
            return self.cursor;
        }
        let alvo = dentro.first().or_else(|| self.doc.blocos.get(pos + quantos)).or_else(|| pos.checked_sub(1).and_then(|i| self.doc.blocos.get(i)));
        match alvo {
            Some(b) => Cursor::em(Pos { bloco: b.id, celula: primeiro_alvo(&b.conteudo).flatten(), desl: 0 }),
            None => self.cursor,
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::inline::analisar;
    use crate::modelo::Documento;

    fn doc(textos: &[&str]) -> Sessao {
        let mut d = Documento::vazio();
        for t in textos {
            d.acrescentar(Conteudo::Paragrafo { texto: analisar(t) });
        }
        Sessao::com_documento(d)
    }

    #[test]
    fn troca_so_o_miolo_e_desfaz_num_passo() {
        let mut s = doc(&["a", "b", "c"]);
        let mut novos = s.documento().blocos.clone();
        novos[1] = Bloco::novo(0, Conteudo::Paragrafo { texto: analisar("B") });
        novos.insert(2, Bloco::novo(0, Conteudo::Paragrafo { texto: analisar("b2") }));
        let m = s.trocar_blocos(novos, None, None);
        let textos: Vec<String> = s.documento().blocos.iter().map(|b| crate::inline::texto_puro(crate::edicao::trechos_de(&b.conteudo).unwrap())).collect();
        assert_eq!(textos, ["a", "B", "b2", "c"]);
        assert_eq!(m.afetados.len(), 3, "o b que saiu e os dois que entraram");
        assert_eq!(s.documento().blocos[0].id, 1, "o começo igual fica com o mesmo id");
        s.desfazer();
        assert_eq!(s.documento().blocos.len(), 3);
    }

    #[test]
    fn nada_mudou_nada_acontece() {
        let mut s = doc(&["a", "b"]);
        let m = s.trocar_blocos(s.documento().blocos.clone(), None, None);
        assert!(m.afetados.is_empty());
        assert!(!s.estado().pode_desfazer);
    }

    #[test]
    fn codigo_vazio_deixa_um_paragrafo() {
        let mut s = doc(&["a"]);
        s.trocar_blocos(Vec::new(), None, None);
        assert_eq!(s.documento().blocos.len(), 1);
    }

    #[test]
    fn numeracao_vinda_do_codigo_volta_no_desfazer() {
        let mut s = doc(&["a"]);
        s.trocar_blocos(s.documento().blocos.clone(), None, Some(Numeracao::Decimal));
        assert_eq!(s.documento().estilos.numeracao, Numeracao::Decimal);
        s.desfazer();
        assert_eq!(s.documento().estilos.numeracao, Numeracao::Nenhuma);
    }
}
