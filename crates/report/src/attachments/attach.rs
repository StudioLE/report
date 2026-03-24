//! Trait for attaching additional context to errors and results.
use crate::prelude::*;

/// Attach additional context to an error or a result containing one.
pub trait Attach: Sized {
    /// Attach a name and value.
    #[must_use]
    fn attach(self, name: impl Into<String>, value: impl Display) -> Self;

    /// Attach a name and lazily evaluated value.
    #[must_use]
    fn attach_with<D: Display>(self, name: impl Into<String>, value: impl FnOnce() -> D) -> Self;

    /// Attach a [`Path`].
    #[must_use]
    fn attach_path(self, path: impl AsRef<Path>) -> Self;
}

impl Attach for StructuredError {
    fn attach(mut self, name: impl Into<String>, value: impl Display) -> Self {
        self.attached.push(Attachment::new(name, value));
        self
    }
    fn attach_with<D: Display>(
        mut self,
        name: impl Into<String>,
        value: impl FnOnce() -> D,
    ) -> Self {
        self.attached.push(Attachment::with(name, value));
        self
    }
    fn attach_path(mut self, path: impl AsRef<Path>) -> Self {
        self.attached.push(Attachment::path(path));
        self
    }
}

impl<T: StdError + Send + Sync + 'static> Attach for Report<T> {
    fn attach(mut self, name: impl Into<String>, value: impl Display) -> Self {
        self.inner.attached.push(Attachment::new(name, value));
        self
    }
    fn attach_with<D: Display>(
        mut self,
        name: impl Into<String>,
        value: impl FnOnce() -> D,
    ) -> Self {
        self.inner.attached.push(Attachment::with(name, value));
        self
    }
    fn attach_path(mut self, path: impl AsRef<Path>) -> Self {
        self.inner.attached.push(Attachment::path(path));
        self
    }
}

impl<T, A: Attach> Attach for Result<T, A> {
    fn attach(self, name: impl Into<String>, value: impl Display) -> Self {
        self.map_err(|e| e.attach(name, value))
    }
    fn attach_with<D: Display>(self, name: impl Into<String>, value: impl FnOnce() -> D) -> Self {
        self.map_err(|e| e.attach_with(name, value))
    }
    fn attach_path(self, path: impl AsRef<Path>) -> Self {
        self.map_err(|e| e.attach_path(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_error_attach() {
        // Arrange
        let error = StructuredError::new(OuterError::Operation);
        // Act
        let error = error.attach("key", "value");
        // Assert
        assert_eq!(error.attached.len(), 1);
    }

    #[test]
    fn report_attach() {
        // Arrange
        let report = Report::new(OuterError::Operation);
        // Act
        let error = report.attach("key", "value");
        // Assert
        assert_eq!(error.attached.len(), 1);
    }

    #[test]
    fn result_attach__on_err() {
        // Arrange
        let result: Result<i32, Report<OuterError>> = Err(Report::new(OuterError::Operation));
        // Act
        let report = result.attach("key", "value");
        // Assert
        assert_eq!(report.expect_err("should be err").attached.len(), 1);
    }

    #[test]
    fn result_attach__on_ok() {
        // Arrange
        let result: Result<i32, Report<OuterError>> = Ok(42);
        // Act
        let result = result.attach("key", "value");
        // Assert
        assert_eq!(result.expect("should be ok"), 42);
    }

    #[test]
    fn result_attach__attach_path() {
        // Arrange
        let result: Result<i32, Report<OuterError>> = Err(Report::new(OuterError::Operation));
        // Act
        let result = result.attach_path("/tmp/data.bin");
        // Assert
        assert_eq!(result.expect_err("should be err").attached.len(), 1);
    }

    #[test]
    fn result_attach__attach_with() {
        // Arrange
        let result: Result<i32, Report<OuterError>> = Err(Report::new(OuterError::Operation));
        // Act
        let result = result.attach_with("count", || 7);
        // Assert
        assert_eq!(result.expect_err("should be err").attached.len(), 1);
    }

    #[test]
    fn result_attach__structured_error() {
        // Arrange
        let result: Result<i32, StructuredError> = Err(StructuredError::new(OuterError::Operation));
        // Act
        let result = result.attach("file", "test.txt");
        // Assert
        assert_eq!(result.expect_err("should be err").attached.len(), 1);
    }
}
