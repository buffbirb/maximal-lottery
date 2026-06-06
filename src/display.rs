use crate::ballot::{CompactBallot, PairPreference};
use crate::maximal_lottery::condorcet_winner;
use crate::types::Lottery;

pub fn print_ballot(ballot: &CompactBallot, n: usize, label: &str) {
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
                match ballot.preference(i, j, n) {
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

pub fn print_margins(margins: &[Vec<i64>]) {
    println!("Margin matrix:");
    for row in margins {
        println!("  {:?}", row);
    }
}

pub fn print_lottery(lottery: &Lottery, margins: &[Vec<i64>]) {
    println!();
    if let Some(winner) = condorcet_winner(margins) {
        println!("Condorcet winner: candidate {}", winner);
    } else {
        println!("Maximal lottery:");
        for (i, prob) in lottery.iter().enumerate() {
            println!("  Candidate {}: {}", i, prob);
        }
    }
}
