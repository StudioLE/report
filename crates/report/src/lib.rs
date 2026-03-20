//! Lightweight error report with diagnostic rendering.
#![cfg_attr(test, allow(non_snake_case))]
mod extensions;
mod helpers;
pub mod prelude;
#[cfg(feature = "render")]
mod render;
mod report;
