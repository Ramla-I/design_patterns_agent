mod cli;
mod parser;
mod navigation;
mod agent;
mod detection;
mod llm;
mod report;
mod translation;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run().await
}
