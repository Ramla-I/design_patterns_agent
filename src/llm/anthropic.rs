use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::types::{LlmClient, LlmRequest, LlmResponse};

pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct AnthropicRequestBody {
    model: String,
    max_tokens: u32,
    temperature: f32,
    system: String,
    messages: Vec<AnthropicMessage>,
}

#[derive(Deserialize)]
struct AnthropicResponseBody {
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: usize,
    output_tokens: usize,
    #[serde(default)]
    cache_read_input_tokens: usize,
    #[serde(default)]
    cache_creation_input_tokens: usize,
}

impl AnthropicClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let body = AnthropicRequestBody {
            model: self.model.clone(),
            max_tokens: 8192,
            temperature: request.temperature,
            system: request.system_prompt,
            messages: vec![AnthropicMessage {
                role: "user",
                content: request.user_prompt,
            }],
        };

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error ({}): {}", status, error_text);
        }

        let parsed: AnthropicResponseBody = resp.json().await?;

        let content = parsed
            .content
            .first()
            .map(|b| b.text.clone())
            .ok_or_else(|| anyhow::anyhow!("No content in Anthropic response"))?;

        Ok(LlmResponse {
            content,
            tokens_used: parsed.usage.input_tokens + parsed.usage.output_tokens,
            prompt_tokens: parsed.usage.input_tokens,
            completion_tokens: parsed.usage.output_tokens,
            cached_tokens: parsed.usage.cache_read_input_tokens + parsed.usage.cache_creation_input_tokens,
            reasoning_tokens: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_client_creation() {
        let client = AnthropicClient::new("test-key".to_string(), "claude-sonnet-4-6".to_string());
        assert_eq!(client.model, "claude-sonnet-4-6");
    }
}
