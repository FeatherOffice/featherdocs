// A janela do Windows não deve abrir um console junto no modo release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    featherdocs_lib::run()
}
