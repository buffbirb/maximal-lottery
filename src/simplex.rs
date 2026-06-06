use num_rational::BigRational;
use num_traits::{One, Zero};

#[derive(Debug, Clone)]
pub struct Tableau {
    /// Constraint matrix in canonical form (B^{-1} * A), m rows, n cols.
    pub a: Vec<Vec<BigRational>>,
    /// Right-hand side vector (values of basic variables).
    pub b: Vec<BigRational>,
    /// Objective coefficients.
    pub c: Vec<BigRational>,
    /// Indices of basic variables.
    pub basis: Vec<usize>,
}

impl Tableau {
    pub fn new(constraints: usize, variables: usize) -> Self {
        Self {
            a: vec![vec![BigRational::zero(); variables]; constraints],
            b: vec![BigRational::zero(); constraints],
            c: vec![BigRational::zero(); variables],
            basis: Vec::new(),
        }
    }

    pub fn n_rows(&self) -> usize {
        self.a.len()
    }

    pub fn n_cols(&self) -> usize {
        if self.a.is_empty() {
            0
        } else {
            self.a[0].len()
        }
    }

    fn reduced_cost(&self, col: usize) -> BigRational {
        let mut rc = self.c[col].clone();
        for (i, &basis_col) in self.basis.iter().enumerate() {
            rc -= self.c[basis_col].clone() * self.a[i][col].clone();
        }
        rc
    }

    fn entering_column(&self) -> Option<usize> {
        for j in 0..self.n_cols() {
            if self.reduced_cost(j) > BigRational::zero() {
                return Some(j);
            }
        }
        None
    }

    fn leaving_row(&self, entering: usize) -> Option<usize> {
        let mut min_ratio: Option<BigRational> = None;
        let mut min_row = None;

        for i in 0..self.n_rows() {
            let a_ij = &self.a[i][entering];
            if *a_ij > BigRational::zero() {
                let ratio = self.b[i].clone() / a_ij.clone();
                match &min_ratio {
                    None => {
                        min_ratio = Some(ratio.clone());
                        min_row = Some(i);
                    }
                    Some(current) if ratio < *current => {
                        min_ratio = Some(ratio);
                        min_row = Some(i);
                    }
                    _ => {}
                }
            }
        }

        min_row
    }

    fn pivot(&mut self, entering: usize, leaving_row: usize) {
        let pivot_val = self.a[leaving_row][entering].clone();

        // Normalize pivot row
        for j in 0..self.n_cols() {
            self.a[leaving_row][j] /= &pivot_val;
        }
        self.b[leaving_row] /= &pivot_val;

        // Eliminate entering column from all other rows
        for i in 0..self.n_rows() {
            if i == leaving_row {
                continue;
            }
            let factor = self.a[i][entering].clone();
            if factor.is_zero() {
                continue;
            }
            for j in 0..self.n_cols() {
                let term = factor.clone() * self.a[leaving_row][j].clone();
                self.a[i][j] -= term;
            }
            let br = self.b[leaving_row].clone();
            self.b[i] -= factor * br;
        }

        self.basis[leaving_row] = entering;
    }

    pub fn optimize(&mut self) -> bool {
        loop {
            let entering = match self.entering_column() {
                Some(col) => col,
                None => return true,
            };

            let leaving = match self.leaving_row(entering) {
                Some(row) => row,
                None => return false,
            };

            self.pivot(entering, leaving);
        }
    }

    pub fn primal_solution(&self) -> Vec<BigRational> {
        let mut x = vec![BigRational::zero(); self.n_cols()];
        for (i, &basis_col) in self.basis.iter().enumerate() {
            x[basis_col] = self.b[i].clone();
        }
        x
    }

    pub fn objective_value(&self) -> BigRational {
        let mut obj = BigRational::zero();
        for (i, &basis_col) in self.basis.iter().enumerate() {
            obj += self.c[basis_col].clone() * self.b[i].clone();
        }
        obj
    }
}

/// Solve a linear program in the form:
///   find x ≥ 0 such that Ax = b, all b_i ≥ 0
///
/// Uses a two-phase simplex method with arbitrary-precision rationals.
/// Returns the primal solution x for the first `n_original` variables, or None if infeasible.
pub fn solve_lp(
    a: &[Vec<BigRational>],
    b: &[BigRational],
    n_original: usize,
    phase2_obj: &[BigRational],
) -> Option<Vec<BigRational>> {
    let m = a.len();
    if m == 0 {
        return Some(vec![]);
    }
    let n_var = a[0].len();
    let n_aug = n_var + m; // original variables + artificials

    // Build augmented tableau [A | I]
    let mut tableau = Tableau::new(m, n_aug);
    for i in 0..m {
        for j in 0..n_var {
            tableau.a[i][j] = a[i][j].clone();
        }
        tableau.a[i][n_var + i] = BigRational::one();
        tableau.b[i] = b[i].clone();
    }

    // Phase I objective: maximize -Σ a_j
    // c_j = -1 for artificials, 0 for originals
    tableau.c = vec![BigRational::zero(); n_aug];
    for j in 0..m {
        tableau.c[n_var + j] = -BigRational::one();
    }
    tableau.basis = (n_var..n_var + m).collect();

    if !tableau.optimize() {
        return None; // unbounded (shouldn't happen)
    }

    // Check infeasibility: if sum of artificials > 0, no feasible solution
    let solution = tableau.primal_solution();
    let art_sum: BigRational = solution[n_var..].iter().sum();
    if art_sum > BigRational::zero() {
        return None;
    }

    // Drive remaining artificials (at value 0) out of basis if possible
    for i in 0..m {
        if tableau.basis[i] >= n_var {
            for j in 0..n_var {
                if !tableau.a[i][j].is_zero() && !tableau.basis.contains(&j) {
                    tableau.pivot(j, i);
                    break;
                }
            }
        }
    }

    // Phase II: switch to original objective
    // Keep artificial columns but set their objective to 0
    tableau.c = vec![BigRational::zero(); n_aug];
    for j in 0..n_original {
        if j < phase2_obj.len() {
            tableau.c[j] = phase2_obj[j].clone();
        }
    }

    if !tableau.optimize() {
        return None;
    }

    let sol = tableau.primal_solution();
    Some(sol[..n_original].to_vec())
}
