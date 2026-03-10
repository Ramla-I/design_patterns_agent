use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub temperature: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LlmResponse {
    pub content: String,
    pub tokens_used: usize,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub cached_tokens: usize,
    pub reasoning_tokens: usize,
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse>;
}

impl LlmRequest {
    pub fn new(system_prompt: impl Into<String>, user_prompt: impl Into<String>) -> Self {
        Self {
            system_prompt: system_prompt.into(),
            user_prompt: user_prompt.into(),
            temperature: 0.7,
        }
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_request_creation() {
        let request = LlmRequest::new("system", "user");
        assert_eq!(request.system_prompt, "system");
        assert_eq!(request.user_prompt, "user");
        assert_eq!(request.temperature, 0.7);
    }

    #[test]
    fn test_llm_request_with_temperature() {
        let request = LlmRequest::new("sys", "usr").with_temperature(0.5);
        assert_eq!(request.temperature, 0.5);
    }
}
