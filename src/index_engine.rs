// index interface
use chrono::Local;
use json::object;
use json::JsonValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

pub struct IndexEngine {
    path: PathBuf,
    name: String,
    version: Uuid,
    db_connection: sqlite::Connection,
    created_at: i64,
    attribute_list: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct Resultset {
    count: i64,
    hits: Vec<HashMap<String, String>>,
    attributes: HashMap<String, String>,

    processing_time_ms: i64,
    query: String,
    parsed_query: String,
    params: String,
}

impl IndexEngine {
    pub fn dump_json(&mut self) -> Result<String, String> {
        let out = object! {
            path: self.path.clone().to_str(),
            name: self.name.clone(),
            version: self.version.clone().to_string(),
            created_at: self.created_at,
            schema: self.attribute_list.clone(),
        };
        Ok(out.dump())
    }

    pub fn load_or_create_index(path: PathBuf, name: String) -> Self {
        let mut path = path;

        if !path.is_file() {
            path.push(format!("{}.db", name));
        }

        IndexEngine {
            path: path.clone(),
            name,
            version: Uuid::new_v4(),
            db_connection: sqlite::open(path.clone()).unwrap(),
            created_at: Local::now().timestamp_millis(),
            attribute_list: Vec::new(),
        }
    } // new index engine

    pub fn new(path: PathBuf, name: String, doc: String) -> Self {
        let mut ie = IndexEngine::load_or_create_index(path, name);
        ie.create_schema_from_string(doc);

        ie
    } // new index engine

    pub fn search(&mut self, qs: String) -> Result<Resultset, String> {
        let query = format!(
            "SELECT * FROM {} WHERE {} MATCH \"{}\"",
            self.name, self.name, qs
        );

        debug!("search query: {}", query);
        let mut rs = Resultset {
            count: 0,
            hits: Vec::new(),
            attributes: HashMap::new(),
            processing_time_ms: 0,
            query: qs.clone(),
            parsed_query: qs.clone(),
            params: qs,
        };

        match self.db_connection.iterate(query, |pairs| {
            let mut new_pairs: HashMap<String, String> = HashMap::new();
            for &(column, value) in pairs.iter() {
                debug!("result: {}:{:?}", column, value);
                new_pairs.insert(column.to_string(), value.unwrap().to_string());
            }
            rs.count += 1;
            rs.hits.push(new_pairs);

            true
        }) {
            Ok(_) => Ok(rs),
            Err(e) => Err(format!("err: {}", e)),
        }

        //serde_json::to_string(&rs)
        // Ok(rs)
    }

    pub fn index_string_document(&mut self, body: String) {
        let doc = json::parse(&body).unwrap();

        self.index_jsonvalue(doc)
    }

    pub fn index_jsonvalue(&mut self, doc: JsonValue) {
        let mut attribute_list: Vec<String> = vec![];
        let mut value_list: Vec<String> = vec![];
        debug!("doc: {}", doc);
        debug!("schema: {:?}", self.attribute_list);

        for tag in doc.entries() {
            println!("Element: {:?}: {:?}", tag.0, tag.1.to_string());
            attribute_list.push(tag.0.to_string());
            value_list.push(format!("'{}'", tag.1));
        }

        let insert_statement = format!(
            "INSERT into {} ({}) VALUES ({})",
            self.name,
            attribute_list.join(","),
            value_list.into_iter().collect::<Vec<String>>().join(",")
        );

        match self.db_connection.execute(insert_statement.clone()) {
            Ok(v) => debug!("ok: {:?} - {}", v, insert_statement),
            Err(e) => info!("error: {} - {}", e, insert_statement),
        };
    }

    pub fn create_schema_from_json(&mut self, doc: JsonValue) {
        let mut attribute_list: Vec<String> = vec![];
        let local_doc = doc.clone();
        debug!("doc: {}", local_doc);

        for tag in local_doc.entries() {
            let tag = tag;
            debug!("Element: {:?}: {:?}", tag.0, tag.1.to_string());
            attribute_list.push(tag.0.to_string());
        }

        let index_statement = format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS {} USING fts5 ({});",
            self.name,
            attribute_list.join(",")
        );
        debug!("creating table: {}", index_statement);

        self.db_connection.execute(index_statement).unwrap();

        self.index_jsonvalue(doc);

        self.attribute_list = attribute_list.clone();
    }

    pub fn create_schema_from_string(&mut self, body: String) {
        match json::parse(&body) {
            Ok(v) => self.create_schema_from_json(v),
            Err(e) => info!("{:?}", e.to_string()),
        };
    }
}

// tests
// curl -vvv localhost:3000/i/livros?q=amigos+';'+select+from+livros
