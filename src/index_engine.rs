// index interface
use chrono::Local;
use json::JsonValue;
use serde::{Deserialize, Serialize};
use sqlite;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct IndexManager {
    pub index: HashMap<String, Arc<Mutex<IndexEngine>>>,
}

pub struct IndexEngine {
    path: String,
    name: String,
    version: Uuid,
    db_connection: sqlite::Connection,
    created_at: i64,
    attribute_list: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct Resultset {
    count: i64,
    rows: HashMap<String, String>,
}

impl IndexEngine {
    pub fn new_blank_index(name: String) -> Self {
        let path = format!("{}.db", name);
        let ie = IndexEngine {
            path: path.clone(),
            name: name,
            version: Uuid::new_v4(),
            db_connection: sqlite::open(path.clone()).unwrap(), // temp config
            created_at: Local::now().timestamp_millis(),
            attribute_list: Vec::new(),
        };
        ie
    } // new index engine

    pub fn new(name: String, doc: String) -> Self {
        let path = format!("{}.db", name);
        let mut ie = IndexEngine {
            path: path.clone(),
            name: name,
            version: Uuid::new_v4(),
            db_connection: sqlite::open(path.clone()).unwrap(), // temp config
            created_at: Local::now().timestamp_millis(),
            attribute_list: Vec::new(),
        };

        ie.create_schema_from_string(doc);

        ie
    } // new index engine

    pub fn search(&mut self, qs: String) -> Result<String, serde_json::Error> {
        let query = format!(
            "SELECT * FROM {} WHERE {} MATCH \"{}\"",
            self.name,
            self.name,
            // self.attribute_list.join(","),
            qs
        );

        println!("search: {}", query);
        let mut rs = Resultset {
            count: 0,
            rows: HashMap::new(),
        };
        self.db_connection
            .iterate(query, |pairs| {
                for &(column, value) in pairs.iter() {
                    rs.rows
                        .insert(column.to_string(), value.unwrap().to_string());
                    rs.count += 1;
                }
                true
            })
            .unwrap();

        serde_json::to_string(&rs)
    }

    pub fn create_standard_schema_index(&mut self) {
        let create_statement = format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS {} USING fts5 (title, body)",
            self.name
        );

        self.db_connection.execute(create_statement).unwrap();
    } //creates a standard (title,body) index

    pub fn index_string_document(&mut self, body: String) {
        let doc = json::parse(&body).unwrap();

        self.index_jsonvalue(doc)
    }

    pub fn index_jsonvalue(&mut self, doc: JsonValue) {
        let mut attribute_list: Vec<String> = vec![];
        let mut value_list: Vec<String> = vec![];
        for tag in doc.entries() {
            println!("Element: {:?}: {:?}", tag.0, tag.1.to_string());
            attribute_list.push(tag.0.to_string());
            value_list.push(format!("'{}'", tag.1.to_string()));
        }

        let insert_statement = format!(
            "INSERT into {} ({}) VALUES ({})",
            self.name,
            attribute_list.join(","),
            value_list.into_iter().collect::<Vec<String>>().join(",")
        );

        self.db_connection.execute(insert_statement).unwrap();
    }

    pub fn create_schema_from_json(&mut self, doc: JsonValue) {
        let mut attribute_list: Vec<String> = vec![];
        let local_doc = doc.clone();

        for tag in local_doc.entries() {
            let tag = tag.clone();
            println!("Element: {:?}: {:?}", tag.0, tag.1.to_string());
            attribute_list.push(tag.0.to_string());
        }

        let index_statement = format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS {} USING fts5 ({});",
            self.name,
            attribute_list.join(",")
        );

        self.db_connection.execute(index_statement).unwrap();

        self.index_jsonvalue(doc);
        // self.attribute_list.clone_from_slice(&attribute_list);
        self.attribute_list = attribute_list.clone();
    }

    pub fn create_schema_from_string(&mut self, body: String) {
        match json::parse(&body) {
            Ok(v) => self.create_schema_from_json(v),
            Err(e) => info!("{:?}", e.to_string()),
        };
    }
}
