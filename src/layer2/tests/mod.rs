//! # Layer 2 Tests - Comprehensive Testing Suite
//!
//! This module contains comprehensive tests for the Layer 2 planning system, including
//! unit tests, integration tests, and performance tests.

pub mod unit_tests;
pub mod integration_tests;
pub mod performance_tests;
pub mod test_utils;

pub use unit_tests::*;
pub use integration_tests::*;
pub use performance_tests::*;
pub use test_utils::*;