use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::report::Invariant;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressEntry {
    pub module_path: String,
    pub status: String,
    #[serde(default)]
    pub invariants_found: usize,
    #[serde(default)]
    pub tokens_used: u64,
    pub timestamp: String,
    #[serde(default)]
    pub error: Option<String>,
}

pub struct ProgressTracker {
    progress_writer: Mutex<BufWriter<File>>,
    invariants_writer: Mutex<BufWriter<File>>,
    completed: Mutex<HashSet<String>>,
    pub total_chunks: usize,
    pub completed_count: AtomicUsize,
    pub failed_count: AtomicUsize,
    pub invariants_found: AtomicUsize,
    pub total_tokens: AtomicU64,
    token_budget: u64,
    pub run_dir: PathBuf,
}

impl ProgressTracker {
    pub fn new(run_dir: &Path, total_chunks: usize, token_budget: u64) -> Result<Self> {
        std::fs::create_dir_all(run_dir)?;

        let progress_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(run_dir.join("progress.jsonl"))?;

        let invariants_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(run_dir.join("invariants.jsonl"))?;

        Ok(Self {
            progress_writer: Mutex::new(BufWriter::new(progress_file)),
            invariants_writer: Mutex::new(BufWriter::new(invariants_file)),
            completed: Mutex::new(HashSet::new()),
            total_chunks,
            completed_count: AtomicUsize::new(0),
            failed_count: AtomicUsize::new(0),
            invariants_found: AtomicUsize::new(0),
            total_tokens: AtomicU64::new(0),
            token_budget,
            run_dir: run_dir.to_path_buf(),
        })
    }

    /// Load a checkpoint from an existing progress.jsonl file.
    /// Only "completed" entries are skipped on resume — "failed" entries are retried,
    /// since failures are often transient (rate limits, timeouts, server errors).
    pub fn load_checkpoint(path: &Path) -> Result<HashSet<String>> {
        let mut completed = HashSet::new();
        if !path.exists() {
            return Ok(completed);
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<ProgressEntry>(&line) {
                if entry.status == "completed" {
                    completed.insert(entry.module_path);
                }
                // "failed" entries are NOT added — they will be retried on resume
            }
        }
        Ok(completed)
    }

    /// Restore completed set from a checkpoint for resume
    pub fn restore_from_checkpoint(&self, checkpoint: HashSet<String>) {
        let count = checkpoint.len();
        let mut completed = self.completed.lock().unwrap();
        *completed = checkpoint;
        self.completed_count.store(count, Ordering::Relaxed);
    }

    pub fn is_completed(&self, module_path: &str) -> bool {
        self.completed.lock().unwrap().contains(module_path)
    }

    pub fn record_result(
        &self,
        module_path: &str,
        status: &str,
        invariant_count: usize,
        tokens: u64,
        error: Option<&str>,
    ) {
        let entry = ProgressEntry {
            module_path: module_path.to_string(),
            status: status.to_string(),
            invariants_found: invariant_count,
            tokens_used: tokens,
            timestamp: chrono::Utc::now().to_rfc3339(),
            error: error.map(|e| e.to_string()),
        };

        if let Ok(json) = serde_json::to_string(&entry) {
            if let Ok(mut writer) = self.progress_writer.lock() {
                let _ = writeln!(writer, "{}", json);
                let _ = writer.flush();
            }
        }

        self.completed.lock().unwrap().insert(module_path.to_string());

        if status == "completed" {
            self.completed_count.fetch_add(1, Ordering::Relaxed);
            self.invariants_found.fetch_add(invariant_count, Ordering::Relaxed);
        } else {
            self.failed_count.fetch_add(1, Ordering::Relaxed);
        }

        self.total_tokens.fetch_add(tokens, Ordering::Relaxed);
    }

    pub fn record_invariant(&self, invariant: &Invariant) {
        if let Ok(json) = serde_json::to_string(invariant) {
            if let Ok(mut writer) = self.invariants_writer.lock() {
                let _ = writeln!(writer, "{}", json);
                let _ = writer.flush();
            }
        }
    }

    pub fn budget_exceeded(&self) -> bool {
        if self.token_budget == 0 {
            return false;
        }
        self.total_tokens.load(Ordering::Relaxed) >= self.token_budget
    }

    pub fn print_status(&self) {
        let completed = self.completed_count.load(Ordering::Relaxed);
        let failed = self.failed_count.load(Ordering::Relaxed);
        let invariants = self.invariants_found.load(Ordering::Relaxed);
        let tokens = self.total_tokens.load(Ordering::Relaxed);

        let token_str = if tokens >= 1_000_000 {
            format!("{:.1}M tokens", tokens as f64 / 1_000_000.0)
        } else if tokens >= 1_000 {
            format!("{:.0}K tokens", tokens as f64 / 1_000.0)
        } else {
            format!("{} tokens", tokens)
        };

        eprintln!(
            "  [{}/{}] {} invariants found | {} | {} failures",
            completed + failed,
            self.total_chunks,
            invariants,
            token_str,
            failed,
        );
    }

    /// Load invariants from invariants.jsonl for building the final report
    pub fn load_invariants(path: &Path) -> Result<Vec<Invariant>> {
        let mut invariants = Vec::new();
        if !path.exists() {
            return Ok(invariants);
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(inv) = serde_json::from_str::<Invariant>(&line) {
                invariants.push(inv);
            }
        }
        Ok(invariants)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_progress_tracker_new() {
        let tmp = TempDir::new().unwrap();
        let tracker = ProgressTracker::new(tmp.path(), 10, 0).unwrap();
        assert_eq!(tracker.total_chunks, 10);
        assert!(!tracker.budget_exceeded());
    }

    #[test]
    fn test_record_and_check_completed() {
        let tmp = TempDir::new().unwrap();
        let tracker = ProgressTracker::new(tmp.path(), 10, 0).unwrap();
        assert!(!tracker.is_completed("test::mod"));
        tracker.record_result("test::mod", "completed", 2, 100, None);
        assert!(tracker.is_completed("test::mod"));
        assert_eq!(tracker.completed_count.load(Ordering::Relaxed), 1);
        assert_eq!(tracker.invariants_found.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_budget_exceeded() {
        let tmp = TempDir::new().unwrap();
        let tracker = ProgressTracker::new(tmp.path(), 10, 500).unwrap();
        assert!(!tracker.budget_exceeded());
        tracker.record_result("a", "completed", 0, 300, None);
        assert!(!tracker.budget_exceeded());
        tracker.record_result("b", "completed", 0, 250, None);
        assert!(tracker.budget_exceeded());
    }

    #[test]
    fn test_load_checkpoint_skips_completed_retries_failed() {
        let tmp = TempDir::new().unwrap();
        let tracker = ProgressTracker::new(tmp.path(), 10, 0).unwrap();
        tracker.record_result("mod_a", "completed", 1, 100, None);
        tracker.record_result("mod_b", "failed", 0, 0, Some("rate limit"));

        // Drop tracker to flush
        drop(tracker);

        let completed = ProgressTracker::load_checkpoint(&tmp.path().join("progress.jsonl")).unwrap();
        assert!(completed.contains("mod_a"), "completed entries should be skipped");
        assert!(!completed.contains("mod_b"), "failed entries should be retried");
    }

    #[test]
    fn test_load_invariants() {
        let tmp = TempDir::new().unwrap();
        let tracker = ProgressTracker::new(tmp.path(), 10, 0).unwrap();

        let inv = Invariant {
            id: 1,
            invariant_type: crate::report::InvariantType::StateMachine,
            title: "Test".to_string(),
            description: "desc".to_string(),
            location: crate::report::Location {
                file_path: "test.rs".to_string(),
                line_start: 1,
                line_end: 10,
            },
            evidence: crate::report::Evidence {
                code_snippet: "code".to_string(),
                explanation: "expl".to_string(),
            },
            suggested_pattern: "typestate".to_string(),
            confidence: crate::report::Confidence::High,
        };
        tracker.record_invariant(&inv);
        drop(tracker);

        let loaded = ProgressTracker::load_invariants(&tmp.path().join("invariants.jsonl")).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "Test");
    }
}
