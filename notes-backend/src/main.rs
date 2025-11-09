pub mod app;
pub mod pages;
pub mod transcription;
pub mod upload;

use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite};

#[derive(Clone)]
struct AppState {
    pool: Pool<Sqlite>,
}

#[cfg(feature = "ssr")]
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    use crate::{app::*, upload::upload};
    use axum::{extract::DefaultBodyLimit, routing::post, Extension, Router};
    use futures_util::StreamExt;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use reqwest::Client;
    use sqlx::query;
    use whisper_rs::install_logging_hooks;
    use std::{env, fs::exists, str::FromStr};
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    install_logging_hooks();

    if !exists("ggml-large-v3-q5_0.bin").unwrap() {
        println!("Downloading https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin");
        let res = Client::new()
            .get("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin")
            .send()
            .await
            .unwrap();

        let mut file = File::create("ggml-large-v3-q5_0.bin").await.unwrap();
        let mut stream = res.bytes_stream();

        while let Some(chunk) = stream.next().await {
            file.write_all(&chunk.unwrap()).await.unwrap();
        }

        println!("Downloaded https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin");
    }

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let pool: Pool<Sqlite> = Pool::connect_with(
        SqliteConnectOptions::from_str("sqlite://db.sqlite".into())
            .unwrap()
            .create_if_missing(true),
    )
    .await
    .unwrap();

    query(
        r#"
        CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file TEXT NOT NULL,
            name TEXT NOT NULL,
            transcript TEXT,
            guide TEXT
        )
    "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let state = AppState { pool };

    let app = Router::new()
        .route("/upload", post(upload))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(Extension(state))
        .layer(DefaultBodyLimit::disable())
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(format!(
        "0.0.0.0:{}",
        env::var("PORT").unwrap_or_else(|_| "8080".to_string())
    ))
    .await
    .unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
