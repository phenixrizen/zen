use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use zen_types::decision::{DatabaseOperator, JoinKind, OrderDirection};

/// Host-supplied database access for [`databaseNode`](zen_types::decision::DecisionNodeKind).
///
/// The engine resolves the node's expressions, validates identifiers and binds every value before
/// calling the handler, so implementors receive a request that is already safe to render into a
/// statement. Mirrors [`HttpHandler`](crate::nodes::http_handler::HttpHandler): the returned future
/// is `Send`, so a handler is free to move work onto a blocking pool.
pub trait DatabaseHandler: Debug + Send + Sync {
    fn query(
        &self,
        request: DatabaseRequest,
    ) -> Pin<Box<dyn Future<Output = Result<DatabaseResponse, String>> + Send + '_>>;
}

pub type DynamicDatabaseHandler = Option<Arc<dyn DatabaseHandler + Send + Sync>>;

/// A fully resolved query. Every value is bound; nothing here is interpolated into statement text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRequest {
    /// Logical source name, already resolved from the node's `source`.
    pub source: String,
    /// Request-scoped relations to make available to the query.
    #[serde(default)]
    pub relations: Vec<ResolvedRelation>,
    pub query: ResolvedQuery,
    /// Engine-enforced ceiling on returned rows. Handlers should stop reading past this.
    pub max_rows: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResolvedQuery {
    Select(ResolvedSelect),
    Raw(ResolvedRaw),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedSelect {
    pub table: String,
    pub columns: Vec<String>,
    pub distinct: bool,
    pub joins: Vec<ResolvedJoin>,
    pub conditions: Vec<ResolvedPredicate>,
    pub order_by: Vec<ResolvedOrder>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedJoin {
    pub table: String,
    pub kind: JoinKind,
    pub on: Vec<(String, String)>,
}

/// A resolved predicate tree. Top-level entries are combined with AND.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResolvedPredicate {
    Condition(ResolvedCondition),
    All { all: Vec<ResolvedPredicate> },
    Any { any: Vec<ResolvedPredicate> },
}

impl ResolvedPredicate {
    /// Every leaf comparison in the tree, in declaration order.
    pub fn conditions(&self) -> Vec<&ResolvedCondition> {
        let mut out = Vec::new();
        self.collect(&mut out);
        out
    }

    fn collect<'a>(&'a self, out: &mut Vec<&'a ResolvedCondition>) {
        match self {
            Self::Condition(condition) => out.push(condition),
            Self::All { all } => all.iter().for_each(|p| p.collect(out)),
            Self::Any { any } => any.iter().for_each(|p| p.collect(out)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedCondition {
    pub column: String,
    pub operator: DatabaseOperator,
    /// Bound values: empty for null checks, one for scalar comparisons, N for `in`/`notIn`.
    pub values: Vec<DatabaseValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedOrder {
    pub column: String,
    pub direction: OrderDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRelation {
    pub name: String,
    pub columns: Vec<ResolvedRelationColumn>,
    pub rows: Vec<Vec<DatabaseValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRelationColumn {
    pub name: String,
    pub value_type: DatabaseValueKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRaw {
    pub sql: String,
    /// Named parameters. A parameter may carry multiple values when it expands a list.
    pub parameters: Vec<(String, Vec<DatabaseValue>)>,
}

/// Declared type of a relation column.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseValueKind {
    Text,
    Integer,
    Number,
    Boolean,
}

/// A bound value.
///
/// Deliberately not `serde_json::Value`: this keeps integers and decimals exact and is unaffected
/// by the `arbitrary_precision` feature, which changes `Value`'s number representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum DatabaseValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Decimal(Decimal),
    Text(String),
    Blob(Vec<u8>),
}

/// Columnar result. Column names are carried once rather than per row.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseResponse {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<DatabaseValue>>,
    /// Set when the handler stopped at `max_rows`.
    #[serde(default)]
    pub truncated: bool,
}

/// Placeholder handler used when the host supplied none.
#[derive(Debug, Default)]
pub struct NoopDatabaseHandler;

impl DatabaseHandler for NoopDatabaseHandler {
    fn query(
        &self,
        _request: DatabaseRequest,
    ) -> Pin<Box<dyn Future<Output = Result<DatabaseResponse, String>> + Send + '_>> {
        Box::pin(async { Err("Database handler not provided".to_string()) })
    }
}
