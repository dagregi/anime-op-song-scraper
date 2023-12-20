use std::path::PathBuf;

use anyhow::Context;
use askama::Template;
use axum::{routing::get, Router};
use scraper::{Html, Selector};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("routing initialized...");
    let app = Router::new()
        .route("/", get(hello))
        // Serve static files from the "assets" directory
        .nest_service("/assets", ServeDir::new(PathBuf::from("./assets")))
        .layer(tower_http::trace::TraceLayer::new_for_http());
    let port = 8000_u16;
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("listening on {}", addr);

    Ok(axum::serve(listener, app).await?)
}

async fn hello() -> HelloTemplate {
    HelloTemplate {}
}

#[derive(Template)]
#[template(path = "index.html")]
struct HelloTemplate;

struct ScrapedData {
    title: String,
    link: String
}
async fn scrape_download_links(url: &str) -> anyhow::Result<Vec<ScrapedData>> {
    let body = reqwest::get(url).await?.text().await?;
    // Parse the body
    let document = Html::parse_document(&body);
    let selector = Selector::parse(".sbutton").unwrap();
    let mut scraped_data = Vec::new();
    // Useing scraper to extract the link and title
    for element in document.select(&selector) {
        let link = element
            .select(&Selector::parse("a").unwrap())
            .next()
            .and_then(|a| a.value().attr("href"))
            .with_context(|| "Link not found")?;

        let title = element
            .select(&Selector::parse(".bottom").unwrap())
            .next()
            .map(|e| e.text().collect::<String>())
            .with_context(|| "Couldn't parse the title")?;

        scraped_data.push(ScrapedData {title, link: link.to_string()});
    }

    Ok(scraped_data)
}
