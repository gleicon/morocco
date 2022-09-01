use actix_web::{middleware, web, App, HttpServer};
use clap::{AppSettings, Parser};

use std::path::PathBuf;
use std::sync::Mutex;

mod handlers;
mod index_engine;
mod index_manager;
mod stats;

#[macro_use]
extern crate log;

#[derive(Parser, Debug)]
#[clap(name = "morocco",author, version, about, long_about = None)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct MoroccoOptions {
    /// data folder
    #[clap(short = 'd', long = "data")]
    data_dir: Option<PathBuf>,

    /// port
    #[clap(short = 'p', long = "port")]
    http_port: Option<u16>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var(
        "RUST_LOG",
       // "actix_web=info,actix_server=info,morocco=info,morocco::handlers=debug,morocco::index_engine=info,morocco::index_manager=info",
         "actix_web=debug,actix_server=debug,morocco=debug,morocco::handlers=debug,morocco::index_engine=debug,morocco::index_manager=debug",

    );
    env_logger::init();
    info!("Morocco search");
    // console_subscriber::init();

    let cli = MoroccoOptions::parse();

    let data_dir = match cli.data_dir {
        Some(v) => v,
        None => std::env::current_dir().unwrap(),
    };
    info!("Data dir: {:?}", data_dir);

    let http_port = cli.http_port.unwrap_or(3000);
    info!("Http port: {}", http_port);

    let data = web::Data::new(Mutex::new(index_manager::IndexManager::new(
        std::env::current_dir().unwrap(),
    )));
    let stats = web::Data::new(Mutex::new(stats::SearchStats::new("main".to_string())));

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(data.clone())
            .app_data(stats.clone())
            .service(handlers::search_index)
            .service(handlers::index_document)
            .service(handlers::index_stats)
            .service(handlers::catch_get)
            .service(handlers::query_index)
            .service(handlers::batch_index)
    })
    .bind(("127.0.0.1", http_port))?
    .run()
    .await
}
