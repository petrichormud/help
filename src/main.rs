use dotenvy::dotenv;
use serde_derive::Deserialize;

use std::fs;

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
            "INSERT INTO help (slug, title, sub, category, pid, raw, html) VALUES (?, ?, ?, ?, ?, ?, ?);",
            doc.slug,
            doc.title,
            doc.sub,
            doc.category,
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
                    "INSERT INTO help_related (slug, related_title, related_sub, related_slug) VALUES (?, ?, ?, ?);",
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

        for tag in doc.tags.iter() {
            if let Err(err) = sqlx::query!(
                "INSERT INTO help_tags (slug, tag) VALUES (?, ?);",
                &doc.slug,
                tag,
            )
            .execute(&pool)
            .await
            {
                panic!("{:?}", err)
            }
        }
    }

    Ok(())
}

async fn get_slugs(path: &str) -> Vec<String> {
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

fn get_slug(file: fs::DirEntry) -> Option<String> {
    file.path()
        .with_extension("")
        .file_name()?
        .to_str()
        .map(|s| s.to_owned())
}

async fn read_help(
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

        let help = Help::builder()
            .slug(slug)
            .title(&metadata.title)
            .sub(&metadata.sub)
            .category(&metadata.category)
            .pid(r.id)
            .raw(&raw)
            .html(&html)
            .tags(metadata.tags)
            .related(metadata.related)
            .build();

        map.insert(slug.to_string(), help);
    }

    map
}

#[derive(Default)]
pub struct HelpBuilder {
    slug: String,
    title: String,
    sub: String,
    category: String,
    pid: i64,
    raw: String,
    html: String,
    tags: Vec<String>,
    related: Vec<String>,
}

impl HelpBuilder {
    fn slug(mut self, slug: &str) -> Self {
        self.slug = slug.to_owned();
        self
    }

    fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    fn sub(mut self, sub: &str) -> Self {
        self.sub = sub.to_owned();
        self
    }

    fn category(mut self, category: &str) -> Self {
        self.category = category.to_owned();
        self
    }

    fn pid(mut self, pid: i64) -> Self {
        self.pid = pid;
        self
    }

    fn raw(mut self, raw: &str) -> Self {
        self.raw = raw.to_owned();
        self
    }

    fn html(mut self, html: &str) -> Self {
        self.html = html.to_owned();
        self
    }

    fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    fn related(mut self, related: Vec<String>) -> Self {
        self.related = related;
        self
    }

    fn build(self) -> Help {
        Help {
            slug: self.slug,
            title: self.title,
            sub: self.sub,
            category: self.category,
            pid: self.pid,
            raw: self.raw,
            html: self.html,
            tags: self.tags,
            related: self.related,
        }
    }
}

#[derive(Debug)]
struct Help {
    slug: String,
    title: String,
    sub: String,
    category: String,
    pid: i64,
    raw: String,
    html: String,
    tags: Vec<String>,
    related: Vec<String>,
}

impl Help {
    fn builder() -> HelpBuilder {
        HelpBuilder::default()
    }
}

#[derive(Debug, Deserialize)]
struct Metadata {
    author: String,
    title: String,
    sub: String,
    category: String,
    tags: Vec<String>,
    related: Vec<String>,
}
