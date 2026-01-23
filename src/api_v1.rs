#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum Response {
    Mod(DataFull),
    ModList(Vec<DataShort>),
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct DataShort {
    name: String,
    author: String,
    icon_src: Option<String>,
    short_desc: String,
}

#[derive(serde::Serialize)]
pub struct DataFull {
    mod_info: ModInfoFull,
    versions: Vec<VersionData>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ModInfoFull {
    name: String,
    author: String,
    icon_src: Option<String>,
    long_desc: String,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct VersionData {
    link: String,
    version: String,
    changelog: String,
}

pub async fn get(
    axum::extract::Path(endpoint): axum::extract::Path<String>,
    state: axum::extract::State<std::sync::Arc<crate::app::State>>,
) -> Result<axum::response::Json<Response>, axum::http::StatusCode> {
    println!(
        "{} GET /api/v1/{}",
        crate::pages::get_time(),
        endpoint.clone()
    );

    if let Some(mod_name) = endpoint.strip_prefix("mod/") {
        if mod_name == "" {
            return Err(axum::http::StatusCode::NO_CONTENT);
        }

        let mod_info: ModInfoFull = match sqlx::query_as(
            "
                SELECT name, author, icon_src, long_desc
                FROM info
                WHERE name LIKE ?
            ",
        )
        .bind(mod_name)
        .persistent(true)
        .fetch_one(&*state.database)
        .await
        {
            Ok(row) => row,
            Err(e) => match e {
                sqlx::Error::RowNotFound => return Err(axum::http::StatusCode::NOT_FOUND),
                _ => {
                    println!("Database error: {}", e);
                    return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
                }
            },
        };

        let versions: Vec<VersionData> = match sqlx::query_as(
            "
                SELECT link, version, changelog
                FROM versions
                WHERE name LIKE ?
                ORDER BY version DESC 
            ",
        )
        .bind(mod_name)
        .persistent(true)
        .fetch_all(&*state.database)
        .await
        {
            Ok(row) => row,
            Err(e) => match e {
                sqlx::Error::RowNotFound => return Err(axum::http::StatusCode::NOT_FOUND),
                _ => {
                    println!("Database error: {}", e);
                    return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
                }
            },
        };

        return Ok(axum::response::Json(Response::Mod(DataFull {
            mod_info,
            versions,
        })));
    }

    if endpoint.as_str() == "mod-list" {
        let mod_list: Vec<DataShort> = match sqlx::query_as(
            "
                SELECT info.name, author, icon_src, short_desc
                FROM info INNER JOIN versions ON info.name = versions.name
                GROUP BY info.name
                ORDER BY MAX(versions.id) DESC
            ",
        )
        .persistent(true)
        .fetch_all(&*state.database)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                println!("Database error: {}", e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        return Ok(axum::response::Json(Response::ModList(mod_list)));
    }

    return Err(axum::http::StatusCode::NOT_FOUND);
}
