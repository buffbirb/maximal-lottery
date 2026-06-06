use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};

use crate::types::Lottery;

#[derive(Debug, thiserror::Error)]
pub enum MaximalLotteryError {
    #[error("no candidates provided")]
    NoCandidates,
    #[error("the LP is infeasible")]
    Infeasible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LotteryMethod {
    /// Return the centroid (center of mass) of the set of all maximal lotteries.
    /// This corresponds to the "canonical" maximal lottery advocated by Fishburn,
    /// Brandl, and others — the uniform average over all optimal strategies.
    Centroid,
}

/// Solve a pairwise margin matrix `M` for a maximal lottery.
///
/// `M` is an n×n skew-symmetric matrix where `M[i][j] = -M[j][i]`.
/// The entry `M[i][j]` is the margin (voters preferring `i` over `j` minus
/// voters preferring `j` over `i`).
///
/// The `method` parameter selects the tie-breaking strategy when multiple
/// lotteries are optimal.
pub fn maximal_lottery(
    margins: &[Vec<i64>],
    method: LotteryMethod,
) -> Result<Lottery, MaximalLotteryError> {
    let n = margins.len();
    if n == 0 {
        return Err(MaximalLotteryError::NoCandidates);
    }
    if n == 1 {
        return Ok(vec![BigRational::one()]);
    }

    if let Some(winner) = condorcet_winner(margins) {
        let mut lottery = vec![BigRational::zero(); n];
        lottery[winner] = BigRational::one();
        return Ok(lottery);
    }

    match method {
        LotteryMethod::Centroid => centroid(margins),
    }
}

/// Return the Condorcet winner (if one exists): a candidate `i` such that
/// `M[i][j] > 0` for all `j ≠ i`.
pub fn condorcet_winner(margins: &[Vec<i64>]) -> Option<usize> {
    let n = margins.len();
    (0..n).find(|&i| (0..n).all(|j| i == j || margins[i][j] > 0))
}

// ── Centroid computation via extreme-point enumeration ──────────────────────

fn centroid(margins: &[Vec<i64>]) -> Result<Lottery, MaximalLotteryError> {
    let n = margins.len();
    let total_vars = 2 * n; // p_0..p_{n-1}, s_0..s_{n-1}
    let zero_count = n.saturating_sub(1);

    // Build the equality system  A · x = b,  where
    //   x = [p_0, …, p_{n-1},  s_0, …, s_{n-1}]
    //
    // Rows 0..n-1 (one per candidate j):
    //     Σ_i M[i][j] · p_i  −  s_j  =  0
    // Row n (sum-to-one):
    //     Σ_i p_i  =  1
    let rows = n + 1;
    let cols = total_vars;

    let a: Vec<Vec<BigRational>> = (0..rows)
        .map(|r| {
            (0..cols)
                .map(|c| {
                    if r < n {
                        if c < n {
                            BigRational::from_integer(BigInt::from(margins[c][r]))
                        } else if c == n + r {
                            -BigRational::one()
                        } else {
                            BigRational::zero()
                        }
                    } else {
                        if c < n {
                            BigRational::one()
                        } else {
                            BigRational::zero()
                        }
                    }
                })
                .collect()
        })
        .collect();

    let b: Vec<BigRational> = {
        let mut b = vec![BigRational::zero(); rows];
        b[n] = BigRational::one();
        b
    };

    let mut extreme_points: Vec<Vec<BigRational>> = Vec::new();
    let mut subset = Vec::with_capacity(zero_count);

    enumerate_subsets(
        zero_count,
        total_vars,
        &mut subset,
        &mut |zeros: &[usize]| {
            let basis: Vec<usize> = (0..total_vars).filter(|c| !zeros.contains(c)).collect();

            let mut sub = vec![vec![BigRational::zero(); rows]; rows];
            for (col_idx, &var) in basis.iter().enumerate() {
                for r in 0..rows {
                    sub[r][col_idx] = a[r][var].clone();
                }
            }

            if let Some(x_basis) = solve_linear(&sub, &b) {
                if x_basis.iter().all(|x| *x >= BigRational::zero()) {
                    let mut p = vec![BigRational::zero(); n];
                    for (col_idx, &var) in basis.iter().enumerate() {
                        if var < n {
                            p[var] = x_basis[col_idx].clone();
                        }
                    }
                    if !extreme_points.contains(&p) {
                        extreme_points.push(p);
                    }
                }
            }
        },
    );

    if extreme_points.is_empty() {
        return Err(MaximalLotteryError::Infeasible);
    }

    // Centroid = average of all unique extreme points
    let m = extreme_points.len();
    let mut result = vec![BigRational::zero(); n];
    for pt in &extreme_points {
        for i in 0..n {
            result[i] += pt[i].clone();
        }
    }
    for i in 0..n {
        result[i] /= BigRational::from_integer(BigInt::from(m));
    }

    Ok(result)
}

// ── Subset enumeration ──────────────────────────────────────────────────────

fn enumerate_subsets(
    size: usize,
    n_total: usize,
    current: &mut Vec<usize>,
    f: &mut impl FnMut(&[usize]),
) {
    if current.len() == size {
        f(current);
        return;
    }
    let start = current.last().map_or(0, |&x| x + 1);
    let end = n_total.saturating_sub(size - current.len());
    for i in start..=end {
        current.push(i);
        enumerate_subsets(size, n_total, current, f);
        current.pop();
    }
}

// ── Gaussian elimination (exact rational arithmetic) ────────────────────────

fn solve_linear(a: &[Vec<BigRational>], b: &[BigRational]) -> Option<Vec<BigRational>> {
    let n = a.len();
    let mut aug = vec![vec![BigRational::zero(); n + 1]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = a[i][j].clone();
        }
        aug[i][n] = b[i].clone();
    }

    for col in 0..n {
        let pivot_row = (col..n).find(|&r| aug[r][col] != BigRational::zero())?;
        if pivot_row != col {
            aug.swap(col, pivot_row);
        }

        let pivot_val = aug[col][col].clone();
        for j in col..=n {
            aug[col][j] /= &pivot_val;
        }

        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row][col].clone();
            if factor == BigRational::zero() {
                continue;
            }
            for j in col..=n {
                let term = factor.clone() * aug[col][j].clone();
                aug[row][j] -= term;
            }
        }
    }

    let mut x = vec![BigRational::zero(); n];
    for i in 0..n {
        x[i] = aug[i][n].clone();
    }
    Some(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_candidate() {
        let margins = vec![vec![0i64]];
        let result = maximal_lottery(&margins, LotteryMethod::Centroid).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], BigRational::one());
    }

    #[test]
    fn test_two_candidates_tie() {
        let margins = vec![vec![0, 0], vec![0, 0]];
        let result = maximal_lottery(&margins, LotteryMethod::Centroid).unwrap();
        assert_eq!(result.len(), 2);

        let half = BigRational::new(BigInt::from(1u8), BigInt::from(2u8));
        assert_eq!(result[0], half);
        assert_eq!(result[1], half);

        let sum: BigRational = result.iter().sum();
        assert_eq!(sum, BigRational::one());

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
        let result = maximal_lottery(&margins, LotteryMethod::Centroid).unwrap();
        assert_eq!(result[0], BigRational::one());
        assert_eq!(result[1], BigRational::zero());
        assert_eq!(result[2], BigRational::zero());
    }

    #[test]
    fn test_wikipedia_example() {
        let margins = vec![vec![0, 1, -1], vec![-1, 0, 1], vec![1, -1, 0]];

        let result = maximal_lottery(&margins, LotteryMethod::Centroid).unwrap();
        assert_eq!(result.len(), 3);

        let sum: BigRational = result.iter().sum();
        assert_eq!(sum, BigRational::one());

        let third = BigRational::new(BigInt::from(1u8), BigInt::from(3u8));
        for p in &result {
            assert_eq!(*p, third);
        }

        for j in 0..3 {
            let mut payoff = BigRational::zero();
            for i in 0..3 {
                payoff +=
                    result[i].clone() * BigRational::from_integer(BigInt::from(margins[i][j]));
            }
            assert!(payoff >= BigRational::zero());
        }
    }

    #[test]
    fn test_empty() {
        let margins: Vec<Vec<i64>> = vec![];
        assert!(maximal_lottery(&margins, LotteryMethod::Centroid).is_err());
    }

    #[test]
    fn test_condorcet_winner_func() {
        let margins = vec![vec![0, 1, 2], vec![-1, 0, 1], vec![-2, -1, 0]];
        assert_eq!(condorcet_winner(&margins), Some(0));
    }

    #[test]
    fn test_no_condorcet() {
        let margins = vec![vec![0, 1, -1], vec![-1, 0, 1], vec![1, -1, 0]];
        assert_eq!(condorcet_winner(&margins), None);
    }
}
