//! Shared error types and utilities for tests.
use crate::prelude::*;

/// Outer error context for testing error chains.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum OuterError {
    #[error("Outer operation failed")]
    Operation,
}

/// Inner error context for testing error chains.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum InnerError {
    #[error("Inner operation failed")]
    Operation,
}

/// Unit struct error for testing non-enum error types.
#[derive(Debug, Error)]
#[error("Unit struct error")]
pub struct UnitError;

const ANSI_ESCAPE: char = '\x1B';

/// Strip ANSI CSI escape sequences from a string.
///
/// Handles sequences of the form `ESC [ <params> <letter>` used for colors,
/// bold, reset, etc. Does not handle OSC sequences (`ESC ]`) or other
/// non-CSI escapes.
pub fn strip_ansi_codes(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == ANSI_ESCAPE {
            // Skip until we hit the terminating letter
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(c);
        }
    }
    output
}
