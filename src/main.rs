use actix_web::{middleware, web, App, HttpServer};
use std::sync::Mutex;

mod handlers;
mod index_engine;
mod index_manager;

#[macro_use]
extern crate log;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        "actix_web=info,actix_server=info,morocco=info,morocco::handlers=info, morocco::index_engine=info",
    );
    env_logger::init();
    info!("Morocco search");
    let data = web::Data::new(Mutex::new(index_manager::IndexManager::new(
        std::env::current_dir().unwrap(),
    )));

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(data.clone())
            .service(handlers::search_index)
            .service(handlers::index_document)
            .service(handlers::index_stats)
            .service(handlers::catch_get)
            .service(handlers::catch_post)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
