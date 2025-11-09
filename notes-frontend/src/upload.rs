use std::{cmp::Reverse, collections::BTreeMap};

use leptos::{prelude::*, reactive::spawn_local};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};

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
    pub name: String,
    pub uploaded: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct Upload {
    url: String,
    files: Vec<u32>,
}

#[component]
pub fn Upload() -> impl IntoView {
    let (files, add_file) = signal::<BTreeMap<Reverse<u32>, Recording>>(BTreeMap::new());
    let (ids, set_ids) = signal::<Vec<u32>>(Vec::new());
    let url = RwSignal::new("".to_string());

    Effect::new(move || {
        spawn_local(async move {
            let cb: Closure<dyn Fn(JsValue)> = Closure::new(move |data: JsValue| {
                let event: Event<RecordingId> = from_value(data).unwrap();
                let rec = event.payload;
                add_file.update(|f| {
                    f.insert(
                        Reverse(rec.id),
                        Recording {
                            file: rec.file,
                            name: rec.name,
                            uploaded: rec.uploaded,
                        },
                    );
                });
            });
            listen("file", cb.as_ref().unchecked_ref()).await;
            cb.forget();
        });
    });
    Effect::new(|| spawn_local(emit_argless("ready")));
    Effect::new(move || {
        spawn_local(async move { url.set(from_value(invoke_argless("get_url").await).unwrap()) })
    });

    view! {
        <div class="upload">
            <input bind:value=url placeholder="prot://ip:port"/>
            <button on:click=move |_| {
                spawn_local(async move {invoke("upload_files", to_value(&Upload {
                    url: url.get_untracked(),
                    files: ids.get_untracked()
                }).unwrap()).await;})
            }>Upload</button>
        </div>
        <ul class="uploads">
            <For
                each=move || files.get()
                    key=|file| file.0
                    children=move |(id, file)| {
                        if !file.uploaded {
                            set_ids.update(|ids| ids.push(id.0));
                        }
                        view! {
                            <li>
                                <p>{file.name}</p>
                                <input type="checkbox" checked=!file.uploaded
                                on:change=move |ev| {
                                    if event_target_checked(&ev) {
                                        set_ids.update(|ids| ids.push(id.0));
                                    } else {
                                        set_ids.update(|ids| {ids.remove(ids.iter().position(|x| *x == id.0).unwrap());});
                                    }
                                }
                                />
                            </li>
                        }
                    }
            />
        </ul>
    }
}
