//! Ballot types and the [`Ballot`] trait.
//!
//! This crate provides [`PairwiseBallot`] for explicit pairwise preferences
//! and [`RankedBallot`] for ranked ballots. Users may also implement the
//! [`Ballot`] trait for custom ballot types.

pub mod pairwise;
pub mod ranked;

pub use pairwise::PairwiseBallot;
pub use ranked::RankedBallot;

use crate::profile::Candidate;

/// A voter's pairwise preference between two candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PairPreference {
    /// No preference expressed for this pair.
    Abstain = 0b00,
    /// The first candidate in the ordered pair is preferred.
    Left = 0b01,
    /// The second candidate in the ordered pair is preferred.
    Right = 0b10,
}

/// A ballot that can be queried for pairwise preferences.
///
/// Implementations define both the number of candidates the ballot refers to
/// and the voter's preference for any ordered pair. Users may implement this
/// trait for custom ballot types; the library only requires that the returned
/// preferences be consistent with the ballot's own semantics.
pub trait Ballot {
    /// Number of candidates this ballot refers to.
    fn n_candidates(&self) -> usize;

    /// The voter's preference for the ordered pair `(i, j)`.
    ///
    /// Returns [`Left`](PairPreference::Left) if `i` is preferred over `j`,
    /// [`Right`](PairPreference::Right) if `j` is preferred over `i`, and
    /// [`Abstain`](PairPreference::Abstain) if no preference is expressed.
    fn preference(&self, i: Candidate, j: Candidate) -> PairPreference;
}
