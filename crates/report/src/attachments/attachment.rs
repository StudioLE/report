//! Single key-value attachment for error context.
use crate::prelude::*;

const PATH_KEY: &str = "path";

/// Key-value pair of additional context on an error.
pub(crate) struct Attachment {
    /// Label describing the attached value.
    pub(crate) name: String,
    /// Stringified value.
    pub(crate) value: String,
}

impl Attachment {
    /// Create a new [`Attachment`] from a name and displayable value.
    pub(crate) fn new(name: impl Into<String>, value: impl Display) -> Self {
        Self {
            name: name.into(),
            value: value.to_string(),
        }
    }

    /// Create a new [`Attachment`] from a name and lazily evaluated value.
    pub(crate) fn with<D: Display>(name: impl Into<String>, value: impl FnOnce() -> D) -> Self {
        Self {
            name: name.into(),
            value: value().to_string(),
        }
    }

    /// Create a new [`Attachment`] for a path.
    pub(crate) fn path(path: impl AsRef<Path>) -> Self {
        Self {
            name: PATH_KEY.to_owned(),
            value: path.as_ref().to_string_lossy().to_string(),
        }
    }
}

impl Display for Attachment {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}: {}", self.name, self.value)
    }
}
