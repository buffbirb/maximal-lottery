use maximal_lottery::ballot::{CompactBallot, PairPreference};
use maximal_lottery::display::{print_ballot, print_lottery, print_margins};
use maximal_lottery::margins::tally_margins;
use maximal_lottery::maximal_lottery::{LotteryMethod, maximal_lottery};
use maximal_lottery::types::PreferenceProfile;

fn main() {
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

    // Voter 2: 0 < 1, 0 < 2, 1 > 2
    let ballot2 = CompactBallot::from_pairs(
        &[
            (0, 1, PairPreference::Right),
            (0, 2, PairPreference::Right),
            (1, 2, PairPreference::Left),
        ],
        n,
    );

    // Voter 3: 0 > 1, 0 < 2, 1 < 2
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

    println!("Ballots:\n");
    for (i, ballot) in profile.ballots.iter().enumerate() {
        print_ballot(ballot, n, &format!("Ballot {}:", i + 1));
    }

    let margins = tally_margins(&profile.ballots, profile.n);
    print_margins(&margins);

    let lottery = maximal_lottery(&margins, LotteryMethod::Centroid)
        .expect("failed to compute maximal lottery");
    print_lottery(&lottery, &margins);
}
