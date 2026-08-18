//! Per-source database handles.
//!
//! Every request is a pure read — relations are rendered as `VALUES` CTEs rather than written
//! into temporary tables — so connections carry no per-evaluation state and need no cleanup.
//! That removes the usual pooling hazard entirely: there is nothing for one evaluation to leave
//! behind for the next.

use ahash::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use turso::{Builder, Connection, Database};

use crate::config::SqliteConfig;
use crate::error::SqliteError;

pub(crate) struct Sources {
    opened: Mutex<HashMap<PathBuf, Database>>,
    config: SqliteConfig,
}

impl Sources {
    pub fn new(config: SqliteConfig) -> Self {
        Self {
            opened: Mutex::new(HashMap::default()),
            config,
        }
    }

    /// Resolves a logical source name to a connection.
    ///
    /// Databases are opened once and cached by resolved path, so two names pointing at the same
    /// file share a handle and two catalog versions do not.
    pub async fn connect(&self, name: &str) -> Result<Connection, SqliteError> {
        let path = self.config.resolve(name)?;

        if !path.exists() {
            return Err(SqliteError::UnknownSource(name.to_string()));
        }

        let cached = self
            .opened
            .lock()
            .map_err(|_| SqliteError::query("source registry is poisoned"))?
            .get(&path)
            .cloned();

        let database = match cached {
            Some(database) => database,
            None => {
                let opened = Builder::new_local(
                    path.to_str()
                        .ok_or_else(|| SqliteError::UnknownSource(name.to_string()))?,
                )
                .build()
                .await
                .map_err(SqliteError::from)?;

                let mut registry = self
                    .opened
                    .lock()
                    .map_err(|_| SqliteError::query("source registry is poisoned"))?;
                registry.entry(path).or_insert(opened).clone()
            }
        };

        database.connect().map_err(SqliteError::from)
    }
}
