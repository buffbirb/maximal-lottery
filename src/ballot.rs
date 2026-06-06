use bitvec::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PairPreference {
    Abstain = 0b00,
    Left = 0b01,
    Right = 0b10,
}

#[derive(Debug, Clone)]
pub enum CompactBallot {
    Tiny(u32),
    Small(u64),
    Large(BitVec<u8, Lsb0>),
}

pub const fn num_pairs(n: usize) -> usize {
    n * (n.saturating_sub(1)) / 2
}

pub const fn ballot_bits(n: usize) -> usize {
    num_pairs(n) * 2
}

pub const fn max_n_u32() -> usize {
    let mut n = 0;
    while n * (n + 1) <= 32 {
        n += 1;
    }
    n
}

pub const fn max_n_u64() -> usize {
    let mut n = 0;
    while n * (n + 1) <= 64 {
        n += 1;
    }
    n
}

#[inline]
pub const fn pair_index(i: usize, j: usize, n: usize) -> usize {
    debug_assert!(i < j);
    debug_assert!(j < n);
    i * n - i * (i + 1) / 2 + (j - i - 1)
}

impl CompactBallot {
    pub fn new(n: usize) -> Self {
        let bits = ballot_bits(n);
        if bits <= 32 {
            CompactBallot::Tiny(0)
        } else if bits <= 64 {
            CompactBallot::Small(0)
        } else {
            CompactBallot::Large(bitvec![u8, Lsb0; 0; bits])
        }
    }

    pub fn from_pairs(pairs: &[(usize, usize, PairPreference)], n: usize) -> Self {
        let mut ballot = Self::new(n);
        for &(i, j, pref) in pairs {
            ballot.set_preference(i, j, n, pref);
        }
        ballot
    }

    #[inline]
    pub fn preference(&self, i: usize, j: usize, n: usize) -> PairPreference {
        if i == j {
            return PairPreference::Abstain;
        }
        let (left, right, swap) = if i < j { (i, j, false) } else { (j, i, true) };
        let idx = pair_index(left, right, n) * 2;
        let bits: u8 = match self {
            CompactBallot::Tiny(bits) => ((bits >> idx) & 0b11) as u8,
            CompactBallot::Small(bits) => ((bits >> idx) & 0b11) as u8,
            CompactBallot::Large(bits) => {
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

    #[inline]
    pub fn set_preference(&mut self, i: usize, j: usize, n: usize, pref: PairPreference) {
        let idx = pair_index(i, j, n) * 2;
        let bits = pref as u8;

        match self {
            CompactBallot::Tiny(val) => {
                *val &= !(0b11 << idx);
                *val |= (bits as u32) << idx;
            }
            CompactBallot::Small(val) => {
                *val &= !(0b11 << idx);
                *val |= (bits as u64) << idx;
            }
            CompactBallot::Large(bv) => {
                let hi = (bits >> 1) & 1 == 1;
                let lo = (bits & 1) == 1;
                bv.set(idx, hi);
                bv.set(idx + 1, lo);
            }
        }
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
        let mut ballot = CompactBallot::new(n);
        assert!(matches!(ballot, CompactBallot::Tiny(0)));

        ballot.set_preference(0, 1, n, PairPreference::Left);
        assert_eq!(ballot.preference(0, 1, n), PairPreference::Left);
        assert_eq!(ballot.preference(0, 2, n), PairPreference::Abstain);

        ballot.set_preference(0, 1, n, PairPreference::Right);
        assert_eq!(ballot.preference(0, 1, n), PairPreference::Right);

        ballot.set_preference(1, 2, n, PairPreference::Left);
        assert_eq!(ballot.preference(1, 2, n), PairPreference::Left);
    }

    #[test]
    fn test_from_pairs() {
        let n = 4;
        let ballot = CompactBallot::from_pairs(
            &[
                (0, 1, PairPreference::Left),
                (0, 2, PairPreference::Left),
                (0, 3, PairPreference::Left),
                (1, 2, PairPreference::Left),
                (1, 3, PairPreference::Left),
                (2, 3, PairPreference::Left),
            ],
            n,
        );
        // 0 > 1, 0 > 3, 1 > 2, 2 > 3
        assert_eq!(ballot.preference(0, 1, n), PairPreference::Left);
        assert_eq!(ballot.preference(0, 3, n), PairPreference::Left);
        assert_eq!(ballot.preference(1, 2, n), PairPreference::Left);
        assert_eq!(ballot.preference(2, 3, n), PairPreference::Left);
        // 3 > 0 (reverse of 0 > 3)
        assert_eq!(ballot.preference(3, 0, n), PairPreference::Right);
    }

    #[test]
    fn test_large_ballot() {
        let n = 10;
        let ballot = CompactBallot::new(n);
        assert!(matches!(ballot, CompactBallot::Large(_)));
    }
}
