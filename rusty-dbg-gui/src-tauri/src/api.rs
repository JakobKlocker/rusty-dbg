use rusty_dbg::core::Debugger;
use rusty_dbg::core::memory::Memory;
use std::sync::Mutex;
use tauri::State;

pub struct DebuggerState {
    pub debugger: Mutex<Option<Debugger>>,
}

#[tauri::command]
pub fn debugger_init(path: String, state: State<DebuggerState>) -> Result<(), String> {
    let mut dbg = state.debugger.lock().unwrap();
    *dbg = Some(Debugger::new(path.clone(), "tauri-debugger".to_string()));
    Ok(())
}

#[tauri::command]
pub fn get_memory(addr: u64, size: usize, state: State<DebuggerState>) -> Result<Vec<u8>, String> {
    let mut dbg = state.debugger.lock().unwrap();
    if let Some(ref mut debugger) = *dbg {
        debugger.dump_hex(&format!("{:#x}", addr), size)
            .map_err(|e| e.to_string())
    } else {
        Err("Debugger not initialized".to_string())
    }
}