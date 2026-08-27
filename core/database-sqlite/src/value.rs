//! Conversion between the engine's transport values and Turso values.

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use turso::Value as TursoValue;
use zen_engine::nodes::database::DatabaseValue;

/// Binds an engine value as a SQL parameter.
///
/// Decimals bind as text rather than as a lossy `f64`: SQLite has no exact decimal type, and
/// reference data compares codes and dates as text, so text preserves equality and ordering for
/// the shapes this driver serves.
pub(crate) fn to_sql(value: &DatabaseValue) -> TursoValue {
    match value {
        DatabaseValue::Null => TursoValue::Null,
        DatabaseValue::Boolean(b) => TursoValue::Integer(i64::from(*b)),
        DatabaseValue::Integer(i) => TursoValue::Integer(*i),
        DatabaseValue::Decimal(d) => TursoValue::Text(d.to_string()),
        DatabaseValue::Text(s) => TursoValue::Text(s.clone()),
        DatabaseValue::Blob(b) => TursoValue::Blob(b.clone()),
    }
}

/// Reads a returned column back into an engine value.
pub(crate) fn from_sql(value: TursoValue) -> DatabaseValue {
    match value {
        TursoValue::Null => DatabaseValue::Null,
        TursoValue::Integer(i) => DatabaseValue::Integer(i),
        TursoValue::Real(f) => Decimal::from_f64(f)
            .map(DatabaseValue::Decimal)
            .unwrap_or(DatabaseValue::Null),
        TursoValue::Text(s) => DatabaseValue::Text(s),
        TursoValue::Blob(b) => DatabaseValue::Blob(b),
    }
}
