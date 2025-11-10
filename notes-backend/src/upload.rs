use axum::{extract::Multipart, http::StatusCode, response::IntoResponse, Extension};
use sqlx::{query, Row};
use tokio::{fs::File, io::AsyncWriteExt, spawn};
use uuid::Uuid;

use crate::{datatypes::AppState, ai::sumarize, transcription::transcribe};

pub async fn upload(
    Extension(state): Extension<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = format!("{}.wav", Uuid::new_v4());
        let mut file = File::create(&file_name).await.unwrap();
        file.write_all(&field.bytes().await.unwrap()).await.unwrap();

        let row = query(
            r#"
            INSERT INTO entries (file, name)
            VALUES (?, ?)
            RETURNING id
        "#,
        )
        .bind(&file_name)
        .bind(name)
        .fetch_one(&state.pool)
        .await
        .unwrap();

        let pool = state.pool.clone();
        spawn(async move {
            let transcript = transcribe(file_name).await;
            let summary = sumarize(transcript.clone()).await;
            query(
                r#"
                UPDATE entries 
                SET transcript = ?, summary = ?
                WHERE id = ?
            "#,
            )
            .bind(transcript).bind(summary)
            .bind(row.get::<u32, &str>("id"))
            .execute(&pool)
            .await
            .unwrap();
        });
    }
    StatusCode::OK
}
