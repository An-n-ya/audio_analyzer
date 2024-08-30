#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod buffer;
mod data;
pub mod data_source;
mod widgets;
pub use app::TemplateApp;
use wasm_bindgen::JsValue;

pub trait Log {
    fn log(msg: &str) {
        if Self::name() != "Data" {
            web_sys::console::debug_1(&JsValue::from_str(&format!("[{}] {}", Self::name(), msg)));
        }
    }
    fn name() -> &'static str;
}
