use actix_web::{get, post, Error, Result};
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use json::JsonValue;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, collections::VecDeque};

mod index_engine;

#[macro_use]
extern crate log;

#[derive(Deserialize, Clone)]
struct PathInfo {
    route: String,
}

/*
index -> file.sqlite
json attributes -> fts5 table

*/
// URI index name and query term
#[derive(Deserialize)]
struct IndexInfo {
    index: String,
    term: String,
}
struct IndexManager {
    index: HashMap<String, Arc<Mutex<index_engine::IndexEngine>>>,
}
// stats route per index:
// top queries with more results, top queries w/o result, top queries with less results
// top terms

// catch all routes
#[get("/{route:.*}")]
async fn catch_get(info: web::Path<PathInfo>) -> Result<HttpResponse, Error> {
    info!("{}", info.route);
    return Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(info.clone().route));
}

#[post("/{route:.*}")]
async fn catch_post(info: web::Path<PathInfo>, body: web::Bytes) -> Result<HttpResponse, Error> {
    let result = json::parse(std::str::from_utf8(&body).unwrap());
    let injson: JsonValue = match result {
        Ok(v) => v,
        Err(e) => json::object! {"err" => e.to_string() },
    };
    info!("{}", info.route);
    info!("{}", injson.dump());
    info!("{}", injson["requests"]);
    for x in injson.entries() {
        info!("{:?}", x);
    }

    return Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(info.clone().route));
}

// rest search
#[get("/i/{index}/{term}")]
async fn search_index(
    info: web::Path<IndexInfo>,
    data: web::Data<Mutex<IndexManager>>,
) -> Result<HttpResponse, Error> {
    let data = data.lock().unwrap();
    let index = data.index.get(&info.index);

    match index {
        Some(vect) => match vect.lock() {
            Ok(mut v) => match v.search(info.term.clone()) {
                Ok(mut payload) => {
                    return Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body(payload));
                }
                Err(e) => {
                    return Ok(HttpResponse::NoContent()
                        .content_type("application/json")
                        .body(e.to_string()))
                }
            },
            Err(e) => {
                return Ok(HttpResponse::BadRequest()
                    .content_type("application/json")
                    .body(format!(
                        "msg: err fetching message from topic {:?} -  {:?}",
                        info.index, e
                    )))
            }
        },
        None => {
            return Ok(HttpResponse::NotFound()
                .content_type("application/json")
                .body(format!("msg: index [{:?}] not found", info.index)))
        }
    }
}

#[post("/i/{index}")]
async fn index_document(
    req_body: String,
    info: web::Path<IndexInfo>,
    data: web::Data<Mutex<IndexManager>>,
) -> Result<HttpResponse, Error> {
    let mut data = data.lock().unwrap();
    let index = data.index.get(&info.index);

    //let json_payload = serde_json::to_value(&req_body.clone());

    match index {
        Some(vect) => match vect.lock() {
            Ok(mut v) => v.index_string_document(req_body.clone()),
            Err(e) => {
                return Ok(HttpResponse::BadRequest()
                    .content_type("application/json")
                    .body(format!("msg: err {:?}", e)))
            }
        },
        None => {
            data.index
                .insert(
                    info.index.clone(),
                    Arc::new(Mutex::new(index_engine::IndexEngine::new(
                        req_body.clone(),
                        req_body.clone(),
                    ))),
                )
                .unwrap();
            ()
        }
    }
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("document {} indexed at {}", req_body, info.index)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        "actix_web=info,actix_server=info,morocco=info,morocco::handlers=info",
    );
    env_logger::init();
    info!("Morocco search");
    let data = web::Data::new(Mutex::new(IndexManager {
        index: HashMap::new(),
    }));

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(data.clone())
            .service(catch_get)
            .service(catch_post)
            .service(search_index)
            .service(index_document)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
