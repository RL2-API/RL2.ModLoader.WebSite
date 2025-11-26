#[derive(Clone)]
pub struct State {
    pub database: std::sync::Arc<sqlx::SqlitePool>,
}

pub enum Error<'a> {
    MissingArg(&'a str),
    BindPort(String),
    DatabaseFailure(sqlx::Error),
    Serve(std::io::Error),
}

impl std::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self {
                Error::MissingArg(arg) => format!("Missing arg {}", arg),
                Error::BindPort(port) => format!("Failed to bind to port {}", port),
                Error::Serve(e) => format!("Failed to serve app with error: {}", e),
                Error::DatabaseFailure(e) => format!("Database operation failed: {}", e),
            }
            .as_str(),
        )
    }
}

impl std::fmt::Debug for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}
