use std::{collections::HashMap, rc::Rc, sync::Arc};

use leptos::{ev, prelude::*, task::spawn_local};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["console"])]
    fn log(data: JsValue);

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
struct Delete {
    id: u32,
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

#[derive(Clone)]
struct Recording {
    pub file: String,
    pub name: ReadSignal<String>,
    pub uploaded: bool,
}

#[derive(Serialize)]
struct NewRec {
    file: String,
    name: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (recording, set_recording) = signal(false);
    let (files, add_file) = signal::<HashMap<u32, Recording>>(HashMap::new());
    let (files_update, add_file_update) =
        signal::<HashMap<u32, WriteSignal<String>>>(HashMap::new());

    Effect::new(move || {
        spawn_local(async move {
            let cb: Closure<dyn Fn(JsValue)> = Closure::new(move |data: JsValue| {
                let event: Event<RecordingId> = from_value(data).unwrap();
                let rec = event.payload;
                match files_update.get_untracked().get(&rec.id) {
                    Some(s) => {
                        s.set(rec.name);
                        return;
                    }
                    None => (),
                };
                add_file.update(|f| {
                    let (name, set_name) = signal(rec.name);
                    add_file_update.update(|f| {
                        f.insert(rec.id, set_name);
                    });
                    f.insert(
                        rec.id,
                        Recording {
                            file: rec.file,
                            name: name,
                            uploaded: rec.uploaded,
                        },
                    );
                });
            });
            listen("file", cb.as_ref().unchecked_ref()).await;
            cb.forget();
        });
    });

    Effect::new(move || {
        spawn_local(async move {
            let cb: Closure<dyn Fn(JsValue)> = Closure::new(move |data: JsValue| {
                let event: Event<u32> = from_value(data).unwrap();
                add_file.update(|f| {
                    f.remove(&event.payload);
                });
                add_file_update.update(|f| {
                    f.remove(&event.payload);
                });
            });
            listen("delete", cb.as_ref().unchecked_ref()).await;
            cb.forget();
        });
    });

    Effect::new(|| spawn_local(emit_argless("ready")));
    let prompt_data = RwSignal::new("".to_string());
    let prompt_show = RwSignal::new(false);

    view! {
        <Prompt
            question="Name".to_string()
            data=prompt_data
            show=prompt_show
            cb={|data: Option<String>| {

            }}
        />
        <Show
            when = move || recording.get()
            fallback = move || view!{<button on:click=move |_| spawn_local(async move {set_recording.set(true); invoke_argless("plugin:mic-recorder|start_recording").await;})>Start</button>}
        >
            <button on:click=move |_| spawn_local(async move {
                let name = match prompt("Name", "") {
                    Some(n) => n,
                    None => return,
                };
                set_recording.set(false);
                invoke("new", to_value(&NewRec {file: invoke_argless("plugin:mic-recorder|stop_recording").await.try_into().unwrap(), name}).unwrap()).await;
            })>Stop</button>
        </Show>
            <ul>
            <For
                each=move || files.get()
                key=|file| file.0
                children=move |(id, file)| {
                    let name = file.name.clone();
                    view!{
                        <li>
                            <p>{file.name}</p>
                            <button on:click=move |_| {
                                let name = match prompt("Name", &name.get()) {
                                    Some(n) => n,
                                    None => return,
                                };
                                spawn_local(async move {invoke("edit", to_value(&Edit{ id, name }).unwrap()).await;});
                            }><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#c0c0c0" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-pencil-icon lucide-pencil"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/></svg></button>
                            <button on:click=move |_| {
                                spawn_local(async move {invoke("delete", to_value(&Delete {id}).unwrap()).await;});
                            }><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#c0c0c0" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-trash2-icon lucide-trash-2"><path d="M10 11v6"/><path d="M14 11v6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg></button>
                        </li>
                    }
                }
            />
        </ul>
    }
}

#[component]
pub fn Prompt(
    question: String,
    data: RwSignal<String>,
    show: RwSignal<bool>,
    cb: impl Fn(Option<String>) + Send + Sync + 'static,
) -> impl IntoView {
    let cb = Arc::new(cb);
    view! {
        <Show when=move || show.get()>
            <div class="prompt">
                <p>{question.clone()}</p>
                <input
                    type="text"
                    bind:value=data
                    on:keydown={
                        let cb = cb.clone();
                        move |ev: ev::KeyboardEvent| {
                        match ev.key().as_str() {
                            "Enter" => {
                                cb(Some(data.get_untracked()));
                                show.set(false);
                            }
                            "Escape" => {
                                cb(None);
                                show.set(false);
                            }
                            _ => {}
                        }
                    }}
                />
                <button on:click={
                        let cb = cb.clone();
                        move |_| {
                            cb(Some(data.get_untracked()));
                            show.set(false);
                    }}></button>
                <button on:click={
                        let cb = cb.clone();
                        move |_| {
                            cb(None);
                            show.set(false);
                    }}></button>
            </div>
        </Show>
    }
}
