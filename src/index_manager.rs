use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct IndexManager {
    pub path: PathBuf,
    pub index: HashMap<String, Arc<Mutex<crate::index_engine::IndexEngine>>>,
}

impl IndexManager {
    pub fn new(path: PathBuf) -> IndexManager {
        let mut im = IndexManager {
            path,
            index: HashMap::new(),
        };
        im.load_persistence();
        im
    }
    pub fn create_new_index(&mut self, index_name: String, doc: String) -> Result<String, String> {
        match self.index.get(&index_name) {
            // check if the index is not there already
            Some(i) => {
                i.lock().unwrap().index_string_document(doc);
                Ok(format!("msg: Index updated {}", index_name.clone()))
            }
            None => {
                self.index.insert(
                    index_name.clone(),
                    Arc::new(Mutex::new(crate::index_engine::IndexEngine::new(
                        self.path.clone(),
                        index_name.clone(),
                        doc,
                    ))),
                );
                Ok(format!("msg: index created {}", index_name.clone()))
            }
        }
    }
    fn load_existing_index(&mut self, index_name: String) -> Result<String, String> {
        // if key exists, just refresh. if not, create it
        let pp = Path::new(&index_name).to_path_buf();
        let index = pp.file_stem().unwrap();
        let clean_name = index.to_os_string().into_string().unwrap();
        match self.index.insert(
            clean_name.clone(),
            Arc::new(Mutex::new(
                crate::index_engine::IndexEngine::load_or_create_index(pp, clean_name),
            )),
        ) {
            Some(_v) => Ok(format!("msg: Index updated {}", index_name)),
            None => Ok(format!("msg: Index loaded {}", index_name)),
        }
    }
    fn load_persistence(&mut self) {
        if !self.path.ends_with("data") {
            self.path.push("data");
        }
        info!("Data path: {:?}", self.path);

        if !Path::new(&self.path).exists() {
            info!("Creating data dir: {:?}", self.path);
            fs::create_dir_all(&self.path).unwrap();
        }

        let dir = &self.path;
        info!("dir: {:?}", dir);
        if dir.is_dir() {
            for entry in fs::read_dir(dir).unwrap() {
                let db_path = entry.unwrap().path();
                if !db_path.is_dir() {
                    let index_name = db_path.to_str().unwrap().to_string();
                    self.load_existing_index(index_name).unwrap();
                };
            }
        }
    }
    //pub fn stats() {}
}
