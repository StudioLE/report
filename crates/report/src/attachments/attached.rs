//! Collection type for storing multiple [`Attachment`] on an error.
use crate::prelude::*;

/// Collection of [`Attachment`] on an error.
pub(crate) struct Attached {
    inner: Vec<Attachment>,
}

impl Attached {
    /// Create an empty collection.
    pub(crate) fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Append an [`Attachment`] to the collection.
    pub(crate) fn push(&mut self, attachment: Attachment) {
        self.inner.push(attachment);
    }

    /// Number of attachments in the collection.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.inner.len()
    }
}

impl Display for Attached {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for attachment in &self.inner {
            write!(f, "\n▷ {attachment}")?;
        }
        Ok(())
    }
}
