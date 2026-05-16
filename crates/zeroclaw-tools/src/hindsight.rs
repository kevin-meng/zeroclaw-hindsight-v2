//! Hindsight long-term memory backend for ZeroClaw.
//!
//! Connects to the Hindsight Cloud API to provide semantic memory storage
//! and retrieval with knowledge graph, entity resolution, and multi-strategy retrieval.
//!
//! ## Configuration
//!
//! ```toml
//! [memory]
//! backend = "hindsight"
//!
//! [memory.hindsight]
//! api_url = "https://api.hindsight.vectorize.io"
//! api_key = "${HINDSIGHT_API_KEY}"   # env var reference
//! bank_id = "zeroclaw"
//! budget = "mid"                     # low | mid | high
//! timeout_secs = 120
//! ```
//!
//! ## Tools
//!
//! Three tools are exposed to the agent:
//! - `hindsight_retain` — store information to long-term memory
//! - `hindsight_recall` — semantic search of long-term memory
//! - `hindsight_reflect` — cross-memory reasoning synthesis

// ─── Shared types ────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

/// Budget level for recall/reflect requests — controls depth of memory search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Budget {
    Low,
    #[default]
    Mid,
    High,
}

impl std::fmt::Display for Budget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Budget::Low => write!(f, "low"),
            Budget::Mid => write!(f, "mid"),
            Budget::High => write!(f, "high"),
        }
    }
}

/// A single recall result from Hindsight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallResult {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Response from a reflect (reasoning synthesis) call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectResponse {
    pub text: String,
    #[serde(default)]
    pub num_memories_used: Option<usize>,
    #[serde(default)]
    pub truncated: Option<bool>,
}

/// Response from a retain call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetainResponse {
    pub success: bool,
    pub bank_id: String,
    pub items_count: usize,
}

/// Metadata attached to a retain call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetainMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retained_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_id: Option<String>,
}