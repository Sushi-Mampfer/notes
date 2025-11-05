use leptos::{prelude::*, task::spawn_local};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = "invoke")]
    async fn invoke_argless(cmd: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["console"])]
    fn log(data: JsValue);

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> js_sys::Function;
}

#[derive(Clone, Serialize, Deserialize)]
struct File {
    pub id: u32,
    pub path: String,
}

#[derive(Serialize)]
struct NewRec {
    pub path: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (files, add_file) = signal::<Vec<File>>(Vec::new());

    Effect::new(move || {
        spawn_local(async move {
            let cb: Closure<dyn Fn(JsValue)> = Closure::new(move |data: JsValue| {
                add_file.update(|f| f.push(from_value(data).unwrap()));
            });
            listen("new_file", cb.as_ref().unchecked_ref()).await;
            cb.forget();
        });
    });

    Effect::new(|| {
        spawn_local(async {
            let cb: Closure<dyn Fn(JsValue)> = Closure::new(|data: JsValue| {
                log(data);
            });

            listen("test", cb.as_ref().unchecked_ref()).await;

            cb.forget()
        })
    });

    view! {
        <button on:click=|_| spawn_local(async {invoke_argless("plugin:mic-recorder|start_recording").await;})>Start</button>
        <button on:click=move |_| spawn_local(async move {invoke("new_rec", to_value(&NewRec {path: invoke_argless("plugin:mic-recorder|stop_recording").await.try_into().unwrap()}).unwrap()).await;})>Stop</button>
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
