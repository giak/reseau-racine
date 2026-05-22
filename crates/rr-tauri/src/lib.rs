#[tauri::command]
fn get_identity() -> Result<String, String> {
    let data_dir = rr_core::identity::IdentityManager::default_data_dir();
    let manager = rr_core::identity::IdentityManager::new(&data_dir);
    match manager.load() {
        Ok(identity) => Ok(identity.public_key_bech32()),
        Err(_) => Err("No identity found. Run `rr init` first.".to_string()),
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_identity])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
