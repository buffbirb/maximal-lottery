//! Ranked ballot implementation.

use crate::ballot::{Ballot, PairPreference};
use crate::error::Error;
use crate::profile::Candidate;

/// A ballot that ranks candidates from most-preferred to least-preferred.
///
/// Missing candidates in a partial ranking are treated as abstentions relative
/// to all other candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedBallot {
    n: usize,
    ranking: Vec<Candidate>,
    unranked: Vec<Candidate>,
}

impl RankedBallot {
    /// Create a ballot that ranks all `n` candidates.
    ///
    /// The `ranking` slice must contain each candidate index `0..n` exactly
    /// once.
    ///
    /// # Errors
    ///
    /// Returns an error if the ranking length does not equal `n`, if any
    /// candidate index is out of range, or if a candidate appears more than
    /// once.
    pub fn total(ranking: &[Candidate], n: usize) -> Result<Self, Error> {
        if ranking.len() != n {
            return Err(Error::InvalidRankingLength(ranking.len(), n));
        }
        Self::partial(ranking, n)
    }

    /// Create a ballot that ranks a subset of the candidates.
    ///
    /// Missing candidates are treated as abstentions relative to all other
    /// candidates.
    ///
    /// # Errors
    ///
    /// Returns an error if any candidate index is out of range or if a
    /// candidate appears more than once.
    pub fn partial(ranking: &[Candidate], n: usize) -> Result<Self, Error> {
        let mut seen = vec![false; n];
        for &c in ranking {
            if c.0 >= n {
                return Err(Error::InvalidCandidate(c, n));
            }
            if seen[c.0] {
                return Err(Error::DuplicateCandidate(c));
            }
            seen[c.0] = true;
        }
        let unranked = seen
            .into_iter()
            .enumerate()
            .filter_map(|(c, present)| if present { None } else { Some(Candidate(c)) })
            .collect();
        Ok(Self {
            n,
            ranking: ranking.to_vec(),
            unranked,
        })
    }

    /// Candidates in rank order.
    pub fn ranking(&self) -> &[Candidate] {
        &self.ranking
    }

    /// Candidates that were omitted from the ranking.
    pub fn unranked(&self) -> &[Candidate] {
        &self.unranked
    }
}

impl Ballot for RankedBallot {
    fn n_candidates(&self) -> usize {
        self.n
    }

    fn preference(&self, i: Candidate, j: Candidate) -> PairPreference {
        let pos_i = self.ranking.iter().position(|&c| c == i);
        let pos_j = self.ranking.iter().position(|&c| c == j);
        match (pos_i, pos_j) {
            (Some(pi), Some(pj)) if pi < pj => PairPreference::Left,
            (Some(pi), Some(pj)) if pi > pj => PairPreference::Right,
            _ => PairPreference::Abstain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_ranking() {
        let ballot = RankedBallot::total(&[Candidate(2), Candidate(0), Candidate(1)], 3).unwrap();
        assert_eq!(
            ballot.preference(Candidate(2), Candidate(0)),
            PairPreference::Left
        );
        assert_eq!(
            ballot.preference(Candidate(0), Candidate(1)),
            PairPreference::Left
        );
        assert_eq!(
            ballot.preference(Candidate(2), Candidate(1)),
            PairPreference::Left
        );
        assert_eq!(
            ballot.preference(Candidate(1), Candidate(2)),
            PairPreference::Right
        );
    }

    #[test]
    fn test_partial_ranking_abstains() {
        let ballot = RankedBallot::partial(&[Candidate(0), Candidate(1)], 3).unwrap();
        assert_eq!(
            ballot.preference(Candidate(0), Candidate(1)),
            PairPreference::Left
        );
        assert_eq!(
            ballot.preference(Candidate(0), Candidate(2)),
            PairPreference::Abstain
        );
        assert_eq!(
            ballot.preference(Candidate(2), Candidate(1)),
            PairPreference::Abstain
        );
    }

    #[test]
    fn test_duplicate_rejected() {
        assert!(RankedBallot::total(&[Candidate(0), Candidate(0), Candidate(1)], 3).is_err());
    }

    #[test]
    fn test_out_of_range_rejected() {
        assert!(RankedBallot::total(&[Candidate(0), Candidate(1), Candidate(3)], 3).is_err());
    }

    #[test]
    fn test_total_length_mismatch() {
        assert!(RankedBallot::total(&[Candidate(0), Candidate(1)], 3).is_err());
    }
}
