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
pub mod algfactor;
pub mod algext;
pub mod rsa_data;
pub mod auri_families;
pub mod groups;
pub mod groups_cmd;
pub mod factor_cmd;
pub mod factorlab;
pub mod labgroups;
pub mod labsearch;
pub mod lableak;
pub mod lab_cmd;
pub mod extra_cmd;
pub mod discover;
pub mod display;
pub mod kernel;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use eval::{Output, Session};
pub use expr::Expr;
