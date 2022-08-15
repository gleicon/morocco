use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct IndexManager {
    pub index: HashMap<String, Arc<Mutex<crate::index_engine::IndexEngine>>>,
}
