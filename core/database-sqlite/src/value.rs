//! Conversion between the engine's transport values and SQLite values.

use rusqlite::types::{Value as SqlValue, ValueRef};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use zen_engine::nodes::database::DatabaseValue;

/// Binds an engine value as a SQL parameter.
///
/// Decimals bind as text rather than as a lossy `f64`: SQLite has no exact decimal type, and
/// reference data compares codes and dates as text, so text preserves equality and ordering for
/// the shapes this driver serves.
pub(crate) fn to_sql(value: &DatabaseValue) -> SqlValue {
    match value {
        DatabaseValue::Null => SqlValue::Null,
        DatabaseValue::Boolean(b) => SqlValue::Integer(i64::from(*b)),
        DatabaseValue::Integer(i) => SqlValue::Integer(*i),
        DatabaseValue::Decimal(d) => SqlValue::Text(d.to_string()),
        DatabaseValue::Text(s) => SqlValue::Text(s.clone()),
        DatabaseValue::Blob(b) => SqlValue::Blob(b.clone()),
    }
}

/// Reads a returned column back into an engine value.
pub(crate) fn from_sql(value: ValueRef<'_>) -> DatabaseValue {
    match value {
        ValueRef::Null => DatabaseValue::Null,
        ValueRef::Integer(i) => DatabaseValue::Integer(i),
        ValueRef::Real(f) => Decimal::from_f64(f)
            .map(DatabaseValue::Decimal)
            .unwrap_or(DatabaseValue::Null),
        ValueRef::Text(bytes) => match std::str::from_utf8(bytes) {
            Ok(s) => DatabaseValue::Text(s.to_string()),
            // SQLite does not guarantee TEXT is valid UTF-8. Surfacing the bytes as a blob keeps
            // the value rather than substituting a lossy replacement string.
            Err(_) => DatabaseValue::Blob(bytes.to_vec()),
        },
        ValueRef::Blob(bytes) => DatabaseValue::Blob(bytes.to_vec()),
    }
}
