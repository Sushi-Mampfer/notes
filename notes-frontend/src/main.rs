mod app;

use app::*;
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <Router>
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=|| view! { <App/> } />
                    <Route path=path!("/upload") view=|| view! { <App/> } />
                </Routes>
            </Router>
        }
    })
}
