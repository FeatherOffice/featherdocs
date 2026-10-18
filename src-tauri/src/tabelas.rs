//! A tabela editada no FeatherSheet (como o Google Docs faz com o Excel).
//!
//! O contrato entre os dois aplicativos é **um arquivo CSV** — nenhum importa código
//! do outro. O FeatherDocs grava a tabela numa pasta própria (`tabelas/<id>/`), abre o
//! FeatherSheet nela e, enquanto a janela de lá estiver aberta, a interface pergunta
//! (`tabela_verificar`) se algum `.csv` novo apareceu na pasta. O CSV mais recente
//! volta ao motor, que troca as células do bloco.
//!
//! Hoje, para devolver as mudanças, o usuário faz **Exportar → CSV** no FeatherSheet
//! (o nome e a pasta que ele sugere já servem). O Ctrl+S gravando direto nesse CSV
//! depende de um modo de edição externa no FeatherSheet, ainda não feito.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::SystemTime;

use tauri::{AppHandle, Manager};

/// Por tabela aberta no FeatherSheet: a pasta e a última gravação já trazida de volta.
#[derive(Default)]
pub struct Vigias(Mutex<HashMap<u64, Vigia>>);

struct Vigia {
    pasta: PathBuf,
    visto: SystemTime,
}

/// O `.csv` mais recente de `pasta` com data posterior a `visto`.
fn csv_novo(pasta: &Path, visto: SystemTime) -> Option<(PathBuf, SystemTime)> {
    std::fs::read_dir(pasta)
        .ok()?
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("csv")))
        .filter_map(|e| Some((e.path(), e.metadata().ok()?.modified().ok()?)))
        .filter(|(_, t)| *t > visto)
        .max_by_key(|(_, t)| *t)
}

/// Onde está o FeatherSheet: `FEATHERSHEET_BIN`, ao lado deste executável (a suíte instalada junta), ou no PATH.
fn achar_feathersheet() -> PathBuf {
    if let Some(p) = std::env::var_os("FEATHERSHEET_BIN") {
        return PathBuf::from(p);
    }
    let nome = if cfg!(windows) { "feathersheet.exe" } else { "feathersheet" };
    std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join(nome))).filter(|p| p.is_file()).unwrap_or_else(|| PathBuf::from(nome))
}

/// Grava a tabela (CSV, vinda do motor) e abre no FeatherSheet.
#[tauri::command]
pub async fn tabela_editar_no_feathersheet(app: AppHandle, id: u64, csv: String) -> Result<(), String> {
    let pasta = app.path().app_data_dir().map_err(|e| format!("Não achei a pasta de dados do aplicativo: {e}"))?.join("tabelas").join(id.to_string());
    std::fs::create_dir_all(&pasta).map_err(|e| format!("Não consegui criar \"{}\": {e}", pasta.display()))?;
    let arquivo = pasta.join("tabela.csv");
    // Ao reabrir, os CSVs de uma vez anterior não voltam por engano.
    for velho in std::fs::read_dir(&pasta).into_iter().flatten().flatten() {
        let _ = std::fs::remove_file(velho.path());
    }
    std::fs::write(&arquivo, csv.as_bytes()).map_err(|e| format!("Não consegui gravar \"{}\": {e}", arquivo.display()))?;
    let visto = std::fs::metadata(&arquivo).and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::now());
    Command::new(achar_feathersheet()).arg(&arquivo).spawn().map_err(|e| format!("Não achei o FeatherSheet instalado ({e}). Instale a suíte ou defina FEATHERSHEET_BIN."))?;
    app.state::<Vigias>().0.lock().unwrap_or_else(|e| e.into_inner()).insert(id, Vigia { pasta, visto });
    Ok(())
}

/// O CSV da tabela, se o FeatherSheet gravou um novo desde a última vez; senão `None`.
#[tauri::command]
pub fn tabela_verificar(app: AppHandle, id: u64) -> Result<Option<String>, String> {
    let estado = app.state::<Vigias>();
    let mut vigias = estado.0.lock().unwrap_or_else(|e| e.into_inner());
    let v = vigias.get_mut(&id).ok_or("Esta tabela não está aberta no FeatherSheet.")?;
    let Some((arquivo, quando)) = csv_novo(&v.pasta, v.visto) else { return Ok(None) };
    let texto = std::fs::read_to_string(&arquivo).map_err(|e| format!("Não consegui ler \"{}\": {e}", arquivo.display()))?;
    v.visto = quando;
    Ok(Some(texto))
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::time::Duration;

    #[test]
    fn so_traz_csv_mais_novo_que_o_visto() {
        let pasta = std::env::temp_dir().join(format!("featherdocs-tabela-{}", std::process::id()));
        std::fs::create_dir_all(&pasta).unwrap();
        let antes = SystemTime::now() - Duration::from_secs(5);
        assert!(csv_novo(&pasta, antes).is_none(), "pasta vazia");
        std::fs::write(pasta.join("a.csv"), "x").unwrap();
        std::fs::write(pasta.join("nota.txt"), "y").unwrap();
        let (achado, quando) = csv_novo(&pasta, antes).unwrap();
        assert!(achado.ends_with("a.csv"));
        assert!(csv_novo(&pasta, quando).is_none(), "já visto");
        std::fs::remove_dir_all(&pasta).unwrap();
    }
}
