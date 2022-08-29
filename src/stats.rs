/*
Search Stats per index
Top X queries sorted by frequency with more results
Top X queries sorted by frequency with zero results
Top X documents/results clicked (?)

General search metrics
Most searched index
Documents per index
HTTP Errors

*/
use json::object;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct SearchStats {
    instance_id: String, // unique instance id
    query_result_counter_per_index: Arc<Mutex<HashMap<String, u64>>>,
    empty_query_result_per_index: Arc<Mutex<HashMap<String, u64>>>,
    index_usage_count: Arc<Mutex<HashMap<String, u64>>>,
    documents_count_per_index: Arc<Mutex<HashMap<String, u64>>>,
    http_4xx_errors: Arc<Mutex<HashMap<String, u64>>>,
    http_5xx_errors: Arc<Mutex<HashMap<String, u64>>>,
}

impl SearchStats {
    pub fn new(instance_id: String) -> SearchStats {
        SearchStats {
            instance_id,
            query_result_counter_per_index: Arc::new(Mutex::new(HashMap::new())),
            empty_query_result_per_index: Arc::new(Mutex::new(HashMap::new())),
            index_usage_count: Arc::new(Mutex::new(HashMap::new())),
            documents_count_per_index: Arc::new(Mutex::new(HashMap::new())),
            http_4xx_errors: Arc::new(Mutex::new(HashMap::new())),
            http_5xx_errors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn increment_result_counter(&mut self, index: String) {
        self.query_result_counter_per_index
            .lock()
            .unwrap()
            .entry(index)
            .and_modify(|count| *count += 1)
            .or_insert(0);
    }

    pub fn increment_index_usage_counter(&mut self, index: String) {
        self.index_usage_count
            .lock()
            .unwrap()
            .entry(index)
            .and_modify(|count| *count += 1)
            .or_insert(0);
    }

    pub fn increment_empty_result_counter(&mut self, index: String) {
        self.empty_query_result_per_index
            .lock()
            .unwrap()
            .entry(index)
            .and_modify(|count| *count += 1)
            .or_insert(0);
    }

    pub fn increment_docs_per_index_counter(&mut self, index: String) {
        self.documents_count_per_index
            .lock()
            .unwrap()
            .entry(index)
            .and_modify(|count| *count += 1)
            .or_insert(0);
    }

    pub fn increment_http_4xx_errors_counter(&mut self, index: String) {
        self.http_4xx_errors
            .lock()
            .unwrap()
            .entry(index)
            .and_modify(|count| *count += 1)
            .or_insert(0);
    }

    pub fn increment_http_5xx_errors_counter(&mut self, index: String) {
        self.http_5xx_errors
            .lock()
            .unwrap()
            .entry(index)
            .and_modify(|count| *count += 1)
            .or_insert(0);
    }

    pub fn dump_json(&mut self) -> Result<String, String> {
        let out = object! {
            instance_id: self.instance_id.clone(),
            query_result_counter_per_index: self.query_result_counter_per_index.lock().unwrap().clone(),
            empty_query_result_per_index: self.empty_query_result_per_index.lock().unwrap().clone(),
            index_usage_count:self.index_usage_count.lock().unwrap().clone(),
            documents_count_per_index:self.documents_count_per_index.lock().unwrap().clone(),
            http_4xx_errors:self.http_4xx_errors.lock().unwrap().clone(),
            http_5xx_errors:self.http_5xx_errors.lock().unwrap().clone(),
        };
        Ok(out.dump())
    }
}
