//! Preferências locais da interface; nunca fazem parte do documento.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Configuracao {
    pub escuro: bool,
    /// Segue o tema do sistema operacional quando o aplicativo é iniciado.
    pub tema_auto: bool,
    pub acento: usize,
    /// None = preferência legada; Some(false) restaura a logo de fábrica
    /// sem alterar a cor de destaque dos controles do aplicativo.
    pub marca_personalizada: Option<bool>,
    pub estrutura: bool,
    /// Ctrl+F1 alterna a faixa, preservado ao reabrir.
    pub faixa_recolhida: bool,
    /// Desenhar minimizar/maximizar/fechar como no FeatherSlide (na próxima abertura).
    pub barra_propria: bool,
    pub zoom: f32,
    /// Arquivos abertos ou salvos por último, o mais recente primeiro.
    pub recentes: Vec<PathBuf>,
}
impl Default for Configuracao {
    fn default() -> Self {
        Self {
            escuro: true,
            tema_auto: false,
            acento: 0,
            marca_personalizada: None,
            estrutura: true,
            faixa_recolhida: false,
            barra_propria: true,
            zoom: 1.0,
            recentes: Vec::new(),
        }
    }
}
fn caminho() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("featherdocs/config.json"))
}
impl Configuracao {
    /// Antes desta preferência existir, acento diferente do padrão era escolha manual.
    pub fn logo_usa_acento(&self) -> bool {
        self.marca_personalizada.unwrap_or(self.acento != 0)
    }

    /// O mesmo caminho de leitura do app, com arquivo explícito para
    /// testar persistência real sem interferir no HOME do usuário.
    fn carregar_de(path: &std::path::Path) -> Self {
        let mut c: Self = std::fs::read(path).ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        if !c.zoom.is_finite() { c.zoom = 1.0; }
        c.zoom = c.zoom.clamp(0.5, 2.0);
        c.acento = c.acento.min(feather_ui::tema::ACENTOS.len() - 1);
        c
    }
    fn gravar_em(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(pasta) = path.parent() {
            std::fs::create_dir_all(pasta)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?)
    }
    /// Põe `p` no topo dos recentes (sem repetir, no máximo 10) e grava.
    pub fn lembrar(&mut self, p: &std::path::Path) {
        self.recentes.retain(|r| r != p);
        self.recentes.insert(0, p.to_path_buf());
        self.recentes.truncate(10);
        self.gravar();
    }
    pub fn carregar() -> Self {
        caminho().map(|p| Self::carregar_de(&p)).unwrap_or_default()
    }
    #[cfg(not(test))]
    pub fn gravar(&self) {
        if let Some(p) = caminho() { let _ = self.gravar_em(&p); }
    }
    #[cfg(test)]
    pub fn gravar(&self) {}
}

#[cfg(test)]
mod testes_ui010 {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static CONTADOR: AtomicU64 = AtomicU64::new(0);
    fn arquivo_teste() -> PathBuf {
        let i = CONTADOR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("featherdocs-ui010-{}-{i}", std::process::id()))
            .join("config.json")
    }

    #[test]
    fn marca_padrao_vs_acento_manual_preserva_preferencias_antigas() {
        let mut c=Configuracao::default();
        assert!(!c.logo_usa_acento());
        c.acento=2; // Preferência legada de uma cor escolhida.
        assert!(c.logo_usa_acento());
        c.marca_personalizada=Some(false);
        assert!(!c.logo_usa_acento()); // Restaurar apenas a logo.
        c.marca_personalizada=Some(true);
        assert!(c.logo_usa_acento()); // Acento explícito, mesmo índice 0.
        c.acento=0;
        assert!(c.logo_usa_acento());
    }
    #[test]
    fn preferencias_zoom_estrutura_e_ribbon_persistem_em_arquivo_isolado() {
        let p = arquivo_teste();
        let conf = Configuracao {
            escuro: false, tema_auto: true, acento: 2, marca_personalizada: Some(true), estrutura: false,
            faixa_recolhida: true, barra_propria: false, zoom: 1.75,
            recentes: vec![PathBuf::from("/tmp/a.fdoz")],
        };
        conf.gravar_em(&p).unwrap();
        let lida = Configuracao::carregar_de(&p);
        assert!(!lida.escuro && lida.tema_auto && !lida.estrutura && lida.faixa_recolhida);
        assert!(!lida.barra_propria);
        assert_eq!(lida.acento, 2);
        assert!((lida.zoom - 1.75).abs() < 0.00001);
        assert_eq!(lida.recentes, conf.recentes);
        std::fs::remove_dir_all(p.parent().unwrap()).unwrap();
    }

    #[test]
    fn preferencias_antigas_e_corrompidas_preservam_limites() {
        let p = arquivo_teste();
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, br#"{"zoom":100.0,"acento":99999,"escuro":false}"#).unwrap();
        let lida = Configuracao::carregar_de(&p);
        assert!(!lida.escuro);
        assert!(!lida.tema_auto);
        assert_eq!(lida.zoom, 2.0);
        assert_eq!(lida.acento, feather_ui::tema::ACENTOS.len()-1);
        assert_eq!(lida.faixa_recolhida, Configuracao::default().faixa_recolhida);
        std::fs::write(&p, b"not json").unwrap();
        assert_eq!(Configuracao::carregar_de(&p).zoom, Configuracao::default().zoom);
        std::fs::remove_dir_all(p.parent().unwrap()).unwrap();
    }
}
