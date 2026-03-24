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
        self.map_err(|error| {
            let inner = StructuredError::new(error).change_context(context);
            Report::from_inner(inner)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

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

    #[test]
    fn result_ext_change_context__works_with_from() {
        // Arrange
        let expected = "broken";
        let result: Result<i32, io::Error> = Err(io::Error::other(expected));
        let report = result
            .change_context(OuterError::Operation)
            .expect_err("should be err");
        // Act
        let error = StructuredError::from(report);
        // Assert
        assert_eq!(error.to_string(), "Outer operation failed");
    }
}
