// SPDX-License-Identifier: PMPL-1.0-or-later
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
//! echidnabot-mcp — Minimal MCP (Model Context Protocol) server exposing
//! echidnabot's capabilities to AI assistants.
//!
//! **Temporary BoJ-only-MCP exception.** The estate-wide policy is that
//! all MCP tools route through the BoJ server (see memory
//! `feedback_boj_only_mcp`). That policy stays canonical — this server
//! exists because the BoJ V-lang adapter was deleted during the V-lang
//! ban and the Zig replacement is not yet built. When BoJ is revived,
//! this binary becomes the implementation inside a BoJ cartridge
//! (or gets retired outright). See docs/MCP-TEMPORARY-EXCEPTION.adoc.
//!
//! Transport: JSON-RPC 2.0 over stdio, newline-delimited.
//!
//! Tools exposed:
//!   - `suggest_tactics`       : call ECHIDNA, rerank by local history
//!   - `record_tactic_outcome` : persist an attempt for future reranking
//!   - `list_outcome_history`  : query the tactic_outcome store
//!   - `corpus_refresh`        : fire `just corpus-refresh` in echidna
//!
//! No authentication layer — stdio is already process-scoped.

use std::sync::Arc;

use anyhow::{anyhow, Context as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdout};

use echidnabot::{
    config::Config,
    dispatcher::{EchidnaClient, ProverKind},
    feedback::{CorpusDelta, DeltaRow, DeltaSource, Reranker},
    store::{models::TacticOutcomeRecord, SqliteStore, Store},
};

const PROTO_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "echidnabot-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Serialize)]
struct RpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

struct ServerState {
    store: Arc<dyn Store>,
    echidna: EchidnaClient,
    reranker: Reranker,
    corpus: Option<CorpusDelta>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Logs to stderr — stdout is reserved for the JSON-RPC channel.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .compact()
        .init();

    let config_path =
        std::env::var("ECHIDNABOT_CONFIG").unwrap_or_else(|_| "echidnabot.toml".to_string());
    let config = Config::load(&config_path).context("loading echidnabot config")?;

    let store = Arc::new(
        SqliteStore::new(&config.database.url)
            .await
            .context("opening store")?,
    ) as Arc<dyn Store>;
    let echidna = EchidnaClient::new(&config.echidna);
    let reranker = Reranker::new(Arc::clone(&store));

    let corpus = build_corpus_delta(&config.corpus)
        .context("building corpus delta writer")?;

    let state = Arc::new(ServerState {
        store,
        echidna,
        reranker,
        corpus,
    });

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();

    tracing::info!(
        server = SERVER_NAME,
        version = SERVER_VERSION,
        proto = PROTO_VERSION,
        "echidnabot-mcp ready on stdio"
    );

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break; // stdin closed
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                write_response(
                    &mut stdout,
                    &error_response(None, -32700, format!("Parse error: {}", e)),
                )
                .await?;
                continue;
            }
        };

        let id = request.id.clone();
        let is_notification = id.is_none();
        let result = dispatch(state.clone(), &request).await;

        if is_notification {
            continue; // JSON-RPC 2.0: notifications get no response
        }

        let resp = match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(value),
                error: None,
            },
            Err(e) => error_response(id, -32603, e.to_string()),
        };
        write_response(&mut stdout, &resp).await?;
    }

    Ok(())
}

fn build_corpus_delta(cfg: &echidnabot::config::CorpusConfig) -> Result<Option<CorpusDelta>> {
    if !cfg.enabled {
        return Ok(None);
    }
    let dir = cfg
        .training_data_dir
        .clone()
        .ok_or_else(|| anyhow!("corpus.enabled but training_data_dir not set"))?;
    let mut cd = CorpusDelta::new(dir);
    if let Some(root) = cfg.echidna_root.clone() {
        cd = cd.with_trigger(root);
    }
    if let Some(t) = cfg.auto_trigger_threshold {
        cd = cd.with_auto_trigger(t);
    }
    Ok(Some(cd))
}

async fn dispatch(state: Arc<ServerState>, req: &JsonRpcRequest) -> Result<Value> {
    match req.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": PROTO_VERSION,
            "capabilities": { "tools": {} },
            "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION }
        })),
        "notifications/initialized" => Ok(Value::Null),
        "tools/list" => Ok(tools_list()),
        "tools/call" => tools_call(state, &req.params).await,
        other => Err(anyhow!("Method not found: {}", other)),
    }
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "suggest_tactics",
                "description":
                    "Request tactic suggestions from ECHIDNA for a proof goal, \
                     reranked against this bot's local tactic-outcome history.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "prover":     { "type": "string", "description": "ProverKind (Coq, Lean, Z3, ...)" },
                        "context":    { "type": "string" },
                        "goal_state": { "type": "string" }
                    },
                    "required": ["prover", "goal_state"]
                }
            },
            {
                "name": "record_tactic_outcome",
                "description":
                    "Persist a tactic attempt outcome (feeds the reranker + corpus-delta writer).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "prover":      { "type": "string" },
                        "goal_state":  { "type": "string" },
                        "tactic":      { "type": "string" },
                        "succeeded":   { "type": "boolean" },
                        "duration_ms": { "type": "integer", "minimum": 0 },
                        "source":      { "type": "string", "enum": ["webhook", "mcp", "cli"] }
                    },
                    "required": ["prover", "goal_state", "tactic", "succeeded", "duration_ms"]
                }
            },
            {
                "name": "list_outcome_history",
                "description":
                    "Query the local tactic_outcome store by (prover, fingerprint-of-goal) or \
                     (prover, tactic).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "prover":     { "type": "string" },
                        "goal_state": { "type": "string" },
                        "tactic":     { "type": "string" },
                        "limit":      { "type": "integer", "minimum": 1, "default": 20 }
                    },
                    "required": ["prover"]
                }
            },
            {
                "name": "corpus_refresh",
                "description":
                    "Fire the configured retrain trigger (default: `just corpus-refresh` in \
                     echidna_root). Errors if corpus is not enabled or echidna_root is unset.",
                "inputSchema": { "type": "object", "properties": {} }
            }
        ]
    })
}

async fn tools_call(state: Arc<ServerState>, params: &Value) -> Result<Value> {
    #[derive(Deserialize)]
    struct Call {
        name: String,
        #[serde(default)]
        arguments: Value,
    }
    let call: Call = serde_json::from_value(params.clone())
        .context("parsing tools/call params")?;

    match call.name.as_str() {
        "suggest_tactics" => tool_suggest_tactics(state, call.arguments).await,
        "record_tactic_outcome" => tool_record_tactic_outcome(state, call.arguments).await,
        "list_outcome_history" => tool_list_outcome_history(state, call.arguments).await,
        "corpus_refresh" => tool_corpus_refresh(state).await,
        other => Err(anyhow!("Unknown tool: {}", other)),
    }
}

fn parse_prover(raw: &str) -> Result<ProverKind> {
    // Accept both Debug-form ("Coq") and lowercase serde form ("coq").
    let normalised = match raw {
        "coq" => "Coq",
        "lean" => "Lean",
        "agda" => "Agda",
        "isabelle" => "Isabelle",
        "z3" => "Z3",
        "cvc5" => "Cvc5",
        "metamath" => "Metamath",
        "hollight" | "hol_light" => "HolLight",
        "mizar" => "Mizar",
        "pvs" => "Pvs",
        "acl2" => "Acl2",
        "hol4" => "Hol4",
        other => other,
    };
    match normalised {
        "Coq" => Ok(ProverKind::Coq),
        "Lean" => Ok(ProverKind::Lean),
        "Agda" => Ok(ProverKind::Agda),
        "Isabelle" => Ok(ProverKind::Isabelle),
        "Z3" => Ok(ProverKind::Z3),
        "Cvc5" => Ok(ProverKind::Cvc5),
        "Metamath" => Ok(ProverKind::Metamath),
        "HolLight" => Ok(ProverKind::HolLight),
        "Mizar" => Ok(ProverKind::Mizar),
        "Pvs" => Ok(ProverKind::Pvs),
        "Acl2" => Ok(ProverKind::Acl2),
        "Hol4" => Ok(ProverKind::Hol4),
        other => Err(anyhow!("Unknown prover: {}", other)),
    }
}

fn parse_source(raw: &str) -> DeltaSource {
    match raw {
        "webhook" => DeltaSource::Webhook,
        "cli" => DeltaSource::Cli,
        _ => DeltaSource::Mcp,
    }
}

async fn tool_suggest_tactics(state: Arc<ServerState>, args: Value) -> Result<Value> {
    #[derive(Deserialize)]
    struct A {
        prover: String,
        #[serde(default)]
        context: String,
        goal_state: String,
    }
    let a: A = serde_json::from_value(args).context("suggest_tactics args")?;
    let prover = parse_prover(&a.prover)?;
    let base = state
        .echidna
        .suggest_tactics(prover, &a.context, &a.goal_state)
        .await?;
    let reranked = state.reranker.rerank(prover, &a.goal_state, base).await?;
    Ok(serde_json::to_value(reranked)?)
}

async fn tool_record_tactic_outcome(state: Arc<ServerState>, args: Value) -> Result<Value> {
    #[derive(Deserialize)]
    struct A {
        prover: String,
        goal_state: String,
        tactic: String,
        succeeded: bool,
        duration_ms: i64,
        #[serde(default)]
        source: Option<String>,
    }
    let a: A = serde_json::from_value(args).context("record_tactic_outcome args")?;
    let prover = parse_prover(&a.prover)?;
    let fingerprint = echidnabot::store::models::goal_fingerprint(&a.goal_state);
    let record = TacticOutcomeRecord::new(
        None,
        prover,
        fingerprint,
        a.tactic.clone(),
        a.succeeded,
        a.duration_ms,
    );
    state.store.record_tactic_outcome(&record).await?;

    // Parallel: write a corpus-delta row so successes eventually feed retrainer.
    if let Some(corpus) = &state.corpus {
        let source = a
            .source
            .as_deref()
            .map(parse_source)
            .unwrap_or(DeltaSource::Mcp);
        let row = DeltaRow::new(
            prover,
            a.goal_state,
            a.tactic,
            a.succeeded,
            a.duration_ms,
            source,
        );
        corpus.record(&row).await?;
    }

    Ok(json!({ "id": record.id, "recorded": true }))
}

async fn tool_list_outcome_history(state: Arc<ServerState>, args: Value) -> Result<Value> {
    #[derive(Deserialize)]
    struct A {
        prover: String,
        #[serde(default)]
        goal_state: Option<String>,
        #[serde(default)]
        tactic: Option<String>,
        #[serde(default = "default_limit")]
        limit: usize,
    }
    fn default_limit() -> usize {
        20
    }

    let a: A = serde_json::from_value(args).context("list_outcome_history args")?;
    let prover = parse_prover(&a.prover)?;

    let rows = match (a.goal_state.as_deref(), a.tactic.as_deref()) {
        (Some(g), _) => {
            let fp = echidnabot::store::models::goal_fingerprint(g);
            state
                .store
                .list_tactic_outcomes_by_fingerprint(prover, &fp, a.limit)
                .await?
        }
        (None, Some(t)) => {
            state
                .store
                .list_tactic_outcomes_by_tactic(prover, t, a.limit)
                .await?
        }
        (None, None) => {
            return Err(anyhow!(
                "list_outcome_history needs at least one of `goal_state` or `tactic`"
            ));
        }
    };
    Ok(serde_json::to_value(rows)?)
}

async fn tool_corpus_refresh(state: Arc<ServerState>) -> Result<Value> {
    let corpus = state
        .corpus
        .as_ref()
        .ok_or_else(|| anyhow!("corpus feature is disabled — set corpus.enabled in config"))?;
    let status = corpus.trigger_refresh().await?;
    Ok(json!({
        "success": status.success,
        "exit_code": status.exit_code,
        "stdout": status.stdout,
        "stderr": status.stderr,
    }))
}

fn error_response(id: Option<Value>, code: i32, message: String) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(RpcError {
            code,
            message,
            data: None,
        }),
    }
}

async fn write_response(stdout: &mut Stdout, resp: &JsonRpcResponse) -> Result<()> {
    let mut line = serde_json::to_vec(resp)?;
    line.push(b'\n');
    stdout.write_all(&line).await?;
    stdout.flush().await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — protocol-level, not spawning the binary. We call dispatch() directly
// with constructed requests and check the resulting JSON.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use echidnabot::config::EchidnaConfig;

    async fn test_state() -> (Arc<ServerState>, std::path::PathBuf) {
        let db_path = std::env::temp_dir()
            .join(format!("mcp-test-{}.db", uuid::Uuid::new_v4()));
        let url = format!("sqlite://{}?mode=rwc", db_path.display());
        let store = Arc::new(SqliteStore::new(&url).await.unwrap()) as Arc<dyn Store>;
        let echidna = EchidnaClient::new(&EchidnaConfig::default());
        let reranker = Reranker::new(Arc::clone(&store));
        let state = Arc::new(ServerState {
            store,
            echidna,
            reranker,
            corpus: None,
        });
        (state, db_path)
    }

    fn request(method: &str, params: Value, id: i64) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(id)),
            method: method.to_string(),
            params,
        }
    }

    #[tokio::test]
    async fn initialize_advertises_server_info() {
        let (state, path) = test_state().await;
        let resp = dispatch(state, &request("initialize", json!({}), 1)).await.unwrap();
        assert_eq!(resp["protocolVersion"], PROTO_VERSION);
        assert_eq!(resp["serverInfo"]["name"], SERVER_NAME);
        assert_eq!(resp["serverInfo"]["version"], SERVER_VERSION);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn tools_list_returns_all_four_tools() {
        let (state, path) = test_state().await;
        let resp = dispatch(state, &request("tools/list", json!({}), 2)).await.unwrap();
        let names: Vec<&str> = resp["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"suggest_tactics"));
        assert!(names.contains(&"record_tactic_outcome"));
        assert!(names.contains(&"list_outcome_history"));
        assert!(names.contains(&"corpus_refresh"));
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn record_and_list_outcome_round_trip() {
        let (state, path) = test_state().await;
        // Record
        let rec = dispatch(
            state.clone(),
            &request(
                "tools/call",
                json!({
                    "name": "record_tactic_outcome",
                    "arguments": {
                        "prover": "Coq",
                        "goal_state": "forall x, x = x",
                        "tactic": "reflexivity",
                        "succeeded": true,
                        "duration_ms": 7
                    }
                }),
                10,
            ),
        )
        .await
        .unwrap();
        assert_eq!(rec["recorded"], true);

        // List by goal_state (fingerprint lookup)
        let list = dispatch(
            state.clone(),
            &request(
                "tools/call",
                json!({
                    "name": "list_outcome_history",
                    "arguments": {
                        "prover": "Coq",
                        "goal_state": "forall x, x = x"
                    }
                }),
                11,
            ),
        )
        .await
        .unwrap();
        let arr = list.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["tactic"], "reflexivity");
        assert_eq!(arr[0]["succeeded"], true);

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn unknown_method_errors() {
        let (state, path) = test_state().await;
        let res = dispatch(state, &request("nope/nope", json!({}), 3)).await;
        let err = res.unwrap_err().to_string();
        assert!(err.contains("Method not found"), "got: {}", err);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn corpus_refresh_errors_when_disabled() {
        let (state, path) = test_state().await;
        let res = dispatch(
            state,
            &request(
                "tools/call",
                json!({ "name": "corpus_refresh", "arguments": {} }),
                4,
            ),
        )
        .await;
        let err = res.unwrap_err().to_string();
        assert!(err.contains("corpus feature is disabled"), "got: {}", err);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn parse_prover_accepts_debug_and_lowercase() {
        assert!(matches!(parse_prover("Coq").unwrap(), ProverKind::Coq));
        assert!(matches!(parse_prover("coq").unwrap(), ProverKind::Coq));
        assert!(matches!(parse_prover("Z3").unwrap(), ProverKind::Z3));
        assert!(matches!(parse_prover("z3").unwrap(), ProverKind::Z3));
        assert!(parse_prover("perl").is_err());
    }
}
