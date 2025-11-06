use std::collections::HashMap;

use leptos::{prelude::*, task::spawn_local};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["console"])]
    fn log(data: JsValue);

    #[wasm_bindgen(js_namespace = ["window"])]
    fn prompt(text: &str, placeholder: &str) -> Option<String>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = "invoke")]
    async fn invoke_argless(cmd: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> js_sys::Function;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], js_name = "emit")]
    async fn emit_argless(event: &str);
}

#[derive(Clone, Serialize, Deserialize)]
struct Edit {
    id: u32,
    name: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct RecordingId {
    pub id: u32,
    pub file: String,
    pub name: String,
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
                    let event: Event<RecordingId> = from_value(data).unwrap();
                    let rec = event.payload;
                    f.insert(
                        rec.id,
                        Recording {
                            file: rec.file,
                            name: rec.name,
                            uploaded: rec.uploaded,
                        },
                    );
                    log(to_value(&f).unwrap());
                });
            });
            listen("file", cb.as_ref().unchecked_ref()).await;
            cb.forget();
        });
    });

    Effect::new(|| spawn_local(emit_argless("ready")));

    view! {
        <button on:click=|_| spawn_local(async {invoke_argless("plugin:mic-recorder|start_recording").await;})>Start</button>
        <button on:click=move |_| spawn_local(async move {
            let name = match prompt("Name", "") {
                Some(n) => n,
                None => return,
            };
            invoke("new", to_value(&NewRec {file: invoke_argless("plugin:mic-recorder|stop_recording").await.try_into().unwrap(), name}).unwrap()).await;
        })>Stop</button>
        <ul>
            <For
                each=move || files.get()
                key=|file| file.0
                children=move |file| {
                    let name = file.1.name.clone();
                    view!{
                        <li>
                            <p>{file.1.name}</p>
                            <button on:click=move |_| {
                                let name = match prompt("Name", &name) {
                                    Some(n) => n,
                                    None => return,
                                };
                                spawn_local(async move {invoke("edit", to_value(&Edit{ id: file.0, name }).unwrap()).await;});
                            }><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#c0c0c0" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-pencil-icon lucide-pencil"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/></svg></button>
                            <button><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#c0c0c0" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-trash2-icon lucide-trash-2"><path d="M10 11v6"/><path d="M14 11v6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg></button>
                        </li>
                    }
                }
            />
        </ul>
    }
}
