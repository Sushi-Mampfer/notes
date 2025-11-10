use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use sqlx::{Pool, Sqlite};

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Sqlite>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Note {
    pub name: String,
    pub transcript: Option<String>,
    pub summary: Option<String>,
}
