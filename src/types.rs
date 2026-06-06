use crate::ballot::CompactBallot;
use num_rational::BigRational;

pub type Candidate = usize;

pub type Lottery = Vec<BigRational>;

#[derive(Debug, Clone)]
pub struct PreferenceProfile {
    pub n: usize,
    pub ballots: Vec<CompactBallot>,
}

impl PreferenceProfile {
    pub fn new(n: usize, ballots: Vec<CompactBallot>) -> Self {
        Self { n, ballots }
    }
}
