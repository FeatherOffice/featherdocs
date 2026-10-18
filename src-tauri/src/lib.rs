//! A casca do FeatherDocs: uma janela e o protocolo `ponte://`, que leva cada
//! comando do TypeScript até o motor Rust e devolve os bytes **na hora**
//! (síncrono do lado da página, como no FeatherSheet e no FeatherCAD).
//!
//! Aqui não há regra de documento: a janela recebe o pedido, chama
//! `Ponte::chamar` e devolve a resposta. O disco e as caixas de diálogo estão em
//! `arquivos.rs`; a tabela editada no FeatherSheet, em `tabelas.rs`.

mod arquivos;
mod tabelas;

use std::sync::RwLock;
use std::time::{Duration, Instant};

use featherdocs_ponte::Ponte;
use tauri::http::{Request, Response};
use tauri::Manager;

/// Acima disto o comando aparece no terminal como `[motor]` (um quadro de tela tem 16,6 ms).
const LIMITE_MOTOR: Duration = Duration::from_millis(16);

/// Uma resposta com os cabeçalhos que a página precisa para ler o XHR.
fn resposta(status: u16, tipo: &str, corpo: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
        .header("Content-Type", tipo)
        .body(corpo)
        .unwrap_or_else(|_| Response::new(Vec::new()))
}

/// Um comando do motor, com a métrica `[motor]`. Os que só leem (`chamar_leitura`) rodam com a
/// trava de leitura, ao mesmo tempo; o resto pega a trava de escrita.
pub(crate) fn executar(app: &tauri::AppHandle, comando: &str, corpo: &[u8]) -> Result<Vec<u8>, String> {
    let estado = app.state::<RwLock<Ponte>>();
    let inicio = Instant::now();
    let resultado = {
        let leitura = estado.read().unwrap_or_else(|e| e.into_inner());
        leitura.chamar_leitura(comando, corpo)
    };
    let resultado = match resultado {
        Some(r) => r,
        None => estado.write().unwrap_or_else(|e| e.into_inner()).chamar(comando, corpo),
    };
    let gasto = inicio.elapsed();
    if gasto > LIMITE_MOTOR {
        eprintln!("[motor] {comando}: {:.1} ms ({} bytes in)", gasto.as_secs_f64() * 1000.0, corpo.len());
    }
    resultado
}

/// Atende um pedido do `ponte://` (XHR síncrono da página): um comando do motor.
fn atender(app: &tauri::AppHandle, req: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    if req.method() == "OPTIONS" {
        return resposta(200, "text/plain", Vec::new());
    }
    let comando = req.uri().path().trim_start_matches('/');
    match executar(app, comando, req.body()) {
        Ok(bytes) => resposta(200, "application/octet-stream", bytes),
        Err(erro) => resposta(400, "text/plain", erro.into_bytes()),
    }
}

/// Pede ao WebKitGTK para compor e desenhar o canvas na **placa de vídeo, sempre**. O padrão
/// (`ON_DEMAND`) só usa a GPU com WebGL; um Canvas 2D comum ficava no processador (visto no FeatherCAD).
#[cfg(target_os = "linux")]
fn ligar_aceleracao_por_gpu(app: &tauri::App) {
    use webkit2gtk::{HardwareAccelerationPolicy, SettingsExt, WebViewExt};
    let Some(janela) = app.get_webview_window("main") else { return };
    let pedido = janela.with_webview(|webview| {
        let Some(ajustes) = webview.inner().settings() else { return };
        ajustes.set_hardware_acceleration_policy(HardwareAccelerationPolicy::Always);
    });
    if let Err(erro) = pedido {
        eprintln!("[FeatherDocs] aceleração por GPU: não consegui acessar a webview: {erro}");
    }
}

/// Sobe a janela do FeatherDocs. O conteúdo é o mesmo `dist/index.html` que roda no navegador.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// A janela da Ajuda (F1): como usar e atalhos. Abre (ou traz para a frente, se já aberta) o `ajuda.html` do `dist/`.
#[tauri::command]
fn abrir_janela_ajuda(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(janela) = app.get_webview_window("ajuda") {
        let _ = janela.show();
        let _ = janela.unminimize();
        let _ = janela.set_focus();
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(&app, "ajuda", tauri::WebviewUrl::App("ajuda.html".into()))
        .title("Ajuda do FeatherDocs")
        .inner_size(1000.0, 720.0)
        .min_inner_size(640.0, 420.0)
        .resizable(true)
        .focused(true)
        .build()
        .map_err(|e| format!("Não consegui abrir a janela da Ajuda: {e}"))?;
    Ok(())
}

pub fn run() {
    eprintln!("[FeatherDocs] Aplicativo iniciado (PID: {}).", std::process::id());
    let executado = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(RwLock::new(Ponte::default()))
        .manage(arquivos::Caminho::default())
        .manage(tabelas::Vigias::default())
        // Assíncrono: o WebKitGTK atende esquema próprio na thread da janela, e esperar a trava ali
        // congelaria a janela. A página continua esperando (o XHR é síncrono), mas a janela não.
        .register_asynchronous_uri_scheme_protocol("ponte", |ctx, req, responder| {
            let app = ctx.app_handle().clone();
            tauri::async_runtime::spawn_blocking(move || responder.respond(atender(&app, &req)));
        })
        .setup(|app| {
            #[cfg(target_os = "linux")]
            ligar_aceleracao_por_gpu(app);
            #[cfg(not(target_os = "linux"))]
            let _ = app;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            abrir_janela_ajuda,
            arquivos::documento_abrir,
            arquivos::documento_inicial,
            arquivos::documento_salvar,
            arquivos::documento_salvar_confirmado,
            arquivos::documento_exportar,
            arquivos::documento_esquecer_caminho,
            arquivos::documento_versoes,
            arquivos::documento_abrir_versao,
            arquivos::documento_comparar_versao,
            arquivos::texto_abrir,
            arquivos::texto_salvar,
            tabelas::tabela_editar_no_feathersheet,
            tabelas::tabela_verificar,
        ])
        .run(tauri::generate_context!());
    if let Err(erro) = executado {
        eprintln!("[FeatherDocs] Não consegui abrir a janela: {erro}");
    }
}
