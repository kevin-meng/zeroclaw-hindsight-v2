//! Hindsight REST API client.
//!
//! Wraps the Hindsight Cloud API endpoints:
//! - `POST /retain` — store memories
//! - `POST /recall` — semantic search
//! - `POST /reflect` — reasoning synthesis
//! - `GET /version` — API capability probe

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::hindsight::{Budget, RecallResult, ReflectResponse, RetainMetadata, RetainResponse};

/// Hindsight API client.
#[derive(Clone)]
pub struct HindsightClient {
    http: Client,
    api_url: String,
    api_key: String,
    #[allow(dead_code)]
    bank_id: String,
    budget: Budget,
    version_cache: Arc<RwLock<HashMap<String, Option<String>>>>,
}

impl HindsightClient {
    /// Create a new Hindsight client.
    pub fn new(api_url: String, api_key: String, bank_id: String, budget: Budget) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("reqwest client should build");
        Self {
            http,
            api_url,
            api_key,
            bank_id,
            budget,
            version_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Build authorization headers.
    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.api_key).parse().unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        headers
    }

    /// Probe the API version.
    pub async fn probe_version(&self) -> Result<Option<String>> {
        let url = format!("{}/version", self.api_url.trim_end_matches('/'));
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("hindsight /version probe failed")?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let body: serde_json::Value = resp.json().await.context("parse /version response")?;
        let version = body
            .get("version")
            .or_else(|| body.get("api_version"))
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(version)
    }

    /// Check if the API supports `update_mode='append'` (≥ 0.5.0).
    pub async fn supports_update_mode_append(&self) -> bool {
        let url = self.api_url.clone();
        let cache = self.version_cache.clone();

        // Fast path: check cache first.
        {
            let cache = cache.read().await;
            if let Some(version) = cache.get(&url) {
                return version.as_ref().is_some_and(|v| compare_versions(v, "0.5.0") >= 0);
            }
        }

        let version = self.probe_version().await.ok().flatten();
        {
            let mut cache = cache.write().await;
            cache.insert(url, version.clone());
        }

        version.is_some_and(|v| compare_versions(&v, "0.5.0") >= 0)
    }

    /// Retain (store) a memory item.
    pub async fn retain(
        &self,
        bank_id: &str,
        content: &str,
        _metadata: RetainMetadata,
        _document_id: Option<&str>,
        _update_mode: Option<&str>,
        tags: Option<&[String]>,
        context: Option<&str>,
    ) -> Result<RetainResponse> {
        let url = format!(
            "{}/v1/default/banks/{}/memories",
            self.api_url.trim_end_matches('/'),
            bank_id
        );

        #[derive(Serialize)]
        struct RetainRequest<'a> {
            items: Vec<RetainItem<'a>>,
        }
        #[derive(Serialize)]
        struct RetainItem<'a> {
            content: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            context: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            tags: Option<&'a [String]>,
        }

        let body = RetainRequest {
            items: vec![RetainItem {
                content,
                context,
                tags,
            }],
        };

        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&body)
            .send()
            .await
            .with_context(|| format!("hindsight /retain failed for bank {bank_id}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("hindsight /retain returned {status}: {body_text}");
        }

        let result: RetainResponse = resp
            .json()
            .await
            .context("parse /retain response")?;

        Ok(result)
    }

    /// Recall (semantic search) memories.
    pub async fn recall(
        &self,
        bank_id: &str,
        query: &str,
        budget: Option<Budget>,
        max_tokens: Option<usize>,
        tags: Option<&[String]>,
        tags_match: Option<&str>,
        types: Option<&[String]>,
    ) -> Result<Vec<RecallResult>> {
        let url = format!(
            "{}/v1/default/banks/{}/memories/recall",
            self.api_url.trim_end_matches('/'),
            bank_id
        );
        let budget = budget.unwrap_or(self.budget);

        #[derive(Serialize)]
        struct RecallRequest<'a> {
            bank_id: &'a str,
            query: &'a str,
            budget: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_tokens: Option<usize>,
            #[serde(skip_serializing_if = "Option::is_none")]
            tags: Option<&'a [String]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            tags_match: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            types: Option<&'a [String]>,
        }

        let body = RecallRequest {
            bank_id,
            query,
            budget: budget.to_string(),
            max_tokens,
            tags,
            tags_match,
            types,
        };

        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&body)
            .send()
            .await
            .with_context(|| format!("hindsight /recall failed for bank {bank_id}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("hindsight /recall returned {status}: {body_text}");
        }

        #[derive(Deserialize)]
        struct RecallResponse {
            results: Vec<RecallResult>,
        }

        let result: RecallResponse = resp
            .json()
            .await
            .context("parse /recall response")?;

        Ok(result.results)
    }

    /// Reflect (reasoning synthesis) across memories.
    pub async fn reflect(
        &self,
        bank_id: &str,
        query: &str,
        budget: Option<Budget>,
    ) -> Result<ReflectResponse> {
        let url = format!(
            "{}/v1/default/banks/{}/memories/reflect",
            self.api_url.trim_end_matches('/'),
            bank_id
        );
        let budget = budget.unwrap_or(self.budget);

        #[derive(Serialize)]
        struct ReflectRequest<'a> {
            bank_id: &'a str,
            query: &'a str,
            budget: String,
        }

        let body = ReflectRequest {
            bank_id,
            query,
            budget: budget.to_string(),
        };

        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&body)
            .send()
            .await
            .with_context(|| format!("hindsight /reflect failed for bank {bank_id}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("hindsight /reflect returned {status}: {body_text}");
        }

        let result: ReflectResponse = resp
            .json()
            .await
            .context("parse /reflect response")?;

        Ok(result)
    }
}

// ─── Semantic version comparison ─────────────────────────────────────────────

fn compare_versions(a: &str, b: &str) -> i8 {
    use std::cmp::Ordering;
    let parse = |s: &str| -> Vec<u64> {
        s.split('-')
            .next()
            .unwrap_or(s)
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect()
    };
    let va = parse(a);
    let vb = parse(b);
    let max_len = va.len().max(vb.len());

    for i in 0..max_len {
        let ai = va.get(i).copied().unwrap_or(0);
        let bi = vb.get(i).copied().unwrap_or(0);
        match ai.cmp(&bi) {
            Ordering::Less => return -1,
            Ordering::Greater => return 1,
            Ordering::Equal => continue,
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert!(compare_versions("0.5.0", "0.5.0") == 0);
        assert!(compare_versions("0.6.1", "0.5.0") > 0);
        assert!(compare_versions("0.4.22", "0.5.0") < 0);
        assert!(compare_versions("1.0.0", "0.9.9") > 0);
    }
}