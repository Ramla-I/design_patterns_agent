use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;

use super::types::{LlmClient, LlmRequest, LlmResponse};

/// Classifies an error as retryable or permanent.
fn is_retryable(err: &anyhow::Error) -> bool {
    let msg = format!("{:#}", err).to_lowercase();

    // HTTP status codes that are retryable
    let retryable_patterns = [
        "429",            // rate limit
        "rate limit",
        "rate_limit",
        "too many requests",
        "500",            // internal server error
        "502",            // bad gateway
        "503",            // service unavailable
        "529",            // overloaded (Anthropic)
        "overloaded",
        "capacity",
        "timeout",
        "timed out",
        "connection reset",
        "connection refused",
        "connection closed",
        "broken pipe",
        "eof",
        "dns error",
        "network",
        "temporarily unavailable",
        "server error",
    ];

    for pattern in &retryable_patterns {
        if msg.contains(pattern) {
            return true;
        }
    }

    false
}

/// Extracts a Retry-After hint (in seconds) from an error message, if present.
fn parse_retry_after(err: &anyhow::Error) -> Option<u64> {
    let msg = format!("{:#}", err);
    // Look for patterns like "retry after 30" or "retry-after: 30" or "try again in 30 seconds"
    let lower = msg.to_lowercase();

    for prefix in &["retry after ", "retry-after: ", "try again in "] {
        if let Some(pos) = lower.find(prefix) {
            let after = &msg[pos + prefix.len()..];
            if let Some(num_str) = after.split(|c: char| !c.is_ascii_digit()).next() {
                if let Ok(secs) = num_str.parse::<u64>() {
                    return Some(secs);
                }
            }
        }
    }

    None
}

/// Wrapper around an LlmClient that retries transient/rate-limit errors
/// with exponential backoff and jitter.
pub struct RetryClient {
    inner: Arc<dyn LlmClient>,
    max_retries: u32,
    base_delay: Duration,
    /// Tracks consecutive rate-limit hits across calls for adaptive backoff
    consecutive_rate_limits: AtomicU64,
}

impl RetryClient {
    pub fn new(inner: Arc<dyn LlmClient>, max_retries: u32, base_delay_secs: u64) -> Self {
        Self {
            inner,
            max_retries,
            base_delay: Duration::from_secs(base_delay_secs),
            consecutive_rate_limits: AtomicU64::new(0),
        }
    }

    fn compute_delay(&self, attempt: u32, err: &anyhow::Error) -> Duration {
        // If the server told us how long to wait, respect it (with a small buffer)
        if let Some(retry_after) = parse_retry_after(err) {
            return Duration::from_secs(retry_after + 1);
        }

        // Exponential backoff: base * 2^attempt
        let exp_delay = self.base_delay.as_millis() as u64 * 2u64.pow(attempt);

        // Add pressure from consecutive rate limits (backs off more aggressively
        // if the whole run is hitting limits, not just this one call)
        let consecutive = self.consecutive_rate_limits.load(Ordering::Relaxed);
        let pressure_multiplier = 1 + consecutive.min(5);

        let delay_ms = exp_delay * pressure_multiplier;

        // Jitter: +/- 25%
        let jitter_range = delay_ms / 4;
        let jitter = if jitter_range > 0 {
            // Simple deterministic-ish jitter from attempt number
            (attempt as u64 * 7 + consecutive * 13) % (jitter_range * 2)
        } else {
            0
        };
        let final_ms = delay_ms - jitter_range + jitter;

        // Cap at 5 minutes
        Duration::from_millis(final_ms.min(300_000))
    }
}

#[async_trait]
impl LlmClient for RetryClient {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let mut last_err = None;

        for attempt in 0..=self.max_retries {
            // Clone the request for each attempt
            let req = LlmRequest {
                system_prompt: request.system_prompt.clone(),
                user_prompt: request.user_prompt.clone(),
                temperature: request.temperature,
            };

            match self.inner.complete(req).await {
                Ok(response) => {
                    // Reset consecutive rate limit counter on success
                    self.consecutive_rate_limits.store(0, Ordering::Relaxed);
                    return Ok(response);
                }
                Err(err) => {
                    if !is_retryable(&err) || attempt == self.max_retries {
                        return Err(err);
                    }

                    let is_rate_limit = {
                        let msg = format!("{:#}", err).to_lowercase();
                        msg.contains("429") || msg.contains("rate limit") || msg.contains("rate_limit")
                            || msg.contains("too many requests") || msg.contains("overloaded") || msg.contains("529")
                    };

                    if is_rate_limit {
                        self.consecutive_rate_limits.fetch_add(1, Ordering::Relaxed);
                    }

                    let delay = self.compute_delay(attempt, &err);
                    eprintln!(
                        "    Retryable error (attempt {}/{}), backing off {:.1}s: {}",
                        attempt + 1,
                        self.max_retries,
                        delay.as_secs_f64(),
                        err,
                    );

                    last_err = Some(err);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("Retry loop exhausted with no error")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    struct FailThenSucceedClient {
        fail_count: AtomicU32,
        target_fails: u32,
        error_msg: String,
    }

    #[async_trait]
    impl LlmClient for FailThenSucceedClient {
        async fn complete(&self, _request: LlmRequest) -> Result<LlmResponse> {
            let count = self.fail_count.fetch_add(1, Ordering::Relaxed);
            if count < self.target_fails {
                anyhow::bail!("{}", self.error_msg);
            }
            Ok(LlmResponse {
                content: "[]".to_string(),
                tokens_used: 100,
                prompt_tokens: 80,
                completion_tokens: 20,
                cached_tokens: 0,
                reasoning_tokens: 0,
            })
        }
    }

    struct AlwaysFailClient {
        error_msg: String,
    }

    #[async_trait]
    impl LlmClient for AlwaysFailClient {
        async fn complete(&self, _request: LlmRequest) -> Result<LlmResponse> {
            anyhow::bail!("{}", self.error_msg);
        }
    }

    #[test]
    fn test_is_retryable() {
        assert!(is_retryable(&anyhow::anyhow!("Anthropic API error (429): rate limit exceeded")));
        assert!(is_retryable(&anyhow::anyhow!("HTTP 529: overloaded")));
        assert!(is_retryable(&anyhow::anyhow!("connection reset by peer")));
        assert!(is_retryable(&anyhow::anyhow!("request timeout")));
        assert!(is_retryable(&anyhow::anyhow!("502 Bad Gateway")));
        assert!(!is_retryable(&anyhow::anyhow!("invalid API key")));
        assert!(!is_retryable(&anyhow::anyhow!("malformed request (400)")));
        assert!(!is_retryable(&anyhow::anyhow!("No response content from OpenAI")));
    }

    #[test]
    fn test_parse_retry_after() {
        assert_eq!(
            parse_retry_after(&anyhow::anyhow!("Rate limited. Retry after 30 seconds.")),
            Some(30)
        );
        assert_eq!(
            parse_retry_after(&anyhow::anyhow!("retry-after: 60")),
            Some(60)
        );
        assert_eq!(
            parse_retry_after(&anyhow::anyhow!("try again in 15 seconds")),
            Some(15)
        );
        assert_eq!(
            parse_retry_after(&anyhow::anyhow!("generic error")),
            None
        );
    }

    #[tokio::test]
    async fn test_retry_on_rate_limit() {
        let inner = Arc::new(FailThenSucceedClient {
            fail_count: AtomicU32::new(0),
            target_fails: 2,
            error_msg: "429 rate limit exceeded".to_string(),
        });
        // Use very short delays for testing
        let client = RetryClient {
            inner,
            max_retries: 3,
            base_delay: Duration::from_millis(1),
            consecutive_rate_limits: AtomicU64::new(0),
        };

        let req = LlmRequest::new("sys", "user");
        let result = client.complete(req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_no_retry_on_permanent_error() {
        let inner = Arc::new(AlwaysFailClient {
            error_msg: "invalid API key (401)".to_string(),
        });
        let client = RetryClient {
            inner,
            max_retries: 3,
            base_delay: Duration::from_millis(1),
            consecutive_rate_limits: AtomicU64::new(0),
        };

        let req = LlmRequest::new("sys", "user");
        let result = client.complete(req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exhausted_retries() {
        let inner = Arc::new(AlwaysFailClient {
            error_msg: "503 service unavailable".to_string(),
        });
        let client = RetryClient {
            inner,
            max_retries: 2,
            base_delay: Duration::from_millis(1),
            consecutive_rate_limits: AtomicU64::new(0),
        };

        let req = LlmRequest::new("sys", "user");
        let result = client.complete(req).await;
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("503"));
    }
}
