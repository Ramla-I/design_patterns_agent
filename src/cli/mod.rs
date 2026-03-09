pub mod config;

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
    #[arg(value_name = "CODEBASE_PATH")]
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

    /// Maximum items to analyze per module (legacy, used in exhaustive mode)
    #[arg(long, default_value = "50", global = true)]
    pub max_items_per_module: usize,

    /// Search mode: "exhaustive" (parse all modules) or "semantic" (use octocode)
    #[arg(long, default_value = "exhaustive", global = true)]
    pub search_mode: String,

    /// Minimum similarity threshold for semantic search (0.0-1.0)
    #[arg(long, default_value = "0.1", global = true)]
    pub similarity_threshold: f32,

    /// LLM provider to use ("openai" or "anthropic")
    #[arg(long, default_value = "openai", global = true)]
    pub provider: String,

    /// API key (or set OPENAI_API_KEY / ANTHROPIC_API_KEY env var)
    #[arg(long, env = "OPENAI_API_KEY", global = true)]
    pub api_key: Option<String>,

    /// LLM model to use
    #[arg(long, default_value = "gpt-5.2", global = true)]
    pub model: String,

    /// Number of concurrent LLM calls
    #[arg(long, default_value = "1", global = true)]
    pub concurrency: usize,

    /// Maximum token budget (0 = unlimited)
    #[arg(long, default_value = "0", global = true)]
    pub token_budget: usize,

    /// Path to existing progress.jsonl to resume from
    #[arg(long, global = true)]
    pub resume: Option<PathBuf>,

    /// Comma-separated module prefixes to prioritize (e.g. sync,io,fs,net,cell)
    #[arg(long, value_delimiter = ',', global = true)]
    pub priority_modules: Vec<String>,

    /// Discover and analyze multiple crates under one directory
    #[arg(long, global = true)]
    pub multi_crate: bool,

    /// Maximum retries per LLM call on transient errors (rate limits, timeouts)
    #[arg(long, default_value = "5", global = true)]
    pub max_retries: u32,

    /// Base delay in seconds for exponential backoff between retries
    #[arg(long, default_value = "2", global = true)]
    pub retry_base_delay: u64,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Analyze a Rust codebase for design patterns and invariants
    Analyze {
        /// Path to the Rust codebase to analyze
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },

    /// Translate C2Rust or C code to idiomatic Rust
    #[cfg(feature = "translation")]
    Translate {
        /// Paths to Public-Tests directories or specific programs
        #[arg(value_name = "PATH", required = true)]
        paths: Vec<PathBuf>,

        /// Maximum number of retry attempts for failed translations
        #[arg(long, default_value = "5")]
        max_retries: usize,

        /// Maximum lines in a source file before skipping
        #[arg(long, default_value = "2000")]
        max_lines: usize,

        /// Run design patterns analysis on successful translations
        #[arg(long)]
        analyze: bool,

        /// Skip running tests (only verify build succeeds)
        #[arg(long)]
        skip_tests: bool,

        /// Translate from C source (test_case/) instead of C2Rust output
        #[arg(long)]
        from_c: bool,

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
    pub provider: String,
    pub api_key: Option<String>,
    pub model: String,
    pub search_mode: String,
    pub similarity_threshold: f32,
    pub concurrency: usize,
    pub token_budget: usize,
    pub resume_path: Option<PathBuf>,
    pub priority_modules: Vec<String>,
    pub multi_crate: bool,
    pub max_retries: u32,
    pub retry_base_delay: u64,
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
            provider: cli.provider.clone(),
            api_key: cli.api_key.clone(),
            model: cli.model.clone(),
            search_mode: cli.search_mode.clone(),
            similarity_threshold: cli.similarity_threshold,
            concurrency: cli.concurrency,
            token_budget: cli.token_budget,
            resume_path: cli.resume.clone(),
            priority_modules: cli.priority_modules.clone(),
            multi_crate: cli.multi_crate,
            max_retries: cli.max_retries,
            retry_base_delay: cli.retry_base_delay,
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
        #[cfg(feature = "translation")]
        Some(Command::Translate { paths, max_retries, max_lines, analyze, skip_tests, from_c, report }) => {
            run_translate(&cli, paths, *max_retries, *max_lines, *analyze, *skip_tests, *from_c, report.as_ref()).await
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
    let (report, run_dir) = crate::agent::analyze_codebase(path, &config).await?;

    // Generate output
    let ext = match args.format {
        OutputFormat::Markdown => "md",
        OutputFormat::Json => "json",
    };
    let output = match args.format {
        OutputFormat::Markdown => crate::report::generate_markdown(&report)?,
        OutputFormat::Json => crate::report::generate_json(&report)?,
    };

    // Write report to the run directory created by the agent
    let run_report_path = run_dir.join(format!("report.{}", ext));
    std::fs::write(&run_report_path, &output)?;
    println!("Report written to: {}", run_report_path.display());

    // If --output was given, also write a copy there
    if let Some(output_path) = args.output {
        std::fs::write(&output_path, &output)?;
        println!("Report copy written to: {}", output_path.display());
    }

    Ok(())
}

#[cfg(feature = "translation")]
async fn run_translate(
    cli: &Cli,
    paths: &[PathBuf],
    max_retries: usize,
    max_lines: usize,
    _analyze: bool,
    skip_tests: bool,
    from_c: bool,
    report_path: Option<&PathBuf>,
) -> Result<()> {
    use llm_translation::{TranslationAgent, TranslationConfig};

    // Validate paths
    for path in paths {
        if !path.exists() {
            anyhow::bail!("Path does not exist: {:?}", path);
        }
    }

    // Resolve API key
    let api_key = cli.api_key.clone().or_else(|| {
        match cli.provider.as_str() {
            "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
            _ => None,
        }
    }).ok_or_else(|| {
        let env_var = match cli.provider.as_str() {
            "anthropic" => "ANTHROPIC_API_KEY",
            _ => "OPENAI_API_KEY",
        };
        anyhow::anyhow!("API key not provided. Set {} or use --api-key", env_var)
    })?;

    // Create translation config
    let translation_config = TranslationConfig {
        max_retries,
        max_lines,
        skip_tests,
        from_c,
        provider: cli.provider.clone(),
        api_key,
        model: cli.model.clone(),
        ..Default::default()
    };

    // Create and run the translation agent
    let agent = TranslationAgent::new(translation_config);
    let report = agent.translate_all(paths).await?;

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

    #[cfg(feature = "translation")]
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
            Some(Command::Translate { paths, max_retries, analyze, .. }) => {
                assert_eq!(paths, vec![PathBuf::from("/path/to/tests")]);
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
