use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct IndexManager {
    pub path: PathBuf,
    pub index: HashMap<String, Arc<Mutex<crate::index_engine::IndexEngine>>>,
}

impl IndexManager {
    pub fn new(path: PathBuf) -> IndexManager {
        let mut im = IndexManager {
            path: path.clone(),
            index: HashMap::new(),
        };
        im.load_persistence();
        im
    }
    pub fn create_new_index(&mut self, index_name: String, doc: String) -> Result<String, String> {
        match self.index.get(&index_name.clone()) {
            // check if the index is not there already
            Some(i) => {
                i.lock().unwrap().index_string_document(doc.clone());
                return Ok(format!("msg: Index updated {}", index_name.clone()));
            }
            None => {
                self.index.insert(
                    index_name.clone(),
                    Arc::new(Mutex::new(crate::index_engine::IndexEngine::new(
                        self.path.clone(),
                        index_name.clone(),
                        doc.clone(),
                    ))),
                );
                return Ok(format!("msg: index created {}", index_name.clone()));
            }
        }
    }
    fn load_existing_index(&mut self, index_name: String) -> Result<String, String> {
        // if key exists, just refresh. if not, create it
        match self.index.insert(
            index_name.clone(),
            Arc::new(Mutex::new(
                crate::index_engine::IndexEngine::load_or_create_index(
                    std::env::current_dir().unwrap(),
                    index_name.clone(),
                ),
            )),
        ) {
            Some(_v) => return Ok(format!("msg: Index updated {}", index_name.clone())),
            None => return Ok(format!("msg: Index loaded {}", index_name.clone())),
        }
    }
    fn load_persistence(&mut self) {
        let dir = &self.path;
        if dir.is_dir() {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    let index_name = path.to_str().unwrap().to_string();
                    self.load_existing_index(index_name).unwrap();
                };
            }
        }
    }
    //pub fn stats() {}
}
