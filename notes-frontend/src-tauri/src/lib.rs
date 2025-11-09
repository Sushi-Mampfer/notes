use reqwest::{multipart::Form, Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use sqlx::{
    query, query_as, sqlite::SqliteConnectOptions, FromRow, Pool, QueryBuilder, Row, Sqlite,
};
use std::{fs, str::FromStr};
use tauri::{async_runtime::spawn, Emitter, Listener, Manager, WebviewWindowBuilder};

struct AppData {
    pub pool: Pool<Sqlite>,
}

#[derive(Clone, Serialize, Deserialize, FromRow)]
struct Recording {
    pub id: u32,
    pub file: String,
    pub name: String,
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
            let app_handle = app.handle().clone();
            spawn(async move {
                query(
                    r#"
                    CREATE TABLE IF NOT EXISTS recordings (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        file TEXT NOT NULL,
                        name TEXT NOT NULL,
                        uploaded INTEGER
                    )
                "#,
                )
                .execute(&pool)
                .await
                .unwrap();
                query(
                    r#"
                    CREATE TABLE IF NOT EXISTS config (
                        id INTEGER PRIMARY KEY CHECK (id = 1),
                        url TEXT
                    )
                "#,
                )
                .execute(&pool)
                .await
                .unwrap();
                query(
                    r#"
                    INSERT OR IGNORE INTO config(id, url)
                    VALUES (1, "")
                "#,
                )
                .execute(&pool)
                .await
                .unwrap();

                let handle = app_handle.clone();
                app_handle.listen_any("ready", move |_| {
                    let handle = handle.clone();
                    spawn(async move {
                        let rows: Vec<Recording> =
                            query_as(r#"SELECT * FROM recordings ORDER BY id"#)
                                .fetch_all(&handle.state::<AppData>().pool)
                                .await
                                .unwrap();
                        for r in &rows {
                            handle.emit("file", r).unwrap();
                        }
                    });
                });
            });
            Ok(())
        })
        .plugin(tauri_plugin_mic_recorder::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            new,
            edit,
            delete,
            upload,
            get_url,
            upload_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn new(app_handle: tauri::AppHandle, file: String, name: String) {
    let row = query(
        r#"
        INSERT INTO recordings (file, name, uploaded)
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
                uploaded: false,
            },
        )
        .unwrap();
}

#[tauri::command]
async fn edit(app_handle: tauri::AppHandle, id: u32, name: String) {
    let rec: Recording = query_as(
        r#"
        UPDATE recordings SET name = ?
        WHERE id = ?
        RETURNING *
    "#,
    )
    .bind(&name)
    .bind(&id)
    .fetch_one(&app_handle.state::<AppData>().pool)
    .await
    .unwrap();
    app_handle.emit("file", rec).unwrap();
}

#[tauri::command]
async fn delete(app_handle: tauri::AppHandle, id: u32) {
    let file: String = query(
        r#"
        DELETE FROM recordings
        WHERE id = ?
        RETURNING file
    "#,
    )
    .bind(&id)
    .fetch_one(&app_handle.state::<AppData>().pool)
    .await
    .unwrap()
    .get("file");
    fs::remove_file(file).unwrap();
    app_handle.emit("delete", id).unwrap();
}

#[tauri::command]
async fn upload(app_handle: tauri::AppHandle) {
    WebviewWindowBuilder::from_config(
        &app_handle,
        &app_handle.config().app.windows.get(1).unwrap().clone(),
    )
    .unwrap()
    .build()
    .unwrap();
}

#[tauri::command]
async fn get_url(app_handle: tauri::AppHandle) -> String {
    query(
        r#"
        SELECT url FROM config
        WHERE id = 1
    "#,
    )
    .fetch_one(&app_handle.state::<AppData>().pool)
    .await
    .unwrap()
    .get("url")
}

#[tauri::command]
async fn upload_files(app_handle: tauri::AppHandle, url: String, files: Vec<u32>) {
    query(
        r#"
        UPDATE config
        SET url = ?
        WHERE id = 1
    "#,
    )
    .bind(&url)
    .execute(&app_handle.state::<AppData>().pool)
    .await
    .unwrap();
    let mut query: QueryBuilder<Sqlite> = QueryBuilder::new(
        r#"
        UPDATE recordings
        SET uploaded = 1
        WHERE id IN (
    "#,
    );
    let mut seperated = query.separated(", ");
    for i in files {
        seperated.push_bind(i);
    }
    seperated.push_unseparated(
        r#")
        RETURNING name, file
    "#,
    );
    let rows = query
        .build()
        .fetch_all(&app_handle.state::<AppData>().pool)
        .await
        .unwrap();
    let mut form = Form::new();
    for i in rows {
        form = form
            .file::<String, String>(i.get("name"), i.get("file"))
            .await
            .unwrap();
    }
    Client::new()
        .post(format!("{}/upload", url))
        .multipart(form)
        .send()
        .await
        .unwrap();
}
