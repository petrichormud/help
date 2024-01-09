use dotenvy::dotenv;
use serde_derive::Deserialize;

use std::fs;

mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::FULL)
        .init();

    let pool =
        sqlx::MySqlPool::connect("mysql://root:pass@localhost:3306/test?parseTime=true").await?;

    if let Err(err) = sqlx::query!("DELETE FROM help;").execute(&pool).await {
        panic!("{:?}", err)
    }

    for (_, doc) in read_help(&pool, "data", &get_slugs("data/help").await).await {
        if let Err(err) = sqlx::query!(
            "INSERT INTO help (slug, pid, raw, html) VALUES (?, ?, ?, ?);",
            doc.slug,
            doc.pid,
            doc.raw,
            doc.html
        )
        .execute(&pool)
        .await
        {
            panic!("{:?}", err)
        }
    }

    Ok(())
}

pub async fn get_slugs(path: &str) -> Vec<String> {
    let mut slugs: Vec<String> = vec![];

    if let Ok(files) = fs::read_dir(path) {
        for file in files.flatten() {
            if let Some(slug) = get_slug(file) {
                slugs.push(slug);
            }
        }
    }

    slugs
}

pub fn get_slug(file: fs::DirEntry) -> Option<String> {
    file.path()
        .with_extension("")
        .file_name()?
        .to_str()
        .map(|s| s.to_owned())
}

pub async fn read_help(
    pool: &sqlx::Pool<sqlx::MySql>,
    dir: &str,
    slugs: &[String],
) -> hashbrown::HashMap<String, Help> {
    let mut map = hashbrown::HashMap::new();

    for slug in slugs {
        let metadata: Metadata = match fs::read_to_string(format!("{dir}/metadata/{slug}.toml")) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(metadata) => metadata,
                Err(err) => {
                    panic!("provided string is not valid TOML, got error: {err}")
                }
            },
            Err(err) => {
                panic!("Could not read file, got error: {:?}", err);
            }
        };

        let raw = match fs::read_to_string(format!("{dir}/help/{slug}.md")) {
            Ok(c) => c,
            Err(err) => {
                panic!("Could not read file, got error: {:?}", err);
            }
        };

        let html = markdown::to_html(&raw);

        let r = match sqlx::query!("SELECT id FROM players WHERE username = ?", metadata.author)
            .fetch_one(pool)
            .await
        {
            Ok(aid) => aid,
            Err(err) => panic!("Err getting author id: {:?}", err),
        };

        map.insert(slug.to_string(), Help::new(slug, r.id, raw, html));
    }

    map
}

#[derive(Debug)]
pub struct Help {
    slug: String,
    pid: i64,
    raw: String,
    html: String,
}

impl Help {
    pub fn new(slug: &str, pid: i64, raw: String, html: String) -> Self {
        Help {
            slug: slug.to_string(),
            pid,
            raw,
            html,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub author: String,
    pub related: Vec<String>,
}
