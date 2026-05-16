//! Hindsight memory configuration schema.
//!
//! This is the runtime config struct used inside zeroclaw-tools.
//! The schema-layer `HindsightSchemaConfig` (in zeroclaw-config) adds the
//! `#[nested]` + `Configurable` derives for config file parsing.

use serde::{Deserialize, Serialize};

/// Configuration for the Hindsight memory backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HindsightConfig {
    /// Hindsight API URL. Defaults to the cloud API.
    #[serde(rename = "apiUrl")]
    pub api_url: Option<String>,

    /// API key for Hindsight Cloud. Can also be set via HINDSIGHT_API_KEY env var.
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,

    /// Memory bank identifier. Defaults to "zeroclaw".
    #[serde(rename = "bankId")]
    pub bank_id: Option<String>,

    /// Recall budget: low, mid, or high. Controls depth of memory search.
    pub budget: Option<String>,

    /// Request timeout in seconds. Defaults to 120.
    #[serde(rename = "timeoutSecs")]
    pub timeout_secs: Option<u64>,

    /// Tags attached to all retained memories.
    #[serde(default)]
    pub retain_tags: Vec<String>,

    /// Source label attached to retained memories.
    #[serde(rename = "retainSource")]
    pub retain_source: Option<String>,

    /// User prefix in retained transcripts. Defaults to "User".
    #[serde(rename = "retainUserPrefix")]
    pub retain_user_prefix: Option<String>,

    /// Assistant prefix in retained transcripts. Defaults to "Assistant".
    #[serde(rename = "retainAssistantPrefix")]
    pub retain_assistant_prefix: Option<String>,

    /// Max tokens for recall response. Defaults to 4096.
    #[serde(rename = "recallMaxTokens")]
    pub recall_max_tokens: Option<usize>,

    /// Max input chars for prefetch query. Defaults to 800.
    #[serde(rename = "recallMaxInputChars")]
    pub recall_max_input_chars: Option<usize>,

    /// Recall prompt preamble.
    #[serde(rename = "recallPromptPreamble")]
    pub recall_prompt_preamble: Option<String>,
}

impl Default for HindsightConfig {
    fn default() -> Self {
        Self {
            api_url: None,
            api_key: None,
            bank_id: Some("zeroclaw".to_string()),
            budget: Some("mid".to_string()),
            timeout_secs: Some(120),
            retain_tags: Vec::new(),
            retain_source: None,
            retain_user_prefix: Some("User".to_string()),
            retain_assistant_prefix: Some("Assistant".to_string()),
            recall_max_tokens: Some(4096),
            recall_max_input_chars: Some(800),
            recall_prompt_preamble: None,
        }
    }
}

impl HindsightConfig {
    /// Resolve the effective API URL.
    pub fn api_url(&self) -> String {
        self.api_url
            .clone()
            .unwrap_or_else(|| "https://api.hindsight.vectorize.io".to_string())
    }

    /// Resolve the effective bank ID.
    pub fn bank_id(&self) -> String {
        self.bank_id.clone().unwrap_or_else(|| "zeroclaw".to_string())
    }

    /// Resolve the effective budget.
    pub fn budget(&self) -> crate::hindsight::Budget {
        match self.budget.as_deref() {
            Some("low") => crate::hindsight::Budget::Low,
            Some("high") => crate::hindsight::Budget::High,
            _ => crate::hindsight::Budget::Mid,
        }
    }

    /// Resolve the effective timeout.
    pub fn timeout(&self) -> u64 {
        self.timeout_secs.unwrap_or(120)
    }

    /// Resolve the effective max recall tokens.
    pub fn recall_max_tokens(&self) -> usize {
        self.recall_max_tokens.unwrap_or(4096)
    }

    /// Resolve the effective max input chars.
    pub fn recall_max_input_chars(&self) -> usize {
        self.recall_max_input_chars.unwrap_or(800)
    }
}