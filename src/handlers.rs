use actix_web::{get, post, Error, Result};
use actix_web::{web, HttpResponse};
use json::JsonValue;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

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
#[get("/i/{index}/{term}")]
async fn search_index(
    info: web::Path<IndexInfo>,
    data: web::Data<Mutex<crate::index_engine::IndexManager>>,
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
    info: web::Path<DocumentInfo>,
    data: web::Data<Mutex<crate::index_engine::IndexManager>>,
) -> Result<HttpResponse, Error> {
    let mut data = data.lock().unwrap();
    let index = data.index.get(&info.index);
    info!("{}", info.index.clone());

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
            info!("none");
            match data.index.insert(
                info.index.clone(),
                Arc::new(Mutex::new(crate::index_engine::IndexEngine::new(
                    info.index.clone(),
                    req_body.clone(),
                ))),
            ) {
                Some(v) => {
                    return Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body(format!("msg: Document updated")))
                }
                None => {
                    return Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body(format!("msg: Document added")))
                }
            }
        }
    }
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("document {} indexed at {}", req_body, info.index)))
}
