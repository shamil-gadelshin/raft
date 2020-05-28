//! # Raft Node Test cases
//!
//! This subproject provides integration tests for the Raft algorithm.

#[macro_use]
extern crate log;
pub mod cases;
mod steps;

pub use self::cases::smoke;
