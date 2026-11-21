//! Galileo: a small computer algebra system.
#![allow(clippy::should_implement_trait, clippy::needless_range_loop, clippy::collapsible_if)]

pub mod build;
pub mod calc;
pub mod expr;
pub mod num;
pub mod print;
pub mod linalg;
pub mod eval;
pub mod help;
pub mod parser;

pub use eval::{Output, Session};
pub use expr::Expr;
