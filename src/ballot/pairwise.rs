//! Pairwise ballot implementation.

use bitvec::prelude::*;

use crate::ballot::{Ballot, PairPreference};
use crate::error::Error;
use crate::profile::Candidate;

/// Number of unordered pairs among `n` candidates.
pub const fn num_pairs(n: usize) -> usize {
    n * (n.saturating_sub(1)) / 2
}

const fn ballot_bits(n: usize) -> usize {
    num_pairs(n) * 2
}

#[inline]
const fn pair_index(i: usize, j: usize, n: usize) -> usize {
    debug_assert!(i < j);
    debug_assert!(j < n);
    i * n - i * (i + 1) / 2 + (j - i - 1)
}

#[derive(Debug, Clone)]
enum BitPackedPairwise {
    Tiny(u32),
    Small(u64),
    Large(BitVec<u8, Lsb0>),
}

impl BitPackedPairwise {
    fn new(n: usize) -> Self {
        let bits = ballot_bits(n);
        if bits <= 32 {
            BitPackedPairwise::Tiny(0)
        } else if bits <= 64 {
            BitPackedPairwise::Small(0)
        } else {
            BitPackedPairwise::Large(bitvec![u8, Lsb0; 0; bits])
        }
    }

    fn preference(&self, i: usize, j: usize, n: usize) -> PairPreference {
        if i == j {
            return PairPreference::Abstain;
        }
        let (left, right, swap) = if i < j { (i, j, false) } else { (j, i, true) };
        let idx = pair_index(left, right, n) * 2;
        let bits: u8 = match self {
            BitPackedPairwise::Tiny(bits) => ((bits >> idx) & 0b11) as u8,
            BitPackedPairwise::Small(bits) => ((bits >> idx) & 0b11) as u8,
            BitPackedPairwise::Large(bits) => {
                let hi = bits[idx] as u8;
                let lo = bits[idx + 1] as u8;
                (hi << 1) | lo
            }
        };

        let pref = match bits {
            0b00 => PairPreference::Abstain,
            0b01 => PairPreference::Left,
            0b10 => PairPreference::Right,
            _ => PairPreference::Abstain,
        };

        if swap {
            match pref {
                PairPreference::Left => PairPreference::Right,
                PairPreference::Right => PairPreference::Left,
                other => other,
            }
        } else {
            pref
        }
    }

    fn set_preference(&mut self, i: usize, j: usize, n: usize, pref: PairPreference) {
        let (left, right, swap) = if i < j { (i, j, false) } else { (j, i, true) };
        let pref = if swap {
            match pref {
                PairPreference::Left => PairPreference::Right,
                PairPreference::Right => PairPreference::Left,
                other => other,
            }
        } else {
            pref
        };
        let idx = pair_index(left, right, n) * 2;
        let bits = pref as u8;

        match self {
            BitPackedPairwise::Tiny(val) => {
                *val &= !(0b11 << idx);
                *val |= (bits as u32) << idx;
            }
            BitPackedPairwise::Small(val) => {
                *val &= !(0b11 << idx);
                *val |= (bits as u64) << idx;
            }
            BitPackedPairwise::Large(bv) => {
                let hi = (bits >> 1) & 1 == 1;
                let lo = (bits & 1) == 1;
                bv.set(idx, hi);
                bv.set(idx + 1, lo);
            }
        }
    }
}

/// A ballot that stores explicit pairwise preferences for every pair of
/// candidates.
///
/// This ballot type allows Condorcet cycles within a single ballot and
/// supports abstentions. Preferences are stored in a compact bit-packed
/// representation.
#[derive(Debug, Clone)]
pub struct PairwiseBallot {
    n: usize,
    inner: BitPackedPairwise,
}

impl PairwiseBallot {
    /// Create a new ballot with all pairs set to [`Abstain`](PairPreference::Abstain).
    pub fn new(n: usize) -> Self {
        Self {
            n,
            inner: BitPackedPairwise::new(n),
        }
    }

    /// Create a ballot from a list of pairwise preferences.
    ///
    /// Each tuple `(i, j, pref)` sets the preference for the ordered pair
    /// `(i, j)`. If contradictory preferences are provided for the same
    /// unordered pair, an error is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if a candidate index is out of range or if
    /// contradictory preferences are supplied for the same pair.
    pub fn from_pairs(
        pairs: &[(Candidate, Candidate, PairPreference)],
        n: usize,
    ) -> Result<Self, Error> {
        let mut ballot = Self::new(n);
        for &(i, j, pref) in pairs {
            if i.0 >= n || j.0 >= n {
                return Err(Error::InvalidCandidate(i.max(j), n));
            }
            let existing = ballot.preference(i, j);
            if existing != PairPreference::Abstain && existing != pref {
                return Err(Error::ContradictoryPair(i.min(j), i.max(j)));
            }
            ballot.set_preference(i, j, pref);
        }
        Ok(ballot)
    }

    /// Set the preference for the ordered pair `(i, j)`.
    pub fn set_preference(&mut self, i: Candidate, j: Candidate, pref: PairPreference) {
        self.inner.set_preference(i.0, j.0, self.n, pref);
    }
}

impl Ballot for PairwiseBallot {
    fn n_candidates(&self) -> usize {
        self.n
    }

    fn preference(&self, i: Candidate, j: Candidate) -> PairPreference {
        self.inner.preference(i.0, j.0, self.n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pair_index() {
        let n = 4;
        assert_eq!(pair_index(0, 1, n), 0);
        assert_eq!(pair_index(0, 2, n), 1);
        assert_eq!(pair_index(0, 3, n), 2);
        assert_eq!(pair_index(1, 2, n), 3);
        assert_eq!(pair_index(1, 3, n), 4);
        assert_eq!(pair_index(2, 3, n), 5);
    }

    #[test]
    fn test_tiny_ballot() {
        let n = 4;
        let mut ballot = PairwiseBallot::new(n);
        assert!(matches!(ballot.inner, BitPackedPairwise::Tiny(0)));

        ballot.set_preference(Candidate(0), Candidate(1), PairPreference::Left);
        assert_eq!(
            ballot.preference(Candidate(0), Candidate(1)),
            PairPreference::Left
        );
        assert_eq!(
            ballot.preference(Candidate(0), Candidate(2)),
            PairPreference::Abstain
        );

        ballot.set_preference(Candidate(0), Candidate(1), PairPreference::Right);
        assert_eq!(
            ballot.preference(Candidate(0), Candidate(1)),
            PairPreference::Right
        );

        ballot.set_preference(Candidate(1), Candidate(2), PairPreference::Left);
        assert_eq!(
            ballot.preference(Candidate(1), Candidate(2)),
            PairPreference::Left
        );
    }

    #[test]
    fn test_from_pairs() {
        let n = 4;
        let ballot = PairwiseBallot::from_pairs(
            &[
                (Candidate(0), Candidate(1), PairPreference::Left),
                (Candidate(0), Candidate(2), PairPreference::Left),
                (Candidate(0), Candidate(3), PairPreference::Left),
                (Candidate(1), Candidate(2), PairPreference::Left),
                (Candidate(1), Candidate(3), PairPreference::Left),
                (Candidate(2), Candidate(3), PairPreference::Left),
            ],
            n,
        )
        .unwrap();
        assert_eq!(
            ballot.preference(Candidate(0), Candidate(1)),
            PairPreference::Left
        );
        assert_eq!(
            ballot.preference(Candidate(0), Candidate(3)),
            PairPreference::Left
        );
        assert_eq!(
            ballot.preference(Candidate(1), Candidate(2)),
            PairPreference::Left
        );
        assert_eq!(
            ballot.preference(Candidate(2), Candidate(3)),
            PairPreference::Left
        );
        assert_eq!(
            ballot.preference(Candidate(3), Candidate(0)),
            PairPreference::Right
        );
    }

    #[test]
    fn test_large_ballot() {
        let n = 10;
        let ballot = PairwiseBallot::new(n);
        assert!(matches!(ballot.inner, BitPackedPairwise::Large(_)));
    }

    #[test]
    fn test_contradictory_pair_rejected() {
        let n = 2;
        let result = PairwiseBallot::from_pairs(
            &[
                (Candidate(0), Candidate(1), PairPreference::Left),
                (Candidate(1), Candidate(0), PairPreference::Left),
            ],
            n,
        );
        assert!(result.is_err());
    }
}
