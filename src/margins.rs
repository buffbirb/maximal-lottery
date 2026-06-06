use crate::ballot::{CompactBallot, PairPreference};

pub fn tally_margins(ballots: &[CompactBallot], n: usize) -> Vec<Vec<i64>> {
    let mut margins = vec![vec![0i64; n]; n];

    for ballot in ballots {
        for i in 0..n {
            for j in (i + 1)..n {
                match ballot.preference(i, j, n) {
                    PairPreference::Left => {
                        margins[i][j] += 1;
                        margins[j][i] -= 1;
                    }
                    PairPreference::Right => {
                        margins[i][j] -= 1;
                        margins[j][i] += 1;
                    }
                    PairPreference::Abstain => {}
                }
            }
        }
    }

    margins
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tally_empty() {
        let margins = tally_margins(&[], 3);
        assert_eq!(margins, vec![vec![0; 3]; 3]);
    }

    #[test]
    fn test_tally_single_ballot() {
        let n = 3;
        // 0 > 1, 0 > 2, 1 > 2
        let ballot = CompactBallot::from_pairs(
            &[
                (0, 1, PairPreference::Left),
                (0, 2, PairPreference::Left),
                (1, 2, PairPreference::Left),
            ],
            n,
        );
        let margins = tally_margins(&[ballot], n);
        assert_eq!(margins[0][1], 1);
        assert_eq!(margins[1][0], -1);
        assert_eq!(margins[0][2], 1);
        assert_eq!(margins[2][0], -1);
        assert_eq!(margins[1][2], 1);
        assert_eq!(margins[2][1], -1);
    }
}
