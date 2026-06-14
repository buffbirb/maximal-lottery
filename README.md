# maximal-lottery

A Rust library for computing maximal lotteries from preference profiles.

Maximal lottery is a probabilistic voting rule: voters express pairwise
preferences between candidates, the profile is turned into a margin matrix,
and that matrix is treated as a symmetric zero-sum game. A maximal lottery
is a maximin strategy of that game.

## Quick start

Add this to your `Cargo.toml`:

```toml
[dependencies]
maximal-lottery = "0.1"
```

```rust
use maximal_lottery::ballot::{PairPreference, PairwiseBallot};
use maximal_lottery::prelude::*;

let n = 3;
let c = Candidate;
let ballot = PairwiseBallot::from_pairs(&[
    (c(0), c(1), PairPreference::Left),
    (c(1), c(2), PairPreference::Left),
    (c(0), c(2), PairPreference::Left),
], n)
.unwrap();

let profile = PreferenceProfile::try_new(vec![ballot]).unwrap();
let margins = profile.tally_margins();
let lottery = CentroidSolver.solve(&margins).unwrap();
```

## Documentation

Run `cargo doc --open` to view the API reference locally, or visit
[docs.rs/maximal-lottery](https://docs.rs/maximal-lottery) once published.

## Examples

See the `examples/` directory for runnable programs.
