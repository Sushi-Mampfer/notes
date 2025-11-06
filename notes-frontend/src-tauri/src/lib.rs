use serde::{Deserialize, Serialize};
use sqlx::{query, sqlite::SqliteConnectOptions, Pool, Row, Sqlite};
use std::{fs, str::FromStr};
use tauri::{async_runtime::spawn, Emitter, Manager};

struct AppData {
    pub pool: Pool<Sqlite>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Recording {
    pub id: u32,
    pub file: String,
    pub name: String,
    pub note: Option<String>,
    pub uploaded: bool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tokio::main]
pub async fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("db.sqlite");
            let pool = Pool::connect_lazy_with(
                SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
                    .create_if_missing(true),
            );
            app.manage(AppData { pool: pool.clone() });
            spawn(async move {
                query(
                    r#"
                    CREATE TABLE IF NOT EXISTS recordings (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        file TEXT NOT NULL,
                        title TEXT NOT NULL,
                        notes TEXT,
                        uploaded INTEGER
                    )
                "#,
                )
                .execute(&pool)
                .await
                .unwrap();
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
async fn new_rec(app_handle: tauri::AppHandle, file: String, name: String) {
    let row = query(
        r#"
        INSERT INTO recordings (file, title, uploaded)
        VALUES (?, ?, 0)
         RETURNING id
    "#,
    )
    .bind(&file)
    .bind(&name)
    .fetch_one(&app_handle.state::<AppData>().pool)
    .await
    .unwrap();
    let id = row.get("id");
    app_handle
        .emit(
            "file",
            Recording {
                id,
                file,
                name,
                note: None,
                uploaded: false,
            },
        )
        .unwrap();
}
