//! `hindsight_reflect` tool — cross-memory reasoning synthesis.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use zeroclaw_api::tool::{Tool, ToolResult};

use crate::hindsight::Budget;
use crate::hindsight_client::HindsightClient;

/// Synthesize a reasoned answer from Hindsight long-term memory using cross-memory
/// reasoning — identifies patterns, relationships, and implications across multiple
/// stored memories rather than just retrieving matching results.
pub struct HindsightReflectTool {
    client: Arc<HindsightClient>,
    bank_id: String,
    default_budget: Budget,
}

impl HindsightReflectTool {
    pub fn new(client: Arc<HindsightClient>, bank_id: String, default_budget: Budget) -> Self {
        Self {
            client,
            bank_id,
            default_budget,
        }
    }
}

#[async_trait]
impl Tool for HindsightReflectTool {
    fn name(&self) -> &'static str {
        "hindsight_reflect"
    }

    fn description(&self) -> &'static str {
        "Synthesize a reasoned answer from Hindsight long-term memory using cross-memory \
         reasoning. Unlike recall which returns matching memories, reflect identifies \
         patterns, relationships, and implications across multiple stored memories. \
         Best for questions that require understanding 'why' or 'how' rather than 'what'."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "A question or topic that requires synthesizing multiple memories. \
                     For example: 'What are this user's long-term goals based on their stated preferences?'"
                },
                "budget": {
                    "type": "string",
                    "enum": ["low", "mid", "high"],
                    "description": "Reasoning depth: low (quick), mid (balanced), high (thorough). \
                     Defaults to configured budget."
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let query = match args.get("query").and_then(|v| v.as_str()) {
            Some(v) if !v.is_empty() => v,
            _ => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing required parameter: query".to_string()),
                });
            }
        };

        let budget = match args.get("budget").and_then(|v| v.as_str()) {
            Some("low") => Budget::Low,
            Some("high") => Budget::High,
            _ => self.default_budget,
        };

        match self
            .client
            .reflect(&self.bank_id, query, Some(budget))
            .await
        {
            Ok(resp) => {
                let mut output = resp.text;
                if let Some(n) = resp.num_memories_used {
                    output.push_str(&format!("\n\n[Synthesized from {} memory/ies]", n));
                }
                if resp.truncated == Some(true) {
                    output.push_str("\n\n[Note: Response was truncated due to token budget]");
                }
                Ok(ToolResult {
                    success: true,
                    output,
                    error: None,
                })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Hindsight reflect failed: {}", e)),
            }),
        }
    }
}