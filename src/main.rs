mod api_v1;
mod api_v2;
mod app;
mod pages;

use axum::response::Redirect;
use axum::routing::get;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> Result<(), app::Error<'static>> {
    let args: std::vec::Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err(app::Error::MissingArg("port"));
    }

    let port = args[1].as_str();
    let listener = match tokio::net::TcpListener::bind(format!("127.0.0.1:{}", &port)).await {
        Ok(tcp) => tcp,
        Err(e) => {
            eprintln!("Tokio error: {}", e);
            return Err(app::Error::BindPort(port.to_owned()));
        }
    };

    let database = match sqlx::SqlitePool::connect("mods.db").await {
        Ok(val) => std::sync::Arc::new(val),
        Err(e) => return Err(app::Error::DatabaseFailure(e)),
    };

    let router = axum::Router::new()
        .route("/mods", get(pages::ModList::get))
        .route("/mod/{name}", get(pages::Mod::get))
        .nest(
            "/api",
            axum::Router::new()
                .route_service("/v2", ServeFile::new("static/api_v2.html"))
                .route("/v2/", get(Redirect::to("/api/v2")))
                .route("/v2/mod-list", get(Redirect::to("/api/v2/mod-list/0")))
                .route("/v2/mod-list/{page}", get(api_v2::mod_list))
                // Legacy routes
                .route("/", get(Redirect::to("/api/v1")))
                .route_service("/v1", ServeFile::new("static/api_v1.html"))
                .route("/{end}", get(api_v1::get))
                .route("/v1/{*end}", get(api_v1::get)),
        )
        .nest_service("/styles", tower_http::services::ServeDir::new("styles"))
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .with_state(std::sync::Arc::new(app::State { database }));

    println!("Serving the server at 127.0.0.1:{}", &port);
    match axum::serve::serve(listener, router).await {
        Ok(_) => Ok(()),
        Err(e) => return Err(app::Error::Serve(e)),
    }
}
