#[derive(sqlx::FromRow, Clone)]
pub struct ModListData {
    name: String,
    icon_src: Option<String>,
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
    author: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct Search {
    search: Option<String>,
    page: Option<u32>,
}

impl ModList {
    pub async fn get(
        author: Option<String>,
        state: axum::extract::State<std::sync::Arc<crate::app::State>>,
        query: axum::extract::Query<Search>,
    ) -> Result<axum::response::Html<String>, axum::http::StatusCode> {
        if let Some(name) = author.clone() {
            println!(
                "{} GET /author/{:<32}\tsearch={}\tpage={}",
                get_time(),
                name,
                query.search.clone().unwrap_or_default(),
                query.page.clone().unwrap_or_default()
            )
        } else {
            println!(
                "{} GET /mods{:<32}\tsearch={}\tpage={}",
                get_time(),
                "",
                query.search.clone().unwrap_or("".to_owned()),
                query.page.unwrap_or(1),
            );
        }

        let mods: std::vec::Vec<ModListData> = match (match query.search.clone() {
            None => sqlx::query_as(MOD_LIST_FULL),
            Some(filter) => sqlx::query_as(MOD_LIST_FILTERED).bind(format!("{}%", filter)),
        })
        .bind(format!("{}", author.clone().unwrap_or("%".to_owned())))
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
            author,
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
    info: ModData,
    versions: Vec<Version>,
    q: ModQuery,
    author: bool,
}

#[derive(sqlx::FromRow)]
pub struct ModData {
    author: String,
    icon_src: Option<String>,
    long_desc: String,
}

#[derive(sqlx::FromRow, serde::Deserialize)]
pub struct Version {
    link: String,
    version: String,
    changelog: String,
}

#[derive(serde::Deserialize)]
pub struct ModQuery {
    changelog: Option<bool>,
    p: Option<u32>,
    f: Option<String>,
}

impl Mod {
    pub async fn delete(
        axum::extract::Path(mod_name): axum::extract::Path<String>,
        state: axum::extract::State<std::sync::Arc<crate::app::State>>,
    ) -> Result<axum::response::Redirect, axum::http::StatusCode> {
        let name = mod_name.as_str();

        match sqlx::query(
            "
                SELECT author, icon_src, long_desc
                FROM info
                WHERE name LIKE ?
            ",
        )
        .bind(name)
        .persistent(true)
        .fetch_one(&*state.database)
        .await
        {
            Err(_) => {
                return Err(axum::http::StatusCode::FORBIDDEN);
            }
            Ok(_) => {}
        };

        match sqlx::query(
            "
                DELETE FROM info
                WHERE name LIKE ?;
                DELETE FROM versions
                WHERE name LIKE ?;
            "
        )
        .bind(name)
        .bind(name)
        .execute(&*state.database)
        .await {
            Err(_) => {
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
            Ok(_) => {}
        }
        
        
        Ok(axum::response::Redirect::to("/mods"))
    }
    
    pub async fn get(
        author: bool,
        axum::extract::Path(name): axum::extract::Path<String>,
        state: axum::extract::State<std::sync::Arc<crate::app::State>>,
        axum::extract::Query(query): axum::extract::Query<ModQuery>,
    ) -> Result<axum::response::Html<String>, axum::http::StatusCode> {
        let changelog = query.changelog.unwrap_or(false);

        let info: ModData = match sqlx::query_as(
            "
                SELECT author, icon_src, long_desc
                FROM info
                WHERE name LIKE ?
            ",
        )
        .bind(name.clone())
        .persistent(true)
        .fetch_one(&*state.database)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                println!("Database error in '/mod/{}': {}", name, e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        if author {
            println!(
                "{} GET /author/{:<32}\tchangelog={}",
                get_time(),
                format!("{}/{}", info.author, name.clone()),
                changelog,
            );
        } else {
            println!(
                "{} GET /mod/{:<32}\tchangelog={}",
                get_time(),
                name.clone(),
                changelog,
            );
        }

        let versions: Vec<Version> = match sqlx::query_as(
            "
                SELECT link, version, changelog
                FROM versions
                WHERE name LIKE ?
                ORDER BY version DESC
            ",
        )
        .bind(name.clone())
        .persistent(true)
        .fetch_all(&*state.database)
        .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(|ver: Version| Version {
                    changelog: markdown::to_html_with_options(
                        ver.changelog.replace("\\n", "\n").as_str(),
                        &markdown::Options::gfm(),
                    )
                    .unwrap_or("ERROR. Please report".to_string()),
                    ..ver
                })
                .collect(),
            Err(e) => {
                println!("Database error in '/mod/{}': {}", name, e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let contents = match askama::Template::render(&Mod {
            name,
            info,
            versions,
            q: query,
            author,
        }) {
            Ok(html) => html,
            Err(_) => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        };
        Ok(axum::response::Html::from(contents))
    }
}

#[derive(serde::Deserialize)]
pub struct NewMod {
    author: String,
    name: String,
    download: String,
    description: String,
}

impl NewMod {
    pub async fn post(
        state: axum::extract::State<std::sync::Arc<crate::app::State>>,
        axum::extract::Query(query): axum::extract::Query<NewMod>,
    ) -> Result<axum::response::Redirect, axum::http::StatusCode> {
        let name = query.name.as_str();
        let desc = query.description.as_str();
        let author = query.author.as_str();
        let link = query.download.as_str();

        match sqlx::query(
            "
                SELECT author, icon_src, long_desc
                FROM info
                WHERE name LIKE ?
            ",
        )
        .bind(name)
        .persistent(true)
        .fetch_one(&*state.database)
        .await
        {
            Err(_) => {}
            Ok(_) => {
                return Err(axum::http::StatusCode::FORBIDDEN);
            }
        };

        match sqlx::query(
            "
            INSERT INTO info (name, author, short_desc, long_desc, icon_src) VALUES (?, ?, ?, ?, ?); 
            INSERT INTO versions (name, link, version, changelog) VALUES (?, ?, ?, ?); 
            ",
        )
        // info
        .bind(name)
        .bind(author)
        .bind(desc)
        .bind(desc)
        .bind(Option::<String>::None)
        // versions
        .bind(name)
        .bind(link)
        .bind("1.0.0")
        .bind("*Initial release*")
        .execute(&*state.database)
        .await
        {
            Ok(_) => {}
            Err(e) => {
                println!("Database error in '/mod/{}': {}", name, e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        Ok(axum::response::Redirect::to(
            format!("/mod/{}", name).as_str(),
        ))
    }
}

impl Version {
    pub async fn post(
        axum::extract::Path(mod_name): axum::extract::Path<String>,
        state: axum::extract::State<std::sync::Arc<crate::app::State>>,
        axum::extract::Query(query): axum::extract::Query<Version>,
    ) -> Result<axum::response::Redirect, axum::http::StatusCode> {
        let version = query.version.as_str();
        let link = query.link.as_str();
        let changelog = query.changelog.as_str();
        let name = mod_name.as_str();

        // Return 404 if mod doesn't exist
        match sqlx::query(
            "
                SELECT author, icon_src, long_desc
                FROM info
                WHERE name LIKE ?
            ",
        )
        .bind(name)
        .persistent(true)
        .fetch_one(&*state.database)
        .await
        {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                return Err(axum::http::StatusCode::NOT_FOUND);
            }
        };

        match sqlx::query(
            "
            INSERT INTO versions (name, link, version, changelog) VALUES (?, ?, ?, ?); 
            ",
        )
        .bind(name)
        .bind(link)
        .bind(version)
        .bind(changelog)
        .execute(&*state.database)
        .await
        {
            Ok(_) => {}
            Err(e) => {
                println!("Database error in '/mod/{}': {}", name, e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        Ok(axum::response::Redirect::to(
            format!("/mod/{}", name).as_str(),
        ))
    }
}

pub const MODS_PER_PAGE: usize = 6;
pub const RELOAD_ON_INPUT: bool = false;

const MOD_LIST_FULL: &'static str = "
    SELECT info.name, info.icon_src, info.author, info.short_desc
    FROM info INNER JOIN versions ON info.name = versions.name
    WHERE info.author LIKE ?
    GROUP BY info.name
    ORDER BY MAX(versions.id) DESC
";

const MOD_LIST_FILTERED: &'static str = r#"
    SELECT info.name, info.icon_src, info.author, info.short_desc
    FROM info INNER JOIN versions ON info.name = versions.name
    WHERE info.name LIKE ? AND info.author LIKE ?
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
    format!("[{:0>2}:{:0>2}]", hours as u64, minutes as u64)
}
