use std::{cmp::Reverse, collections::BTreeMap, sync::Arc};

use leptos::{ev, html, prelude::*, task::spawn_local};
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
    let (files, add_file) = signal::<BTreeMap<Reverse<u32>, Recording>>(BTreeMap::new());
    let (files_update, add_file_update) =
        signal::<BTreeMap<u32, WriteSignal<String>>>(BTreeMap::new());

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
                        Reverse(rec.id),
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
                    f.remove(&Reverse(event.payload));
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
    let prompt_show = RwSignal::new(None);
    let new_prompt_data = RwSignal::new("".to_string());
    let new_prompt_show = RwSignal::new(None);

    view! {
        <Prompt
            question="Name".to_string()
            data=prompt_data
            show=prompt_show
            cb={|data: Option<String>, id: u32| {
                let name = match data {
                    Some(n) => n,
                    None => return,
                };
                spawn_local(async move {invoke("edit", to_value(&Edit{ id, name }).unwrap()).await;});
            }}
        />
        <Prompt
            question="Name".to_string()
            data=new_prompt_data
            show=new_prompt_show
            cb={move |data: Option<String>, _id: u32| {
                spawn_local(async move {
                    let name = match data {
                        Some(n) => n,
                        None => return,
                    };
                    set_recording.set(false);
                    invoke("new", to_value(&NewRec {file: invoke_argless("plugin:mic-recorder|stop_recording").await.try_into().unwrap(), name}).unwrap()).await;
                })
            }}
        />
        <div class="top">
            <Show
                when = move || recording.get()
                fallback = move || view!{<button class="storp" on:click=move |_| spawn_local(async move {set_recording.set(true); invoke_argless("plugin:mic-recorder|start_recording").await;})><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#c0c0c0" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-mic-off-icon lucide-mic-off"><path d="M12 19v3"/><path d="M15 9.34V5a3 3 0 0 0-5.68-1.33"/><path d="M16.95 16.95A7 7 0 0 1 5 12v-2"/><path d="M18.89 13.23A7 7 0 0 0 19 12v-2"/><path d="m2 2 20 20"/><path d="M9 9v3a3 3 0 0 0 5.12 2.12"/></svg></button>}
            >
                <button class="storp" on:click=move |_| {
                    new_prompt_data.set("".to_string());
                    new_prompt_show.set(Some(0));
                }><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#c0c0c0" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-mic-icon lucide-mic"><path d="M12 19v3"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><rect x="9" y="2" width="6" height="13" rx="3"/></svg></button>
            </Show>
            <button class="upload_btn" on:click=|_| {spawn_local(async {
                invoke_argless("upload").await;
            });}>Upload</button>
        </div>
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
                                prompt_data.set(name.get_untracked());
                                prompt_show.set(Some(id.0));
                            }><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#c0c0c0" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-pencil-icon lucide-pencil"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/></svg></button>
                            <button on:click=move |_| {
                                spawn_local(async move {invoke("delete", to_value(&Delete { id: id.0}).unwrap()).await;});
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
    show: RwSignal<Option<u32>>,
    cb: impl Fn(Option<String>, u32) + Send + Sync + 'static,
) -> impl IntoView {
    let cb = Arc::new(cb);
    let node_ref = NodeRef::<html::Input>::new();
    Effect::new(move |_| {
        if show.get().is_some() {
            if let Some(node) = node_ref.get_untracked() {
                let _ = node.focus();
            }
        }
    });
    view! {
        <Show when=move || show.get().is_some()>
            <div class="prompt">
                <h1>{question.clone()}</h1>
                <input
                    type="text"
                    node_ref=node_ref
                    bind:value=data
                    on:keydown={
                        let cb = cb.clone();
                        move |ev: ev::KeyboardEvent| {
                        match ev.key().as_str() {
                            "Enter" => {
                                cb(Some(data.get_untracked()), show.get().unwrap());
                                show.set(None);
                            }
                            "Escape" => {
                                cb(None, show.get().unwrap());
                                show.set(None);
                            }
                            _ => {}
                        }
                    }}
                />
                <div>
                    <button on:click={
                            let cb = cb.clone();
                            move |_| {
                                cb(Some(data.get_untracked()), show.get().unwrap());
                                show.set(None);
                        }}>Ok</button>
                    <button on:click={
                            let cb = cb.clone();
                            move |_| {
                                cb(None, show.get().unwrap());
                                show.set(None);
                        }}>Cancel</button>
                </div>
            </div>
        </Show>
    }
}
