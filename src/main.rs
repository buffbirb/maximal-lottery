use voting_systems::ballot::{CompactBallot, PairPreference};
use voting_systems::margins::tally_margins;
use voting_systems::maximal_lottery::maximal_lottery;
use voting_systems::types::PreferenceProfile;

fn main() {
    // Three voters, three candidates (0, 1, 2), Condorcet cycle.
    // Each voter's preferences expressed as explicit pairwise comparisons.
    let n = 3;

    // Voter 1: 0 > 1, 0 > 2, 1 > 2
    let ballot1 = CompactBallot::from_pairs(
        &[
            (0, 1, PairPreference::Left),
            (0, 2, PairPreference::Left),
            (1, 2, PairPreference::Left),
        ],
        n,
    );

    // Voter 2: 1 > 0, 1 > 2, 2 > 0
    let ballot2 = CompactBallot::from_pairs(
        &[
            (0, 1, PairPreference::Right),
            (0, 2, PairPreference::Right),
            (1, 2, PairPreference::Left),
        ],
        n,
    );

    // Voter 3: 2 > 1, 2 > 0, 0 > 1
    let ballot3 = CompactBallot::from_pairs(
        &[
            (0, 1, PairPreference::Left),
            (0, 2, PairPreference::Right),
            (1, 2, PairPreference::Right),
        ],
        n,
    );

    let ballots = vec![ballot1, ballot2, ballot3];
    let profile = PreferenceProfile::new(n, ballots);

    let margins = tally_margins(&profile.ballots, profile.n);
    println!("Margin matrix:");
    for row in &margins {
        println!("  {:?}", row);
    }

    let lottery = maximal_lottery(&margins).expect("failed to compute maximal lottery");

    println!("\nMaximal lottery:");
    for (i, prob) in lottery.iter().enumerate() {
        println!("  Candidate {}: {}", i, prob);
    }
}
