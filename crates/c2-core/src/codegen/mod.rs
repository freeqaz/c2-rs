//! PPC (Xbox 360, big-endian) instruction selection.
//!
//! `.text` payload is stored **big-endian** (unlike the little-endian COFF
//! struct fields). Bit numbering in the encoders is IBM/PPC convention
//! (bit 0 = MSB).
//!
//! # The split
//!
//! This was one 4,718-line file until `docs/ARCHITECTURE_SEAMS.md` §2.1. It was
//! contended all day on 2026-07-30 — but almost never for the same *fact*: one
//! agent would be in a leaf lowering while another was in the frame path. That
//! is file-shaped contention, and the split removes it. Nothing moved between
//! modules semantically; every public path (`c2_core::codegen::encode_add`,
//! `…::select_function`, …) is preserved by the re-exports below.
//!
//! | module | what lives there | who edits it |
//! |---|---|---|
//! | [`encode`] | ~50 word encoders, no dependencies | anyone, additively |
//! | [`labels`] | the label→offset map (`CFG_SHAPE.md` §6.2 item B) | any rung that emits a branch |
//! | [`select`] | the ordered dispatch + `fits_i16`/`out_of_class`/`ARG_REGS` | every rung, one line |
//! | [`straightline`] | `select_text`/`combine`/the depth-2 tree — one unit | the integer lane |
//! | [`leaf`] | one file per leaf shape, each self-contained | the leaf lane, one file per rung |
//! | [`frame`] | `FrameLayout`, the captured thresholds | **the serial spine** |
//! | [`calls`] | framed bodies, call sequences, tail calls, permutation | **the serial spine** |
//! | [`store_run_call`] | board #844's composition seam: a scheduled run as the MIDDLE of a framed body | **the serial spine** |
//!
//! A new leaf rung touches: its own new `leaf/<shape>.rs`, one arm in
//! [`select`], one `pub mod` line — and nothing else in `c2-core`.

pub mod alloc;
pub mod calls;
pub mod cond_tail;
pub mod div_mod_leaf;
pub mod encode;
pub mod frame;
/// A MEASUREMENT, not an emitter — see the module header. `cfg(test)` only, so
/// it cannot be reached from a release build even by accident.
#[cfg(test)]
pub(crate) mod frontier_bytes;
pub mod alloc_init_or_fail;
pub mod guard_ret_chain;
pub mod osf_handle_guard;
pub mod xlrc_create_guard;
pub mod json_utf8_copy;
pub mod guard_chain_shared_tail;
pub mod counted_accum_loop;
pub mod float_walk_loop;
pub mod ctor_forward_call;
pub mod fp_store_diamond;
pub mod if_call_join;
pub mod memcpy_tail;
pub mod nonce_add_run;
pub mod xtea_round_loop;
pub mod labels;
pub mod leaf;
pub mod order;
pub mod pool_ctor_chain;
pub mod pool_free_list;
pub mod ptr_walk_chain_loop;
pub mod ptr_walk_loop;
pub mod schedule;
pub mod select;
pub mod static_scan_loop;
pub mod store_run_call;
pub mod straightline;
#[cfg(test)]
pub(crate) mod testutil;

// Re-exports: every path that worked against the single-file
// `codegen.rs` still works. The split is a pure move.
pub use calls::*;
pub use cond_tail::*;
pub use encode::*;
pub use frame::*;
pub use labels::*;
pub use leaf::*;
pub use select::*;
pub use straightline::*;

// Items that were private to the single file and are now referenced
// across modules. NOT public: the crate-external surface is unchanged.
#[allow(unused_imports)]
pub(crate) use encode::encode_ldr;
#[allow(unused_imports)]
pub(crate) use select::{ARG_REGS, RET_REG, SCRATCH_REG, fits_i16, out_of_class};
#[allow(unused_imports)]
pub(crate) use straightline::{Base, emit_load_imm};
