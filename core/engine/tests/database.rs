//! Engine-side behaviour of `databaseNode`.
//!
//! The handler is faked throughout: what matters here is that the engine resolves expressions
//! against the right scope, rejects unsafe identifiers before any driver sees them, binds every
//! value, and shapes results correctly.

use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use zen_engine::model::DecisionContent;
use zen_engine::nodes::database::{
    DatabaseHandler, DatabaseRequest, DatabaseResponse, DatabaseValue, ResolvedQuery,
};
use zen_engine::{Decision, Variable};

/// Captures the request the engine produced and replays a canned response.
#[derive(Debug, Default)]
struct RecordingHandler {
    requests: Mutex<Vec<DatabaseRequest>>,
    response: Mutex<DatabaseResponse>,
}

impl RecordingHandler {
    fn with_rows(columns: &[&str], rows: Vec<Vec<DatabaseValue>>) -> Arc<Self> {
        Arc::new(Self {
            requests: Mutex::new(Vec::new()),
            response: Mutex::new(DatabaseResponse {
                columns: columns.iter().map(|c| c.to_string()).collect(),
                rows,
                truncated: false,
            }),
        })
    }

    fn empty() -> Arc<Self> {
        Self::with_rows(&["code"], Vec::new())
    }

    fn taken(&self) -> Vec<DatabaseRequest> {
        self.requests.lock().unwrap().clone()
    }

    fn only(&self) -> DatabaseRequest {
        let taken = self.taken();
        assert_eq!(taken.len(), 1, "expected exactly one handler call");
        taken.into_iter().next().unwrap()
    }

    fn select(&self) -> zen_engine::nodes::database::ResolvedSelect {
        match self.only().query {
            ResolvedQuery::Select(select) => select,
            ResolvedQuery::Raw(_) => panic!("expected a select query"),
        }
    }
}

impl DatabaseHandler for RecordingHandler {
    fn query(
        &self,
        request: DatabaseRequest,
    ) -> Pin<Box<dyn Future<Output = Result<DatabaseResponse, String>> + Send + '_>> {
        self.requests.lock().unwrap().push(request);
        let response = self.response.lock().unwrap().clone();
        Box::pin(async move { Ok(response) })
    }
}

fn graph(node_content: serde_json::Value) -> DecisionContent {
    graph_with_params(node_content, None)
}

fn graph_with_params(
    node_content: serde_json::Value,
    params: Option<serde_json::Value>,
) -> DecisionContent {
    let mut document = json!({
        "nodes": [
            { "id": "input", "name": "Request", "type": "inputNode", "content": { "schema": "" } },
            { "id": "db", "name": "Lookup", "type": "databaseNode", "content": node_content },
            { "id": "output", "name": "Response", "type": "outputNode" }
        ],
        "edges": [
            { "id": "e1", "sourceId": "input", "targetId": "db", "type": "edge" },
            { "id": "e2", "sourceId": "db", "targetId": "output", "type": "edge" }
        ]
    });

    if let Some(params) = params {
        document["params"] = params;
    }

    serde_json::from_value(document).expect("graph fixture should deserialize")
}

async fn run(
    handler: Arc<RecordingHandler>,
    node_content: serde_json::Value,
    context: serde_json::Value,
) -> Result<Variable, String> {
    run_with_params(handler, node_content, context, None).await
}

async fn run_with_params(
    handler: Arc<RecordingHandler>,
    node_content: serde_json::Value,
    context: serde_json::Value,
    params: Option<serde_json::Value>,
) -> Result<Variable, String> {
    let content = graph_with_params(node_content, params);
    let decision = Decision::from(content.as_graph().unwrap().clone())
        .with_database_handler(Some(handler.clone()));

    decision
        .evaluate(context.into())
        .await
        .map(|response| response.result)
        .map_err(|err| format!("{err:?}"))
}

fn select_node(conditions: serde_json::Value, result: &str) -> serde_json::Value {
    json!({
        "source": "catalog",
        "query": { "type": "select", "table": "fees_short", "columns": ["CODE"], "conditions": conditions },
        "result": result,
        "outputPath": "lookup"
    })
}

// ---------------------------------------------------------------- binding

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn binds_scalar_condition_from_input() {
    let handler = RecordingHandler::empty();
    let node = select_node(
        json!([{ "id": "c1", "column": "CODE", "operator": "eq", "value": "clue.code", "as": "text" }]),
        "rows",
    );

    run(
        handler.clone(),
        node,
        json!({ "clue": { "code": "11000" } }),
    )
    .await
    .expect("evaluation should succeed");

    let select = handler.select();
    assert_eq!(select.table, "fees_short");
    assert_eq!(select.conditions.len(), 1);
    assert_eq!(
        select.conditions[0].values,
        vec![DatabaseValue::Text("11000".into())]
    );
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn expands_in_list_into_individual_bound_values() {
    let handler = RecordingHandler::empty();
    let node = select_node(
        json!([{ "id": "c1", "column": "CODE", "operator": "in", "value": "map(ocls, #.code)", "as": "text" }]),
        "rows",
    );

    run(
        handler.clone(),
        node,
        json!({ "ocls": [{ "code": "1" }, { "code": "2" }, { "code": "3" }] }),
    )
    .await
    .expect("evaluation should succeed");

    // Each element is bound separately; nothing is interpolated into statement text.
    assert_eq!(
        handler.select().conditions[0].values,
        vec![
            DatabaseValue::Text("1".into()),
            DatabaseValue::Text("2".into()),
            DatabaseValue::Text("3".into()),
        ]
    );
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn empty_in_list_binds_no_values() {
    let handler = RecordingHandler::empty();
    let node = select_node(
        json!([{ "id": "c1", "column": "CODE", "operator": "in", "value": "codes", "as": "text" }]),
        "rows",
    );

    run(handler.clone(), node, json!({ "codes": [] }))
        .await
        .expect("evaluation should succeed");

    assert!(handler.select().conditions[0].values.is_empty());
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn coerces_values_by_declared_type() {
    let handler = RecordingHandler::empty();
    let node = select_node(
        json!([
            { "id": "c1", "column": "a", "operator": "eq", "value": "n", "as": "integer" },
            { "id": "c2", "column": "b", "operator": "eq", "value": "n", "as": "text" },
            { "id": "c3", "column": "c", "operator": "eq", "value": "flag", "as": "boolean" }
        ]),
        "rows",
    );

    run(handler.clone(), node, json!({ "n": 42, "flag": true }))
        .await
        .expect("evaluation should succeed");

    let conditions = handler.select().conditions;
    assert_eq!(conditions[0].values, vec![DatabaseValue::Integer(42)]);
    assert_eq!(conditions[1].values, vec![DatabaseValue::Text("42".into())]);
    assert_eq!(conditions[2].values, vec![DatabaseValue::Boolean(true)]);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn null_check_operators_bind_nothing() {
    let handler = RecordingHandler::empty();
    let node = select_node(
        json!([{ "id": "c1", "column": "CODE", "operator": "isNull" }]),
        "rows",
    );

    run(handler.clone(), node, json!({}))
        .await
        .expect("evaluation should succeed");

    assert!(handler.select().conditions[0].values.is_empty());
}

// ---------------------------------------------------------------- safety

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn rejects_unsafe_identifiers_before_calling_the_handler() {
    for bad in [
        "users; DROP TABLE users",
        "a b",
        "a\"b",
        "naïve",
        "",
        "x".repeat(64).as_str(),
    ] {
        let handler = RecordingHandler::empty();
        let node = json!({
            "source": "catalog",
            "query": { "type": "select", "table": bad, "columns": ["a"] },
            "result": "rows"
        });

        let result = run(handler.clone(), node, json!({})).await;

        assert!(result.is_err(), "table name {bad:?} should be rejected");
        assert!(
            handler.taken().is_empty(),
            "handler must not be called for table name {bad:?}"
        );
    }
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn missing_handler_is_a_clean_error_not_a_panic() {
    let content = graph(select_node(json!([]), "rows"));
    let decision = Decision::from(content.as_graph().unwrap().clone());

    let result = decision.evaluate(json!({}).into()).await;
    assert!(result.is_err(), "evaluation without a handler should fail");
}

// ---------------------------------------------------------------- results

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn shapes_results_per_result_mode() {
    let rows = vec![
        vec![DatabaseValue::Text("11000".into())],
        vec![DatabaseValue::Text("11001".into())],
    ];

    for (mode, expected) in [
        (
            "rows",
            json!({ "lookup": [{ "CODE": "11000" }, { "CODE": "11001" }] }),
        ),
        ("first", json!({ "lookup": { "CODE": "11000" } })),
        ("scalar", json!({ "lookup": "11000" })),
        ("exists", json!({ "lookup": true })),
        ("count", json!({ "lookup": 2 })),
    ] {
        let handler = RecordingHandler::with_rows(&["CODE"], rows.clone());
        let output = run(handler, select_node(json!([]), mode), json!({}))
            .await
            .unwrap_or_else(|e| panic!("mode {mode} should evaluate: {e}"));

        assert_eq!(output, expected.into(), "result mode {mode}");
    }
}

/// A null result under `outputPath` leaves the key absent rather than explicitly null, because
/// the engine merges node output with JSON-merge-patch semantics — a null in the patch removes
/// the key (`Variable::merge`). This matches every other node kind; downstream reads of a missing
/// key yield null anyway, so consumers cannot tell the difference.
#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn shapes_zero_row_results() {
    for (mode, expected) in [
        ("rows", json!({ "lookup": [] })),
        ("first", json!({})),
        ("scalar", json!({})),
        ("exists", json!({ "lookup": false })),
        ("count", json!({ "lookup": 0 })),
    ] {
        let handler = RecordingHandler::empty();
        let output = run(handler, select_node(json!([]), mode), json!({}))
            .await
            .unwrap_or_else(|e| panic!("mode {mode} should evaluate: {e}"));

        assert_eq!(output, expected.into(), "zero-row result mode {mode}");
    }
}

// ---------------------------------------------------------------- relations

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn materializes_a_relation_from_graph_data() {
    let handler = RecordingHandler::empty();
    let node = json!({
        "source": "catalog",
        "relations": [{
            "name": "ocl",
            "rows": "ocls",
            "columns": [
                { "name": "code", "type": "text", "value": "code" },
                { "name": "units", "type": "integer", "value": "units" }
            ]
        }],
        "query": {
            "type": "select", "table": "fees_short", "columns": ["CODE"],
            "joins": [{ "table": "ocl", "kind": "inner", "on": [{ "left": "CODE", "right": "ocl.code" }] }]
        },
        "result": "rows"
    });

    run(
        handler.clone(),
        node,
        json!({ "ocls": [{ "code": "11000", "units": 2 }, { "code": "99213", "units": 1 }] }),
    )
    .await
    .expect("evaluation should succeed");

    let request = handler.only();
    assert_eq!(request.relations.len(), 1);
    let relation = &request.relations[0];
    assert_eq!(relation.name, "ocl");
    assert_eq!(
        relation.rows,
        vec![
            vec![
                DatabaseValue::Text("11000".into()),
                DatabaseValue::Integer(2)
            ],
            vec![
                DatabaseValue::Text("99213".into()),
                DatabaseValue::Integer(1)
            ],
        ]
    );

    let select = handler.select();
    assert_eq!(select.joins.len(), 1);
    assert_eq!(select.joins[0].table, "ocl");
}

// ---------------------------------------------------------------- raw

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn raw_named_parameters_expand_without_touching_sql_text() {
    let handler = RecordingHandler::empty();
    let node = json!({
        "source": "catalog",
        "query": {
            "type": "raw",
            "sql": "SELECT CODE FROM fees_short WHERE CODE IN (:codes) AND GLOBDAYS = :gd",
            "parameters": [
                { "id": "p1", "name": "codes", "value": "codes", "expand": true, "as": "text" },
                { "id": "p2", "name": "gd", "value": "'090'", "as": "text" }
            ]
        },
        "result": "rows"
    });

    run(handler.clone(), node, json!({ "codes": ["1", "2"] }))
        .await
        .expect("evaluation should succeed");

    let ResolvedQuery::Raw(raw) = handler.only().query else {
        panic!("expected a raw query");
    };

    // The statement text is passed through verbatim; only the bindings carry data.
    assert!(raw.sql.contains(":codes"));
    assert_eq!(
        raw.parameters,
        vec![
            (
                "codes".to_string(),
                vec![
                    DatabaseValue::Text("1".into()),
                    DatabaseValue::Text("2".into())
                ]
            ),
            ("gd".to_string(), vec![DatabaseValue::Text("090".into())]),
        ]
    );
}

// ---------------------------------------------------------------- transform attributes

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn pass_through_merges_result_into_input() {
    let handler =
        RecordingHandler::with_rows(&["CODE"], vec![vec![DatabaseValue::Text("11000".into())]]);
    let mut node = select_node(json!([]), "exists");
    node["passThrough"] = json!(true);

    let output = run(handler, node, json!({ "clue": { "code": "11000" } }))
        .await
        .expect("evaluation should succeed");

    assert_eq!(
        output,
        json!({ "clue": { "code": "11000" }, "lookup": true }).into()
    );
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn loop_mode_issues_one_query_per_element() {
    let handler = RecordingHandler::empty();
    let mut node = select_node(
        json!([{ "id": "c1", "column": "CODE", "operator": "eq", "value": "code", "as": "text" }]),
        "exists",
    );
    node["inputField"] = json!("ocls");
    node["executionMode"] = json!("loop");

    run(
        handler.clone(),
        node,
        json!({ "ocls": [{ "code": "a" }, { "code": "b" }] }),
    )
    .await
    .expect("evaluation should succeed");

    let requests = handler.taken();
    assert_eq!(requests.len(), 2, "one query per array element");
}

// ---------------------------------------------------------------- serde

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn node_content_round_trips_through_serde() {
    let original = json!({
        "nodes": [
            { "id": "input", "name": "Request", "type": "inputNode", "content": { "schema": "" } },
            { "id": "db", "name": "Lookup", "type": "databaseNode", "content": {
                "source": "catalog",
                "relations": [{ "name": "ocl", "rows": "ocls",
                    "columns": [{ "name": "code", "type": "text", "value": "code" }] }],
                "query": { "type": "select", "table": "fees_short", "columns": ["CODE"],
                    "distinct": true,
                    "joins": [{ "table": "ocl", "kind": "inner",
                        "on": [{ "left": "CODE", "right": "ocl.code" }] }],
                    "conditions": [{ "id": "c1", "column": "CODE", "operator": "in",
                        "value": "map(ocls, #.code)", "as": "text" }],
                    "orderBy": [{ "column": "CODE", "direction": "asc" }],
                    "limit": 100 },
                "result": "rows",
                "inputField": null, "outputPath": "lookup",
                "executionMode": "single", "passThrough": true
            }},
            { "id": "output", "name": "Response", "type": "outputNode" }
        ],
        "edges": []
    });

    let content: DecisionContent =
        serde_json::from_value(original.clone()).expect("should deserialize");
    let reserialized = serde_json::to_value(&content).expect("should serialize");

    assert_eq!(
        reserialized["nodes"][1]["content"], original["nodes"][1]["content"],
        "databaseNode content must survive a serde round-trip unchanged"
    );
}

// ---------------------------------------------------------------- params

/// Decision-level constants reach the node's expressions as `$params`, so a policy can carry
/// its own code sets and effective dates instead of relying on the host to inject them.
#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn params_are_readable_from_conditions() {
    let handler = RecordingHandler::empty();
    let node = select_node(
        json!([
            { "id": "c1", "column": "CODE", "operator": "in",
              "value": "$params.codes", "as": "text" },
            { "id": "c2", "column": "EFFECTIVESTARTDATE", "operator": "lte",
              "value": "$params.effectiveDate", "as": "text" }
        ]),
        "rows",
    );

    run_with_params(
        handler.clone(),
        node,
        json!({ "clue": { "code": "11000" } }),
        Some(json!({ "codes": ["11000", "11001"], "effectiveDate": "2026-01-01" })),
    )
    .await
    .expect("evaluation should succeed");

    let conditions = handler.select().conditions;
    assert_eq!(
        conditions[0].values,
        vec![
            DatabaseValue::Text("11000".into()),
            DatabaseValue::Text("11001".into())
        ]
    );
    assert_eq!(
        conditions[1].values,
        vec![DatabaseValue::Text("2026-01-01".into())]
    );
}

/// The data source itself can be selected per policy, which is how a decision pins the
/// reference-data vintage it was authored against.
#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn params_can_select_the_source() {
    let handler = RecordingHandler::empty();
    let node = json!({
        "source": { "expression": "$params.catalogDb" },
        "query": { "type": "select", "table": "fees_short", "columns": ["CODE"] },
        "result": "rows"
    });

    run_with_params(
        handler.clone(),
        node,
        json!({}),
        Some(json!({ "catalogDb": "catalog_2026q1" })),
    )
    .await
    .expect("evaluation should succeed");

    assert_eq!(handler.only().source, "catalog_2026q1");
}

/// `$params` is a reserved key: it must not leak into the decision result.
#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn params_do_not_leak_into_output() {
    let handler = RecordingHandler::empty();
    let mut node = select_node(json!([]), "count");
    node["passThrough"] = json!(true);

    let output = run_with_params(
        handler,
        node,
        json!({ "clue": { "code": "11000" } }),
        Some(json!({ "secret": "should-not-appear" })),
    )
    .await
    .expect("evaluation should succeed");

    assert_eq!(
        output,
        json!({ "clue": { "code": "11000" }, "lookup": 0 }).into(),
        "$params must be stripped from the result"
    );
}
