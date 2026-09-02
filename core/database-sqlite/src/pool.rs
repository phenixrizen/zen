//! Per-source database handles and connection reuse.
//!
//! Every request is a pure read — relations are rendered as `VALUES` CTEs rather than written
//! into temporary tables — so connections carry no per-evaluation state and can be handed out
//! and taken back with no cleanup. That makes reuse purely a performance concern rather than a
//! correctness one.

use ahash::HashMap;
use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use crate::config::SqliteConfig;
use crate::error::SqliteError;

pub(crate) struct Source {
    path: PathBuf,
    /// Connections are relatively expensive to create and hold no per-request state, so they are
    /// recycled. The lock is held only to pop or push a handle, never across a query.
    idle: Mutex<Vec<Connection>>,
    max_idle: usize,
}

pub(crate) struct Sources {
    /// Read-mostly: written once per distinct source, read on every request. A plain Mutex here
    /// serialized all queries and made throughput fall as threads were added.
    ///
    /// `Arc`, not a leaked `&'static`: a lease has to outlive the map borrow it was taken under,
    /// and an `Arc` clone is one atomic increment per request — nothing next to a query. The
    /// leak that used to buy the same thing kept every source, and every idle connection in it,
    /// alive for the life of the process: a host that builds engines with distinct roots leaked
    /// one per engine, and on Windows the still-open file could not be deleted after the engine
    /// was freed. Dropping the handler now drops the sources and closes their connections.
    sources: RwLock<HashMap<PathBuf, Arc<Source>>>,
    config: SqliteConfig,
}

/// Opens a connection read-only.
///
/// `SQLITE_OPEN_READ_ONLY` alone still permits writes to temporary storage; this driver never
/// needs any, and refusing them keeps a malformed request from mutating reference data.
fn open(path: &Path) -> Result<Connection, SqliteError> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    // Reference data is read concurrently and never written here, so the rollback journal and
    // synchronous writes are pure overhead.
    conn.pragma_update(None, "query_only", true)?;
    Ok(conn)
}

/// A connection borrowed from a source, returned on drop.
pub(crate) struct Lease {
    source: Arc<Source>,
    conn: Option<Connection>,
}

impl Lease {
    /// Takes the connection out of the lease so it can move into a blocking task.
    ///
    /// rusqlite is synchronous, so queries run on a blocking thread rather than the async
    /// executor. The connection is `Send` but not `Sync`, so it moves rather than being borrowed.
    pub fn take(&mut self) -> Connection {
        self.conn.take().expect("connection held for the lease")
    }

    /// Hands a connection back after a blocking task has finished with it.
    pub fn restore(&mut self, conn: Connection) {
        self.conn = Some(conn);
    }

    pub fn source(&self) -> Arc<Source> {
        Arc::clone(&self.source)
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

/// Returns a connection to a source's idle set, used when the lease itself has been consumed.
pub(crate) fn release(source: Arc<Source>, conn: Connection) {
    if let Ok(mut idle) = source.idle.lock() {
        if idle.len() < source.max_idle {
            idle.push(conn);
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

    fn source(&self, name: &str) -> Result<Arc<Source>, SqliteError> {
        let path = self.config.resolve(name)?;

        // Fast path: a shared read, which is what every request after the first takes.
        if let Ok(sources) = self.sources.read() {
            if let Some(source) = sources.get(&path) {
                return Ok(Arc::clone(source));
            }
        }

        if !path.exists() {
            return Err(SqliteError::UnknownSource(name.to_string()));
        }

        // Open once here so a bad path fails now rather than on first query.
        let probe = open(&path)?;

        let mut sources = self
            .sources
            .write()
            .map_err(|_| SqliteError::query("source registry is poisoned"))?;

        // Another thread may have won the race while this one was opening.
        if let Some(existing) = sources.get(&path) {
            return Ok(Arc::clone(existing));
        }

        // Sources live as long as the handler, which lives as long as the engine — and no longer.
        let source = Arc::new(Source {
            path: path.clone(),
            idle: Mutex::new(vec![probe]),
            max_idle: self.config.max_connections,
        });
        sources.insert(path, Arc::clone(&source));

        Ok(source)
    }

    /// Borrows a connection for one request.
    pub fn lease(&self, name: &str) -> Result<Lease, SqliteError> {
        let source = self.source(name)?;

        let pooled = source.idle.lock().ok().and_then(|mut idle| idle.pop());
        let conn = match pooled {
            Some(conn) => conn,
            None => open(&source.path)?,
        };

        Ok(Lease {
            source,
            conn: Some(conn),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The property the leak used to violate: once the handler is gone, so is every source and
    /// every connection it pooled. Observed through a `Weak`, which is exactly what a leaked
    /// `&'static` could never let you observe.
    #[test]
    fn dropping_the_sources_drops_the_source_and_its_connections() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("catalog.db");
        Connection::open(&path)
            .expect("create")
            .execute_batch("CREATE TABLE t (x TEXT); INSERT INTO t VALUES ('a');")
            .expect("fixture");

        let config = SqliteConfig::from_json(&format!(
            r#"{{"root": {:?}}}"#,
            dir.path().to_string_lossy()
        ))
        .expect("config");
        let sources = Sources::new(config);

        let weak = {
            let lease = sources.lease("catalog").expect("lease");
            Arc::downgrade(&lease.source())
        }; // lease dropped: its connection goes back to the idle set
        assert!(weak.upgrade().is_some(), "the map still owns the source");

        drop(sources);
        assert!(
            weak.upgrade().is_none(),
            "dropping the handler's sources must free the source and close its connections"
        );
        // and on every platform the file is now deletable
        std::fs::remove_file(&path).expect("file closed");
    }
}
