//! How logical source names map to database files.

use ahash::HashMap;
use std::path::{Path, PathBuf};

use crate::error::SqliteError;

#[derive(Debug, Clone)]
pub struct SqliteConfig {
    sources: SourceResolution,
    /// Maximum connections held per source. Defaults to the machine's parallelism.
    pub max_connections: usize,
    /// Whether `raw` queries are permitted. Off by default: a raw statement is opaque to static
    /// analysis, so enabling it is a deliberate choice.
    pub allow_raw: bool,
    /// Per-connection page cache, in kibibytes (negative `cache_size` in SQLite terms).
    pub cache_size_kib: u32,
    /// Memory-mapped I/O size in bytes. Zero disables it.
    pub mmap_size: u64,
    /// Busy timeout in milliseconds.
    pub busy_timeout_ms: u32,
}

#[derive(Debug, Clone)]
enum SourceResolution {
    /// Every source named explicitly. Nothing else is reachable.
    Explicit(HashMap<String, PathBuf>),
    /// `<root>/<name>.db`. Names are validated, so a name can never escape the root.
    Root(PathBuf),
}

impl SqliteConfig {
    /// Resolves `<root>/<name>.db`.
    pub fn with_root(root: impl AsRef<Path>) -> Self {
        Self::new(SourceResolution::Root(root.as_ref().to_path_buf()))
    }

    /// Resolves only the names given. Preferred when the set of databases is known.
    pub fn with_sources<I, K, V>(sources: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<PathBuf>,
    {
        let map = sources
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Self::new(SourceResolution::Explicit(map))
    }

    fn new(sources: SourceResolution) -> Self {
        Self {
            sources,
            max_connections: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            allow_raw: false,
            cache_size_kib: 65_536,
            mmap_size: 268_435_456,
            busy_timeout_ms: 5_000,
        }
    }

    pub fn allow_raw(mut self, allow: bool) -> Self {
        self.allow_raw = allow;
        self
    }

    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = max.max(1);
        self
    }

    /// Maps a logical source name to a file.
    ///
    /// Names are restricted to `[A-Za-z_][A-Za-z0-9_]*`, so a name cannot contain a path
    /// separator, a parent-directory segment, or a URI parameter.
    pub(crate) fn resolve(&self, name: &str) -> Result<PathBuf, SqliteError> {
        let safe = !name.is_empty()
            && name.len() <= 63
            && name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && name
                .chars()
                .skip(1)
                .all(|c| c.is_ascii_alphanumeric() || c == '_');

        if !safe {
            return Err(SqliteError::Identifier(name.to_string()));
        }

        match &self.sources {
            SourceResolution::Explicit(map) => map
                .get(name)
                .cloned()
                .ok_or_else(|| SqliteError::UnknownSource(name.to_string())),
            SourceResolution::Root(root) => Ok(root.join(format!("{name}.db"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_cannot_escape_the_root() {
        let config = SqliteConfig::with_root("/catalog");
        for bad in ["../etc/passwd", "a/b", "a.db", "", "a b", "file:x?mode=rw"] {
            assert!(
                config.resolve(bad).is_err(),
                "source name {bad:?} should be rejected"
            );
        }
        assert_eq!(
            config.resolve("fees_short").expect("valid name"),
            PathBuf::from("/catalog/fees_short.db")
        );
    }

    #[test]
    fn explicit_sources_reject_unknown_names() {
        let config = SqliteConfig::with_sources([("catalog", "/tmp/catalog.db")]);
        assert!(config.resolve("catalog").is_ok());
        assert!(config.resolve("other").is_err());
    }
}
