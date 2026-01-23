mod openai;
mod types;

pub use openai::OpenAIClient;
pub use types::{LlmClient, LlmRequest, LlmResponse};

use anyhow::Result;

/// Create an LLM client based on the configuration
pub fn create_client(api_key: String, model: String) -> Result<Box<dyn LlmClient>> {
    Ok(Box::new(OpenAIClient::new(api_key, model)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_client() {
        let client = create_client("test-key".to_string(), "gpt-4".to_string());
        assert!(client.is_ok());
    }
}
