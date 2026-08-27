//! SQLite handler for the ZEN engine's `databaseNode`.
//!
//! Backed by SQLite itself, compiled from the bundled amalgamation so every platform runs an
//! identical, pinned version rather than whatever the host happens to provide.
//!
//! rusqlite is synchronous while `DatabaseHandler` is async, so each query runs on a blocking
//! thread. A local SQLite read is short, but blocking the executor for it would stall every other
//! evaluation sharing that thread.
//!
//! ```no_run
//! use std::sync::Arc;
//! use zen_engine::DecisionEngine;
//! use zen_database_sqlite::{SqliteConfig, SqliteDatabaseHandler};
//!
//! let handler = SqliteDatabaseHandler::new(SqliteConfig::with_root("/catalog"));
//! let engine = DecisionEngine::default().with_database_handler(Some(Arc::new(handler)));
//! ```
//!
//! Every request is a pure read. Graph-side data supplied as `relations` is rendered into the
//! statement as a `VALUES` common table expression rather than written to a temporary table, so
//! concurrent evaluations never contend on a write lock and no per-evaluation state is left on a
//! shared connection.

mod config;
mod error;
mod pool;
mod render;
mod value;

pub use config::SqliteConfig;
pub use error::SqliteError;

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use zen_engine::nodes::database::{
    DatabaseHandler, DatabaseRequest, DatabaseResponse, DatabaseValue, ResolvedQuery,
};

use crate::pool::Sources;

/// Conservative default, below SQLite's modern SQLITE_MAX_VARIABLE_NUMBER.
const VARIABLE_LIMIT: usize = 32_766;

#[derive(Debug)]
pub struct SqliteDatabaseHandler {
    sources: Sources,
    allow_raw: bool,
}

impl Debug for Sources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Sources")
    }
}

impl SqliteDatabaseHandler {
    pub fn new(config: SqliteConfig) -> Self {
        Self {
            allow_raw: config.allow_raw,
            sources: Sources::new(config),
        }
    }

    async fn run(&self, request: DatabaseRequest) -> Result<DatabaseResponse, SqliteError> {
        let mut lease = self.sources.lease(&request.source)?;

        let mut params: Vec<DatabaseValue> = Vec::new();
        let (prefix, relation_names) =
            render::render_relations(&request.relations, &mut params, VARIABLE_LIMIT)?;

        let rendered = match &request.query {
            ResolvedQuery::Select(select) => {
                render::render_select(select, &relation_names, VARIABLE_LIMIT)?
            }
            ResolvedQuery::Raw(raw) => {
                if !self.allow_raw {
                    return Err(SqliteError::RawDisabled);
                }
                render::render_raw(raw, VARIABLE_LIMIT)?
            }
        };

        // Relation bindings come first because the CTE precedes the statement they feed.
        params.extend(rendered.params);
        let sql = format!("{prefix}{}", rendered.sql);

        let bound: Vec<rusqlite::types::Value> = params.iter().map(value::to_sql).collect();
        let max_rows = request.max_rows;

        let conn = lease.take();
        let source = lease.source();

        // rusqlite is synchronous. Running it on a blocking thread keeps the async executor free,
        // and the connection moves with it because it is Send but not Sync.
        let outcome = tokio::task::spawn_blocking(move || {
            let result = execute(&conn, &sql, bound, max_rows);
            (conn, result)
        })
        .await;

        match outcome {
            Ok((conn, result)) => {
                pool::release(source, conn);
                result
            }
            // The connection was lost with the panicking task, so there is nothing to return to
            // the pool; the next request opens a fresh one.
            Err(join) => Err(SqliteError::query(format!("query task failed: {join}"))),
        }
    }
}

impl DatabaseHandler for SqliteDatabaseHandler {
    fn query(
        &self,
        request: DatabaseRequest,
    ) -> Pin<Box<dyn Future<Output = Result<DatabaseResponse, String>> + Send + '_>> {
        Box::pin(async move { self.run(request).await.map_err(|err| err.to_string()) })
    }
}

/// Runs one prepared statement to completion. Synchronous: called from a blocking thread.
fn execute(
    conn: &rusqlite::Connection,
    sql: &str,
    bound: Vec<rusqlite::types::Value>,
    max_rows: u32,
) -> Result<DatabaseResponse, SqliteError> {
    let mut statement = conn.prepare(sql)?;

    // Column names come from the prepared statement so they are known even when the query
    // matches nothing - a zero-row result still has a shape.
    let columns: Vec<String> = statement
        .column_names()
        .into_iter()
        .map(str::to_string)
        .collect();

    let mut rows = statement.query(rusqlite::params_from_iter(bound))?;

    let mut collected: Vec<Vec<DatabaseValue>> = Vec::new();
    let mut truncated = false;

    while let Some(row) = rows.next()? {
        if collected.len() as u32 >= max_rows {
            truncated = true;
            break;
        }

        let mut values = Vec::with_capacity(columns.len());
        for index in 0..columns.len() {
            values.push(value::from_sql(row.get_ref(index)?));
        }
        collected.push(values);
    }

    Ok(DatabaseResponse {
        columns,
        rows: collected,
        truncated,
    })
}
