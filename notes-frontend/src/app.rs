use leptos::{prelude::*, task::spawn_local};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = "invoke")]
    async fn invoke_argless(cmd: &str) -> JsValue;

    fn listen() 
}

#[derive(Clone)]
struct File {
    pub id: u32,
    pub path: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (files, add_file) = signal::<Vec<File>>(Vec::new());
    let (count, set_count) = signal(0);

    view! {
        <button on:click=|_| spawn_local(async {invoke_argless("plugin:mic-recorder|start_recording").await;})>Start</button>
        <button on:click=move |_| spawn_local(async move {invoke("new_rec", invoke_argless("plugin:mic-recorder|stop_recording").await).await;})>Stop</button>
        <ul>
            <For
                each=move || files.get()
                key=|file| file.id
                children=move |file| {
                    view!{
                        <li>{file.path}</li>
                    }
                }
            />
        </ul>
    }
}
