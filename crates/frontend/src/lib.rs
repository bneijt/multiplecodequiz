mod app;
mod components;
mod quiz;

pub use app::App;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    leptos::mount::mount_to_body(App);
}
