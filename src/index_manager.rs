use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct IndexManager {
    pub path: PathBuf,
    pub index: HashMap<String, Arc<Mutex<crate::index_engine::IndexEngine>>>,
}

impl IndexManager {
    pub fn new(path: PathBuf) -> IndexManager {
        let im = IndexManager {
            path: path.clone(),
            index: HashMap::new(),
        };
        im
    }
    pub fn create_new_index() {}
    fn load_existing_indexes() {}
    pub fn stats() {}
}
