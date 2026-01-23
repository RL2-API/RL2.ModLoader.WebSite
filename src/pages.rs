#[derive(sqlx::FromRow, Clone)]
pub struct ModListData {
    name: String,
    icon_src: String,
    author: String,
    short_desc: String,
}

#[derive(askama::Template)]
#[template(path = "mods.html")]
pub struct ModList {
    mods: std::vec::Vec<ModListData>,
    total_count: usize,
    filter: String,
    page: usize,
}

#[derive(serde::Deserialize)]
pub struct Search {
    search: Option<String>,
    page: Option<u32>,
}

impl ModList {
    pub async fn get(
        state: axum::extract::State<std::sync::Arc<crate::app::State>>,
        query: axum::extract::Query<Search>,
    ) -> Result<axum::response::Html<String>, axum::http::StatusCode> {
        println!(
            "{} GET /mods?search={}&page={}",
            get_time(),
            query.search.clone().unwrap_or("".to_owned()),
            query.page.unwrap_or(0)
        );

        let mods: std::vec::Vec<ModListData> = match (match query.search.clone() {
            None => sqlx::query_as(MOD_LIST_FULL),
            Some(filter) => sqlx::query_as(MOD_LIST_FILTERED).bind(format!("{}%", filter)),
        })
        .persistent(true)
        .fetch_all(&*state.database)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                println!("Database error in '/': {}", e);
                vec![]
            }
        };

        let page: usize = query.page.unwrap_or(1) as usize;
        let mut first: usize = (page - 1) * MODS_PER_PAGE;
        let mut last: usize = page * MODS_PER_PAGE;
        if first >= mods.len() {
            first = 0;
            last = 0;
        } else if last > mods.len() {
            last = mods.len();
        }

        let contents = match askama::Template::render(&ModList {
            mods: mods[first..last].to_vec(),
            total_count: mods.len(),
            filter: query.search.clone().unwrap_or("".to_owned()),
            page,
        }) {
            Ok(html) => html,
            Err(_) => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        };
        Ok(axum::response::Html::from(contents))
    }
}

#[derive(askama::Template)]
#[template(path = "mod.html")]
pub struct Mod {
    name: String,
}

// #[derive(serde::Deserialize)]
// pub struct ModQuery {}

impl Mod {
    pub async fn get(
        axum::extract::Path(name): axum::extract::Path<String>,
        // state: axum::extract::State<std::sync::Arc<crate::app::State>>,
        // query: axum::extract::Query<ModQuery>,
    ) -> Result<axum::response::Html<String>, axum::http::StatusCode> {
        println!("{} GET /mod/{}", get_time(), name);
        let contents = match askama::Template::render(&Mod { name }) {
            Ok(html) => html,
            Err(_) => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        };
        Ok(axum::response::Html::from(contents))
    }
}

pub const MODS_PER_PAGE: usize = 6;
pub const RELOAD_ON_INPUT: bool = false;

const MOD_LIST_FULL: &'static str = "
    SELECT info.name, info.icon_src, info.author, info.short_desc
    FROM info INNER JOIN versions ON info.name = versions.name
    GROUP BY info.name
    ORDER BY MAX(versions.id) DESC
";

const MOD_LIST_FILTERED: &'static str = r#"
    SELECT info.name, info.icon_src, info.author, info.short_desc
    FROM info INNER JOIN versions ON info.name = versions.name
    WHERE info.name LIKE ?
    GROUP BY info.name
    ORDER BY MAX(versions.id) DESC
"#;

pub fn get_time() -> String {
    let current_time: f64 = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        % (24 * 3600)) as f64
        / 3600.0;
    let hours = current_time.floor();
    let minutes = ((current_time - hours) * 60.0).floor();
    format!("[{:?}:{:?}]", hours as u64, minutes as u64)
}
