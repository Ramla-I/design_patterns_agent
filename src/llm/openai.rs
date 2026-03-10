use anyhow::Result;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;

use super::types::{LlmClient, LlmRequest, LlmResponse};

pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAIClient {
    pub fn new(api_key: String, model: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);

        Self { client, model }
    }

    /// Returns true if the model is known to reject custom temperature values.
    fn model_rejects_temperature(&self) -> bool {
        let m = self.model.as_str();
        m.starts_with("o1")
            || m.starts_with("o3")
            || m.starts_with("o4")
            || m.starts_with("gpt-5")
    }
}

#[async_trait]
impl LlmClient for OpenAIClient {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(request.system_prompt)
                    .build()?,
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(request.user_prompt)
                    .build()?,
            ),
        ];

        let mut builder = CreateChatCompletionRequestArgs::default();
        builder.model(&self.model).messages(messages);
        // Some models (o1, o3, gpt-5, etc.) don't support custom temperature.
        // Only set it when the model is known to accept it.
        if let Some(temp) = request.temperature {
            if !self.model_rejects_temperature() {
                builder.temperature(temp);
            }
        }
        let chat_request = builder.build()?;

        let response = self.client.chat().create(chat_request).await?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response content from OpenAI"))?;

        let (tokens_used, prompt_tokens, completion_tokens, cached_tokens, reasoning_tokens) = response
            .usage
            .map(|u| {
                let cached = u.prompt_tokens_details
                    .as_ref()
                    .and_then(|d| d.cached_tokens)
                    .unwrap_or(0) as usize;
                let reasoning = u.completion_tokens_details
                    .as_ref()
                    .and_then(|d| d.reasoning_tokens)
                    .unwrap_or(0) as usize;
                (u.total_tokens as usize, u.prompt_tokens as usize, u.completion_tokens as usize, cached, reasoning)
            })
            .unwrap_or((0, 0, 0, 0, 0));

        Ok(LlmResponse {
            content,
            tokens_used,
            prompt_tokens,
            completion_tokens,
            cached_tokens,
            reasoning_tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_client_creation() {
        let client = OpenAIClient::new("test-key".to_string(), "gpt-4".to_string());
        assert_eq!(client.model, "gpt-4");
    }

    // Note: Actual API tests would require a real API key and should be integration tests
    // We can add those later with proper mocking or feature flags
}
