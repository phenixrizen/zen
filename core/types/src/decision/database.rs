use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{TransformAttributes, empty_string_is_none};

/// Content of a `databaseNode`.
///
/// A database node resolves a logical data source, evaluates its parameter expressions against
/// the node input, and delegates execution to the host-supplied
/// [`DatabaseHandler`](https://docs.rs/zen-engine) extension. The engine itself never speaks a
/// database protocol and carries no driver dependency.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseNodeContent {
    /// Logical name of the data source. Resolved by the host; never a path or connection string.
    pub source: DatabaseSource,
    /// Ephemeral, request-scoped relations built from graph data so a query can join against them.
    #[serde(default)]
    pub relations: Arc<Vec<DatabaseRelation>>,
    pub query: DatabaseQuery,
    #[serde(default)]
    pub result: DatabaseResultShape,
    #[serde(flatten)]
    pub transform_attributes: TransformAttributes,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum DatabaseSource {
    /// A literal source name.
    Name(Arc<str>),
    /// An expression evaluating to a source name.
    Expression { expression: Arc<str> },
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DatabaseQuery {
    Select(SelectQuery),
    Raw(RawQuery),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectQuery {
    pub table: Arc<str>,
    /// Columns to project. An empty list selects everything and forfeits output type inference.
    #[serde(default)]
    pub columns: Arc<Vec<Arc<str>>>,
    #[serde(default)]
    pub distinct: bool,
    #[serde(default)]
    pub joins: Arc<Vec<SelectJoin>>,
    #[serde(default)]
    pub conditions: Arc<Vec<DatabaseCondition>>,
    #[serde(default)]
    pub order_by: Arc<Vec<DatabaseOrder>>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectJoin {
    /// Table or relation to join against.
    pub table: Arc<str>,
    #[serde(default)]
    pub kind: JoinKind,
    pub on: Arc<Vec<JoinCondition>>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum JoinKind {
    #[default]
    Inner,
    Left,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinCondition {
    pub left: Arc<str>,
    pub right: Arc<str>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseCondition {
    pub id: Arc<str>,
    pub column: Arc<str>,
    #[serde(default)]
    pub operator: DatabaseOperator,
    /// Expression producing the bound value(s). Never interpolated into statement text.
    #[serde(default, deserialize_with = "empty_string_is_none")]
    pub value: Option<Arc<str>>,
    #[serde(rename = "as", default)]
    pub value_type: Option<DatabaseValueType>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseOperator {
    #[default]
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
    In,
    NotIn,
    Like,
    IsNull,
    IsNotNull,
}

impl DatabaseOperator {
    /// Number of bound values the operator expects: `None` means variadic.
    pub const fn arity(&self) -> Option<usize> {
        match self {
            Self::IsNull | Self::IsNotNull => Some(0),
            Self::In | Self::NotIn => None,
            _ => Some(1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseValueType {
    Text,
    Integer,
    Number,
    Boolean,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseOrder {
    pub column: Arc<str>,
    #[serde(default)]
    pub direction: OrderDirection,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum OrderDirection {
    #[default]
    Asc,
    Desc,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseResultShape {
    /// All matching rows, as an array of objects.
    #[default]
    Rows,
    /// The first row, or null.
    First,
    /// The first column of the first row, or null.
    Scalar,
    /// Whether any row matched.
    Exists,
    /// Number of matching rows.
    Count,
}

/// A request-scoped relation materialized from graph data.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRelation {
    pub name: Arc<str>,
    /// Expression producing an array of objects.
    pub rows: Arc<str>,
    pub columns: Arc<Vec<RelationColumn>>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationColumn {
    pub name: Arc<str>,
    #[serde(rename = "type")]
    pub column_type: DatabaseValueType,
    /// Expression evaluated against each row element.
    pub value: Arc<str>,
}

/// A raw statement, for queries the declarative form cannot express.
///
/// Opaque to static analysis, and handlers may refuse it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawQuery {
    pub sql: Arc<str>,
    #[serde(default)]
    pub parameters: Arc<Vec<RawParameter>>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawParameter {
    pub id: Arc<str>,
    /// Named placeholder this parameter binds to.
    pub name: Arc<str>,
    pub value: Arc<str>,
    /// Expand an array value into one placeholder per element.
    #[serde(default)]
    pub expand: bool,
    #[serde(rename = "as", default)]
    pub value_type: Option<DatabaseValueType>,
}
