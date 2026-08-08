//! `c2rs` subcommand handlers, one module per command group.
//!
//! Split out of `main.rs` (lane `w-mod`) purely to bound the file size; the
//! dispatch `match`, `print_usage` and — load-bearing — `mod argv` all stay in
//! the crate root. `mod argv` is the binary's ONLY argument parser and
//! `Args::toolchain` its ONLY producer of a `Toolchain`, so every handler here
//! reaches the toolchain through a `&self` on an already-parsed, already-
//! validated `Args`. "Parse and validate, THEN locate" stays the only order this
//! binary can express (boards #194/#195), and
//! `tests/cli_flags.rs::locate_is_reachable_only_through_the_arg_seam` now scans
//! `main.rs` *and every file in this directory* to keep it that way.

pub(crate) mod census;
pub(crate) mod corpus;
pub(crate) mod factors;
pub(crate) mod gap;
pub(crate) mod listing;
pub(crate) mod perf;
pub(crate) mod prefilter;
pub(crate) mod reference;
pub(crate) mod retrieve;
pub(crate) mod search;
pub(crate) mod util;
