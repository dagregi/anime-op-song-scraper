mod args;
mod downloader;
mod scrapers;
mod structs;

use clap::Parser;
use scrapers::search_page::search_results;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli_args = args::Arguments::parse();
    if let Some(search) = cli_args.search {
        search_results(&search, cli_args.all, cli_args.type_).await?;
    }
    Ok(())
}
