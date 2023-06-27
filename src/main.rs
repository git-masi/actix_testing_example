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
        // Adding they type here is not ideal
        let api_client: std::sync::Arc<dyn posts::PostAdder> =
            std::sync::Arc::new(posts::ApiClient::new());
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web::middleware::Logger::new("%a %{User-Agent}i"))
            // This doesn't work
            // .app_data(web::Data::new(posts::ApiClient::new()))
            .app_data(web::Data::from(api_client))
            .route("/", web::get().to(actix_web::HttpResponse::Ok))
            .route("/", web::post().to(posts::add_post))
    })
    .bind(server_config.to_address())?
    .run()
    .await
}
