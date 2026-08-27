//! Per-source database handles and connection reuse.
//!
//! Every request is a pure read — relations are rendered as `VALUES` CTEs rather than written
//! into temporary tables — so connections carry no per-evaluation state and can be handed out
//! and taken back with no cleanup. That makes reuse purely a performance concern rather than a
//! correctness one.

use ahash::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};
use turso::{Builder, Connection, Database};

use crate::config::SqliteConfig;
use crate::error::SqliteError;

struct Source {
    database: Database,
    /// Connections are relatively expensive to create and hold no per-request state, so they are
    /// recycled. The lock is held only to pop or push a handle, never across a query.
    idle: Mutex<Vec<Connection>>,
    max_idle: usize,
}

pub(crate) struct Sources {
    /// Read-mostly: written once per distinct source, read on every request. A plain Mutex here
    /// serialized all queries and made throughput fall as threads were added.
    sources: RwLock<HashMap<PathBuf, &'static Source>>,
    config: SqliteConfig,
}

/// A connection borrowed from a source, returned on drop.
pub(crate) struct Lease {
    source: &'static Source,
    conn: Option<Connection>,
}

impl Lease {
    pub fn conn(&self) -> &Connection {
        self.conn.as_ref().expect("connection held for the lease")
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        let Some(conn) = self.conn.take() else {
            return;
        };
        if let Ok(mut idle) = self.source.idle.lock() {
            if idle.len() < self.source.max_idle {
                idle.push(conn);
            }
        }
    }
}

impl Sources {
    pub fn new(config: SqliteConfig) -> Self {
        Self {
            sources: RwLock::new(HashMap::default()),
            config,
        }
    }

    async fn source(&self, name: &str) -> Result<&'static Source, SqliteError> {
        let path = self.config.resolve(name)?;

        // Fast path: a shared read, which is what every request after the first takes.
        if let Ok(sources) = self.sources.read() {
            if let Some(source) = sources.get(&path) {
                return Ok(*source);
            }
        }

        if !path.exists() {
            return Err(SqliteError::UnknownSource(name.to_string()));
        }

        let database = Builder::new_local(
            path.to_str()
                .ok_or_else(|| SqliteError::UnknownSource(name.to_string()))?,
        )
        .build()
        .await
        .map_err(SqliteError::from)?;

        let mut sources = self
            .sources
            .write()
            .map_err(|_| SqliteError::query("source registry is poisoned"))?;

        // Another thread may have won the race while this one was opening.
        if let Some(existing) = sources.get(&path) {
            return Ok(*existing);
        }

        // Sources live as long as the handler, which lives as long as the engine; leaking keeps
        // leases borrow-free without an Arc clone on every request.
        let source: &'static Source = Box::leak(Box::new(Source {
            database,
            idle: Mutex::new(Vec::new()),
            max_idle: self.config.max_connections,
        }));
        sources.insert(path, source);

        Ok(source)
    }

    /// Borrows a connection for one request.
    pub async fn lease(&self, name: &str) -> Result<Lease, SqliteError> {
        let source = self.source(name).await?;

        let pooled = source.idle.lock().ok().and_then(|mut idle| idle.pop());
        let conn = match pooled {
            Some(conn) => conn,
            None => source.database.connect().map_err(SqliteError::from)?,
        };

        Ok(Lease {
            source,
            conn: Some(conn),
        })
    }
}
