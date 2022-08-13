// index interface
use chrono::Local;
use json::object;
use json::JsonValue;
use serde::{Deserialize, Serialize};
use sqlite;
use std::collections::HashMap;
use uuid::Uuid;

pub struct IndexEngine {
    name: String,
    version: Uuid,
    db_connection: sqlite::Connection,
    created_at: i64,
}
#[derive(Serialize, Deserialize)]
struct Resultset {
    count: i64,
    rows: HashMap<String, String>,
}

impl IndexEngine {
    pub fn new_blank_index(name: String) -> Self {
        let ie = IndexEngine {
            name: name,
            version: Uuid::new_v4(),
            db_connection: sqlite::open(":memory:").unwrap(), // temp config
            created_at: Local::now().timestamp_millis(),
        };
        ie
        // ie.create_schema();
    } // new index engine
    pub fn new(name: String, doc: String) -> Self {
        let mut ie = IndexEngine {
            name: name,
            version: Uuid::new_v4(),
            db_connection: sqlite::open(":memory:").unwrap(), // temp config
            created_at: Local::now().timestamp_millis(),
        };
        // let result = json::parse(&doc); //std::str::from_utf8(&doc).unwrap());
        ie.create_schema_from_string(doc);

        ie
    } // new index engine

    pub fn search(&mut self, qs: String) -> Result<String, serde_json::Error> {
        let query = format!("SELECT * FROM ? WHERE {} MATCH ?", self.name);

        // let mut scursor = self
        //     .db_connection
        //     .prepare(query)
        //     .unwrap()
        //     .into_cursor()
        //     .bind(&[
        //         Value::String(self.name),
        //         Value::String("\"fts5\"".to_string()),
        //     ])
        //     .unwrap();

        // // let results = 0;
        // // let Resultset
        // scursor.
        // while let Some(Ok(row)) = scursor.next() {
        //     println!("Title = {}", row.get::<String, _>(0));
        //     println!("Body = {}", row.get::<String, _>(1));
        // }

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

        for tag in doc.entries() {
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
    }

    pub fn create_schema_from_string(&mut self, body: String) {
        match json::parse(&body) {
            Ok(v) => self.create_schema_from_json(v),
            Err(e) => info!("{:?}", e.to_string()),
        };
    }
}
