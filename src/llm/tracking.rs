use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

use super::types::{LlmClient, LlmRequest, LlmResponse};

/// Serializable snapshot of token usage counters.
#[derive(Debug, Clone, Serialize)]
pub struct TokenStatsSnapshot {
    pub input_tokens: u64,
    pub cached_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_tokens: u64,
    pub total_tokens: u64,
}

/// Granular token usage counters, safe for concurrent access.
#[derive(Debug)]
pub struct TokenStats {
    pub input_tokens: AtomicU64,
    pub cached_tokens: AtomicU64,
    pub output_tokens: AtomicU64,
    pub reasoning_tokens: AtomicU64,
    pub total_tokens: AtomicU64,
}

impl TokenStats {
    pub fn new() -> Self {
        Self {
            input_tokens: AtomicU64::new(0),
            cached_tokens: AtomicU64::new(0),
            output_tokens: AtomicU64::new(0),
            reasoning_tokens: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
        }
    }

    fn record(&self, response: &LlmResponse) {
        self.input_tokens.fetch_add(response.prompt_tokens as u64, Ordering::Relaxed);
        self.cached_tokens.fetch_add(response.cached_tokens as u64, Ordering::Relaxed);
        self.output_tokens.fetch_add(response.completion_tokens as u64, Ordering::Relaxed);
        self.reasoning_tokens.fetch_add(response.reasoning_tokens as u64, Ordering::Relaxed);
        self.total_tokens.fetch_add(response.tokens_used as u64, Ordering::Relaxed);
    }

    /// Return a serializable snapshot of the current counters.
    pub fn snapshot(&self) -> TokenStatsSnapshot {
        TokenStatsSnapshot {
            input_tokens: self.input_tokens.load(Ordering::Relaxed),
            cached_tokens: self.cached_tokens.load(Ordering::Relaxed),
            output_tokens: self.output_tokens.load(Ordering::Relaxed),
            reasoning_tokens: self.reasoning_tokens.load(Ordering::Relaxed),
            total_tokens: self.total_tokens.load(Ordering::Relaxed),
        }
    }

    /// Print a labeled summary of token usage. Returns total tokens.
    pub fn print_summary(&self, label: &str) -> u64 {
        let total = self.total_tokens.load(Ordering::Relaxed);
        if total == 0 {
            return 0;
        }
        let input = self.input_tokens.load(Ordering::Relaxed);
        let cached = self.cached_tokens.load(Ordering::Relaxed);
        let output = self.output_tokens.load(Ordering::Relaxed);
        let reasoning = self.reasoning_tokens.load(Ordering::Relaxed);

        println!("  {} tokens:", label);
        println!("    Input:     {:>10}", input);
        if cached > 0 {
            println!("    Cached:    {:>10}", cached);
        }
        println!("    Output:    {:>10}", output);
        if reasoning > 0 {
            println!("    Reasoning: {:>10}", reasoning);
        }
        println!("    Total:     {:>10}", total);
        total
    }
}

/// Wrapper around an LlmClient that transparently tracks token usage.
pub struct TokenTrackingClient {
    inner: Arc<dyn LlmClient>,
    pub stats: Arc<TokenStats>,
}

impl TokenTrackingClient {
    pub fn new(inner: Arc<dyn LlmClient>, stats: Arc<TokenStats>) -> Self {
        Self { inner, stats }
    }
}

#[async_trait]
impl LlmClient for TokenTrackingClient {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let response = self.inner.complete(request).await?;
        self.stats.record(&response);
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
                cached_tokens: 20,
                reasoning_tokens: 10,
            })
        }
    }

    #[tokio::test]
    async fn test_token_tracking() {
        let stats = Arc::new(TokenStats::new());
        let client = TokenTrackingClient::new(Arc::new(MockClient), stats.clone());

        let req = LlmRequest::new("sys", "user");
        client.complete(req).await.unwrap();
        assert_eq!(stats.total_tokens.load(Ordering::Relaxed), 150);
        assert_eq!(stats.input_tokens.load(Ordering::Relaxed), 100);
        assert_eq!(stats.output_tokens.load(Ordering::Relaxed), 50);
        assert_eq!(stats.cached_tokens.load(Ordering::Relaxed), 20);
        assert_eq!(stats.reasoning_tokens.load(Ordering::Relaxed), 10);

        let req2 = LlmRequest::new("sys", "user");
        client.complete(req2).await.unwrap();
        assert_eq!(stats.total_tokens.load(Ordering::Relaxed), 300);
        assert_eq!(stats.input_tokens.load(Ordering::Relaxed), 200);
        assert_eq!(stats.output_tokens.load(Ordering::Relaxed), 100);
    }
}
