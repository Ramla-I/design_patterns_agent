use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use super::types::{LlmClient, LlmRequest, LlmResponse};

/// Wrapper around an LlmClient that transparently tracks total tokens used.
pub struct TokenTrackingClient {
    inner: Arc<dyn LlmClient>,
    pub total_tokens: Arc<AtomicU64>,
}

impl TokenTrackingClient {
    pub fn new(inner: Arc<dyn LlmClient>, total_tokens: Arc<AtomicU64>) -> Self {
        Self {
            inner,
            total_tokens,
        }
    }
}

#[async_trait]
impl LlmClient for TokenTrackingClient {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let response = self.inner.complete(request).await?;
        self.total_tokens
            .fetch_add(response.tokens_used as u64, Ordering::Relaxed);
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockClient;

    #[async_trait]
    impl LlmClient for MockClient {
        async fn complete(&self, _request: LlmRequest) -> Result<LlmResponse> {
            Ok(LlmResponse {
                content: "[]".to_string(),
                tokens_used: 150,
                prompt_tokens: 100,
                completion_tokens: 50,
                cached_tokens: 0,
                reasoning_tokens: 0,
            })
        }
    }

    #[tokio::test]
    async fn test_token_tracking() {
        let total = Arc::new(AtomicU64::new(0));
        let client = TokenTrackingClient::new(Arc::new(MockClient), total.clone());

        let req = LlmRequest::new("sys", "user");
        client.complete(req).await.unwrap();
        assert_eq!(total.load(Ordering::Relaxed), 150);

        let req2 = LlmRequest::new("sys", "user");
        client.complete(req2).await.unwrap();
        assert_eq!(total.load(Ordering::Relaxed), 300);
    }
}
