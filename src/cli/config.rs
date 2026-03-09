use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm: LlmConfig,
    pub exploration: ExplorationConfig,
    pub detection: DetectionConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    #[serde(default)]
    pub token_budget: usize,
    #[serde(default)]
    pub resume_path: Option<std::path::PathBuf>,
    #[serde(default)]
    pub priority_modules: Vec<String>,
    #[serde(default)]
    pub multi_crate: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_base_delay")]
    pub retry_base_delay: u64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            concurrency: default_concurrency(),
            token_budget: 0,
            resume_path: None,
            priority_modules: vec![],
            multi_crate: false,
            max_retries: default_max_retries(),
            retry_base_delay: default_retry_base_delay(),
        }
    }
}

fn default_concurrency() -> usize { 1 }
fn default_max_retries() -> u32 { 5 }
fn default_retry_base_delay() -> u64 { 2 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_search_mode")]
    pub mode: SearchMode,
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,
    #[serde(default = "default_max_results_per_query")]
    pub max_results_per_query: usize,
    #[serde(default = "default_context_lines")]
    pub context_lines: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            mode: default_search_mode(),
            similarity_threshold: default_similarity_threshold(),
            max_results_per_query: default_max_results_per_query(),
            context_lines: default_context_lines(),
        }
    }
}

fn default_search_mode() -> SearchMode { SearchMode::Exhaustive }
fn default_similarity_threshold() -> f32 { 0.1 }
fn default_max_results_per_query() -> usize { 20 }
fn default_context_lines() -> usize { 30 }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    Exhaustive,
    Semantic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationConfig {
    pub max_depth: usize,
    pub max_items_per_module: usize,
    pub context_window_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    pub focus: Vec<InvariantType>,
    #[serde(default = "default_min_confidence")]
    pub min_confidence: MinConfidence,
}

fn default_min_confidence() -> MinConfidence {
    MinConfidence::Medium
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvariantType {
    TemporalOrdering,
    ResourceLifecycle,
    StateMachine,
    Precondition,
    Protocol,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MinConfidence {
    High,
    Medium,
    Low,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn from_args(args: &super::Args) -> Result<Self> {
        let api_key = args.api_key.clone().or_else(|| {
            match args.provider.as_str() {
                "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
                _ => None,
            }
        }).ok_or_else(|| {
            let env_var = match args.provider.as_str() {
                "anthropic" => "ANTHROPIC_API_KEY",
                _ => "OPENAI_API_KEY",
            };
            anyhow::anyhow!("API key not provided. Set {} or use --api-key", env_var)
        })?;

        let search_mode = match args.search_mode.as_str() {
            "semantic" => SearchMode::Semantic,
            _ => SearchMode::Exhaustive,
        };

        Ok(Config {
            llm: LlmConfig {
                provider: args.provider.clone(),
                api_key,
                model: args.model.clone(),
            },
            exploration: ExplorationConfig {
                max_depth: args.max_depth,
                max_items_per_module: args.max_items_per_module,
                context_window_tokens: 4000,
            },
            detection: DetectionConfig {
                focus: vec![
                    InvariantType::TemporalOrdering,
                    InvariantType::ResourceLifecycle,
                    InvariantType::StateMachine,
                    InvariantType::Precondition,
                    InvariantType::Protocol,
                ],
                min_confidence: MinConfidence::Medium,
            },
            search: SearchConfig {
                mode: search_mode,
                similarity_threshold: args.similarity_threshold,
                ..Default::default()
            },
            execution: ExecutionConfig {
                concurrency: args.concurrency,
                token_budget: args.token_budget,
                resume_path: args.resume_path.clone(),
                priority_modules: args.priority_modules.clone(),
                multi_crate: args.multi_crate,
                max_retries: args.max_retries,
                retry_base_delay: args.retry_base_delay,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_config_from_toml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
[llm]
provider = "openai"
api_key = "test-key"
model = "gpt-4"

[exploration]
max_depth = 10
max_items_per_module = 50
context_window_tokens = 4000

[detection]
focus = ["temporal_ordering", "resource_lifecycle", "state_machine"]
min_confidence = "medium"
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.model, "gpt-4");
        assert_eq!(config.exploration.max_depth, 10);
        assert_eq!(config.detection.focus.len(), 3);
        assert_eq!(config.detection.min_confidence, MinConfidence::Medium);
    }

    #[test]
    fn test_config_with_semantic_search() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
[llm]
provider = "openai"
api_key = "test-key"
model = "gpt-4"

[exploration]
max_depth = 10
max_items_per_module = 50
context_window_tokens = 4000

[detection]
focus = ["temporal_ordering"]
min_confidence = "low"

[search]
mode = "semantic"
similarity_threshold = 0.2
max_results_per_query = 30
context_lines = 40
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(config.search.mode, SearchMode::Semantic);
        assert!((config.search.similarity_threshold - 0.2).abs() < 0.01);
        assert_eq!(config.search.max_results_per_query, 30);
        assert_eq!(config.search.context_lines, 40);
    }

    #[test]
    fn test_config_defaults_without_search_section() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
[llm]
provider = "openai"
api_key = "test-key"
model = "gpt-4"

[exploration]
max_depth = 10
max_items_per_module = 50
context_window_tokens = 4000

[detection]
focus = ["temporal_ordering"]
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(config.search.mode, SearchMode::Exhaustive);
        assert!((config.search.similarity_threshold - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_invariant_type_deserialization() {
        let json = r#""temporal_ordering""#;
        let invariant_type: InvariantType = serde_json::from_str(json).unwrap();
        assert_eq!(invariant_type, InvariantType::TemporalOrdering);
    }
}
