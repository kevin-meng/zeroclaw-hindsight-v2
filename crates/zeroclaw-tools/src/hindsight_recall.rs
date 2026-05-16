//! `hindsight_recall` tool — semantic search of long-term memory.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use zeroclaw_api::tool::{Tool, ToolResult};

use crate::hindsight::Budget;
use crate::hindsight_client::HindsightClient;

/// Search Hindsight long-term memory for relevant facts, preferences, or context.
pub struct HindsightRecallTool {
    client: Arc<HindsightClient>,
    bank_id: String,
    default_budget: Budget,
    max_tokens: usize,
    max_input_chars: usize,
}

impl HindsightRecallTool {
    pub fn new(
        client: Arc<HindsightClient>,
        bank_id: String,
        default_budget: Budget,
        max_tokens: usize,
        max_input_chars: usize,
    ) -> Self {
        Self {
            client,
            bank_id,
            default_budget,
            max_tokens,
            max_input_chars,
        }
    }

    fn format_results(&self, results: Vec<crate::hindsight::RecallResult>) -> String {
        if results.is_empty() {
            return "No matching memories found.".to_string();
        }
        let mut output = String::new();
        for (i, result) in results.iter().enumerate() {
            let score_str = result
                .score
                .map(|s| format!(" [{:.2}]", s))
                .unwrap_or_default();
            let tags_str = if result.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", result.tags.join(", "))
            };
            output.push_str(&format!(
                "{}. {}{}{}\n",
                i + 1,
                result.text,
                score_str,
                tags_str
            ));
        }
        output
    }
}

#[async_trait]
impl Tool for HindsightRecallTool {
    fn name(&self) -> &'static str {
        "hindsight_recall"
    }

    fn description(&self) -> &'static str {
        "Search Hindsight long-term memory for relevant facts, preferences, \
         or context. Returns scored results ranked by relevance using semantic \
         search. Supports tag filtering and budget levels (low/mid/high)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Keywords, phrase, or question to search for in memory. \
                     Omit or pass '*' to return recent memories."
                },
                "budget": {
                    "type": "string",
                    "enum": ["low", "mid", "high"],
                    "description": "Search depth: low (fast), mid (balanced), high (thorough). \
                     Defaults to the configured budget."
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results to return. Note: the effective limit is also \
                     constrained by max_tokens."
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Filter to memories with any of these tags."
                },
                "tags_match": {
                    "type": "string",
                    "enum": ["any", "all"],
                    "description": "Whether tags filter requires 'any' (OR) or 'all' (AND). \
                     Defaults to 'any'."
                },
                "types": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Filter by memory types (e.g. 'fact', 'preference', 'context')."
                }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("*");

        // Truncate long queries to avoid token bloat.
        let query = if query.len() > self.max_input_chars {
            &query[..self.max_input_chars]
        } else {
            query
        };

        // Handle "recent only" query (bare * or empty).
        let query = if query.trim().is_empty() || query == "*" {
            ""
        } else {
            query
        };

        let budget = match args.get("budget").and_then(|v| v.as_str()) {
            Some("low") => Budget::Low,
            Some("high") => Budget::High,
            _ => self.default_budget,
        };

        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let tags: Option<Vec<String>> = args
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        let tags_match = args.get("tags_match").and_then(|v| v.as_str());

        let types: Option<Vec<String>> = args
            .get("types")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        let max_tokens = limit.map(|l| l * 200).unwrap_or(self.max_tokens);

        match self
            .client
            .recall(
                &self.bank_id,
                query,
                Some(budget),
                Some(max_tokens),
                tags.as_deref(),
                tags_match,
                types.as_deref(),
            )
            .await
        {
            Ok(results) => {
                let limited = if let Some(limit) = limit {
                    results
                        .into_iter()
                        .take(limit)
                        .collect::<Vec<crate::hindsight::RecallResult>>()
                } else {
                    results
                };
                Ok(ToolResult {
                    success: true,
                    output: self.format_results(limited),
                    error: None,
                })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Hindsight recall failed: {}", e)),
            }),
        }
    }
}