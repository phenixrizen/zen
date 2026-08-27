use thiserror::Error;

#[derive(Debug, Error)]
pub enum SqliteError {
    #[error("unknown data source \"{0}\"")]
    UnknownSource(String),

    #[error("invalid identifier \"{0}\"")]
    Identifier(String),

    #[error("{message}")]
    Query { message: String },

    #[error("raw queries are not enabled for this handler")]
    RawDisabled,

    #[error("sqlite: {0}")]
    Database(#[from] rusqlite::Error),
}

impl SqliteError {
    pub(crate) fn query(message: impl Into<String>) -> Self {
        Self::Query {
            message: message.into(),
        }
    }
}
