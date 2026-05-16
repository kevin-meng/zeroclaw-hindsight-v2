//! `hindsight_retain` tool — store information to long-term memory.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use zeroclaw_api::tool::{Tool, ToolResult};

use crate::hindsight::RetainMetadata;
use crate::hindsight_client::HindsightClient;

/// Store a fact, preference, or note in Hindsight long-term memory.
pub struct HindsightRetainTool {
    client: Arc<HindsightClient>,
    bank_id: String,
    session_id: Option<String>,
    platform: Option<String>,
    user_id: Option<String>,
}

impl HindsightRetainTool {
    pub fn new(
        client: Arc<HindsightClient>,
        bank_id: String,
        session_id: Option<String>,
        platform: Option<String>,
        user_id: Option<String>,
    ) -> Self {
        Self {
            client,
            bank_id,
            session_id,
            platform,
            user_id,
        }
    }

    fn build_metadata(&self, message_count: usize, turn_index: usize) -> RetainMetadata {
        RetainMetadata {
            retained_at: Some(chrono::Utc::now().to_rfc3339()),
            message_count: Some(message_count),
            turn_index: Some(turn_index),
            session_id: self.session_id.clone(),
            platform: self.platform.clone(),
            user_id: self.user_id.clone(),
            assistant_id: None,
        }
    }
}

#[async_trait]
impl Tool for HindsightRetainTool {
    fn name(&self) -> &'static str {
        "hindsight_retain"
    }

    fn description(&self) -> &'static str {
        "Store a fact, preference, or note in Hindsight long-term memory. \
         Use to remember user preferences, decisions, context, or anything \
         that should persist across sessions. Tags can be added to organize \
         memories by session, topic, or source."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "The information to remember. Can be a single fact, a structured note, or a summary of a conversation."
                },
                "context": {
                    "type": "string",
                    "description": "Optional context describing what this memory means or when it's relevant."
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional tags to attach (e.g. ['session:abc123', 'topic:preferences'])."
                },
                "document_id": {
                    "type": "string",
                    "description": "Optional session-scoped document ID. If omitted, a new document is created."
                }
            },
            "required": ["content"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let content = match args.get("content").and_then(|v| v.as_str()) {
            Some(v) if !v.is_empty() => v,
            _ => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing required parameter: content".to_string()),
                });
            }
        };

        let context = args.get("context").and_then(|v| v.as_str()).filter(|s| !s.is_empty());
        let tags: Option<Vec<String>> = args
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });
        let document_id = args
            .get("document_id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let metadata = self.build_metadata(1, 0);

        match self
            .client
            .retain(
                &self.bank_id,
                content,
                metadata,
                document_id,
                None, // update_mode: None = let client decide based on version probe
                tags.as_deref(),
                context,
            )
            .await
        {
            Ok(resp) => Ok(ToolResult {
                success: true,
                output: format!("Stored to Hindsight: bank={}, items_count={}", resp.bank_id, resp.items_count),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Hindsight retain failed: {}", e)),
            }),
        }
    }
}