// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
