//! Type-erased error with chained sources, attachments, and diagnostics.

use crate::prelude::*;

/// Type-erased error with chained sources, attachments, and diagnostic rendering.
pub struct StructuredError {
    /// Boxed error context.
    pub(crate) context: Box<dyn StdError + Send + Sync + 'static>,
    /// Pre-captured diagnostic code from the original typed context.
    code: String,
    /// Key-value pairs of additional context.
    pub(crate) attached: Attached,
    /// Underlying error in the source chain.
    source: Option<Box<dyn StdError + Send + Sync + 'static>>,
}

impl StructuredError {
    /// Create from any error, capturing its diagnostic code before type erasure.
    pub fn new<T: StdError + Send + Sync + 'static>(context: T) -> Self {
        let code = short_code(&context);
        Self {
            context: Box::new(context),
            code,
            attached: Attached::new(),
            source: None,
        }
    }

    /// Wrap this error as the source of a new context.
    #[must_use]
    pub fn change_context<T: StdError + Send + Sync + 'static>(
        self,
        new_context: T,
    ) -> StructuredError {
        StructuredError {
            source: Some(Box::new(self)),
            ..StructuredError::new(new_context)
        }
    }

    /// Get the current context.
    #[must_use]
    #[expect(clippy::borrowed_box)]
    pub fn current_context(&self) -> &Box<dyn StdError + Send + Sync> {
        &self.context
    }
}

impl Display for StructuredError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.context, f)?;
        write!(f, "{}", self.attached)?;
        Ok(())
    }
}

impl Debug for StructuredError {
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

impl StdError for StructuredError {
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

impl Diagnostic for StructuredError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(self.code.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_error_display() {
        // Arrange
        let error = StructuredError::new(OuterError::Operation);
        // Act
        let display = error.to_string();
        // Assert
        assert_snapshot!(display, @"Outer operation failed");
    }

    #[test]
    fn structured_error_display__with_attach() {
        // Arrange
        let error = StructuredError::new(OuterError::Operation)
            .attach("path", "/tmp/file.txt")
            .attach("retries", 3);
        // Act
        let display = error.to_string();
        // Assert
        assert_snapshot!(display);
    }

    #[test]
    fn structured_error_debug__with_source() {
        // Arrange
        let inner = StructuredError::new(InnerError::Operation);
        let outer = inner.change_context(OuterError::Operation);
        // Act
        let debug = format!("{outer:?}");
        // Assert
        assert_snapshot!(debug);
    }

    #[test]
    fn structured_error_diagnostic_code() {
        // Arrange
        let error = StructuredError::new(OuterError::Operation);
        // Act
        let code = error.code().expect("should have code").to_string();
        // Assert
        assert_eq!(code, "studiole_report::OuterError::Operation");
    }

    #[test]
    fn structured_error_current_context() {
        // Arrange
        let error = StructuredError::new(OuterError::Operation);
        // Act
        let context = error.current_context();
        // Assert
        let downcast = context
            .downcast_ref::<OuterError>()
            .expect("should be OuterError");
        assert_eq!(downcast, &OuterError::Operation);
    }
}
