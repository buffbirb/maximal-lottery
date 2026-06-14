use maximal_lottery::ballot::{PairPreference, PairwiseBallot};
use maximal_lottery::display::{print_ballot, print_lottery, print_margins};
use maximal_lottery::prelude::*;

fn main() {
    let n = 3;
    let c = Candidate;

    // Voters 1-3: 0 > 1 > 2
    let voter_a = PairwiseBallot::from_pairs(
        &[
            (c(0), c(1), PairPreference::Left),
            (c(0), c(2), PairPreference::Left),
            (c(1), c(2), PairPreference::Left),
        ],
        n,
    )
    .unwrap();

    // Voters 4-5: 1 > 2 > 0
    let voter_b = PairwiseBallot::from_pairs(
        &[
            (c(0), c(1), PairPreference::Right),
            (c(0), c(2), PairPreference::Right),
            (c(1), c(2), PairPreference::Left),
        ],
        n,
    )
    .unwrap();

    let ballots = vec![
        voter_a.clone(),
        voter_a.clone(),
        voter_a.clone(),
        voter_b.clone(),
        voter_b.clone(),
    ];
    let profile = PreferenceProfile::try_new(ballots).unwrap();

    println!("Ballots:\n");
    for (i, ballot) in profile.ballots().iter().enumerate() {
        print_ballot(ballot, &format!("Ballot {}:", i + 1));
    }

    let margins = profile.tally_margins();
    print_margins(&margins);

    let lottery = CentroidSolver
        .solve(&margins)
        .expect("failed to compute maximal lottery");
    print_lottery(&lottery, &margins);
}
