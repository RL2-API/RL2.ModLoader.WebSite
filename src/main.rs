mod api;
mod app;
mod pages;

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
        .route("/", axum::routing::get(pages::ModList::get))
        .route("/mod/{name}", axum::routing::get(pages::Mod::get))
        .route("/api", axum::routing::get(api::Homepage::get))
        .route("/api/", axum::routing::get(api::Homepage::get))
        .route("/api/{*endpoint}", axum::routing::get(api::get))
        .nest_service("/styles", tower_http::services::ServeDir::new("styles"))
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .with_state(std::sync::Arc::new(app::State { database }));

    println!("Serving the server at 127.0.0.1:{}", &port);
    match axum::serve::serve(listener, router).await {
        Ok(_) => Ok(()),
        Err(e) => return Err(app::Error::Serve(e)),
    }
}
