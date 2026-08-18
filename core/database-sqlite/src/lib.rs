//! Pure-Rust SQLite handler for the ZEN engine's `databaseNode`.
//!
//! Backed by [Turso](https://github.com/tursodatabase/turso), a Rust reimplementation of SQLite,
//! so there is no C toolchain, no vendored amalgamation, and no `unsafe` FFI in the build.
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

/// Conservative default; Turso follows SQLite's modern parameter ceiling.
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
        let conn = self.sources.connect(&request.source).await?;

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

        let bound: Vec<turso::Value> = params.iter().map(value::to_sql).collect();
        let mut statement = conn.prepare(&sql).await.map_err(SqliteError::from)?;

        // Column names come from the prepared statement so they are known even when the query
        // matches nothing - a zero-row result still has a shape.
        let columns: Vec<String> = statement
            .columns()
            .iter()
            .map(|column| column.name().to_string())
            .collect();

        let mut rows = statement
            .query(turso::params_from_iter(bound))
            .await
            .map_err(SqliteError::from)?;

        let mut collected: Vec<Vec<DatabaseValue>> = Vec::new();
        let mut truncated = false;

        while let Some(row) = rows.next().await.map_err(SqliteError::from)? {
            if collected.len() as u32 >= request.max_rows {
                truncated = true;
                break;
            }

            let mut values = Vec::with_capacity(columns.len());
            for index in 0..columns.len() {
                let raw = row.get_value(index).map_err(SqliteError::from)?;
                values.push(value::from_sql(raw));
            }
            collected.push(values);
        }

        Ok(DatabaseResponse {
            columns,
            rows: collected,
            truncated,
        })
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
