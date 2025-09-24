pub use fundsp::hacker32::*;

// The main eval module is always available since crossbeam-channel is a regular dependency
pub mod eval;

// Re-export the parser type and useful functions
pub use eval::Lapis as LapisParser;
pub use eval::{eval_net, eval_stmt};