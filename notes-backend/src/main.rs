pub mod app;
pub mod pages;
pub mod upload;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::env;
    use axum::{Router, routing::post};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use crate::{app::*, upload::upload};

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .route("upload", post(upload))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", env::var("PORT").unwrap_or_else(|_| "8080".to_string()))).await.unwrap();
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
