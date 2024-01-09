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

    if let Err(err) = sqlx::query!("DELETE FROM help_related;")
        .execute(&pool)
        .await
    {
        panic!("{:?}", err)
    }

    if let Err(err) = sqlx::query!("DELETE FROM help;").execute(&pool).await {
        panic!("{:?}", err)
    }

    let help = read_help(&pool, "data", &get_slugs("data/help").await).await;
    let docs = help.values();

    for doc in docs {
        if let Err(err) = sqlx::query!(
            "INSERT INTO help (slug, title, sub, pid, raw, html) VALUES (?, ?, ?, ?, ?, ?);",
            doc.slug,
            doc.title,
            doc.sub,
            doc.pid,
            doc.raw,
            doc.html
        )
        .execute(&pool)
        .await
        {
            panic!("{:?}", err)
        }

        for related_slug in doc.related.iter() {
            if let Some(related_doc) = help.get(related_slug) {
                if let Err(err) = sqlx::query!(
                    "INSERT INTO help_related (slug, related_title, related_sub, related) VALUES (?, ?, ?, ?);",
                    &doc.slug,
                    &related_doc.title,
                    &related_doc.sub,
                    related_slug,
                )
                .execute(&pool)
                .await
                {
                    panic!("{:?}", err)
                }
            }
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

        map.insert(
            slug.to_string(),
            Help::new(
                slug,
                &metadata.title,
                &metadata.sub,
                r.id,
                raw,
                html,
                metadata.related,
            ),
        );
    }

    map
}

#[derive(Debug)]
pub struct Help {
    slug: String,
    title: String,
    sub: String,
    pid: i64,
    raw: String,
    html: String,
    related: Vec<String>,
}

impl Help {
    pub fn new(
        slug: &str,
        title: &str,
        sub: &str,
        pid: i64,
        raw: String,
        html: String,
        related: Vec<String>,
    ) -> Self {
        Help {
            slug: slug.to_string(),
            title: title.to_string(),
            sub: sub.to_string(),
            pid,
            raw,
            html,
            related,
        }
    }

    pub fn add_related(&mut self, slug: &str) {
        self.related.push(slug.to_string());
    }
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub author: String,
    pub title: String,
    pub sub: String,
    pub related: Vec<String>,
}
