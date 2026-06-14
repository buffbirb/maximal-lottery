//! Exact centroid maximal-lottery solver.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};

use crate::error::Error;
use crate::profile::{Lottery, MarginMatrix};
use crate::solver::LotterySolver;

/// A maximal-lottery solver that returns the centroid of all optimal
/// strategies.
///
/// The centroid is the uniform average over all extreme points of the
/// optimal polytope. It corresponds to the canonical maximal lottery
/// advocated by Fishburn, Brandl, and others.
#[derive(Debug, Clone, Copy, Default)]
pub struct CentroidSolver;

impl LotterySolver for CentroidSolver {
    type Error = Error;

    fn solve(&self, margins: &MarginMatrix) -> Result<Lottery, Self::Error> {
        centroid(margins)
    }
}

fn centroid(margins: &MarginMatrix) -> Result<Lottery, Error> {
    let margins_vec = margins.margins();
    let n = margins_vec.len();
    if n == 0 {
        return Err(Error::NoCandidates);
    }
    if n == 1 {
        return Ok(Lottery::new(vec![BigRational::one()]));
    }

    if let Some(winner) = margins.condorcet_winner() {
        let mut lottery = vec![BigRational::zero(); n];
        lottery[winner.0] = BigRational::one();
        return Ok(Lottery::new(lottery));
    }

    let total_vars = 2 * n;
    let zero_count = n.saturating_sub(1);

    // Build the equality system A · x = b, where
    //   x = [p_0, …, p_{n-1}, s_0, …, s_{n-1}]
    //
    // Rows 0..n-1 (one per candidate j):
    //     Σ_i M[i][j] · p_i − s_j = 0
    // Row n (sum-to-one):
    //     Σ_i p_i = 1
    let rows = n + 1;
    let cols = total_vars;

    let a: Vec<Vec<BigRational>> = (0..rows)
        .map(|r| {
            (0..cols)
                .map(|c| {
                    if r < n {
                        if c < n {
                            BigRational::from_integer(BigInt::from(margins_vec[c][r]))
                        } else if c == n + r {
                            -BigRational::one()
                        } else {
                            BigRational::zero()
                        }
                    } else if c < n {
                        BigRational::one()
                    } else {
                        BigRational::zero()
                    }
                })
                .collect()
        })
        .collect();

    let mut b = vec![BigRational::zero(); rows];
    b[n] = BigRational::one();

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

            if let Some(x_basis) = solve_linear(&sub, &b)
                && x_basis.iter().all(|x| *x >= BigRational::zero())
            {
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
        },
    );

    if extreme_points.is_empty() {
        return Err(Error::Infeasible);
    }

    let m = extreme_points.len();
    let mut result = vec![BigRational::zero(); n];
    for pt in &extreme_points {
        for i in 0..n {
            result[i] += pt[i].clone();
        }
    }
    for r in result.iter_mut().take(n) {
        *r /= BigRational::from_integer(BigInt::from(m));
    }

    Ok(Lottery::new(result))
}

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

fn solve_linear(a: &[Vec<BigRational>], b: &[BigRational]) -> Option<Vec<BigRational>> {
    let n = a.len();
    let mut aug: Vec<Vec<BigRational>> = a
        .iter()
        .zip(b)
        .map(|(row, rhs)| {
            let mut extended = row.clone();
            extended.push(rhs.clone());
            extended
        })
        .collect();

    for col in 0..n {
        let pivot_row = (col..n).find(|&r| aug[r][col] != BigRational::zero())?;
        if pivot_row != col {
            aug.swap(col, pivot_row);
        }

        let pivot_val = aug[col][col].clone();
        for entry in aug[col].iter_mut().skip(col) {
            *entry /= &pivot_val;
        }

        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row][col].clone();
            if factor.is_zero() {
                continue;
            }

            // Borrow the source row (col) immutably and the destination row
            // (row) mutably without overlapping borrows.
            let split = col.max(row);
            let (top, bottom) = aug.split_at_mut(split);
            let (src_row, dst_row) = if col < row {
                (&top[col], &mut bottom[0])
            } else {
                (&bottom[0], &mut top[row])
            };

            for (src, dst) in src_row.iter().skip(col).zip(dst_row.iter_mut().skip(col)) {
                let term = factor.clone() * src.clone();
                *dst -= term;
            }
        }
    }

    Some(aug.into_iter().map(|row| row[n].clone()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Candidate;

    fn margins(m: Vec<Vec<i64>>) -> MarginMatrix {
        MarginMatrix::from_vec(m).unwrap()
    }

    #[test]
    fn test_single_candidate() {
        let matrix = margins(vec![vec![0i64]]);
        let result = CentroidSolver.solve(&matrix).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(Candidate(0)).unwrap(), &BigRational::one());
    }

    #[test]
    fn test_two_candidates_tie() {
        let matrix = margins(vec![vec![0, 0], vec![0, 0]]);
        let result = CentroidSolver.solve(&matrix).unwrap();
        assert_eq!(result.len(), 2);

        let half = BigRational::new(BigInt::from(1u8), BigInt::from(2u8));
        assert_eq!(result.get(Candidate(0)).unwrap(), &half);
        assert_eq!(result.get(Candidate(1)).unwrap(), &half);

        let sum: BigRational = result.probabilities().iter().sum();
        assert_eq!(sum, BigRational::one());

        let margins_vec = matrix.margins();
        for row in margins_vec.iter().take(2) {
            let payoff: BigRational = row
                .iter()
                .zip(result.probabilities().iter())
                .map(|(margin, prob)| {
                    prob.clone() * BigRational::from_integer(BigInt::from(*margin))
                })
                .sum();
            assert!(payoff >= BigRational::zero());
        }
    }

    #[test]
    fn test_condorcet_winner() {
        let matrix = margins(vec![vec![0, 10, 20], vec![-10, 0, 5], vec![-20, -5, 0]]);
        let result = CentroidSolver.solve(&matrix).unwrap();
        assert_eq!(result.get(Candidate(0)).unwrap(), &BigRational::one());
        assert_eq!(result.get(Candidate(1)).unwrap(), &BigRational::zero());
        assert_eq!(result.get(Candidate(2)).unwrap(), &BigRational::zero());
    }

    #[test]
    fn test_wikipedia_example() {
        let matrix = margins(vec![vec![0, 1, -1], vec![-1, 0, 1], vec![1, -1, 0]]);

        let result = CentroidSolver.solve(&matrix).unwrap();
        assert_eq!(result.len(), 3);

        let sum: BigRational = result.probabilities().iter().sum();
        assert_eq!(sum, BigRational::one());

        let third = BigRational::new(BigInt::from(1u8), BigInt::from(3u8));
        for p in result.probabilities() {
            assert_eq!(p, &third);
        }

        let margins_vec = matrix.margins();
        for row in margins_vec.iter().take(3) {
            let payoff: BigRational = row
                .iter()
                .zip(result.probabilities().iter())
                .map(|(margin, prob)| {
                    prob.clone() * BigRational::from_integer(BigInt::from(*margin))
                })
                .sum();
            assert!(payoff >= BigRational::zero());
        }
    }

    #[test]
    fn test_empty() {
        let matrix = margins(Vec::new());
        assert!(CentroidSolver.solve(&matrix).is_err());
    }
}
