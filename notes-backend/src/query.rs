use leptos::prelude::*;

#[cfg(feature = "ssr")]
use sqlx::query_as;

use crate::datatypes::Note;

#[cfg(feature = "ssr")]
use crate::datatypes::AppState;

#[server]
pub async fn query() -> Result<Vec<Note>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let state = expect_context::<AppState>();

        let out = query_as(
            r#"
        SELECT name, transcript, summary FROM entries
    "#,
        )
        .fetch_all(&state.pool)
        .await
        .unwrap();

        Ok(out)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not on server".to_string()))
    }
}
