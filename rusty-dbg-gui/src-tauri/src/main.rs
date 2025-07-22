// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;

fn main() {
    tauri::Builder::default()
        .manage(api::DebuggerState {
            debugger: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            api::debugger_init,
            api::get_memory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
