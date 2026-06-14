//! Helper functions for printing ballots, margins, and lotteries.

use crate::ballot::{Ballot, PairPreference};
use crate::profile::{Candidate, Lottery, MarginMatrix};

/// Print a ballot as a grid of pairwise comparisons.
pub fn print_ballot(ballot: &impl Ballot, label: &str) {
    let n = ballot.n_candidates();
    println!("{}", label);
    print!("    ");
    for j in 0..n {
        print!(" {:>2} ", j);
    }
    println!();
    for i in 0..n {
        print!(" {:>2} ", i);
        for j in 0..n {
            if i == j {
                print!("  - ");
            } else {
                match ballot.preference(Candidate(i), Candidate(j)) {
                    PairPreference::Left => print!("  > "),
                    PairPreference::Right => print!("  < "),
                    PairPreference::Abstain => print!("  . "),
                }
            }
        }
        println!();
    }
    println!();
}

/// Print a margin matrix.
pub fn print_margins(margins: &MarginMatrix) {
    println!("Margin matrix:");
    for row in margins.margins() {
        println!("  {:?}", row);
    }
}

/// Print the result of a maximal-lottery computation.
///
/// If the margin matrix has a Condorcet winner, that candidate is printed.
/// Otherwise, the lottery probabilities are printed.
pub fn print_lottery(lottery: &Lottery, margins: &MarginMatrix) {
    println!();
    if let Some(winner) = margins.condorcet_winner() {
        println!("Condorcet winner: candidate {}", winner);
    } else {
        println!("Maximal lottery:");
        for (i, prob) in lottery.probabilities().iter().enumerate() {
            println!("  Candidate {}: {}", i, prob);
        }
    }
}
