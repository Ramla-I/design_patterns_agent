use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::oneshot;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A search result from octocode semantic search
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub similarity: f32,
    pub line_start: usize,
    pub line_end: usize,
    pub snippet: String,
}

/// MCP client for communicating with the octocode subprocess via JSON-RPC 2.0
pub struct OctocodeClient {
    child: Child,
    stdin: ChildStdin,
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<serde_json::Value>>>>,
    _reader_task: tokio::task::JoinHandle<()>,
}

impl OctocodeClient {
    /// Spawn the octocode MCP server for a given repository path.
    /// Requires `octocode` to be installed and the repo to be indexed.
    pub async fn new(repo_path: &Path) -> Result<Self> {
        // Verify octocode is available
        let check = tokio::process::Command::new("which")
            .arg("octocode")
            .output()
            .await;

        if check.is_err() || !check.unwrap().status.success() {
            anyhow::bail!(
                "octocode not found. Install with:\n  \
                 curl -fsSL https://raw.githubusercontent.com/Muvon/octocode/master/install.sh | sh\n\
                 Then index your codebase:\n  \
                 cd {} && octocode index",
                repo_path.display()
            );
        }

        let mut child = tokio::process::Command::new("octocode")
            .args(["mcp", "--path", &repo_path.to_string_lossy()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn octocode MCP server")?;

        let stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;

        let pending: Arc<Mutex<HashMap<String, oneshot::Sender<serde_json::Value>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Spawn reader task to process responses
        let pending_clone = Arc::clone(&pending);
        let reader_task = tokio::spawn(async move {
            Self::reader_loop(stdout, pending_clone).await;
        });

        Ok(Self {
            child,
            stdin,
            pending,
            _reader_task: reader_task,
        })
    }

    async fn reader_loop(
        stdout: ChildStdout,
        pending: Arc<Mutex<HashMap<String, oneshot::Sender<serde_json::Value>>>>,
    ) {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) | Err(_) => {
                    // EOF or error — clean up pending requests
                    let mut map = pending.lock().await;
                    for (_, tx) in map.drain() {
                        let _ = tx.send(json!({"error": "MCP server closed"}));
                    }
                    return;
                }
                Ok(_) => {}
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(id) = msg.get("id").and_then(|v| v.as_str()) {
                    let mut map = pending.lock().await;
                    if let Some(tx) = map.remove(id) {
                        let _ = tx.send(msg);
                    }
                }
            }
        }
    }

    /// Send a JSON-RPC request and wait for the response
    async fn request(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let req_id = uuid::Uuid::new_v4().to_string();

        let req = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": method,
            "params": params,
        });

        let (tx, rx) = oneshot::channel();
        {
            let mut map = self.pending.lock().await;
            map.insert(req_id.clone(), tx);
        }

        let req_bytes = format!("{}\n", serde_json::to_string(&req)?);
        self.stdin.write_all(req_bytes.as_bytes()).await?;
        self.stdin.flush().await?;

        let resp = tokio::time::timeout(std::time::Duration::from_secs(60), rx)
            .await
            .context("Timeout waiting for octocode response")?
            .context("MCP channel closed")?;

        if let Some(err) = resp.get("error") {
            anyhow::bail!("MCP error: {}", err);
        }

        Ok(resp.get("result").cloned().unwrap_or(resp))
    }

    /// Perform a semantic search query
    pub async fn search(
        &mut self,
        query: &str,
        mode: &str,
        threshold: f32,
        max_results: usize,
    ) -> Result<Vec<SearchResult>> {
        let result = self
            .request(
                "tools/call",
                json!({
                    "name": "semantic_search",
                    "arguments": {
                        "query": query,
                        "mode": mode,
                        "threshold": threshold,
                        "detail_level": "partial",
                        "max_results": max_results,
                    }
                }),
            )
            .await?;

        // Extract text content from MCP response
        let text = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        Ok(parse_search_results(text))
    }

    /// Shut down the octocode subprocess
    pub async fn close(mut self) -> Result<()> {
        let _ = self.child.kill().await;
        Ok(())
    }
}

/// Parse octocode's text output into structured SearchResults.
///
/// Format:
/// ```text
/// 1. src/foo.rs
///    | Similarity 0.85
/// 42: pub struct Foo {
/// 43:     bar: i32,
/// ```
fn parse_search_results(text: &str) -> Vec<SearchResult> {
    let file_re = Regex::new(r"(\d+)\.\s+([^\s]+\.rs)").unwrap();
    let sim_re = Regex::new(r"Similarity\s+([0-9.]+)").unwrap();
    let line_re = Regex::new(r"(?m)^(\d+):").unwrap();

    let chunks: Vec<&str> = text.split("\n\n").collect();
    let mut results = Vec::new();

    // Sometimes entries are separated by double or triple newlines; merge adjacent chunks
    // that belong to the same entry
    let mut i = 0;
    while i < chunks.len() {
        let chunk = chunks[i];

        if let Some(file_match) = file_re.captures(chunk) {
            let file_path = file_match[2].to_string();
            let similarity = sim_re
                .captures(chunk)
                .and_then(|m| m[1].parse::<f32>().ok())
                .unwrap_or(0.0);

            let line_numbers: Vec<usize> = line_re
                .captures_iter(chunk)
                .filter_map(|m| m[1].parse::<usize>().ok())
                .collect();

            // Extract code lines (skip the header lines)
            let code_lines: Vec<&str> = chunk
                .lines()
                .filter(|l| line_re.is_match(l))
                .collect();
            let snippet = code_lines.join("\n");

            if !line_numbers.is_empty() {
                results.push(SearchResult {
                    file_path,
                    similarity,
                    line_start: *line_numbers.iter().min().unwrap(),
                    line_end: *line_numbers.iter().max().unwrap(),
                    snippet,
                });
            }
        }

        i += 1;
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_results() {
        let text = r#"1. src/connection.rs
   | Similarity 0.82
42: pub struct Connection {
43:     is_open: bool,
44:     fd: i32,
45: }

2. src/state.rs
   | Similarity 0.71
10: enum State {
11:     Open,
12:     Closed,
13: }"#;

        let results = parse_search_results(text);
        assert_eq!(results.len(), 2);

        assert_eq!(results[0].file_path, "src/connection.rs");
        assert!((results[0].similarity - 0.82).abs() < 0.01);
        assert_eq!(results[0].line_start, 42);
        assert_eq!(results[0].line_end, 45);

        assert_eq!(results[1].file_path, "src/state.rs");
        assert_eq!(results[1].line_start, 10);
        assert_eq!(results[1].line_end, 13);
    }

    #[test]
    fn test_parse_empty_results() {
        let results = parse_search_results("");
        assert!(results.is_empty());
    }
}
