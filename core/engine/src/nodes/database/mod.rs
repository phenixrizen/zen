use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use zen_expression::variable::{ToVariable, Variable};
use zen_types::decision::{
    DatabaseCondition, DatabaseNodeContent, DatabasePredicate, DatabaseQuery, DatabaseRelation,
    DatabaseResultShape, DatabaseSource, DatabaseValueType, RawQuery, SelectQuery,
    TransformAttributes,
};
use zen_types::symbol::Symbol;

use crate::nodes::context::{NodeContext, NodeContextExt};
use crate::nodes::definition::NodeHandler;
use crate::nodes::result::NodeResult;

pub mod handler;

pub use handler::{
    DatabaseHandler, DatabaseRequest, DatabaseResponse, DatabaseValue, DatabaseValueKind,
    DynamicDatabaseHandler, NoopDatabaseHandler, ResolvedCondition, ResolvedJoin, ResolvedOrder,
    ResolvedPredicate, ResolvedQuery, ResolvedRaw, ResolvedRelation, ResolvedRelationColumn,
    ResolvedSelect,
};

use handler::DatabaseValue as Val;

#[derive(Debug, Clone)]
pub struct DatabaseNodeHandler;

pub type DatabaseNodeData = DatabaseNodeContent;

#[derive(Debug, Clone, Default, ToVariable)]
pub struct DatabaseNodeTrace {
    source: Variable,
    query: Variable,
    row_count: Variable,
    truncated: Variable,
}

/// Identifiers reaching a handler are validated here rather than in each driver, so that
/// injection safety is a property of the engine and not of any one implementation.
fn validate_identifier(kind: &str, value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > 63 {
        return Err(format!(
            "invalid {kind} \"{value}\": must be 1-63 characters"
        ));
    }

    let mut chars = value.chars();
    let valid_start = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
    let valid_rest = chars.all(|c| c.is_ascii_alphanumeric() || c == '_');

    if !valid_start || !valid_rest {
        return Err(format!(
            "invalid {kind} \"{value}\": expected [A-Za-z_][A-Za-z0-9_]*"
        ));
    }

    Ok(())
}

/// A qualified column reference: either `column` or `relation.column`.
fn validate_column_ref(value: &str) -> Result<(), String> {
    match value.split_once('.') {
        Some((qualifier, column)) => {
            validate_identifier("table qualifier", qualifier)?;
            validate_identifier("column", column)
        }
        None => validate_identifier("column", value),
    }
}

fn coerce(value: Variable, hint: Option<DatabaseValueType>) -> Result<Val, String> {
    let coerced = match (&value, hint) {
        (Variable::Null, _) => Val::Null,
        (_, Some(DatabaseValueType::Text)) => Val::Text(variable_to_string(&value)?),
        (_, Some(DatabaseValueType::Boolean)) => match value {
            Variable::Bool(b) => Val::Boolean(b),
            _ => return Err(format!("expected boolean, got {}", type_name(&value))),
        },
        (_, Some(DatabaseValueType::Integer)) => match &value {
            // to_i64 truncates, so a non-integral value must be rejected rather than silently
            // rounded - binding 42.9 as 42 would be data corruption, not coercion.
            Variable::Number(n) => n
                .is_integer()
                .then(|| n.to_i64())
                .flatten()
                .map(Val::Integer)
                .ok_or_else(|| format!("value {n} is not representable as an integer"))?,
            Variable::String(s) => s
                .parse::<i64>()
                .map(Val::Integer)
                .map_err(|_| format!("value \"{s}\" is not an integer"))?,
            _ => return Err(format!("expected integer, got {}", type_name(&value))),
        },
        (_, Some(DatabaseValueType::Number)) => match &value {
            Variable::Number(n) => Val::Decimal(*n),
            Variable::String(s) => s
                .parse::<Decimal>()
                .map(Val::Decimal)
                .map_err(|_| format!("value \"{s}\" is not a number"))?,
            _ => return Err(format!("expected number, got {}", type_name(&value))),
        },
        // No hint: map the natural representation.
        (Variable::Bool(b), None) => Val::Boolean(*b),
        (Variable::Number(n), None) => match n.is_integer().then(|| n.to_i64()).flatten() {
            Some(i) => Val::Integer(i),
            None => Val::Decimal(*n),
        },
        (Variable::String(s), None) => Val::Text(s.to_string()),
        _ => {
            return Err(format!(
                "cannot bind {} as a database value",
                type_name(&value)
            ))
        }
    };

    Ok(coerced)
}

fn variable_to_string(value: &Variable) -> Result<String, String> {
    match value {
        Variable::String(s) => Ok(s.to_string()),
        Variable::Number(n) => Ok(n.to_string()),
        Variable::Bool(b) => Ok(b.to_string()),
        _ => Err(format!("cannot bind {} as text", type_name(value))),
    }
}

fn type_name(value: &Variable) -> &'static str {
    match value {
        Variable::Null => "null",
        Variable::Bool(_) => "boolean",
        Variable::Number(_) => "number",
        Variable::String(_) => "string",
        Variable::Array(_) => "array",
        Variable::Object(_) => "object",
        Variable::Dynamic(_) => "dynamic",
    }
}

fn value_to_variable(value: &Val) -> Variable {
    match value {
        Val::Null => Variable::Null,
        Val::Boolean(b) => Variable::Bool(*b),
        Val::Integer(i) => Variable::Number(Decimal::from(*i)),
        Val::Decimal(d) => Variable::Number(*d),
        Val::Text(s) => Variable::String(Symbol::from(s.as_str())),
        // Blobs have no faithful Variable representation. Erasing them silently would be data
        // loss, so the node rejects them and leaves any encoding decision to the graph author.
        Val::Blob(_) => Variable::Null,
    }
}

fn row_has_blob(row: &[Val]) -> bool {
    row.iter().any(|value| matches!(value, Val::Blob(_)))
}

impl NodeHandler for DatabaseNodeHandler {
    type NodeData = DatabaseNodeData;
    type TraceData = DatabaseNodeTrace;

    fn transform_attributes(
        &self,
        ctx: &NodeContext<Self::NodeData, Self::TraceData>,
    ) -> Option<TransformAttributes> {
        Some(ctx.node.transform_attributes.clone())
    }

    async fn handle(&self, ctx: NodeContext<Self::NodeData, Self::TraceData>) -> NodeResult {
        let Some(handler) = ctx.extensions.database_handler().clone() else {
            return ctx.error("Database handler not provided".to_string());
        };

        let mut isolate = ctx.isolate();

        let source = match &ctx.node.source {
            DatabaseSource::Name(name) => name.to_string(),
            DatabaseSource::Expression { expression } => {
                let value = isolate
                    .run_standard(expression)
                    .with_node_context(&ctx, |_| {
                        format!(r#"Failed to evaluate source expression: "{expression}""#)
                    })?;

                match value {
                    Variable::String(s) => s.to_string(),
                    other => {
                        return ctx.error(format!(
                            "source expression must produce a string, got {}",
                            type_name(&other)
                        ))
                    }
                }
            }
        };
        validate_identifier("source", &source).node_context(&ctx)?;

        let mut relations = Vec::with_capacity(ctx.node.relations.len());
        for relation in ctx.node.relations.iter() {
            relations.push(resolve_relation(&ctx, &mut isolate, relation)?);
        }

        let query = match &ctx.node.query {
            DatabaseQuery::Select(select) => {
                ResolvedQuery::Select(resolve_select(&ctx, &mut isolate, select)?)
            }
            DatabaseQuery::Raw(raw) => ResolvedQuery::Raw(resolve_raw(&ctx, &mut isolate, raw)?),
        };

        let max_rows = ctx.config.database_max_rows;
        let request = DatabaseRequest {
            source: source.clone(),
            relations,
            query,
            max_rows,
        };

        ctx.trace(|trace| {
            trace.source = Variable::String(Symbol::from(source.as_str()));
            trace.query = serde_json::to_value(&request.query)
                .map(Variable::from)
                .unwrap_or(Variable::Null);
        });

        let mut response = handler
            .query(request)
            .await
            .map_err(|err| ctx.make_error(err))?;

        // max_rows is advertised as engine-enforced, so enforce it here rather than trusting the
        // handler to have honoured it.
        let limit = max_rows as usize;
        if response.rows.len() > limit {
            response.rows.truncate(limit);
            response.truncated = true;
        }

        ctx.trace(|trace| {
            trace.row_count = Variable::Number(Decimal::from(response.rows.len()));
            trace.truncated = Variable::Bool(response.truncated);
        });

        let output = shape_result(&ctx.node.result, response).node_context(&ctx)?;
        ctx.success(output)
    }
}

fn shape_result(
    shape: &DatabaseResultShape,
    response: DatabaseResponse,
) -> Result<Variable, String> {
    // Shapes that only look at row counts need no cell inspection.
    match shape {
        DatabaseResultShape::Exists => return Ok(Variable::Bool(!response.rows.is_empty())),
        DatabaseResultShape::Count => {
            if response.truncated {
                return Err(
                    "row count is unreliable: the result was truncated at the configured maximum"
                        .to_string(),
                );
            }
            return Ok(Variable::Number(Decimal::from(response.rows.len())));
        }
        _ => {}
    }

    let width = response.columns.len();
    for (index, row) in response.rows.iter().enumerate() {
        if row.len() != width {
            return Err(format!(
                "row {index} has {} values but {width} columns were declared",
                row.len()
            ));
        }
        if row_has_blob(row) {
            return Err(format!(
                "row {index} contains a blob, which has no representation in a decision value"
            ));
        }
    }

    let row_to_object = |row: &Vec<Val>| {
        let object = Variable::empty_object();
        for (column, value) in response.columns.iter().zip(row.iter()) {
            object.dot_insert(column, value_to_variable(value));
        }
        object
    };

    let output = match shape {
        DatabaseResultShape::Rows => Variable::from_array(
            response
                .rows
                .iter()
                .map(row_to_object)
                .collect::<Vec<Variable>>(),
        ),
        DatabaseResultShape::First => response.rows.first().map(row_to_object).unwrap_or_default(),
        DatabaseResultShape::Scalar => response
            .rows
            .first()
            .and_then(|row| row.first())
            .map(value_to_variable)
            .unwrap_or_default(),
        // Handled above.
        DatabaseResultShape::Exists | DatabaseResultShape::Count => Variable::Null,
    };

    Ok(output)
}

fn resolve_relation(
    ctx: &NodeContext<DatabaseNodeData, DatabaseNodeTrace>,
    isolate: &mut zen_expression::Isolate,
    relation: &DatabaseRelation,
) -> Result<ResolvedRelation, crate::nodes::result::NodeError> {
    validate_identifier("relation", &relation.name).node_context(ctx)?;

    let rows_value = isolate
        .run_standard(&relation.rows)
        .with_node_context(ctx, |_| {
            format!(r#"Failed to evaluate relation rows: "{}""#, &relation.rows)
        })?;

    let Variable::Array(rows) = rows_value else {
        return Err(ctx.make_error(format!(
            "relation \"{}\" rows must be an array, got {}",
            relation.name,
            type_name(&rows_value)
        )));
    };

    let mut columns = Vec::with_capacity(relation.columns.len());
    for column in relation.columns.iter() {
        validate_identifier("relation column", &column.name).node_context(ctx)?;
        columns.push(ResolvedRelationColumn {
            name: column.name.to_string(),
            value_type: match column.column_type {
                DatabaseValueType::Text => DatabaseValueKind::Text,
                DatabaseValueType::Integer => DatabaseValueKind::Integer,
                DatabaseValueType::Number => DatabaseValueKind::Number,
                DatabaseValueType::Boolean => DatabaseValueKind::Boolean,
            },
        });
    }

    // Column expressions are scoped to the row element, but must still see the reserved
    // handles and reuse the graph's compiled bytecode, like every other expression in the node.
    let mut element_isolate = ctx.isolate();
    let mut resolved_rows = Vec::new();
    for element in rows.borrow().iter() {
        element_isolate.set_environment(element.shallow_clone());

        let mut values = Vec::with_capacity(relation.columns.len());
        for column in relation.columns.iter() {
            let value = element_isolate
                .run_standard(&column.value)
                .with_node_context(ctx, |_| {
                    format!(
                        r#"Failed to evaluate relation column "{}": "{}""#,
                        &column.name, &column.value
                    )
                })?;
            values.push(coerce(value, Some(column.column_type)).node_context(ctx)?);
        }
        resolved_rows.push(values);
    }

    Ok(ResolvedRelation {
        name: relation.name.to_string(),
        columns,
        rows: resolved_rows,
    })
}

fn resolve_select(
    ctx: &NodeContext<DatabaseNodeData, DatabaseNodeTrace>,
    isolate: &mut zen_expression::Isolate,
    select: &SelectQuery,
) -> Result<ResolvedSelect, crate::nodes::result::NodeError> {
    validate_identifier("table", &select.table).node_context(ctx)?;
    for column in select.columns.iter() {
        validate_column_ref(column).node_context(ctx)?;
    }

    let mut joins = Vec::with_capacity(select.joins.len());
    for join in select.joins.iter() {
        validate_identifier("join table", &join.table).node_context(ctx)?;
        let mut on = Vec::with_capacity(join.on.len());
        for condition in join.on.iter() {
            validate_column_ref(&condition.left).node_context(ctx)?;
            validate_column_ref(&condition.right).node_context(ctx)?;
            on.push((condition.left.to_string(), condition.right.to_string()));
        }
        joins.push(ResolvedJoin {
            table: join.table.to_string(),
            kind: join.kind.clone(),
            on,
        });
    }

    let mut conditions = Vec::with_capacity(select.conditions.len());
    for predicate in select.conditions.iter() {
        conditions.push(resolve_predicate(ctx, isolate, predicate)?);
    }

    let mut order_by = Vec::with_capacity(select.order_by.len());
    for order in select.order_by.iter() {
        validate_column_ref(&order.column).node_context(ctx)?;
        order_by.push(ResolvedOrder {
            column: order.column.to_string(),
            direction: order.direction,
        });
    }

    Ok(ResolvedSelect {
        table: select.table.to_string(),
        columns: select.columns.iter().map(|c| c.to_string()).collect(),
        distinct: select.distinct,
        joins,
        conditions,
        order_by,
        limit: select.limit,
    })
}

fn resolve_predicate(
    ctx: &NodeContext<DatabaseNodeData, DatabaseNodeTrace>,
    isolate: &mut zen_expression::Isolate,
    predicate: &DatabasePredicate,
) -> Result<ResolvedPredicate, crate::nodes::result::NodeError> {
    let resolved = match predicate {
        DatabasePredicate::Condition(condition) => {
            ResolvedPredicate::Condition(resolve_condition(ctx, isolate, condition)?)
        }
        DatabasePredicate::All { all } => {
            if all.is_empty() {
                return Err(ctx.make_error("an `all` predicate group is empty".to_string()));
            }
            let mut nested = Vec::with_capacity(all.len());
            for inner in all.iter() {
                nested.push(resolve_predicate(ctx, isolate, inner)?);
            }
            ResolvedPredicate::All { all: nested }
        }
        DatabasePredicate::Any { any } => {
            if any.is_empty() {
                return Err(ctx.make_error("an `any` predicate group is empty".to_string()));
            }
            let mut nested = Vec::with_capacity(any.len());
            for inner in any.iter() {
                nested.push(resolve_predicate(ctx, isolate, inner)?);
            }
            ResolvedPredicate::Any { any: nested }
        }
    };

    Ok(resolved)
}

fn resolve_condition(
    ctx: &NodeContext<DatabaseNodeData, DatabaseNodeTrace>,
    isolate: &mut zen_expression::Isolate,
    condition: &DatabaseCondition,
) -> Result<ResolvedCondition, crate::nodes::result::NodeError> {
    validate_column_ref(&condition.column).node_context(ctx)?;

    let expected = condition.operator.arity();
    let values = match (&condition.value, expected) {
        (None, Some(0)) => Vec::new(),
        (None, _) => {
            return Err(ctx.make_error(format!(
                "condition on \"{}\" requires a value",
                condition.column
            )))
        }
        (Some(_), Some(0)) => {
            return Err(ctx.make_error(format!(
                "condition on \"{}\" must not carry a value",
                condition.column
            )))
        }
        (Some(expression), arity) => {
            let value = isolate
                .run_standard(expression)
                .with_node_context(ctx, |_| {
                    format!(r#"Failed to evaluate condition: "{expression}""#)
                })?;

            match (arity, value) {
                // in / notIn: variadic, must be a list
                (None, Variable::Array(items)) => {
                    let items = items.borrow();
                    let mut values = Vec::with_capacity(items.len());
                    for item in items.iter() {
                        values.push(
                            coerce(item.shallow_clone(), condition.value_type).node_context(ctx)?,
                        );
                    }
                    values
                }
                (None, other) => {
                    return Err(ctx.make_error(format!(
                        "operator requires an array, got {}",
                        type_name(&other)
                    )))
                }
                (_, other) => vec![coerce(other, condition.value_type).node_context(ctx)?],
            }
        }
    };

    Ok(ResolvedCondition {
        column: condition.column.to_string(),
        operator: condition.operator,
        values,
    })
}

fn resolve_raw(
    ctx: &NodeContext<DatabaseNodeData, DatabaseNodeTrace>,
    isolate: &mut zen_expression::Isolate,
    raw: &RawQuery,
) -> Result<ResolvedRaw, crate::nodes::result::NodeError> {
    let mut parameters = Vec::with_capacity(raw.parameters.len());
    for parameter in raw.parameters.iter() {
        validate_identifier("parameter", &parameter.name).node_context(ctx)?;

        let value = isolate
            .run_standard(&parameter.value)
            .with_node_context(ctx, |_| {
                format!(r#"Failed to evaluate parameter: "{}""#, &parameter.value)
            })?;

        let values = match (parameter.expand, value) {
            (true, Variable::Array(items)) => {
                let items = items.borrow();
                let mut values = Vec::with_capacity(items.len());
                for item in items.iter() {
                    values.push(
                        coerce(item.shallow_clone(), parameter.value_type).node_context(ctx)?,
                    );
                }
                values
            }
            (true, other) => {
                return Err(ctx.make_error(format!(
                    "parameter \"{}\" is expandable and requires an array, got {}",
                    parameter.name,
                    type_name(&other)
                )))
            }
            (false, other) => vec![coerce(other, parameter.value_type).node_context(ctx)?],
        };

        parameters.push((parameter.name.to_string(), values));
    }

    Ok(ResolvedRaw {
        sql: raw.sql.to_string(),
        parameters,
    })
}
