use leptos::{prelude::*, task::spawn_local};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = "invoke")]
    async fn invoke_argless(cmd: &str) -> JsValue;
}

#[component]
pub fn App() -> impl IntoView {
    let (path, set_path) = signal("".to_string());

    view! {
        <button on:click=|_| spawn_local(async {invoke_argless("plugin:mic-recorder|start_recording").await;})>Start</button>
        <button on:click=move |_| spawn_local(async move {set_path.set(invoke_argless("plugin:mic-recorder|stop_recording").await.try_into().unwrap())})>Stop</button>
        <p>{path}</p>
    }
}
