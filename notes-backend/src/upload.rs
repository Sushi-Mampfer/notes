use axum::{http::StatusCode, response::IntoResponse};

pub async fn upload() -> impl IntoResponse {
    
    StatusCode::OK
}