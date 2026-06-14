//! Compute maximal lotteries from preference profiles.
//!
//! Maximal lottery is a probabilistic voting rule that selects a lottery
//! over candidates by interpreting the pairwise margin matrix as a symmetric
//! zero-sum game and computing a maximin (Nash equilibrium) strategy.
//!
//! # Quick start
//!
//! ```
//! use maximal_lottery::ballot::{PairPreference, PairwiseBallot};
//! use maximal_lottery::prelude::*;
//!
//! let n = 3;
//! let c = Candidate;
//! let ballot = PairwiseBallot::from_pairs(&[
//!     (c(0), c(1), PairPreference::Left),
//!     (c(1), c(2), PairPreference::Left),
//!     (c(0), c(2), PairPreference::Left),
//! ], n)
//! .unwrap();
//!
//! let profile = PreferenceProfile::try_new(vec![ballot]).unwrap();
//! let margins = profile.tally_margins();
//! let lottery = CentroidSolver.solve(&margins).unwrap();
//! assert_eq!(lottery.len(), n);
//! ```
//!
//! See the `examples/` directory for more detailed usage.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod ballot;
pub mod display;
pub mod error;
pub mod prelude;
pub mod profile;
pub mod solver;
