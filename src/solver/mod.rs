//! Maximal-lottery solvers.
//!
//! The [`LotterySolver`] trait is the extension point for algorithms that
//! compute a maximal lottery. This module provides [`CentroidSolver`], which
//! computes the exact centroid of all optimal strategies.

use crate::profile::{Lottery, MarginMatrix};

pub mod centroid;

pub use centroid::CentroidSolver;

/// A solver that computes a maximal lottery from a margin matrix.
///
/// Implementations are free to choose their numeric representation and
/// tie-breaking policy. For example, [`CentroidSolver`] computes the exact
/// centroid of all optimal strategies, while a floating-point LP solver
/// could return an approximate lottery.
pub trait LotterySolver {
    /// Error type returned when solving fails.
    type Error: std::error::Error + 'static;

    /// Compute a maximal lottery for the given margin matrix.
    fn solve(&self, margins: &MarginMatrix) -> Result<Lottery, Self::Error>;
}
