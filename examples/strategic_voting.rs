use maximal_lottery::ballot::{PairPreference, PairwiseBallot};
use maximal_lottery::display::{print_ballot, print_lottery, print_margins};
use maximal_lottery::prelude::*;

fn main() {
    let n = 3;
    let c = Candidate;

    // Voter B's ballot expresses a cycle: 0 > 1 > 2 > 0.
    let voter_b = PairwiseBallot::from_pairs(
        &[
            (c(0), c(1), PairPreference::Left),
            (c(0), c(2), PairPreference::Right),
            (c(1), c(2), PairPreference::Left),
        ],
        n,
    )
    .unwrap();

    // Voter A's true preference is 0 > 1 > 2, but they strategically abstain
    // on the 0-vs-1 comparison.
    let voter_a_strategic = PairwiseBallot::from_pairs(
        &[
            (c(0), c(1), PairPreference::Abstain),
            (c(0), c(2), PairPreference::Left),
            (c(1), c(2), PairPreference::Left),
        ],
        n,
    )
    .unwrap();

    // Voter A's faithful ballot: 0 > 1 > 2.
    let voter_a_faithful = PairwiseBallot::from_pairs(
        &[
            (c(0), c(1), PairPreference::Left),
            (c(0), c(2), PairPreference::Left),
            (c(1), c(2), PairPreference::Left),
        ],
        n,
    )
    .unwrap();

    let run = |label: &str, ballots: Vec<PairwiseBallot>| {
        println!("=== {} ===\n", label);

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
        println!();
    };

    run(
        "Strategic abstention (A withholds 0-vs-1)",
        vec![voter_a_strategic, voter_b.clone()],
    );
    run(
        "Faithful voting (A reports true 0 > 1 > 2)",
        vec![voter_a_faithful, voter_b],
    );
}
