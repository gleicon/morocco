use actix_web::{middleware, web, App, HttpServer};
use clap::{AppSettings, Parser};

use hostname;
use std::path::PathBuf;
use std::process;
use std::sync::Mutex;

use crate::stats::SearchStats;

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
        "actix_web=info,actix_server=info,morocco=info,morocco::handlers=debug,morocco::index_engine=info,morocco::index_manager=info",
    );
    env_logger::init();
    info!("Morocco search");

    let cli = MoroccoOptions::parse();

    let data_dir = match cli.data_dir {
        Some(v) => v,
        None => std::env::current_dir().unwrap(),
    };
    info!("Data dir: {:?}", data_dir);

    let http_port = match cli.http_port {
        Some(hp) => hp,
        None => 3000,
    };
    info!("http port: {}", http_port);

    let id = format!(
        "{}-{:?}",
        hostname::get().unwrap().to_string_lossy(),
        process::id()
    );
    info!("instance id: {}", id.clone());

    let data = web::Data::new(Mutex::new(index_manager::IndexManager::new(
        std::env::current_dir().unwrap(),
    )));
    let stats = web::Data::new(Mutex::new(stats::SearchStats::new(id.clone())));

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(data.clone())
            .app_data(stats.clone())
            .service(handlers::search_index)
            .service(handlers::index_document)
            .service(handlers::index_stats)
            .service(handlers::catch_get)
            .service(handlers::catch_post)
    })
    .bind(("127.0.0.1", http_port))?
    .run()
    .await
}
