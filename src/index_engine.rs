// index interface
use chrono::Local;
use json::JsonValue;
use sqlite;
use sqlite::Value;
use uuid::Uuid;

pub struct IndexEngine {
    name: String,
    version: Uuid,
    db_connection: sqlite::Connection,
    created_at: i64,
}

impl IndexEngine {
    pub fn new(name: String) -> Self {
        let ie = IndexEngine {
            name: name,
            version: Uuid::new_v4(),
            db_connection: sqlite::open(":memory:").unwrap(), // temp config
            created_at: Local::now().timestamp_millis(),
        };
        ie
        // ie.create_schema();
    } // new index engine
    pub fn search(&mut self, qs: String) {
        let mut scursor = self
            .db_connection
            .prepare("SELECT * FROM ? WHERE posts MATCH ?")
            .unwrap()
            .into_cursor()
            .bind(&[
                Value::String(self.name),
                Value::String("\"fts5\"".to_string()),
            ])
            .unwrap();

        while let Some(Ok(row)) = scursor.next() {
            println!("Title = {}", row.get::<String, _>(0));
            println!("Body = {}", row.get::<String, _>(1));
        }
    }

    pub fn create_standard_schema_index(&mut self) {
        let create_statement = format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS {} USING fts5 (title, body)",
            self.name
        );

        self.db_connection.execute(create_statement).unwrap();
    } //creates a standard (title,body) index
    pub fn index_document(&mut self, doc: JsonValue) {
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

        println!("{}", insert_statement);

        self.db_connection.execute(insert_statement).unwrap();
    }

    pub fn create_schema_from_json(&mut self, doc: JsonValue) {
        let mut attribute_list: Vec<String> = vec![];
        let mut value_list: Vec<String> = vec![];

        for tag in doc.entries() {
            println!("Element: {:?}: {:?}", tag.0, tag.1.to_string());
            attribute_list.push(tag.0.to_string());
            value_list.push(format!("'{}'", tag.1.to_string()));
        }

        let index_statement = format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS {} USING fts5 ({});",
            self.name,
            attribute_list.join(",")
        );

        let insert_statement = format!(
            "INSERT into {} ({}) VALUES ({})",
            self.name,
            attribute_list.join(","),
            value_list.into_iter().collect::<Vec<String>>().join(",")
        );

        self.db_connection.execute(index_statement).unwrap();
        self.db_connection.execute(insert_statement).unwrap();
    }
}
