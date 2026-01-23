mod config;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

pub use config::Config;

#[derive(Parser, Debug)]
#[command(name = "design_patterns_agent")]
#[command(about = "Analyze Rust codebases to discover invariants", long_about = None)]
pub struct Args {
    /// Path to the Rust codebase to analyze
    #[arg(value_name = "PATH")]
    pub codebase_path: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value = "markdown")]
    pub format: OutputFormat,

    /// Output file path (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Maximum exploration depth
    #[arg(long, default_value = "10")]
    pub max_depth: usize,

    /// Maximum items to analyze per module
    #[arg(long, default_value = "50")]
    pub max_items_per_module: usize,

    /// OpenAI API key (or set OPENAI_API_KEY env var)
    #[arg(long, env = "OPENAI_API_KEY")]
    pub api_key: Option<String>,

    /// LLM model to use
    #[arg(long, default_value = "gpt-4")]
    pub model: String,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Markdown,
    Json,
}

pub async fn run() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = if let Some(config_path) = args.config {
        Config::from_file(&config_path)?
    } else {
        Config::from_args(&args)?
    };

    // Validate codebase path
    if !args.codebase_path.exists() {
        anyhow::bail!("Codebase path does not exist: {:?}", args.codebase_path);
    }

    // Run the analysis
    let report = crate::agent::analyze_codebase(&args.codebase_path, &config).await?;

    // Generate output
    let output = match args.format {
        OutputFormat::Markdown => crate::report::generate_markdown(&report)?,
        OutputFormat::Json => crate::report::generate_json(&report)?,
    };

    // Write to file or stdout
    if let Some(output_path) = args.output {
        std::fs::write(output_path, output)?;
    } else {
        println!("{}", output);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(&["prog", "/path/to/codebase"]);
        assert_eq!(args.codebase_path, PathBuf::from("/path/to/codebase"));
        assert_eq!(args.max_depth, 10);
    }

    #[test]
    fn test_args_with_options() {
        let args = Args::parse_from(&[
            "prog",
            "/path/to/codebase",
            "--format",
            "json",
            "--max-depth",
            "5",
            "--model",
            "gpt-4-turbo",
        ]);
        assert_eq!(args.max_depth, 5);
        assert_eq!(args.model, "gpt-4-turbo");
    }
}
