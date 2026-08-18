//! End-to-end: a JDM graph querying a real SQLite reference database.

use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use zen_database_sqlite::{SqliteConfig, SqliteDatabaseHandler};
use zen_engine::model::DecisionContent;
use zen_engine::{DecisionEngine, Variable};

/// A miniature stand-in for the real fee-schedule table, built at test time from SQL so no
/// binary fixture is committed.
/// Built by the real `sqlite3` binary, so the fixture is authored by neither the driver nor its
/// engine - this is a genuine file-format compatibility test, not a round-trip.
fn catalog() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("catalog.db");

    let status = std::process::Command::new("sqlite3")
        .arg(&path)
        .arg(
            "CREATE TABLE fees_short (code TEXT, globdays TEXT, \
                 effective_start INTEGER, effective_end INTEGER); \
             INSERT INTO fees_short VALUES \
                 ('11000','090',20200101,20301231), \
                 ('11001','010',20200101,20301231), \
                 ('99213','000',20200101,20301231), \
                 ('27130','090',20200101,20301231); \
             CREATE INDEX idx_code ON fees_short (code);",
        )
        .status()
        .expect("sqlite3 CLI is required to build the fixture");
    assert!(status.success(), "fixture creation failed");

    dir
}

fn engine(dir: &TempDir) -> DecisionEngine {
    let handler = SqliteDatabaseHandler::new(SqliteConfig::with_root(dir.path()));
    DecisionEngine::default().with_database_handler(Some(Arc::new(handler)))
}

fn graph(node: serde_json::Value) -> Arc<DecisionContent> {
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

async fn run(dir: &TempDir, node: serde_json::Value, input: serde_json::Value) -> Variable {
    engine(dir)
        .create_decision(graph(node))
        .expect("decision")
        .evaluate(input.into())
        .await
        .expect("evaluation should succeed")
        .result
}

#[tokio::test]
async fn looks_up_reference_data_by_code() {
    let dir = catalog();
    let output = run(
        &dir,
        json!({
            "source": "catalog",
            "query": { "type": "select", "table": "fees_short", "columns": ["code", "globdays"],
                "conditions": [
                    { "id": "c1", "column": "code", "operator": "eq",
                      "value": "clue.procedureCode", "as": "text" }
                ] },
            "result": "first",
            "outputPath": "fees"
        }),
        json!({ "clue": { "procedureCode": "11000" } }),
    )
    .await;

    assert_eq!(
        output,
        json!({ "fees": { "code": "11000", "globdays": "090" } }).into()
    );
}

/// The date-effective shape that motivated the whole design: a temporal query the input
/// envelope cannot express without shipping the table.
#[tokio::test]
async fn filters_by_effective_date_window() {
    let dir = catalog();
    let node = |dos: i64| {
        json!({
            "source": "catalog",
            "query": { "type": "select", "table": "fees_short", "columns": ["code"],
                "conditions": [
                    { "id": "c1", "column": "code", "operator": "eq", "value": "'11000'", "as": "text" },
                    { "id": "c2", "column": "effective_start", "operator": "lte", "value": format!("{dos}"), "as": "integer" },
                    { "id": "c3", "column": "effective_end", "operator": "gte", "value": format!("{dos}"), "as": "integer" }
                ] },
            "result": "exists",
            "outputPath": "effective"
        })
    };

    assert_eq!(
        run(&dir, node(20250601), json!({})).await,
        json!({ "effective": true }).into()
    );
    assert_eq!(
        run(&dir, node(20190601), json!({})).await,
        json!({ "effective": false }).into()
    );
}

#[tokio::test]
async fn binds_an_in_list_without_touching_statement_text() {
    let dir = catalog();
    let output = run(
        &dir,
        json!({
            "source": "catalog",
            "query": { "type": "select", "table": "fees_short", "columns": ["code"],
                "distinct": true,
                "conditions": [
                    { "id": "c1", "column": "code", "operator": "in",
                      "value": "map(ocls, #.procedureCode)", "as": "text" },
                    { "id": "c2", "column": "globdays", "operator": "eq", "value": "'090'", "as": "text" }
                ],
                "orderBy": [{ "column": "code", "direction": "asc" }] },
            "result": "rows",
            "outputPath": "matches"
        }),
        json!({ "ocls": [
            { "procedureCode": "11000" }, { "procedureCode": "99213" }, { "procedureCode": "27130" }
        ] }),
    )
    .await;

    // Only the two with a 090 global period come back.
    assert_eq!(
        output,
        json!({ "matches": [{ "code": "11000" }, { "code": "27130" }] }).into()
    );
}

/// The capability nothing in JDM could express before: joining the claim's own lines against
/// reference data, in the database, with the list bound rather than interpolated.
#[tokio::test]
async fn joins_claim_lines_against_reference_data() {
    let dir = catalog();
    let output = run(
        &dir,
        json!({
            "source": "catalog",
            "relations": [{
                "name": "ocl",
                "rows": "ocls",
                "columns": [{ "name": "code", "type": "text", "value": "procedureCode" }]
            }],
            "query": {
                "type": "select", "table": "fees_short", "columns": ["fees_short.code", "fees_short.globdays"],
                "joins": [{ "table": "ocl", "kind": "inner",
                    "on": [{ "left": "fees_short.code", "right": "ocl.code" }] }],
                "conditions": [
                    { "id": "c1", "column": "fees_short.globdays", "operator": "eq", "value": "'090'", "as": "text" }
                ],
                "orderBy": [{ "column": "fees_short.code", "direction": "asc" }]
            },
            "result": "rows",
            "outputPath": "surgical"
        }),
        json!({ "ocls": [
            { "procedureCode": "11000" }, { "procedureCode": "99213" }, { "procedureCode": "27130" }
        ] }),
    )
    .await;

    assert_eq!(
        output,
        json!({ "surgical": [
            { "code": "11000", "globdays": "090" },
            { "code": "27130", "globdays": "090" }
        ] })
        .into()
    );
}

/// Relations are rendered as read-only VALUES CTEs, so an evaluation cannot leave rows behind
/// for the next one. This is a correctness requirement in claims adjudication, and here it holds
/// structurally rather than by cleanup.
#[tokio::test]
async fn relations_do_not_leak_between_evaluations() {
    let dir = catalog();
    let eng = engine(&dir);

    let with_relation = graph(json!({
        "source": "catalog",
        "relations": [{ "name": "ocl", "rows": "ocls",
            "columns": [{ "name": "code", "type": "text", "value": "procedureCode" }] }],
        "query": { "type": "select", "table": "ocl", "columns": ["code"] },
        "result": "count", "outputPath": "n"
    }));

    let first = eng
        .create_decision(with_relation.clone())
        .expect("decision")
        .evaluate(json!({ "ocls": [{ "procedureCode": "a" }, { "procedureCode": "b" }] }).into())
        .await
        .expect("first evaluation")
        .result;
    assert_eq!(first, json!({ "n": 2 }).into());

    // A second evaluation with fewer rows must see only its own.
    let second = eng
        .create_decision(with_relation)
        .expect("decision")
        .evaluate(json!({ "ocls": [{ "procedureCode": "z" }] }).into())
        .await
        .expect("second evaluation")
        .result;
    assert_eq!(
        second,
        json!({ "n": 1 }).into(),
        "a relation from a previous evaluation must not survive in the pooled connection"
    );
}

#[tokio::test]
async fn raw_queries_are_refused_unless_enabled() {
    let dir = catalog();
    let node = json!({
        "source": "catalog",
        "query": { "type": "raw", "sql": "SELECT code FROM fees_short", "parameters": [] },
        "result": "rows"
    });

    let refused = engine(&dir)
        .create_decision(graph(node.clone()))
        .expect("decision")
        .evaluate(json!({}).into())
        .await;
    assert!(refused.is_err(), "raw must be off by default");

    let handler = SqliteDatabaseHandler::new(SqliteConfig::with_root(dir.path()).allow_raw(true));
    let permitted = DecisionEngine::default()
        .with_database_handler(Some(Arc::new(handler)))
        .create_decision(graph(node))
        .expect("decision")
        .evaluate(json!({}).into())
        .await;
    assert!(permitted.is_ok(), "raw must work once enabled");
}

#[tokio::test]
async fn unknown_source_is_an_error_not_a_path_probe() {
    let dir = catalog();
    let result = engine(&dir)
        .create_decision(graph(json!({
            "source": "does_not_exist",
            "query": { "type": "select", "table": "fees_short", "columns": ["code"] },
            "result": "rows"
        })))
        .expect("decision")
        .evaluate(json!({}).into())
        .await;

    assert!(result.is_err());
}

/// Claims are adjudicated concurrently, so the driver must not serialize.
///
/// This is the property that ruled out materializing relations into temporary tables: writing a
/// temp table takes a database-wide lock, which failed 15 of 16 concurrent evaluations. Rendering
/// relations as read-only VALUES CTEs keeps every request a pure read.
///
/// Graph evaluation is `!Send` (Variable is Rc-based), so this mirrors how the bindings actually
/// run: one OS thread per worker, each with its own current-thread runtime, all sharing a single
/// engine - the same shape as `LocalPoolHandle::spawn_pinned`.
#[test]
fn concurrent_evaluations_with_relations_do_not_contend() {
    let dir = catalog();
    let eng = Arc::new(engine(&dir));

    let node = json!({
        "source": "catalog",
        "relations": [{ "name": "ocl", "rows": "ocls",
            "columns": [{ "name": "code", "type": "text", "value": "procedureCode" }] }],
        "query": { "type": "select", "table": "fees_short", "columns": ["fees_short.code"],
            "joins": [{ "table": "ocl", "kind": "inner",
                "on": [{ "left": "fees_short.code", "right": "ocl.code" }] }] },
        "result": "count",
        "outputPath": "n"
    });

    let mut workers = Vec::new();
    for i in 0..16u32 {
        let eng = Arc::clone(&eng);
        let node = node.clone();
        workers.push(std::thread::spawn(move || -> Result<(), String> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;

            // Alternate claim sizes so a leaked relation shows up as a wrong count rather than
            // passing by coincidence.
            let (codes, expected) = if i % 2 == 0 {
                (json!([{ "procedureCode": "11000" }]), 1)
            } else {
                (
                    json!([{ "procedureCode": "11000" }, { "procedureCode": "27130" }]),
                    2,
                )
            };

            runtime.block_on(async move {
                let output = eng
                    .create_decision(graph(node))
                    .map_err(|e| e.to_string())?
                    .evaluate(json!({ "ocls": codes }).into())
                    .await
                    .map_err(|e| format!("{e:?}"))?
                    .result;

                if output == json!({ "n": expected }).into() {
                    Ok(())
                } else {
                    Err(format!("expected n={expected}, got {output:?}"))
                }
            })
        }));
    }

    let failures: Vec<String> = workers
        .into_iter()
        .filter_map(|w| w.join().expect("worker should not panic").err())
        .collect();

    assert!(
        failures.is_empty(),
        "{} of 16 concurrent evaluations failed: {:?}",
        failures.len(),
        &failures[..failures.len().min(3)]
    );
}
