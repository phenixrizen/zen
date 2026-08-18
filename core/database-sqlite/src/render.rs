//! Renders a resolved query into SQL text plus an ordered parameter list.
//!
//! Nothing from the decision document or from evaluation input is ever interpolated into the
//! statement: identifiers are quoted after being re-validated here, and every value becomes a
//! bound parameter. This module is deliberately free of any driver types so the same rendering
//! can serve a different SQLite backend later.

use ahash::HashSet;
use zen_engine::nodes::database::{DatabaseValue, ResolvedRaw, ResolvedRelation, ResolvedSelect};
use zen_types::decision::{DatabaseOperator, JoinKind, OrderDirection};

use crate::error::SqliteError;

pub(crate) struct Rendered {
    pub sql: String,
    pub params: Vec<DatabaseValue>,
}

/// Quotes an identifier. The engine validates identifiers before the request reaches a driver;
/// this re-checks rather than trusting it, because a driver is a public extension point.
fn quote(identifier: &str) -> Result<String, SqliteError> {
    let valid = !identifier.is_empty()
        && identifier.len() <= 63
        && identifier
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && identifier
            .chars()
            .skip(1)
            .all(|c| c.is_ascii_alphanumeric() || c == '_');

    if !valid {
        return Err(SqliteError::Identifier(identifier.to_string()));
    }

    Ok(format!("\"{identifier}\""))
}

/// Quotes a possibly-qualified column reference (`col` or `table.col`).
fn quote_column(reference: &str, _relations: &HashSet<String>) -> Result<String, SqliteError> {
    match reference.split_once('.') {
        Some((qualifier, column)) => Ok(format!("{}.{}", quote(qualifier)?, quote(column)?)),
        None => quote(reference),
    }
}

fn quote_table(name: &str, _relations: &HashSet<String>) -> Result<String, SqliteError> {
    quote(name)
}

/// Renders relations as a read-only `WITH name(cols) AS (VALUES (...), (...))` prefix.
///
/// Materializing into a temporary table would be a write, which serializes concurrent
/// evaluations and leaves state on a pooled connection. A CTE keeps the whole request a pure
/// read: no locking, and no way for one evaluation to observe another\'s rows.
pub(crate) fn render_relations(
    relations: &[ResolvedRelation],
    params: &mut Vec<DatabaseValue>,
    variable_limit: usize,
) -> Result<(String, HashSet<String>), SqliteError> {
    let mut names: HashSet<String> = HashSet::default();
    if relations.is_empty() {
        return Ok((String::new(), names));
    }

    let mut clauses = Vec::with_capacity(relations.len());
    for relation in relations {
        if !names.insert(relation.name.clone()) {
            return Err(SqliteError::query(format!(
                "relation \"{}\" is declared more than once",
                relation.name
            )));
        }
        if relation.columns.is_empty() {
            return Err(SqliteError::query(format!(
                "relation \"{}\" declares no columns",
                relation.name
            )));
        }
        if relation.rows.is_empty() {
            return Err(SqliteError::query(format!(
                "relation \"{}\" has no rows; an empty VALUES list cannot be rendered",
                relation.name
            )));
        }

        let width = relation.columns.len();
        let needed = width * relation.rows.len();
        if params.len() + needed > variable_limit {
            return Err(SqliteError::query(format!(
                "relation \"{}\" needs {needed} bound values, which exceeds the parameter \
                 limit of {variable_limit}; narrow the set before the query",
                relation.name
            )));
        }

        let columns = relation
            .columns
            .iter()
            .map(|c| quote(&c.name))
            .collect::<Result<Vec<_>, _>>()?;

        let mut tuples = Vec::with_capacity(relation.rows.len());
        for row in &relation.rows {
            if row.len() != width {
                return Err(SqliteError::query(format!(
                    "relation \"{}\" row has {} values but {width} columns",
                    relation.name,
                    row.len()
                )));
            }
            params.extend(row.iter().cloned());
            tuples.push(format!("({})", vec!["?"; width].join(", ")));
        }

        clauses.push(format!(
            "{}({}) AS (VALUES {})",
            quote(&relation.name)?,
            columns.join(", "),
            tuples.join(", ")
        ));
    }

    Ok((format!("WITH {} ", clauses.join(", ")), names))
}

pub(crate) fn render_select(
    select: &ResolvedSelect,
    relations: &HashSet<String>,
    variable_limit: usize,
) -> Result<Rendered, SqliteError> {
    let mut params: Vec<DatabaseValue> = Vec::new();
    let mut sql = String::from("SELECT ");

    if select.distinct {
        sql.push_str("DISTINCT ");
    }

    if select.columns.is_empty() {
        sql.push('*');
    } else {
        let columns = select
            .columns
            .iter()
            .map(|c| quote_column(c, relations))
            .collect::<Result<Vec<_>, _>>()?;
        sql.push_str(&columns.join(", "));
    }

    sql.push_str(" FROM ");
    sql.push_str(&quote_table(&select.table, relations)?);

    for join in &select.joins {
        let keyword = match join.kind {
            JoinKind::Inner => "INNER JOIN",
            JoinKind::Left => "LEFT JOIN",
        };
        sql.push(' ');
        sql.push_str(keyword);
        sql.push(' ');
        sql.push_str(&quote_table(&join.table, relations)?);

        if join.on.is_empty() {
            return Err(SqliteError::query(format!(
                "join on \"{}\" has no conditions; an unconstrained join is refused",
                join.table
            )));
        }

        let predicates = join
            .on
            .iter()
            .map(|(left, right)| {
                Ok(format!(
                    "{} = {}",
                    quote_column(left, relations)?,
                    quote_column(right, relations)?
                ))
            })
            .collect::<Result<Vec<String>, SqliteError>>()?;
        sql.push_str(" ON ");
        sql.push_str(&predicates.join(" AND "));
    }

    if !select.conditions.is_empty() {
        let mut predicates = Vec::with_capacity(select.conditions.len());
        for condition in &select.conditions {
            let column = quote_column(&condition.column, relations)?;
            let predicate = render_condition(
                &column,
                condition.operator,
                &condition.values,
                &mut params,
                variable_limit,
            )?;
            predicates.push(predicate);
        }
        sql.push_str(" WHERE ");
        sql.push_str(&predicates.join(" AND "));
    }

    if !select.order_by.is_empty() {
        let orders = select
            .order_by
            .iter()
            .map(|order| {
                let direction = match order.direction {
                    OrderDirection::Asc => "ASC",
                    OrderDirection::Desc => "DESC",
                };
                Ok(format!(
                    "{} {direction}",
                    quote_column(&order.column, relations)?
                ))
            })
            .collect::<Result<Vec<String>, SqliteError>>()?;
        sql.push_str(" ORDER BY ");
        sql.push_str(&orders.join(", "));
    }

    if let Some(limit) = select.limit {
        sql.push_str(" LIMIT ?");
        params.push(DatabaseValue::Integer(i64::from(limit)));
    }

    Ok(Rendered { sql, params })
}

fn render_condition(
    column: &str,
    operator: DatabaseOperator,
    values: &[DatabaseValue],
    params: &mut Vec<DatabaseValue>,
    variable_limit: usize,
) -> Result<String, SqliteError> {
    let scalar = |op: &str, params: &mut Vec<DatabaseValue>| -> Result<String, SqliteError> {
        let [value] = values else {
            return Err(SqliteError::query(format!(
                "operator {op} on {column} expects exactly one value, got {}",
                values.len()
            )));
        };
        params.push(value.clone());
        Ok(format!("{column} {op} ?"))
    };

    let predicate = match operator {
        DatabaseOperator::Eq => scalar("=", params)?,
        DatabaseOperator::Ne => scalar("<>", params)?,
        DatabaseOperator::Lt => scalar("<", params)?,
        DatabaseOperator::Lte => scalar("<=", params)?,
        DatabaseOperator::Gt => scalar(">", params)?,
        DatabaseOperator::Gte => scalar(">=", params)?,
        DatabaseOperator::Like => scalar("LIKE", params)?,
        DatabaseOperator::IsNull => format!("{column} IS NULL"),
        DatabaseOperator::IsNotNull => format!("{column} IS NOT NULL"),
        DatabaseOperator::In | DatabaseOperator::NotIn => {
            let negated = matches!(operator, DatabaseOperator::NotIn);

            // `IN ()` is not valid SQL in any dialect, so the empty set is normalized to its
            // logical value: nothing is a member of the empty set, and everything is outside it.
            if values.is_empty() {
                return Ok(if negated { "1 = 1" } else { "0 = 1" }.to_string());
            }

            if values.len() > variable_limit {
                return Err(SqliteError::query(format!(
                    "{} values bound for {column} exceeds the SQLite parameter limit of \
                     {variable_limit}; materialize the list as a relation and join against it \
                     instead",
                    values.len()
                )));
            }

            let placeholders = vec!["?"; values.len()].join(", ");
            params.extend(values.iter().cloned());
            let keyword = if negated { "NOT IN" } else { "IN" };
            format!("{column} {keyword} ({placeholders})")
        }
    };

    Ok(predicate)
}

/// Rewrites named placeholders in a raw statement, expanding any parameter that carries several
/// values into one placeholder per value.
///
/// Only placeholder tokens are rewritten; the rest of the statement is passed through untouched.
pub(crate) fn render_raw(
    raw: &ResolvedRaw,
    variable_limit: usize,
) -> Result<Rendered, SqliteError> {
    let mut seen: HashSet<&str> = HashSet::default();
    for (name, _) in &raw.parameters {
        if !seen.insert(name.as_str()) {
            return Err(SqliteError::query(format!(
                "raw parameter \"{name}\" is declared more than once"
            )));
        }
    }

    let mut sql = String::with_capacity(raw.sql.len());
    let mut params: Vec<DatabaseValue> = Vec::new();
    let mut rest = raw.sql.as_str();

    while let Some(index) = rest.find(':') {
        let (before, after) = rest.split_at(index);
        sql.push_str(before);

        let name_len = after[1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .map(char::len_utf8)
            .sum::<usize>();

        if name_len == 0 {
            // A lone colon is not a placeholder; emit it and continue.
            sql.push(':');
            rest = &after[1..];
            continue;
        }

        let name = &after[1..1 + name_len];
        let Some((_, values)) = raw.parameters.iter().find(|(n, _)| n == name) else {
            return Err(SqliteError::query(format!(
                "statement references :{name}, which is not declared as a parameter"
            )));
        };

        if values.is_empty() {
            return Err(SqliteError::query(format!(
                "parameter :{name} expanded to no values; an empty list cannot be rendered"
            )));
        }
        if params.len() + values.len() > variable_limit {
            return Err(SqliteError::query(format!(
                "binding :{name} exceeds the SQLite parameter limit of {variable_limit}"
            )));
        }

        sql.push_str(&vec!["?"; values.len()].join(", "));
        params.extend(values.iter().cloned());
        rest = &after[1 + name_len..];
    }

    sql.push_str(rest);

    Ok(Rendered { sql, params })
}
