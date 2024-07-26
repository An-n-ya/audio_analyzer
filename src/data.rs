use std::sync::{Arc, Mutex};

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbObjectStore, IdbRequest, IdbTransactionMode};

use crate::Log;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Data {
    pub current_chunks: Vec<Chunk>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Chunk {
    pub id: usize,
    pub data: Vec<u8>,
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
    pub const MAX_SIZE: usize = 10;
    pub fn clear(&mut self) {
        self.current_chunks.clear();
        Self::request_db(|store| {
            store.clear().unwrap();
        })
    }
    pub fn push(&mut self, data: Chunk) {
        let id = data.id;
        self.current_chunks.push(data);
        if self.current_chunks.len() == Self::MAX_SIZE {
            Self::log(&format!("data_id: {}", id));
            let chunks = self.current_chunks.clone();
            Self::request_db(move |store| {
                let value = serde_wasm_bindgen::to_value(&chunks).unwrap();
                if let Ok(_) = store.add_with_key(&value, &JsValue::from_f64(id as f64)) {
                    Self::log("write to indexedDB success");
                }
            });
            self.current_chunks.clear();
        }
    }

    pub fn get_from_db(&self, id: usize, container: Arc<Mutex<Vec<Chunk>>>) {
        if self.current_chunks.len() > 0 && self.current_chunks[0].id <= id {
            let mut a = container.lock().unwrap();
            *a = self.current_chunks.clone();
            return;
        }
        Self::request_db(move |store| {
            let request = store
                .get_all_with_key(&JsValue::from_f64(id as f64))
                .unwrap();
            let container = container.clone();
            let on_success = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let target = event.target().unwrap();
                let request = target.dyn_into::<IdbRequest>().unwrap();
                let res = request.result().unwrap();
                let chunk: Vec<Chunk> = serde_wasm_bindgen::from_value(res).unwrap();
                let mut a = container.lock().unwrap();
                *a = chunk;
            }) as Box<dyn FnMut(_)>);

            request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
            on_success.forget();
        });
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
