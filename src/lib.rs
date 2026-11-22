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
pub mod ratfun;
pub mod latex;
pub mod numeric;
pub mod plot;
pub mod geometry;
pub mod ode;
pub mod commands;
pub mod cas;
pub mod cas_cmd;
pub mod numtheory;
pub mod extra_cmd;
pub mod discover;
pub mod display;
pub mod kernel;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use eval::{Output, Session};
pub use expr::Expr;
