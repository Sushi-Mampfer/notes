use leptos::{prelude::*, task::spawn_local};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[wasm_bindgen(module = "tauri-plugin-mic-recorder-api")]
extern "C" {
    #[wasm_bindgen(js_name = startRecording)]
    pub async fn start_recording();

    #[wasm_bindgen(js_name = stopRecording)]
    pub async fn stop_recording() -> JsValue;
}

#[component]
pub fn App() -> impl IntoView {
    let (path, set_path) = signal("".to_string());

    view! {
        <button on:click=|_| spawn_local(start_recording())>Start</button>
        <button on:click=move |_| spawn_local(async move {set_path.set(stop_recording().await.try_into().unwrap())})>Start</button>
        <p>{path}</p>
    }
}
