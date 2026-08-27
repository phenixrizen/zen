//! Corpus-wide differential: every convertible production query, checked against real SQLite.
//!
//! A manifest produced from the engine corpus supplies, for each `legacy lookup callback` site: the
//! declarative `databaseNode` it converts to, a sample input drawn from the real table, and the
//! result real SQLite gives for the original SQL under production binding semantics.
//!
//! Each node is evaluated through the engine and compared against that oracle.
//!
//! Set ZEN_CORPUS_MANIFEST to the oracle file and ZEN_CATALOG_DIR to the catalog bundle.
//! Skipped when either is absent.

use serde_json::{json, Value};
use std::sync::Arc;
use zen_database_sqlite::{SqliteConfig, SqliteDatabaseHandler};
use zen_engine::model::DecisionContent;
use zen_engine::DecisionEngine;

fn manifest_path() -> Option<String> {
    std::env::var("ZEN_CORPUS_MANIFEST")
        .ok()
        .filter(|p| std::path::Path::new(p).exists())
}

fn catalog_dir() -> Option<String> {
    std::env::var("ZEN_CATALOG_DIR")
        .ok()
        .filter(|p| std::path::Path::new(p).exists())
}

fn graph(node: &Value) -> Arc<DecisionContent> {
    Arc::new(
        serde_json::from_value(json!({
            "nodes": [
                { "id": "input", "name": "Claim", "type": "inputNode", "content": { "schema": "" } },
                { "id": "db", "name": "Lookup", "type": "databaseNode", "content": node },
                { "id": "output", "name": "Insight", "type": "outputNode" }
            ],
            "edges": [
                { "id": "e1", "sourceId": "input", "targetId": "db", "type": "edge" },
                { "id": "e2", "sourceId": "db", "targetId": "output", "type": "edge" }
            ]
        }))
        .expect("graph"),
    )
}

/// SQLite is dynamically typed and the oracle reads through Python, so compare on value rather
/// than representation: "090" and 90 are the same cell.
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
        }
        (Value::String(s), Value::Number(n)) | (Value::Number(n), Value::String(s)) => {
            s == &n.to_string() || s.parse::<f64>().ok() == n.as_f64()
        }
        _ => a == b,
    }
}

#[tokio::test]
async fn every_convertible_production_query_matches_sqlite() {
    let (Some(manifest), Some(dir)) = (manifest_path(), catalog_dir()) else {
        eprintln!("corpus manifest or catalog absent; skipping");
        return;
    };

    let entries: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(&manifest).expect("read manifest"))
            .expect("parse manifest");

    let handler = SqliteDatabaseHandler::new(SqliteConfig::with_root(&dir).allow_raw(true));
    let engine = DecisionEngine::default().with_database_handler(Some(Arc::new(handler)));

    // The same site is evaluated twice: as a declarative query and as raw SQL passed through
    // verbatim. Both are compared against the same SQLite oracle, so a disagreement between them
    // is as interesting as a disagreement with SQLite.
    let mut raw_matched = 0usize;
    let mut raw_total = 0usize;
    let mut raw_errors: Vec<String> = Vec::new();
    let mut matched = 0usize;
    let mut non_empty_matched = 0usize;
    let mut mismatches: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for entry in &entries {
        let node = &entry["node"];
        let input = entry["input"].clone();
        let expected = &entry["expected"];
        let key = node["outputPath"].as_str().unwrap_or("rd");
        let label = format!(
            "{}::{}",
            entry["engine"].as_str().unwrap_or("?"),
            entry["nodeId"].as_str().unwrap_or("?")
        );

        let decision = match engine.create_decision(graph(node)) {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!("{label}: create decision: {e:?}"));
                continue;
            }
        };

        let result = match decision.evaluate(input.into()).await {
            Ok(r) => r.result,
            Err(e) => {
                errors.push(format!("{label}: evaluate: {e:?}"));
                continue;
            }
        };

        let actual_json: Value = serde_json::to_value(&result).expect("result to json");
        // A null result is dropped by the engine's merge-patch semantics, so an absent key and
        // an expected null are the same outcome.
        let actual = actual_json.get(key).cloned().unwrap_or(Value::Null);

        if same(&actual, expected) {
            matched += 1;
            if entry["nonEmpty"].as_bool() == Some(true) {
                non_empty_matched += 1;
            }
        } else if mismatches.len() < 8 {
            // fallthrough below
            mismatches.push(format!(
                "{label}\n      expected: {}\n      actual:   {}",
                serde_json::to_string(expected).unwrap_or_default(),
                serde_json::to_string(&actual).unwrap_or_default()
            ));
        }

        // Now the same site as raw SQL.
        let Some(raw_node) = entry.get("rawNode").filter(|v| !v.is_null()) else {
            continue;
        };
        raw_total += 1;
        let raw_decision = match engine.create_decision(graph(raw_node)) {
            Ok(d) => d,
            Err(e) => {
                raw_errors.push(format!("{label}: raw create: {e:?}"));
                continue;
            }
        };
        let raw_result = match raw_decision.evaluate(entry["input"].clone().into()).await {
            Ok(r) => r.result,
            Err(e) => {
                raw_errors.push(format!("{label}: raw evaluate: {e:?}"));
                continue;
            }
        };
        let raw_json: Value = serde_json::to_value(&raw_result).expect("raw result to json");
        let raw_actual = raw_json.get(key).cloned().unwrap_or(Value::Null);
        if same(&raw_actual, expected) {
            raw_matched += 1;
        } else if raw_errors.len() < 8 {
            raw_errors.push(format!(
                "{label}: RAW MISMATCH\n      expected: {}\n      actual:   {}",
                serde_json::to_string(expected).unwrap_or_default(),
                serde_json::to_string(&raw_actual).unwrap_or_default()
            ));
        }
    }

    let total = entries.len();
    let non_empty_total = entries
        .iter()
        .filter(|e| e["nonEmpty"].as_bool() == Some(true))
        .count();

    println!("\n=== corpus differential ===");
    println!(
        "  RAW      matched {raw_matched}/{raw_total}, errors {}",
        raw_errors.len()
    );
    for e in raw_errors.iter().take(6) {
        println!("    {e}");
    }
    println!("  matched          {matched}/{total}");
    println!("  of the {non_empty_total} that return rows: {non_empty_matched} matched");
    println!("  errors           {}", errors.len());
    for e in errors.iter().take(6) {
        println!("    {e}");
    }
    for m in &mismatches {
        println!("    MISMATCH {m}");
    }

    assert!(
        raw_errors.is_empty() && raw_matched == raw_total,
        "raw path: {raw_matched}/{raw_total} matched, {} errors",
        raw_errors.len()
    );
    assert!(
        errors.is_empty() && matched == total,
        "{} mismatched, {} errored, out of {total}",
        total - matched - errors.len(),
        errors.len()
    );
}
