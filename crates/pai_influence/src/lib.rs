#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InfluenceEventType {
    SuggestionOffered,
    RewriteProposed,
    StructureChangeProposed,
    MemoryAccessed,
    GoalDeclared,
    AmbiguityReturned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfluenceEvent {
    pub session_id: String,
    pub timestamp: i64,
    pub event_type: InfluenceEventType,
    pub affected_span_hash: Option<String>,
    pub source: String,
    pub requires_confirmation: bool,
    pub confirmed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfluenceEntry {
    pub prev_hash: String,
    pub hash: String,
    pub event: InfluenceEvent,
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[derive(Debug, Error)]
pub enum InfluenceError {
    #[error("verification failed at index {0}")]
    VerifyFailed(usize),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde error: {0}")]
    Serde(String),
}

#[derive(Debug, Default)]
pub struct InfluenceLog {
    entries: Vec<InfluenceEntry>,
}

impl InfluenceLog {
    pub fn entries(&self) -> &[InfluenceEntry] {
        &self.entries
    }

    pub fn append(&mut self, event: InfluenceEvent) -> Result<(), InfluenceError> {
        let prev = self
            .entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| "GENESIS".into());
        let payload = serde_json::to_vec(&(prev.clone(), &event))
            .map_err(|e| InfluenceError::Serde(e.to_string()))?;
        let hash = sha256_hex(&payload);
        self.entries.push(InfluenceEntry {
            prev_hash: prev,
            hash,
            event,
        });
        Ok(())
    }

    pub fn verify(&self) -> Result<(), InfluenceError> {
        let mut prev = "GENESIS".to_string();
        for (i, e) in self.entries.iter().enumerate() {
            let payload = serde_json::to_vec(&(prev.clone(), &e.event))
                .map_err(|er| InfluenceError::Serde(er.to_string()))?;
            let expected = sha256_hex(&payload);
            if e.prev_hash != prev || e.hash != expected {
                return Err(InfluenceError::VerifyFailed(i));
            }
            prev = e.hash.clone();
        }
        Ok(())
    }

    pub fn export_jsonl(&self, path: impl AsRef<std::path::Path>) -> Result<(), InfluenceError> {
        use std::io::Write;
        let mut f = std::fs::File::create(path)?;
        for e in &self.entries {
            let line =
                serde_json::to_string(e).map_err(|er| InfluenceError::Serde(er.to_string()))?;
            f.write_all(line.as_bytes())?;
            f.write_all(b"\n")?;
        }
        f.sync_all()?;
        Ok(())
    }
}
