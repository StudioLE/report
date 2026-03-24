//! Trait for attaching additional context to errors and results.
use crate::prelude::*;

/// Attach additional context to an error or a result containing one.
pub trait Attach: Sized {
    /// Mutable access to the underlying [`Attached`], if present.
    fn as_attached_mut(&mut self) -> Option<&mut Attached>;

    /// Attach a name and value.
    #[must_use]
    fn attach(mut self, name: impl Into<String>, value: impl Display) -> Self {
        if let Some(attached) = self.as_attached_mut() {
            attached.push(Attachment::new(name, value));
        }
        self
    }

    /// Attach a name and lazily evaluated value.
    #[must_use]
    fn attach_with<D: Display>(
        mut self,
        name: impl Into<String>,
        value: impl FnOnce() -> D,
    ) -> Self {
        if let Some(attached) = self.as_attached_mut() {
            attached.push(Attachment::with(name, value));
        }
        self
    }

    /// Attach a [`Path`].
    #[must_use]
    fn attach_path(mut self, path: impl AsRef<Path>) -> Self {
        if let Some(attached) = self.as_attached_mut() {
            attached.push(Attachment::path(path));
        }
        self
    }
}

impl Attach for StructuredError {
    fn as_attached_mut(&mut self) -> Option<&mut Attached> {
        Some(&mut self.attached)
    }
}

impl<T: StdError + Send + Sync + 'static> Attach for Report<T> {
    fn as_attached_mut(&mut self) -> Option<&mut Attached> {
        Some(&mut self.inner.attached)
    }
}

impl<T, A: Attach> Attach for Result<T, A> {
    fn as_attached_mut(&mut self) -> Option<&mut Attached> {
        self.as_mut().err()?.as_attached_mut()
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
