use std::collections::HashMap;

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
struct RecordingId {
    pub id: u32,
    pub file: String,
    pub name: String,
    pub note: Option<String>,
    pub uploaded: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct Event<T> {
    pub event: String,
    pub payload: T,
    pub id: u32,
}
#[derive(Clone, Serialize, Deserialize)]
struct Recording {
    pub file: String,
    pub name: String,
    pub note: Option<String>,
    pub uploaded: bool,
}

#[derive(Serialize)]
struct NewRec {
    file: String,
    name: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (files, add_file) = signal::<HashMap<u32, Recording>>(HashMap::new());

    Effect::new(move || {
        spawn_local(async move {
            let cb: Closure<dyn Fn(JsValue)> = Closure::new(move |data: JsValue| {
                add_file.update(|f| {
                    log(data.clone());
                    let event: Event<RecordingId> = from_value(data).unwrap();
                    let rec = event.payload;
                    f.insert(
                        rec.id,
                        Recording {
                            file: rec.file,
                            name: rec.name,
                            note: rec.note,
                            uploaded: rec.uploaded,
                        },
                    );
                });
            });
            listen("file", cb.as_ref().unchecked_ref()).await;
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
        <button on:click=move |_| spawn_local(async move {invoke("new_rec", to_value(&NewRec {file: invoke_argless("plugin:mic-recorder|stop_recording").await.try_into().unwrap(), name: "test".to_string()}).unwrap()).await;})>Stop</button>
        <ul>
            <For
                each=move || files.get()
                key=|file| file.0
                children=move |file| {
                    view!{
                        <li>{file.1.file}</li>
                    }
                }
            />
        </ul>
    }
}
