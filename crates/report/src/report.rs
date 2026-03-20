//! Error report with diagnostic rendering.
use crate::prelude::*;

/// Error report that wraps a typed context with an optional source chain.
pub struct Report<T> {
    /// Typed error context.
    pub(crate) context: T,
    /// Key-value pairs of additional context.
    pub(crate) attachments: Vec<(String, String)>,
    /// Underlying error in the source chain.
    pub(crate) source: Option<Box<dyn StdError + Send + Sync>>,
}

impl<T: StdError + Send + Sync + 'static> Report<T> {
    /// Create a report from the given error context with no source.
    pub fn new(context: T) -> Self {
        Self {
            context,
            attachments: Vec::new(),
            source: None,
        }
    }

    /// The typed context stored in this report.
    pub fn current_context(&self) -> &T {
        &self.context
    }

    /// Wrap this report as the source of a new context.
    pub fn change_context<U: StdError + Send + Sync + 'static>(self, new_context: U) -> Report<U> {
        Report {
            context: new_context,
            attachments: Vec::new(),
            source: Some(Box::new(self)),
        }
    }

    /// Add a key-value pair of additional context.
    #[must_use]
    pub fn attach(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attachments.push((key.into(), value.into()));
        self
    }

    /// Add a path as additional context under the key `"path"`.
    #[must_use]
    pub fn attach_path(mut self, value: impl AsRef<Path>) -> Self {
        self.attachments.push((
            "path".to_owned(),
            value.as_ref().to_string_lossy().to_string(),
        ));
        self
    }

    /// Add a key-value pair of additional context with a lazily evaluated value.
    #[must_use]
    pub fn attach_with(mut self, key: impl Into<String>, value: impl FnOnce() -> String) -> Self {
        self.attachments.push((key.into(), value()));
        self
    }
}

impl<T: StdError + Send + Sync + 'static> From<T> for Report<T> {
    fn from(error: T) -> Self {
        Self::new(error)
    }
}

impl<T: StdError + Send + Sync + 'static> Debug for Report<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(self, f)?;
        let mut source = self.source();
        while let Some(err) = source {
            write!(f, "\n  Caused by: {err}")?;
            source = err.source();
        }
        Ok(())
    }
}

impl<T: Display> Display for Report<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.context, f)?;
        for (key, value) in &self.attachments {
            write!(f, "\n▷ {key}: {value}")?;
        }
        Ok(())
    }
}

impl<T: StdError + Send + Sync + 'static> StdError for Report<T> {
    #[expect(
        clippy::as_conversions,
        reason = "cast from boxed trait object to trait reference"
    )]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|s| s.as_ref() as &(dyn StdError + 'static))
    }
}

impl<T: StdError + Send + Sync + 'static> Diagnostic for Report<T> {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(short_code(&self.context)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_display() {
        // Arrange
        let report = Report::new(OuterError::Operation);
        // Act
        let display = report.to_string();
        // Assert
        assert_snapshot!(display, @"Outer operation failed");
    }

    #[test]
    fn report_display__with_attach() {
        // Arrange
        let report = Report::new(OuterError::Operation)
            .attach("path", "/tmp/file.txt")
            .attach("retries", "3");
        // Act
        let display = report.to_string();
        // Assert
        assert_snapshot!(display);
    }

    #[test]
    fn report_display__with_attach_with() {
        // Arrange
        let report = Report::new(OuterError::Operation).attach_with("count", || String::from("42"));
        // Act
        let display = report.to_string();
        // Assert
        assert_snapshot!(display);
    }

    #[test]
    fn report_debug() {
        // Arrange
        let report = Report::new(OuterError::Operation);
        // Act
        let debug = format!("{report:?}");
        // Assert
        assert_snapshot!(debug, @"Outer operation failed");
    }

    #[test]
    fn report_debug__with_source() {
        // Arrange
        let inner = Report::new(InnerError::Operation);
        let outer = inner.change_context(OuterError::Operation);
        // Act
        let debug = format!("{outer:?}");
        // Assert
        assert_snapshot!(debug);
    }

    #[test]
    fn report_debug__with_additional_and_source() {
        // Arrange
        let inner = Report::new(InnerError::Operation).attach("key", "inner_val");
        let outer = inner
            .change_context(OuterError::Operation)
            .attach("key", "outer_val");
        // Act
        let debug = format!("{outer:?}");
        // Assert
        assert_snapshot!(debug);
    }

    #[test]
    fn report_source__none_without_wrapping() {
        // Arrange
        let report = Report::new(OuterError::Operation);
        // Act
        let source = report.source();
        // Assert
        assert!(source.is_none());
    }

    #[test]
    fn report_source__set_after_change_context() {
        // Arrange
        let inner = Report::new(InnerError::Operation);
        let outer = inner.change_context(OuterError::Operation);
        // Act
        let source = outer.source().expect("should have source");
        // Assert
        assert_eq!(source.to_string(), "Inner operation failed");
    }

    #[test]
    fn report_change_context__preserves_context() {
        // Arrange
        let inner = Report::new(InnerError::Operation);
        // Act
        let outer = inner.change_context(OuterError::Operation);
        // Assert
        assert_eq!(*outer.current_context(), OuterError::Operation);
    }

    #[test]
    fn report_change_context__clears_additional() {
        // Arrange
        let inner = Report::new(InnerError::Operation).attach("key", "value");
        // Act
        let outer = inner.change_context(OuterError::Operation);
        // Assert
        assert!(outer.attachments.is_empty());
    }

    #[test]
    fn report_attach_path() {
        // Arrange
        let report = Report::new(OuterError::Operation).attach_path("/tmp/data.bin");
        // Act
        let display = report.to_string();
        // Assert
        assert_snapshot!(display);
    }

    #[test]
    fn report_from__converts_error_via_question_mark() {
        // Arrange
        fn fallible() -> Result<(), OuterError> {
            Err(OuterError::Operation)
        }
        fn wrapper() -> Result<(), Report<OuterError>> {
            fallible()?;
            Ok(())
        }
        // Act
        let report = wrapper().expect_err("should be err");
        // Assert
        assert_eq!(*report.current_context(), OuterError::Operation);
        assert!(report.source().is_none());
    }
}
