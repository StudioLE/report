//! Extension traits for converting and attaching context to [`Report`].
use crate::prelude::*;

/// Convert fallible results into [`Report`] by changing the error context.
pub trait ResultExt<T> {
    /// Wrap the error in a [`Report`] with the given context.
    fn change_context<C: StdError + Send + Sync + 'static>(
        self,
        context: C,
    ) -> Result<T, Report<C>>;
}

impl<T, E: StdError + Send + Sync + 'static> ResultExt<T> for Result<T, E> {
    fn change_context<C: StdError + Send + Sync + 'static>(
        self,
        context: C,
    ) -> Result<T, Report<C>> {
        self.map_err(|error| Report {
            context,
            attachments: Vec::new(),
            source: Some(Box::new(error)),
        })
    }
}

/// Attach additional context to a [`Report`] inside a `Result`.
pub trait ReportResultExt<T, E> {
    /// Add a key-value pair of additional context.
    #[must_use]
    fn attach(self, key: impl Into<String>, value: impl Into<String>) -> Self;
    /// Add a key-value pair of additional context with a lazily evaluated value.
    #[must_use]
    fn attach_with(self, key: impl Into<String>, value: impl FnOnce() -> String) -> Self;
}

impl<T, E: StdError + Send + Sync + 'static> ReportResultExt<T, E> for Result<T, Report<E>> {
    fn attach(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.map_err(|report| report.attach(key, value))
    }

    fn attach_with(self, key: impl Into<String>, value: impl FnOnce() -> String) -> Self {
        self.map_err(|report| report.attach_with(key, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn report_result_ext_attach() {
        // Arrange
        let input: Result<i32, Report<OuterError>> = Err(Report::new(OuterError::Operation));
        // Act
        let report = input.attach("file", "test.txt").expect_err("should be err");
        // Assert
        assert_eq!(report.attachments.len(), 1);
        let first = report.attachments.first().expect("should have attachment");
        assert_eq!(*first, ("file".to_owned(), "test.txt".to_owned()));
    }

    #[test]
    fn report_result_ext_attach_with() {
        // Arrange
        let input: Result<i32, Report<OuterError>> = Err(Report::new(OuterError::Operation));
        // Act
        let report = input
            .attach_with("count", || String::from("7"))
            .expect_err("should be err");
        // Assert
        assert_eq!(report.attachments.len(), 1);
        let first = report.attachments.first().expect("should have attachment");
        assert_eq!(*first, ("count".to_owned(), "7".to_owned()));
    }

    #[test]
    fn result_ext_change_context() {
        // Arrange
        let expected = "broken";
        let input: Result<i32, io::Error> = Err(io::Error::other(expected));
        // Act
        let report = input
            .change_context(OuterError::Operation)
            .expect_err("should be err");
        // Assert
        assert_eq!(*report.current_context(), OuterError::Operation);
        let source = report.source().expect("should have source");
        assert_eq!(source.to_string(), expected);
    }
}
