use std::{
    collections::LinkedList,
    sync::{Arc, Mutex},
};

use crate::{
    app::View,
    data::{Chunk, Data},
    Log,
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Buffer {
    buf: LinkedList<Arc<Mutex<Vec<Chunk>>>>,
    db: Data,
    max_id: usize,
}

impl Log for Buffer {
    fn name() -> &'static str {
        "Buffer"
    }
}

pub struct DataRange {
    pub data: Vec<u8>,
    pub time_range: Option<(f32, f32)>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            buf: LinkedList::new(),
            db: Data::default(),
            max_id: 0,
        }
    }

    pub fn clear(&mut self) {
        self.buf.clear();
        self.db.clear();
        self.max_id = 0;
    }

    pub fn push(&mut self, chunk: Chunk) {
        self.db.push(chunk.clone());
        if self.buf.len() == 0 {
            self.buf.push_back(Arc::new(Mutex::new(vec![])));
        }
        let back = self.buf.back().unwrap().clone();
        let mut back = back.lock().unwrap();
        if back.len() == Data::MAX_SIZE {
            let c = Arc::new(Mutex::new(vec![chunk]));
            self.buf.push_back(c);
            return;
        }
        back.push(chunk);
    }

    pub fn set_max_id(&mut self, id: usize) {
        self.max_id = id;
    }

    fn front(&self) -> usize {
        if self.buf.len() == 0 {
            return 1;
        }
        let front = self.buf.front().expect("don't have data");
        let front = front.lock().unwrap();
        if front.len() == 0 {
            return 1;
        }
        front[0].id
    }
    fn end(&self) -> usize {
        if self.buf.len() == 0 {
            return 1;
        }
        let end = self.buf.back().expect("don't have data");
        let end = end.lock().unwrap();
        if end.len() == 0 {
            return 1;
        }
        end[end.len() - 1].id
    }

    fn fetch_data(&mut self, id: usize) {
        // Self::log(&format!(
        //     "fetch id: {}, front: {}, end: {}",
        //     id,
        //     self.front(),
        //     self.end()
        // ));
        if id < self.front() {
            let mut fetch_id = self.front() - 1;
            while fetch_id > id {
                let c = Arc::new(Mutex::new(vec![]));
                self.buf.push_front(c);
                self.db
                    .get_from_db(fetch_id, self.buf.front().unwrap().clone());
                fetch_id -= Data::MAX_SIZE;
            }
        } else if id > self.end() {
            let mut fetch_id = self.end() + Data::MAX_SIZE;
            if fetch_id > self.max_id {
                return;
            }
            while fetch_id < id {
                let c = Arc::new(Mutex::new(vec![]));
                self.buf.push_back(c);
                self.db
                    .get_from_db(fetch_id, self.buf.back().unwrap().clone());
                fetch_id += Data::MAX_SIZE;
            }
        } else {
            // no need for fetch data
        }
    }

    pub fn get_data(&mut self, view: &View) -> Vec<u8> {
        assert!(view.end > view.start);
        // Self::log(&format!("get data {:?}", view));
        let mut res = vec![];
        let (start, end) = (view.start, view.end);
        self.fetch_data(view.start);
        self.fetch_data(self.max_id.min(view.end));
        // Self::log(&format!(
        //     "i: {}, start: {}, start_id: {}, size: {}",
        //     i, start, start_id, size
        // ));
        // Self::log(&format!("buf_len {}", self.buf.len()));
        while let Some(id) = self.get_first_chunk_last_id() {
            if id < start {
                self.buf.pop_front();
            } else {
                break;
            }
        }
        let data = self.buf.iter().fold(vec![], |mut acc, value| {
            let data = value.lock().unwrap();
            let data = data.iter().fold(vec![], |mut acc, value| {
                if value.id >= start && value.id <= end {
                    acc.extend(value.data.clone());
                }
                acc
            });
            acc.extend(data);
            acc
        });
        res.extend(data);
        if self.max_id + 1 < view.end {
            res.extend(vec![128; Data::CHUNK_SIZE * (view.end - self.max_id - 1)]);
        }
        // Self::log(&format!("data_len {}", res.len()));
        // Self::log(&format!("buf_len {}", self.buf.len()));
        res
    }

    fn get_first_chunk_last_id(&self) -> Option<usize> {
        if let Some(v) = self.buf.front() {
            let v = v.lock().unwrap();
            if v.len() == 0 {
                // is waiting for data
                return None;
            }
            // assert!(v.len() > 0);
            let last = &v[v.len() - 1];
            Some(last.id)
        } else {
            None
        }
    }
}
