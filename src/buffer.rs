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
        back.push(chunk);
        Self::log(&format!("back_len {}", back.len()));
        if back.len() == Data::MAX_SIZE {
            let c = Arc::new(Mutex::new(vec![]));
            self.buf.push_back(c);
        }
    }

    pub fn set_max_id(&mut self, id: usize) {
        self.max_id = id;
    }

    fn front(&self) -> usize {
        if self.buf.len() == 0 {
            return 0;
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
            return 0;
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
                // if self.buf.len() == 0 {
                //     let c = Arc::new(Mutex::new(vec![]));
                //     self.buf.push_back(c);
                // }
                // let back = self.buf.back().unwrap().clone();
                // let mut back = back.lock().unwrap();
                // Self::log(&format!("back_len {}", back.len()));
                // if back.len() + 1 == Data::MAX_SIZE {
                //     let c = Arc::new(Mutex::new(self.db.current_chunks.clone()));
                //     self.buf.push_back(c.clone());
                //     return;
                // }
                // *back = self.db.current_chunks.clone();
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
        Self::log(&format!("get data {:?}", view));
        let mut res = vec![];
        if view.start < 0 {
            res = vec![128; view.start.abs() as usize];
        }
        let start = if view.start <= 0 {
            1
        } else {
            view.start as usize
        };
        let end = view.end as usize;
        self.fetch_data(start);
        self.fetch_data(end);
        let start_id = self.front();
        assert!(start >= start_id);
        let mut i = 0;
        while start >= start_id + i * Data::MAX_SIZE {
            i += 1;
        }
        let mut length = end - start + 1;
        if length < Data::MAX_SIZE {
            length += Data::MAX_SIZE;
        }
        // assert!(length >= Data::MAX_SIZE);
        let size = 1 + (length - Data::MAX_SIZE) / Data::MAX_SIZE;
        let data = self
            .buf
            .iter()
            .skip(i)
            .enumerate()
            .filter(|(ind, _)| *ind >= size)
            .fold(vec![], |mut acc, (_, value)| {
                let data = value.lock().unwrap();
                let data = data.iter().fold(vec![], |mut acc, v| {
                    acc.extend(v.data.clone());
                    acc
                });
                acc.extend(data);
                acc
            });
        res.extend(data);
        res
    }
}
