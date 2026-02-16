mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub use config::Config;

#[derive(Parser, Debug)]
#[command(name = "design_patterns_agent")]
#[command(about = "Analyze Rust codebases to discover invariants and translate C2Rust code", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Path to the Rust codebase to analyze (for analyze command)
    #[arg(value_name = "PATH", global = true)]
    pub codebase_path: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "markdown", global = true)]
    pub format: OutputFormat,

    /// Output file path (stdout if not specified)
    #[arg(short, long, global = true)]
    pub output: Option<PathBuf>,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Maximum exploration depth
    #[arg(long, default_value = "10", global = true)]
    pub max_depth: usize,

    /// Maximum items to analyze per module
    #[arg(long, default_value = "50", global = true)]
    pub max_items_per_module: usize,

    /// OpenAI API key (or set OPENAI_API_KEY env var)
    #[arg(long, env = "OPENAI_API_KEY", global = true)]
    pub api_key: Option<String>,

    /// LLM model to use
    #[arg(long, default_value = "gpt-5.2", global = true)]
    pub model: String,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Analyze a Rust codebase for design patterns and invariants
    Analyze {
        /// Path to the Rust codebase to analyze
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },

    /// Translate C2Rust code to idiomatic Rust
    Translate {
        /// Path to Public-Tests directory or a specific program
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Maximum number of retry attempts for failed translations
        #[arg(long, default_value = "5")]
        max_retries: usize,

        /// Maximum lines in a source file before skipping
        #[arg(long, default_value = "1000")]
        max_lines: usize,

        /// Run design patterns analysis on successful translations
        #[arg(long)]
        analyze: bool,

        /// Skip running tests (only verify build succeeds)
        #[arg(long)]
        skip_tests: bool,

        /// Output summary report file
        #[arg(long)]
        report: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Markdown,
    Json,
}

/// Legacy Args struct for backward compatibility
#[derive(Debug)]
pub struct Args {
    pub codebase_path: PathBuf,
    pub format: OutputFormat,
    pub output: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub max_depth: usize,
    pub max_items_per_module: usize,
    pub api_key: Option<String>,
    pub model: String,
}

impl Args {
    pub fn from_cli(cli: &Cli, path: PathBuf) -> Self {
        Self {
            codebase_path: path,
            format: cli.format,
            output: cli.output.clone(),
            config: cli.config.clone(),
            max_depth: cli.max_depth,
            max_items_per_module: cli.max_items_per_module,
            api_key: cli.api_key.clone(),
            model: cli.model.clone(),
        }
    }

    #[cfg(test)]
    pub fn parse_from<I, T>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let cli = Cli::parse_from(iter);
        let path = cli.codebase_path.clone().unwrap_or_default();
        Self::from_cli(&cli, path)
    }
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Command::Analyze { path }) => {
            run_analyze(&cli, path).await
        }
        Some(Command::Translate { path, max_retries, max_lines, analyze, skip_tests, report }) => {
            run_translate(&cli, path, *max_retries, *max_lines, *analyze, *skip_tests, report.as_ref()).await
        }
        None => {
            // Default behavior: analyze if path provided
            if let Some(path) = &cli.codebase_path {
                run_analyze(&cli, path).await
            } else {
                anyhow::bail!("No command or path specified. Use --help for usage information.")
            }
        }
    }
}

async fn run_analyze(cli: &Cli, path: &PathBuf) -> Result<()> {
    let args = Args::from_cli(cli, path.clone());

    // Load configuration
    let config = if let Some(config_path) = &args.config {
        Config::from_file(config_path)?
    } else {
        Config::from_args(&args)?
    };

    // Validate codebase path
    if !path.exists() {
        anyhow::bail!("Codebase path does not exist: {:?}", path);
    }

    // Run the analysis
    let report = crate::agent::analyze_codebase(path, &config).await?;

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

async fn run_translate(
    cli: &Cli,
    path: &PathBuf,
    max_retries: usize,
    max_lines: usize,
    analyze: bool,
    skip_tests: bool,
    report_path: Option<&PathBuf>,
) -> Result<()> {
    use crate::translation::{TranslationAgent, TranslationConfig};

    // Validate path
    if !path.exists() {
        anyhow::bail!("Path does not exist: {:?}", path);
    }

    // Create a dummy Args for Config creation
    let args = Args::from_cli(cli, path.clone());

    // Load configuration
    let llm_config = if let Some(config_path) = &args.config {
        Config::from_file(config_path)?
    } else {
        Config::from_args(&args)?
    };

    // Create translation config
    let translation_config = TranslationConfig {
        max_retries,
        max_lines,
        analyze_patterns: analyze,
        skip_tests,
        ..Default::default()
    };

    // Create and run the translation agent
    let agent = TranslationAgent::new(translation_config, llm_config);
    let report = agent.translate_all(path).await?;

    // Write report
    let output = match cli.format {
        OutputFormat::Markdown => report.to_markdown(),
        OutputFormat::Json => serde_json::to_string_pretty(&report)?,
    };

    // Auto-place report in run directory
    if let Some(run_dir) = &report.run_dir {
        let ext = match cli.format {
            OutputFormat::Markdown => "md",
            OutputFormat::Json => "json",
        };
        let run_report_path = std::path::Path::new(run_dir).join(format!("report.{}", ext));
        std::fs::write(&run_report_path, &output)?;
        println!("\n📄 Report written to: {}", run_report_path.display());
    }

    // If --report was given, also write a copy there
    if let Some(report_path) = report_path {
        std::fs::write(report_path, &output)?;
        println!("📄 Report copy written to: {}", report_path.display());
    } else if cli.output.is_some() {
        std::fs::write(cli.output.as_ref().unwrap(), &output)?;
    }

    // Print run directory for easy access
    if let Some(run_dir) = &report.run_dir {
        println!("\n📂 All outputs saved to: {}", run_dir);
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

    #[test]
    fn test_translate_command_parsing() {
        let cli = Cli::parse_from(&[
            "prog",
            "translate",
            "/path/to/tests",
            "--max-retries",
            "3",
            "--analyze",
        ]);

        match cli.command {
            Some(Command::Translate { path, max_retries, analyze, .. }) => {
                assert_eq!(path, PathBuf::from("/path/to/tests"));
                assert_eq!(max_retries, 3);
                assert!(analyze);
            }
            _ => panic!("Expected Translate command"),
        }
    }

    #[test]
    fn test_analyze_command_parsing() {
        let cli = Cli::parse_from(&[
            "prog",
            "analyze",
            "/path/to/codebase",
        ]);

        match cli.command {
            Some(Command::Analyze { path }) => {
                assert_eq!(path, PathBuf::from("/path/to/codebase"));
            }
            _ => panic!("Expected Analyze command"),
        }
    }
}
