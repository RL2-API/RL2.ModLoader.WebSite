use axum::http::StatusCode;

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    message: String,
}

const MODS_PER_PAGE: u32 = 10;

pub async fn mod_list(
    axum::extract::Path(page): axum::extract::Path<u32>,
    axum::extract::State(state): axum::extract::State<std::sync::Arc<crate::app::State>>,
) -> impl axum::response::IntoResponse {
    println!("{} GET /api/v2/mod-list/{}", crate::pages::get_time(), page);

    let data: Vec<crate::api_v1::DataShort> = match sqlx::query_as(
        "
            SELECT info.name, author, icon_src, short_desc
            FROM info INNER JOIN versions ON info.name = versions.name
            GROUP BY info.name
            ORDER BY MAX(versions.id) DESC
            LIMIT ? OFFSET ?
        ",
    )
    .bind(MODS_PER_PAGE)
    .bind(MODS_PER_PAGE * page)
    .persistent(true)
    .fetch_all(&*state.database)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            println!("Database error: {}", e);
            return Err((
                [("status", StatusCode::INTERNAL_SERVER_ERROR.as_u16())],
                axum::Json(ErrorResponse {
                    message: "A database error occured".to_string(),
                }),
            ));
        }
    };
    if data.len() == 0 {
        return Err((
            [("status", StatusCode::NOT_FOUND.as_u16())],
            axum::Json(ErrorResponse {
                message: format!("Page {} out of bounds", page).to_string(),
            }),
        ));
    }

    return Ok(([("status", StatusCode::OK.as_u16())], axum::Json(data)));
}
