mod anthropic;
mod openai;
pub mod retry;
pub mod tracking;
mod types;

pub use anthropic::AnthropicClient;
pub use openai::OpenAIClient;
pub use retry::RetryClient;
pub use tracking::{TokenStats, TokenTrackingClient};
pub use types::{LlmClient, LlmRequest};

use anyhow::Result;

/// Create an LLM client based on the provider name
pub fn create_client(provider: &str, api_key: String, model: String) -> Result<Box<dyn LlmClient>> {
    match provider {
        "anthropic" => Ok(Box::new(AnthropicClient::new(api_key, model))),
        "openai" | _ => Ok(Box::new(OpenAIClient::new(api_key, model))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_openai_client() {
        let client = create_client("openai", "test-key".to_string(), "gpt-4".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_create_anthropic_client() {
        let client = create_client("anthropic", "test-key".to_string(), "claude-sonnet-4-6".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_create_default_client() {
        let client = create_client("unknown", "test-key".to_string(), "model".to_string());
        assert!(client.is_ok());
    }
}
