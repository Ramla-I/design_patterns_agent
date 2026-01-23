use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm: LlmConfig,
    pub exploration: ExplorationConfig,
    pub detection: DetectionConfig,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvariantType {
    StateMachine,
    LinearTypes,
    Ownership,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn from_args(args: &super::Args) -> Result<Self> {
        let api_key = args
            .api_key
            .clone()
            .ok_or_else(|| anyhow::anyhow!("API key not provided. Set OPENAI_API_KEY or use --api-key"))?;

        Ok(Config {
            llm: LlmConfig {
                provider: "openai".to_string(),
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
                    InvariantType::StateMachine,
                    InvariantType::LinearTypes,
                    InvariantType::Ownership,
                ],
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
focus = ["state_machine", "linear_types", "ownership"]
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.model, "gpt-4");
        assert_eq!(config.exploration.max_depth, 10);
        assert_eq!(config.detection.focus.len(), 3);
    }

    #[test]
    fn test_invariant_type_deserialization() {
        let json = r#""state_machine""#;
        let invariant_type: InvariantType = serde_json::from_str(json).unwrap();
        assert_eq!(invariant_type, InvariantType::StateMachine);
    }
}
