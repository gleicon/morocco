use actix_web::{get, post, Error, Result};
use actix_web::{web, HttpResponse};
use json::JsonValue;
use regex::Regex;
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
pub struct Query {
    q: String,
}

#[derive(Deserialize)]
pub struct IndexInfo {
    index: String,
    term: String,
}

#[derive(Deserialize, Clone)]
struct PathInfo {
    route: String,
}

#[derive(Deserialize)]
struct DocumentInfo {
    index: String,
}

// stats route per index:
// top queries with more results, top queries w/o result, top queries with less results
// top terms

// catch all routes for client compatibility
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

// rest search routes
// resembles restmq on simplicity and routing
// querystring is provided by the ?q= query parameter
#[get("/i/{index}")]
async fn search_index(
    info: web::Path<DocumentInfo>,
    data: web::Data<Mutex<crate::index_manager::IndexManager>>,
    stats: web::Data<Mutex<crate::stats::SearchStats>>,
    query: web::Query<Query>,
) -> Result<HttpResponse, Error> {
    let data = data.lock().unwrap();
    let index = data.index.get(&info.index);
    let query = query.q.clone();
    debug!("query string: {}", query);

    let re = Regex::new(r"\W+").unwrap();
    let caps: Vec<&str> = re.split(&query).collect();
    let query = caps.join(" ");

    debug!("filtered query string:{:?}", query);

    match index {
        Some(indexengine) => match indexengine.lock() {
            Ok(mut ie) => match ie.search(query.clone()) {
                Ok(payload) => {
                    stats
                        .lock()
                        .unwrap()
                        .increment_index_usage_counter(info.index.clone());
                    return Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body(payload));
                }
                Err(e) => {
                    stats
                        .lock()
                        .unwrap()
                        .increment_http_4xx_errors_counter(info.index.clone());
                    return Ok(HttpResponse::NoContent()
                        .content_type("application/json")
                        .body(e.to_string()));
                }
            },
            Err(e) => {
                stats
                    .lock()
                    .unwrap()
                    .increment_http_5xx_errors_counter(info.index.clone());
                return Ok(HttpResponse::BadRequest()
                    .content_type("application/json")
                    .body(format!(
                        "msg: err fetching data from index {:?} -  {:?}",
                        info.index, e
                    )));
            }
        },
        None => {
            stats
                .lock()
                .unwrap()
                .increment_http_4xx_errors_counter(info.index.clone());
            return Ok(HttpResponse::NotFound()
                .content_type("application/json")
                .body(format!("msg: index [{:?}] not found", info.index)));
        }
    }
}

#[post("/i/{index}")]
async fn index_document(
    req_body: String,
    info: web::Path<DocumentInfo>,
    index_manager: web::Data<Mutex<crate::index_manager::IndexManager>>,
    stats: web::Data<Mutex<crate::stats::SearchStats>>,
) -> Result<HttpResponse, Error> {
    let mut index_manager = index_manager.lock().unwrap();
    let index = index_manager.index.get(&info.index);
    info!("{}", info.index.clone());

    match index {
        Some(index_engine) => match index_engine.lock() {
            Ok(mut ie) => {
                ie.index_string_document(req_body.clone());
                return Ok(HttpResponse::Ok()
                    .content_type("application/json")
                    .body(format!("msg: Document updated")));
            }
            Err(e) => {
                return Ok(HttpResponse::BadRequest()
                    .content_type("application/json")
                    .body(format!("msg: err {:?}", e)))
            }
        },
        None => {
            index_manager
                .create_new_index(info.index.clone(), req_body.clone())
                .unwrap(); // TODO: improve error handling

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(format!("document {} indexed at {}", req_body, info.index)))
        }
    }
}

#[get("/stats/{index}")]
async fn index_stats(
    info: web::Path<DocumentInfo>,
    data: web::Data<Mutex<crate::index_manager::IndexManager>>,
) -> Result<HttpResponse, Error> {
    let data = data.lock().unwrap();
    let index = data.index.get(&info.index);

    match index {
        Some(vect) => match vect.lock() {
            Ok(mut v) => match v.to_json() {
                Ok(payload) => {
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
