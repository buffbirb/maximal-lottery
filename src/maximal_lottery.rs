use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};

use crate::simplex;
use crate::types::Lottery;

#[derive(Debug, thiserror::Error)]
pub enum MaximalLotteryError {
    #[error("no candidates provided")]
    NoCandidates,
    #[error("the LP is infeasible")]
    Infeasible,
}

/// Solve a pairwise margin matrix M for a maximal lottery.
///
/// M is an n×n skew-symmetric matrix where M[i][j] = -M[j][i].
/// The entry M[i][j] is the margin (voters preferring i over j minus voters preferring j over i).
///
/// Returns a `Lottery` — a probability distribution over candidates with exact rational values.
pub fn maximal_lottery(margins: &[Vec<i64>]) -> Result<Lottery, MaximalLotteryError> {
    let n = margins.len();
    if n == 0 {
        return Err(MaximalLotteryError::NoCandidates);
    }
    if n == 1 {
        return Ok(vec![BigRational::one()]);
    }

    // Check for Condorcet winner: candidate i* such that M[i*][j] > 0 for all j ≠ i*
    for i in 0..n {
        let mut is_winner = true;
        for j in 0..n {
            if i != j && margins[i][j] <= 0 {
                is_winner = false;
                break;
            }
        }
        if is_winner {
            let mut lottery = vec![BigRational::zero(); n];
            lottery[i] = BigRational::one();
            return Ok(lottery);
        }
    }

    solve_lp(margins)
}

/// Build the LP and solve for a maximal lottery.
///
/// The feasibility LP:
/// Find p_i ≥ 0 such that:
///   Σ_i p_i = 1
///   Σ_i p_i * M[i][j] ≥ 0   for all j
///
/// Standard form Ax = b, x ≥ 0:
///   Σ_i p_i * M[i][j] - s_j = 0   for all j
///   Σ_i p_i = 1
///   p_i ≥ 0, s_j ≥ 0
fn solve_lp(margins: &[Vec<i64>]) -> Result<Lottery, MaximalLotteryError> {
    let n = margins.len();
    let n_vars = 2 * n; // p_0..p_{n-1}, s_0..s_{n-1}
    let n_cons = n + 1;

    let mut a = vec![vec![BigRational::zero(); n_vars]; n_cons];
    let mut b = vec![BigRational::zero(); n_cons];

    // Constraints for j = 0..n-1: Σ_i p_i * M[i][j] - s_j = 0
    for j in 0..n {
        for i in 0..n {
            a[j][i] = BigRational::from_integer(BigInt::from(margins[i][j]));
        }
        a[j][n + j] = -BigRational::one();
    }

    // Constraint n: Σ_i p_i = 1
    for i in 0..n {
        a[n][i] = BigRational::one();
    }
    b[n] = BigRational::one();

    // Phase II objective: all zeros (feasibility only)
    let phase2_obj = vec![BigRational::zero(); n_vars];

    let solution =
        simplex::solve_lp(&a, &b, n, &phase2_obj).ok_or(MaximalLotteryError::Infeasible)?;

    Ok(solution)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_candidate() {
        let margins = vec![vec![0i64]];
        let result = maximal_lottery(&margins).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], BigRational::one());
    }

    #[test]
    fn test_two_candidates_tie() {
        let margins = vec![vec![0, 0], vec![0, 0]];
        let result = maximal_lottery(&margins).unwrap();
        assert_eq!(result.len(), 2);
        for p in &result {
            assert!(*p >= BigRational::zero());
        }
        let sum: BigRational = result.iter().sum();
        assert_eq!(sum, BigRational::one());

        // Equilibrium: Σ_i p_i * M[i][j] ≥ 0 for all j
        for j in 0..2 {
            let mut payoff = BigRational::zero();
            for i in 0..2 {
                payoff +=
                    result[i].clone() * BigRational::from_integer(BigInt::from(margins[i][j]));
            }
            assert!(payoff >= BigRational::zero());
        }
    }

    #[test]
    fn test_condorcet_winner() {
        // Candidate 0 beats 1 and 2; 1 beats 2
        let margins = vec![vec![0, 10, 20], vec![-10, 0, 5], vec![-20, -5, 0]];
        let result = maximal_lottery(&margins).unwrap();
        assert_eq!(result[0], BigRational::one());
        assert_eq!(result[1], BigRational::zero());
        assert_eq!(result[2], BigRational::zero());
    }

    #[test]
    fn test_wikipedia_example() {
        let margins = vec![vec![0, 1, -1], vec![-1, 0, 1], vec![1, -1, 0]];

        let result = maximal_lottery(&margins).unwrap();
        assert_eq!(result.len(), 3);

        let sum: BigRational = result.iter().sum();
        assert_eq!(sum, BigRational::one());

        // For this symmetric cycle, the unique maximal lottery is (1/3, 1/3, 1/3)
        let third = BigRational::new(BigInt::from(1u8), BigInt::from(3u8));
        for p in &result {
            assert_eq!(*p, third, "expected 1/3 for each candidate");
        }

        // Equilibrium condition: for all j, Σ_i p_i * M[i][j] ≥ 0
        for j in 0..3 {
            let mut payoff = BigRational::zero();
            for i in 0..3 {
                payoff +=
                    result[i].clone() * BigRational::from_integer(BigInt::from(margins[i][j]));
            }
            assert!(
                payoff >= BigRational::zero(),
                "Violation at candidate {}: payoff = {}",
                j,
                payoff
            );
        }
    }

    #[test]
    fn test_empty() {
        let margins: Vec<Vec<i64>> = vec![];
        assert!(maximal_lottery(&margins).is_err());
    }
}
