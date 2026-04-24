#![cfg(target_arch = "wasm32")]

mod gui;
use gui::Gui;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), wasm_bindgen::JsValue> {
    let web_options = eframe::WebOptions::default();
    eframe::WebRunner::new()
        .start(
            "canvas",
            web_options,
            Box::new(|_cc| Box::new(Gui::new())),
        )
        .await
}
