//! Owner: native hardening policy coverage grouped by responsibility.
#![allow(clippy::expect_used)]
use super::*;
use crate::{
    BenchmarkMatrixRow, BenchmarkMatrixSummary, BenchmarkReadiness, BenchmarkSurfaceMatch,
};
mod matrix;
use matrix::*;
mod admission;
mod comparisons;
mod corpora;
mod databases;
mod diagnostics;
mod evidence;
mod execution;
mod operations;
mod planning;
mod security;
