#[derive(serde::Serialize)]
pub struct ModList {
    status: u16,
    page: u32,
    data: Vec<crate::api_v1::DataShort>,
}

#[derive(serde::Deserialize)]
pub struct ModListRequest {
    mods_per_page: Option<u32>,
}

pub async fn mod_list(
    axum::extract::Path(page): axum::extract::Path<u32>,
    axum::extract::State(state): axum::extract::State<std::sync::Arc<crate::app::State>>,
    req: Option<axum::extract::Json<ModListRequest>>,
) -> Result<axum::response::Json<ModList>, (axum::http::StatusCode, String)> {
    println!("{} GET /api/v2/mod-list/{}", crate::pages::get_time(), page);

    let mut mods_per_page = 10;
    if let Some(axum::Json(list_req)) = req {
        println!(
            "    mods_per_page: {:?}",
            list_req.mods_per_page.unwrap_or(10)
        );
        mods_per_page = list_req.mods_per_page.unwrap_or(10);
    }

    let data: Vec<crate::api_v1::DataShort> = match sqlx::query_as(
        "
            SELECT info.name, author, icon_src, short_desc
            FROM info INNER JOIN versions ON info.name = versions.name
            GROUP BY info.name
            ORDER BY MAX(versions.id) DESC
            LIMIT ? OFFSET ?
        ",
    )
    .bind(mods_per_page)
    .bind(mods_per_page * page)
    .persistent(true)
    .fetch_all(&*state.database)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            let err = format!("Database error: {}", e);
            println!("{}", err);
            return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, err));
        }
    };
    return Ok(axum::response::Json(ModList {
        page,
        data,
        status: axum::http::StatusCode::OK.as_u16(),
    }));
}
