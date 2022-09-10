use actix_web::{get, post, Error, Result};
use actix_web::{web, HttpResponse};
use chrono::DateTime;
use chrono::Utc;
use json::object;
use json::JsonValue;
use std::time::SystemTime;

use regex::Regex;
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
pub struct Query {
    q: String,
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

#[post("/1/indexes/{route}/query")]
async fn query_index(
    info: web::Path<PathInfo>,
    index_manager: web::Data<Mutex<crate::index_manager::IndexManager>>,
    body: web::Bytes,
) -> Result<HttpResponse, Error> {
    let result = json::parse(std::str::from_utf8(&body).unwrap());
    let data = index_manager.lock().unwrap();
    debug!("index: {}", info.route);
    debug!("body: {:?}", result);

    let injson: JsonValue = match result {
        Ok(v) => v,
        Err(e) => json::object! {"err" => e.to_string() },
    };

    if !injson["query"].is_null() {
        let query = injson["query"].clone().to_string();
        let re = Regex::new(r"\W+").unwrap();
        let caps: Vec<&str> = re.split(&query).collect();
        let query = caps.join(" ");

        let index_name = info.route.clone();
        let index = data.index.get(&index_name);

        match index {
            Some(index_engine) => match index_engine.lock() {
                Ok(mut p) => {
                    let pp = p.search(query);
                    let rs = serde_json::to_string(&pp.unwrap());

                    let rs = object! {
                        hits: rs.unwrap(),
                    };
                    return Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body(rs.to_string()));
                }
                Err(e) => {
                    return Ok(HttpResponse::NoContent()
                        .content_type("application/json")
                        .body(e.to_string()))
                }
            },
            None => {
                return Ok(HttpResponse::NotFound()
                    .content_type("application/json")
                    .body(format!("msg: index [{:?}] not found", index_name)))
            }
        };
    } else {
        // defaults to not found
        return Ok(HttpResponse::NotFound()
            .content_type("application/json")
            .body(format!("route not found: {}", info.clone().route)));
    }
}

#[post("/1/indexes/{route}/batch")]
async fn batch_index(
    info: web::Path<PathInfo>,
    index_manager: web::Data<Mutex<crate::index_manager::IndexManager>>,
    body: web::Bytes,
) -> Result<HttpResponse, Error> {
    let result = json::parse(std::str::from_utf8(&body).unwrap());
    debug!("route: {}", info.route);
    debug!("payload: {:?}", &body);

    let injson: JsonValue = match result {
        Ok(v) => v,
        Err(e) => {
            return Ok(HttpResponse::BadRequest()
                .content_type("application/json")
                .body(format!("msg: error {:?}", e)))
        } //json::object! {"err" => e.to_string() },
    };

    if !injson["requests"].is_null() {
        let request = injson["requests"].clone();
        let index_name = info.route.clone();

        let mut index_manager = index_manager.lock().unwrap();
        let index = index_manager.index.get(&index_name);

        match index {
            Some(index_engine) => match index_engine.lock() {
                Ok(mut ie) => {
                    //ie.index_string_document(request[0]["body"].to_string());
                    ie.index_jsonvalue(request[0]["body"].clone());

                    let now = SystemTime::now();
                    let now: DateTime<Utc> = now.into();
                    let now = now.to_rfc3339();

                    let rs = object! {
                        updatedAt: now,
                        taskID:1,
                        objectIDs: [request[0]["body"]["ObjectID"].clone()],
                    };
                    return Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body(rs.to_string()));
                }
                Err(e) => {
                    return Ok(HttpResponse::BadRequest()
                        .content_type("application/json")
                        .body(format!("msg: err {:?}", e)))
                }
            },
            None => {
                index_manager
                    .create_new_index(index_name.to_string(), request[0]["body"].to_string())
                    .unwrap(); // TODO: improve error handling

                return Ok(HttpResponse::Ok()
                    .content_type("application/json")
                    .body(request[0]["body"].to_string()));
            }
        }
    };

    // defaults to not found
    return Ok(HttpResponse::NotFound()
        .content_type("application/json")
        .body(format!("index/route not found: {}", info.clone().route)));
}

// rest search routes
// resembles restmq on simplicity and routing
// querystring is provided by the ?q= query parameter
#[get("/i/{index}")]
async fn search_index(
    info: web::Path<DocumentInfo>,
    index_manager: web::Data<Mutex<crate::index_manager::IndexManager>>,
    stats: web::Data<Mutex<crate::stats::SearchStats>>,
    query: web::Query<Query>,
) -> Result<HttpResponse, Error> {
    let data = index_manager.lock().unwrap();
    let index = data.index.get(&info.index);
    let query = query.q.clone();

    debug!("raw query string: {}", query);
    let re = Regex::new(r"\W+").unwrap();
    let caps: Vec<&str> = re.split(&query).collect();
    let query = caps.join(" ");

    debug!("filtered query string:{:?}", query);

    match index {
        Some(indexengine) => match indexengine.lock() {
            Ok(mut ie) => match ie.search(query) {
                Ok(payload) => {
                    stats
                        .lock()
                        .unwrap()
                        .increment_index_usage_counter(info.index.clone());
                    let rs = serde_json::to_string(&payload);

                    let rs = object! {
                        results: rs.unwrap(),
                    };
                    return Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body(rs.to_string()));
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
                        "msg: error fetching data from index {:?} -  {:?}",
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

    stats
        .lock()
        .unwrap()
        .increment_index_usage_counter(info.index.clone());

    match index {
        Some(index_engine) => match index_engine.lock() {
            Ok(mut ie) => {
                ie.index_string_document(req_body);
                return Ok(HttpResponse::Ok()
                    .content_type("application/json")
                    .body("msg: Document updated"));
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
            Ok(mut v) => match v.dump_json() {
                Ok(payload) => {
                    return Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body(payload));
                }
                Err(e) => {
                    return Ok(HttpResponse::NoContent()
                        .content_type("application/json")
                        .body(e))
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
