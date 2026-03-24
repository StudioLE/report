//! Typed error report wrapping [`StructuredError`].
use crate::prelude::*;
use std::marker::PhantomData;
use std::ops::Deref;

/// Typed error report that wraps a [`StructuredError`] with compile-time type information.
pub struct Report<T> {
    /// The underlying type-erased error.
    pub(crate) inner: StructuredError,
    /// Marker for the typed context.
    _marker: PhantomData<T>,
}

impl<T: StdError + Send + Sync + 'static> Report<T> {
    /// Create a report from the given error context with no source.
    pub fn new(context: T) -> Self {
        Self {
            inner: StructuredError::new(context),
            _marker: PhantomData,
        }
    }

    /// Create a report from a pre-built [`StructuredError`].
    pub(crate) fn from_inner(inner: StructuredError) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    /// Get the current context.
    #[must_use]
    pub fn current_context(&self) -> &T {
        self.inner
            .context
            .downcast_ref::<T>()
            .expect("Report<T> inner context should be of type T")
    }

    /// Wrap this report as the source of a new context.
    pub fn change_context<U: StdError + Send + Sync + 'static>(self, new_context: U) -> Report<U> {
        Report {
            inner: self.inner.change_context(new_context),
            _marker: PhantomData,
        }
    }
}

impl<T: StdError + Send + Sync + 'static> From<Report<T>> for StructuredError {
    fn from(report: Report<T>) -> Self {
        report.inner
    }
}

impl<T: StdError + Send + Sync + 'static> Deref for Report<T> {
    type Target = StructuredError;

    fn deref(&self) -> &StructuredError {
        &self.inner
    }
}

impl<T: StdError + Send + Sync + 'static> From<T> for Report<T> {
    fn from(error: T) -> Self {
        Self::new(error)
    }
}

impl<T: StdError + Send + Sync + 'static> Debug for Report<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.inner, f)
    }
}

impl<T: StdError + Send + Sync + 'static> Display for Report<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.inner, f)
    }
}

impl<T: StdError + Send + Sync + 'static> StdError for Report<T> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.inner.source()
    }
}

impl<T: StdError + Send + Sync + 'static> Diagnostic for Report<T> {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.inner.code()
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
            .attach("retries", 3);
        // Act
        let display = report.to_string();
        // Assert
        assert_snapshot!(display);
    }

    #[test]
    fn report_display__with_attach_with() {
        // Arrange
        let report = Report::new(OuterError::Operation).attach_with("count", || 42);
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
        assert_eq!(outer.inner.attached.len(), 0);
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

    #[test]
    fn structured_error_from() {
        // Arrange
        let report = Report::new(OuterError::Operation).attach("key", "value");
        // Act
        let error = StructuredError::from(report);
        // Assert
        assert_eq!(error.to_string(), "Outer operation failed\n▷ key: value");
    }
}
