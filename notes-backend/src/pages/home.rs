use crate::query::query;
use leptos::{component, prelude::*, view, IntoView};
use leptos_use::{use_clipboard, UseClipboardReturn};

#[component]
pub fn HomePage() -> impl IntoView {
    let data = Resource::new(|| (), |_| async move { query().await.ok() });

    #[cfg(feature = "ssr")]
    let copy = |_data: &str| {};

    #[cfg(not(feature = "ssr"))]
    let UseClipboardReturn {
        is_supported: _,
        text: _,
        copied: _,
        copy,
    } = use_clipboard();

    view! {
        <Suspense fallback=|| {
            view! { <h1>Loading...</h1> }
        }>
            {move || {
                data.get()
                    .map(|items| {
                        view! {
                            <ul>
                                {items
                                    .map(|n| {
                                        n.into_iter()
                                            .map(|n| {
                                                if n.summary.is_some() {
                                                    view! {
                                                        <li class="grid grid-cols-[1fr_10em_10em] p-3 bg-gray-800 text-gray-200 h-14 m-1">
                                                                <p class="leading-8 h-8">{n.name.clone()}</p>
                                                                <button class="leading-8 h-8 text-left" on:click={
                                                                    let copy = copy.clone();
                                                                    move |_| copy(&n.transcript.clone().unwrap())
                                                                }>Transcript</button>
                                                                <button class="leading-8 h-8 text-left" on:click={
                                                                    let copy = copy.clone();
                                                                    move |_| copy(&n.summary.clone().unwrap())
                                                                }>Summary</button>
                                                        </li>
                                                    }
                                                } else {
                                                    view! {
                                                        <li class="grid grid-cols-[1fr_20em] p-3 bg-gray-800 text-gray-200 h-14 m-1">
                                                                <p class="leading-8 h-8">{n.name.clone()}</p>
                                                                <p class="leading-8 h-8">Transcribing...</p>
                                                        </li>
                                                    }
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                    })}
                            </ul>
                        }
                    })
            }}
        </Suspense>
    }
}
