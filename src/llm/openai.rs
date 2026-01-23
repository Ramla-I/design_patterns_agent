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

        let chat_request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .temperature(request.temperature)
            .build()?;

        let response = self.client.chat().create(chat_request).await?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response content from OpenAI"))?;

        let tokens_used = response.usage.map(|u| u.total_tokens as usize).unwrap_or(0);

        Ok(LlmResponse {
            content,
            tokens_used,
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
