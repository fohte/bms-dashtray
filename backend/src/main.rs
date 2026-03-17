// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "public API will be consumed by future modules")
)]
mod db_reader;

fn main() {
    if let Err(e) = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
    {
        eprintln!("error while running tauri application: {e}");
        std::process::exit(1);
    }
}
