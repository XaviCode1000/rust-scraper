//! CLI module — argument parsing, error handling, completions, config.
//!
//! Clean Architecture Adapters layer: all CLI-related utilities.

pub mod args;
pub mod completions;
pub mod config;
pub mod error;
pub mod summary;
pub mod wizard;

pub use args::{Args, Commands, Shell};
