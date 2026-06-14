//! Core types: [`Candidate`], [`Lottery`], [`MarginMatrix`], and
//! [`PreferenceProfile`].

use num_rational::BigRational;

use crate::ballot::{Ballot, PairPreference};
use crate::error::Error;

/// A candidate identifier.
///
/// Candidates are represented by integer indices into the candidate set
/// `0..n`. The field is public for convenience, but callers should still
/// ensure that the index is within the valid range for a given election.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Candidate(pub usize);

impl From<usize> for Candidate {
    fn from(index: usize) -> Self {
        Candidate(index)
    }
}

impl From<Candidate> for usize {
    fn from(candidate: Candidate) -> Self {
        candidate.0
    }
}

impl std::fmt::Display for Candidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A probability distribution over candidates.
///
/// The probabilities are exact rational numbers produced by the chosen
/// [`LotterySolver`](crate::solver::LotterySolver).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Lottery(Vec<BigRational>);

impl Lottery {
    /// Create a lottery from a vector of probabilities.
    ///
    /// The caller is responsible for ensuring the probabilities are valid
    /// (non-negative and sum to one). Solvers in this crate always return
    /// valid lotteries.
    pub fn new(probabilities: Vec<BigRational>) -> Self {
        Self(probabilities)
    }

    /// Number of candidates in the lottery.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the lottery contains no candidates.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Probability assigned to `candidate`, if present.
    pub fn get(&self, candidate: Candidate) -> Option<&BigRational> {
        self.0.get(candidate.0)
    }

    /// All probabilities, indexed by candidate.
    pub fn probabilities(&self) -> &[BigRational] {
        &self.0
    }

    /// Consume the lottery and return the underlying probability vector.
    pub fn into_inner(self) -> Vec<BigRational> {
        self.0
    }
}

/// A skew-symmetric pairwise margin matrix.
///
/// For `n` candidates, the matrix is `n × n` with `M[i][j] = -M[j][i]` and
/// `M[i][i] = 0`. The entry `M[i][j]` is the number of voters preferring
/// candidate `i` over candidate `j` minus those preferring `j` over `i`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarginMatrix(Vec<Vec<i64>>);

impl MarginMatrix {
    /// Construct a margin matrix from a raw matrix.
    ///
    /// Returns an error if the matrix is not square, has a non-zero diagonal,
    /// or is not skew-symmetric.
    pub fn from_vec(margins: Vec<Vec<i64>>) -> Result<Self, Error> {
        let n = margins.len();
        for (i, row) in margins.iter().enumerate() {
            if row.len() != n {
                return Err(Error::InvalidMargins);
            }
            if row[i] != 0 {
                return Err(Error::InvalidMargins);
            }
            for (j, &entry) in row.iter().enumerate() {
                if margins[j][i] != -entry {
                    return Err(Error::InvalidMargins);
                }
            }
        }
        Ok(Self(margins))
    }

    /// Number of candidates.
    pub fn n_candidates(&self) -> usize {
        self.0.len()
    }

    /// Raw margin rows.
    pub fn margins(&self) -> &[Vec<i64>] {
        &self.0
    }

    /// Margin `M[i][j]`, if the indices are in range.
    pub fn get(&self, i: Candidate, j: Candidate) -> Option<i64> {
        self.0.get(i.0)?.get(j.0).copied()
    }

    /// The Condorcet winner, if one exists.
    ///
    /// A Condorcet winner is a candidate that beats every other candidate in
    /// pairwise comparison.
    pub fn condorcet_winner(&self) -> Option<Candidate> {
        let n = self.n_candidates();
        (0..n)
            .find(|&i| (0..n).all(|j| i == j || self.0[i][j] > 0))
            .map(Candidate)
    }
}

/// A collection of ballots over the same set of candidates.
#[derive(Debug, Clone)]
pub struct PreferenceProfile<B: Ballot> {
    n: usize,
    ballots: Vec<B>,
}

impl<B: Ballot> PreferenceProfile<B> {
    /// Create a new preference profile.
    ///
    /// Returns an error if the ballots do not all refer to the same number of
    /// candidates.
    pub fn try_new(ballots: Vec<B>) -> Result<Self, Error> {
        let n = ballots.first().map(Ballot::n_candidates).unwrap_or(0);
        if ballots.iter().any(|b| b.n_candidates() != n) {
            return Err(Error::InconsistentCandidateCount);
        }
        Ok(Self { n, ballots })
    }

    /// Number of candidates in the election.
    pub fn n_candidates(&self) -> usize {
        self.n
    }

    /// Borrow the ballots.
    pub fn ballots(&self) -> &[B] {
        &self.ballots
    }

    /// Tally the pairwise margins from all ballots.
    pub fn tally_margins(&self) -> MarginMatrix {
        let n = self.n;
        let mut margins = vec![vec![0i64; n]; n];
        for ballot in &self.ballots {
            for (i, j) in unordered_pairs(n) {
                match ballot.preference(Candidate(i), Candidate(j)) {
                    PairPreference::Left => {
                        margins[i][j] += 1;
                        margins[j][i] -= 1;
                    }
                    PairPreference::Right => {
                        margins[i][j] -= 1;
                        margins[j][i] += 1;
                    }
                    PairPreference::Abstain => {}
                }
            }
        }
        // Internally constructed margins are always valid.
        MarginMatrix::from_vec(margins).expect("internally constructed margins are valid")
    }
}

/// Iterate over all unordered pairs `(i, j)` with `i < j < n`.
fn unordered_pairs(n: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..n).flat_map(move |i| (i + 1..n).map(move |j| (i, j)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ballot::{PairPreference, PairwiseBallot};

    #[test]
    fn test_empty_profile() {
        let profile = PreferenceProfile::<PairwiseBallot>::try_new(vec![]).unwrap();
        assert_eq!(profile.n_candidates(), 0);
    }

    #[test]
    fn test_inconsistent_candidate_count() {
        let b1 = PairwiseBallot::new(3);
        let b2 = PairwiseBallot::new(4);
        assert!(PreferenceProfile::try_new(vec![b1, b2]).is_err());
    }

    #[test]
    fn test_tally_empty() {
        let profile = PreferenceProfile::<PairwiseBallot>::try_new(vec![]).unwrap();
        let margins = profile.tally_margins();
        assert_eq!(margins.margins(), &Vec::<Vec<i64>>::new());
    }

    #[test]
    fn test_tally_single_ballot() {
        let n = 3;
        let ballot = PairwiseBallot::from_pairs(
            &[
                (Candidate(0), Candidate(1), PairPreference::Left),
                (Candidate(0), Candidate(2), PairPreference::Left),
                (Candidate(1), Candidate(2), PairPreference::Left),
            ],
            n,
        )
        .unwrap();
        let profile = PreferenceProfile::try_new(vec![ballot]).unwrap();
        let margins = profile.tally_margins();
        assert_eq!(margins.get(Candidate(0), Candidate(1)), Some(1));
        assert_eq!(margins.get(Candidate(1), Candidate(0)), Some(-1));
        assert_eq!(margins.get(Candidate(0), Candidate(2)), Some(1));
        assert_eq!(margins.get(Candidate(2), Candidate(0)), Some(-1));
        assert_eq!(margins.get(Candidate(1), Candidate(2)), Some(1));
        assert_eq!(margins.get(Candidate(2), Candidate(1)), Some(-1));
    }

    #[test]
    fn test_margin_matrix_validation() {
        let valid = vec![vec![0, 1, -1], vec![-1, 0, 1], vec![1, -1, 0]];
        assert!(MarginMatrix::from_vec(valid).is_ok());

        let bad_diag = vec![vec![1, 0], vec![0, 0]];
        assert!(MarginMatrix::from_vec(bad_diag).is_err());

        let not_skew = vec![vec![0, 1], vec![1, 0]];
        assert!(MarginMatrix::from_vec(not_skew).is_err());
    }

    #[test]
    fn test_condorcet_winner() {
        let margins =
            MarginMatrix::from_vec(vec![vec![0, 1, 2], vec![-1, 0, 1], vec![-2, -1, 0]]).unwrap();
        assert_eq!(margins.condorcet_winner(), Some(Candidate(0)));
    }

    #[test]
    fn test_no_condorcet() {
        let margins =
            MarginMatrix::from_vec(vec![vec![0, 1, -1], vec![-1, 0, 1], vec![1, -1, 0]]).unwrap();
        assert_eq!(margins.condorcet_winner(), None);
    }
}
