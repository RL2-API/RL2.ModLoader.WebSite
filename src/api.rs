pub enum ApiResponse {
    Mod(DataFull),
    ModList(DataShort)
}

pub async fn get(axum::extract::Path(endpoint): axum::extract::Path<String>) -> Result<serde::Json<ApiResponse>, axum::http::StatusCode> {
    return Err(axum::http::StatusCode::NOT_FOUND)
}

#[derive(askama::Template)]
#[template(path = "api_homepage.html")]
pub struct Homepage();

impl Homepage {
    pub async fn get() -> Result<axum::response::Html<String>, axum::http::StatusCode> {
        let contents = match askama::Template::render(&ApiHomepage {}) {
            Ok(html) => html,
            Err(_) => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        };
        Ok(axum::response::Html::from(contents))
    }
}
