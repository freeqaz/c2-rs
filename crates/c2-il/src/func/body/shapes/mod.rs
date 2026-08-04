//! Body-shape recognizers, one file per shape.
//!
//! This was one 4,599-line file until `docs/ARCHITECTURE_SEAMS.md` §2.2. The
//! `try_parse_*` recognizers are deliberately **non-committal** — cursor copy,
//! `Option` return, no side effects — so they are independent by construction,
//! and their adjacency in one file bought nothing but merge conflicts whose
//! hunks shared a closing brace (that brace was the single 1,242-line
//! `mod tests`'). Each shape now ends at a file boundary and carries its own
//! tests; a new rung is a new file plus one line in
//! [`super::parse_segment_shape`]'s ladder.
//!
//! The genuinely **shared** facts are small and are named modules with named
//! consumers, which is the review affordance §4.1 asks for — a rung file that
//! parses an offset chain without importing [`designator`] is visibly
//! reinventing it:
//!
//! * [`designator`] — how a byte offset into an object is spelled (four consumers)
//! * [`this_binding`] — the implicit first argument (the line-70 lesson)
//! * [`params`] — the formal list, read once
//! * [`calls`] — the unified call shape, the ONE copy (GAPS §6 instance #9)
//!
//! `super::parse_segment_shape` stays the owner of the dispatch **order**,
//! which is load-bearing and must stay readable in one screen (§3.2).


pub(crate) mod assign;
pub(crate) mod calls;
pub(crate) mod cond_tail;
pub(crate) mod control_flow;
pub(crate) mod ctor_dtor;
pub(crate) mod designator;
pub(crate) mod early_return;
pub(crate) mod guarded_seq;
pub(crate) mod leaf_addr;
pub(crate) mod leaf_compare;
pub(crate) mod leaf_float;
pub(crate) mod leaf_fp_tail;
pub(crate) mod leaf_load;
pub(crate) mod leaf_store;
pub(crate) mod mcall_chain;
pub(crate) mod mcall_cmp;
pub(crate) mod mcall_tail;
pub(crate) mod params;
pub(crate) mod this_binding;
#[cfg(test)]
pub(crate) mod testutil;

// Re-exports: every `shapes::<name>` path that worked against the
// single-file `shapes.rs` still works. The split is a pure move.
#[allow(unused_imports)]
pub(crate) use assign::*;
#[allow(unused_imports)]
pub(crate) use calls::*;
#[allow(unused_imports)]
pub(crate) use cond_tail::*;
#[allow(unused_imports)]
pub(crate) use control_flow::*;
#[allow(unused_imports)]
pub(crate) use ctor_dtor::*;
#[allow(unused_imports)]
pub(crate) use designator::*;
#[allow(unused_imports)]
pub(crate) use early_return::*;
#[allow(unused_imports)]
pub(crate) use guarded_seq::*;
#[allow(unused_imports)]
pub(crate) use leaf_addr::*;
#[allow(unused_imports)]
pub(crate) use leaf_compare::*;
#[allow(unused_imports)]
pub(crate) use mcall_chain::*;
#[allow(unused_imports)]
pub(crate) use mcall_cmp::*;
#[allow(unused_imports)]
pub(crate) use leaf_float::*;
#[allow(unused_imports)]
pub(crate) use leaf_fp_tail::*;
#[allow(unused_imports)]
pub(crate) use leaf_load::*;
#[allow(unused_imports)]
pub(crate) use leaf_store::*;
pub(crate) use mcall_tail::*;
#[allow(unused_imports)]
pub(crate) use params::*;
#[allow(unused_imports)]
pub(crate) use this_binding::*;

#[cfg(test)]
mod tests {
    //! Tests that live in `shapes` by history rather than by shape: the
    //! value class below is admitted by the straight-line chain and refused
    //! by the expression layer, so it has no recognizer of its own here.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::func::body::{parse_segment, parse_segment_detail, BodyShape};
    #[allow(unused_imports)]
    use crate::func::IlOp;
    #[allow(unused_imports)]
    use crate::func::bundle::LO_MARKER;
    #[allow(unused_imports)]
    use crate::func::readers::find_subslice;
    #[allow(unused_imports)]
    use crate::func::sy::{Formals, SyView};
    #[allow(unused_imports)]
    use crate::func::test_fixtures::*;
    /// W26: `bool` / `unsigned char` as a value class — free inside the class,
    /// and a real `rlwinm` on the way out of it.
    #[test]
    fn bool_value_class_is_free_inside_and_refuses_the_widening() {
        assert_eq!(
            parse_segment(BOOL_LIT, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![],
                ops: vec![IlOp::Lit(0)],
            })
        );
        assert_eq!(
            parse_segment(BOOL_ID, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0xE409],
                ops: vec![IlOp::Load(0xE409)],
            })
        );
        // The conversion OUT of the class is `clrlwi r3,r3,24`, and it arrives as
        // the same `2C … 00` that is free between the two width-4 classes. It must
        // refuse in the PARSER, under a key that names the target.
        assert_eq!(parse_segment(BOOL_WIDEN_NEG, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(BOOL_WIDEN_NEG, NO_LOCALS)
                .unwrap_err()
                .feature(),
            "expr-convert-target-8641"
        );
    }

}
