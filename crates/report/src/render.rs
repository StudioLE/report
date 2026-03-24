//! Render errors as rich diagnostic strings using miette.

use crate::prelude::*;
use miette::{GraphicalReportHandler, GraphicalTheme};
use std::env;

impl StructuredError {
    /// Render the error chain as a graphical diagnostic string.
    ///
    /// Uses unicode characters with ANSI colors.
    #[must_use]
    pub fn render(&self) -> String {
        let diagnostic: &dyn Diagnostic = self;
        let mut output = String::new();
        GraphicalReportHandler::new_themed(theme())
            .render_report(&mut output, diagnostic)
            .expect("should be able to render report");
        let trimmed = output.trim_end().len();
        output.truncate(trimmed);
        output
    }
}

fn theme() -> GraphicalTheme {
    if is_no_color() {
        GraphicalTheme::unicode_nocolor()
    } else {
        GraphicalTheme::unicode()
    }
}

fn is_no_color() -> bool {
    let Ok(value) = env::var("NO_COLOR") else {
        return false;
    };
    value != "0" && value != "false" && value != "no"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_render() {
        // Arrange
        let report = Report::new(OuterError::Operation);
        // Act
        let output = report.render();
        // Preview
        eprintln!("{output}");
        // Assert
        let output = strip_ansi_codes(&output);
        assert_snapshot!(output);
    }

    #[test]
    fn report_render__with_source_and_additional() {
        // Arrange
        let inner = Report::new(InnerError::Operation).attach("path", "/tmp/data.bin");
        let outer = inner
            .change_context(OuterError::Operation)
            .attach("retries", "3");
        // Act
        let output = outer.render();
        // Preview
        eprintln!("{output}");
        // Assert
        let output = strip_ansi_codes(&output);
        assert_snapshot!(output);
    }

    #[test]
    fn structured_error_render() {
        // Arrange
        let error = StructuredError::new(OuterError::Operation);
        // Act
        let output = error.render();
        // Preview
        eprintln!("{output}");
        // Assert
        let output = strip_ansi_codes(&output);
        assert_snapshot!(output, @r"
        studiole_report::OuterError::Operation

          × Outer operation failed
        ");
    }
}
