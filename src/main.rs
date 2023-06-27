#[macro_use]
extern crate log;

use actix_web::{web, App, HttpServer};

mod config;
mod posts;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set the level to log with `RUST_LOG` (e.g. `export RUST_LOG=error`)
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let server_config = config::ServerConfig::new();

    info!(
        "Server running on {}:{}",
        &server_config.host_name, &server_config.port
    );

    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web::middleware::Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(posts::ApiClient::new()))
            .route("/", web::post().to(posts::add_post))
    })
    .bind(server_config.to_address())?
    .run()
    .await
}
