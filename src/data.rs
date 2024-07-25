use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{IdbDatabase, IdbObjectStore, IdbTransactionMode};

use crate::Log;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Data {
    current_chunks: Vec<Chunk>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Chunk {
    id: usize,
    data: Vec<u8>,
}

impl Chunk {
    pub fn new(id: usize, data: Vec<u8>) -> Self {
        Self { id, data }
    }
}

impl Log for Data {
    fn name() -> &'static str {
        "Data"
    }
}

impl Default for Data {
    fn default() -> Self {
        Self {
            current_chunks: Default::default(),
        }
    }
}

impl Data {
    const MAX_SIZE: usize = 10;
    pub fn clear(&mut self) {
        self.current_chunks.clear();
    }
    pub fn push(&mut self, data: Chunk) {
        let id = data.id;
        self.current_chunks.push(data);
        Self::log(&format!("{}", self.current_chunks.len()));
        if self.current_chunks.len() == Self::MAX_SIZE {
            Self::log("hello");
            let chunks = self.current_chunks.clone();
            Self::request_db(move |store| {
                let value = serde_wasm_bindgen::to_value(&chunks).unwrap();
                store
                    .add_with_key(&value, &JsValue::from_f64(id as f64))
                    .unwrap();
                Self::log("write to indexedDB success");
            });
            self.current_chunks.clear();
        }
    }

    fn get_from_db(id: usize) {
        Self::request_db(move |store| {
            let request = store
                .get_all_with_key(&JsValue::from_f64(id as f64))
                .unwrap();
        })
    }

    fn request_db(f: impl Fn(IdbObjectStore) + 'static) {
        let window = web_sys::window().unwrap();
        let db = window.indexed_db().unwrap().unwrap();
        let request = db.open("audio_db").unwrap();

        let on_success = Closure::wrap(Box::new(move |event: JsValue| {
            let target = JsValue::from_str("target");
            let result = JsValue::from_str("result");
            let target = js_sys::Reflect::get(&event, &target).unwrap();
            let db = js_sys::Reflect::get(&target, &result).unwrap();
            let db = IdbDatabase::from(db);

            let transaction = db
                .transaction_with_str_and_mode("audio_store", IdbTransactionMode::Readwrite)
                .unwrap();
            let store = transaction.object_store("audio_store").unwrap();
            f(store);
        }) as Box<dyn FnMut(JsValue)>);
        let on_upgrade = Closure::wrap(Box::new(move |event: JsValue| {
            Self::log("on upgrade");
            let target = JsValue::from_str("target");
            let result = JsValue::from_str("result");
            let target = js_sys::Reflect::get(&event, &target).unwrap();
            let db = js_sys::Reflect::get(&target, &result).unwrap();
            let db = IdbDatabase::from(db);
            db.create_object_store("audio_store").unwrap();
        }) as Box<dyn FnMut(JsValue)>);
        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
        on_success.forget();
        on_upgrade.forget();
    }
}
