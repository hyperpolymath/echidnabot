//! Client for communicating with ECHIDNA Core

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{ProofResult, ProofStatus, ProverKind, TacticSuggestion};
use crate::config::EchidnaConfig;
use crate::error::{Error, Result};

/// Client for ECHIDNA Core GraphQL API
pub struct EchidnaClient {
    client: Client,
    endpoint: String,
    timeout: Duration,
}

impl EchidnaClient {
    /// Create a new ECHIDNA client
    pub fn new(config: &EchidnaConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            endpoint: config.endpoint.clone(),
            timeout: Duration::from_secs(config.timeout_secs),
        }
    }

    /// Verify a proof using ECHIDNA Core
    pub async fn verify_proof(&self, prover: ProverKind, content: &str) -> Result<ProofResult> {
        let query = GraphQLRequest {
            query: r#"
                mutation VerifyProof($prover: String!, $content: String!) {
                    verifyProof(prover: $prover, content: $content) {
                        status
                        message
                        proverOutput
                        durationMs
                        artifacts
                    }
                }
            "#
            .to_string(),
            variables: serde_json::json!({
                "prover": format!("{:?}", prover).to_lowercase(),
                "content": content
            }),
        };

        let response = self
            .client
            .post(&self.endpoint)
            .json(&query)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            return Err(Error::Echidna(format!(
                "ECHIDNA returned status {}",
                response.status()
            )));
        }

        let gql_response: GraphQLResponse<VerifyProofResponse> =
            response.json().await.map_err(Error::Http)?;

        if let Some(errors) = gql_response.errors {
            return Err(Error::Echidna(
                errors.into_iter().map(|e| e.message).collect::<Vec<_>>().join(", "),
            ));
        }

        let data = gql_response
            .data
            .ok_or_else(|| Error::Echidna("No data in response".to_string()))?;

        Ok(ProofResult {
            status: parse_proof_status(&data.verify_proof.status),
            message: data.verify_proof.message,
            prover_output: data.verify_proof.prover_output,
            duration_ms: data.verify_proof.duration_ms,
            artifacts: data.verify_proof.artifacts,
        })
    }

    /// Request tactic suggestions from ECHIDNA's Julia ML component
    pub async fn suggest_tactics(
        &self,
        prover: ProverKind,
        context: &str,
        goal_state: &str,
    ) -> Result<Vec<TacticSuggestion>> {
        let query = GraphQLRequest {
            query: r#"
                mutation SuggestTactics($prover: String!, $context: String!, $goalState: String!) {
                    suggestTactics(prover: $prover, context: $context, goalState: $goalState) {
                        tactic
                        confidence
                        explanation
                    }
                }
            "#
            .to_string(),
            variables: serde_json::json!({
                "prover": format!("{:?}", prover).to_lowercase(),
                "context": context,
                "goalState": goal_state
            }),
        };

        let response = self
            .client
            .post(&self.endpoint)
            .json(&query)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            return Err(Error::Echidna(format!(
                "ECHIDNA returned status {}",
                response.status()
            )));
        }

        let gql_response: GraphQLResponse<SuggestTacticsResponse> =
            response.json().await.map_err(Error::Http)?;

        if let Some(errors) = gql_response.errors {
            return Err(Error::Echidna(
                errors.into_iter().map(|e| e.message).collect::<Vec<_>>().join(", "),
            ));
        }

        let data = gql_response
            .data
            .ok_or_else(|| Error::Echidna("No data in response".to_string()))?;

        Ok(data
            .suggest_tactics
            .into_iter()
            .map(|s| TacticSuggestion {
                tactic: s.tactic,
                confidence: s.confidence,
                explanation: s.explanation,
            })
            .collect())
    }

    /// Check if ECHIDNA Core is available and healthy
    pub async fn health_check(&self) -> Result<bool> {
        let query = GraphQLRequest {
            query: "{ __typename }".to_string(),
            variables: serde_json::json!({}),
        };

        let response = self
            .client
            .post(&self.endpoint)
            .json(&query)
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Check prover availability
    pub async fn prover_status(&self, prover: ProverKind) -> Result<ProverStatus> {
        let query = GraphQLRequest {
            query: r#"
                query ProverStatus($prover: String!) {
                    proverStatus(prover: $prover) {
                        available
                        message
                    }
                }
            "#
            .to_string(),
            variables: serde_json::json!({
                "prover": format!("{:?}", prover).to_lowercase()
            }),
        };

        let response = self
            .client
            .post(&self.endpoint)
            .json(&query)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            return Ok(ProverStatus::Unavailable);
        }

        let gql_response: GraphQLResponse<ProverStatusResponse> =
            response.json().await.map_err(Error::Http)?;

        match gql_response.data {
            Some(data) if data.prover_status.available => Ok(ProverStatus::Available),
            Some(_) => Ok(ProverStatus::Unavailable),
            None => Ok(ProverStatus::Unknown),
        }
    }
}

// =============================================================================
// GraphQL Types
// =============================================================================

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: serde_json::Value,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Deserialize)]
struct VerifyProofResponse {
    #[serde(rename = "verifyProof")]
    verify_proof: VerifyProofData,
}

#[derive(Deserialize)]
struct VerifyProofData {
    status: String,
    message: String,
    #[serde(rename = "proverOutput")]
    prover_output: String,
    #[serde(rename = "durationMs")]
    duration_ms: u64,
    artifacts: Vec<String>,
}

#[derive(Deserialize)]
struct SuggestTacticsResponse {
    #[serde(rename = "suggestTactics")]
    suggest_tactics: Vec<TacticSuggestionData>,
}

#[derive(Deserialize)]
struct TacticSuggestionData {
    tactic: String,
    confidence: f64,
    explanation: Option<String>,
}

#[derive(Deserialize)]
struct ProverStatusResponse {
    #[serde(rename = "proverStatus")]
    prover_status: ProverStatusData,
}

#[derive(Deserialize)]
struct ProverStatusData {
    available: bool,
    #[allow(dead_code)]
    message: Option<String>,
}

/// Prover availability status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProverStatus {
    Available,
    Degraded,
    Unavailable,
    Unknown,
}

fn parse_proof_status(s: &str) -> ProofStatus {
    match s.to_uppercase().as_str() {
        "VERIFIED" | "PASS" | "SUCCESS" => ProofStatus::Verified,
        "FAILED" | "FAIL" => ProofStatus::Failed,
        "TIMEOUT" => ProofStatus::Timeout,
        "ERROR" => ProofStatus::Error,
        _ => ProofStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prover_file_extensions() {
        assert!(ProverKind::Metamath.file_extensions().contains(&".mm"));
        assert!(ProverKind::Lean.file_extensions().contains(&".lean"));
        assert!(ProverKind::Coq.file_extensions().contains(&".v"));
    }

    #[test]
    fn test_prover_from_extension() {
        assert_eq!(ProverKind::from_extension(".mm"), Some(ProverKind::Metamath));
        assert_eq!(ProverKind::from_extension("lean"), Some(ProverKind::Lean));
        assert_eq!(ProverKind::from_extension(".xyz"), None);
    }

    #[test]
    fn test_prover_tier() {
        assert_eq!(ProverKind::Metamath.tier(), 2);
        assert_eq!(ProverKind::Lean.tier(), 1);
        assert_eq!(ProverKind::Hol4.tier(), 3);
    }
}
