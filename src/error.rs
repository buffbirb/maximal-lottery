//! Error types for the crate.

use thiserror::Error;

use crate::profile::Candidate;

/// Errors that can occur when constructing ballots, profiles, margins, or
/// solving for a maximal lottery.
#[derive(Debug, Error)]
pub enum Error {
    /// A ballot declared contradictory preferences for the same unordered pair
    /// of candidates.
    #[error("ballot contains contradictory preferences for pair ({0}, {1})")]
    ContradictoryPair(Candidate, Candidate),

    /// A ranked ballot listed the same candidate more than once.
    #[error("ballot contains duplicate candidate {0}")]
    DuplicateCandidate(Candidate),

    /// A candidate index was outside the valid range `0..n`.
    #[error("candidate index {0} is out of range for {1} candidates")]
    InvalidCandidate(Candidate, usize),

    /// A ranked ballot did not list the expected number of candidates.
    #[error("ranking length {0} does not match candidate count {1}")]
    InvalidRankingLength(usize, usize),

    /// A profile contained ballots with differing candidate counts.
    #[error("ballots disagree on the number of candidates")]
    InconsistentCandidateCount,

    /// A margin matrix violated skew-symmetry or had a non-zero diagonal.
    #[error("margin matrix is not skew-symmetric")]
    InvalidMargins,

    /// An election had no candidates.
    #[error("no candidates provided")]
    NoCandidates,

    /// The maximal lottery program was infeasible.
    #[error("the maximal lottery is infeasible")]
    Infeasible,
}
