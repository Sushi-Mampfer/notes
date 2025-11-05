use sqlx::{Pool, Sqlite};
use tauri::{Emitter, Manager};

#[derive(Debug)]
struct AppData {
    pub pool: Pool<Sqlite>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tokio::main]
pub async fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppData {
                pool: Pool::connect_lazy(&dbg!(format!(
                    "sqlite://{}\\db.sqlite",
                    app.path()
                        .app_data_dir()?
                        .to_str()
                        .ok_or("Path not valid")?
                )))?,
            });
            Ok(())
        })
        .plugin(tauri_plugin_mic_recorder::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![new_rec])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn new_rec(app_handle: tauri::AppHandle, path: String) {
    app_handle.emit("test", path).unwrap();
}
