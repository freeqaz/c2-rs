//! **The `.text` EMISSION ORDER** — which function c2 writes first.
//!
//! The port used to emit the TU's functions in `.ex` order, which is the order
//! the front end wrote them and (usually) the order they are defined in the
//! source. That is right most of the time and **silently wrong** the rest: the
//! obj still links, the relocations still resolve, and only a byte compare says
//! anything. It was a live `Port=Mismatch` on master (board row **X-d**), and
//! the family is bigger than the one case that surfaced it — see
//! `docs/rungs/2026-08-04-w-order.md` §2 for the six reproducers.
//!
//! # The rule, as measured
//!
//! c2 emits a function only once every function it **references** that is also
//! **defined in this TU** has already been emitted, scanning the `.ex` order
//! repeatedly until nothing more becomes ready:
//!
//! ```text
//!   remaining := functions in .ex order
//!   repeat:
//!       scan remaining forwards; emit each whose local references are all emitted
//!   until a whole pass emits nothing
//! ```
//!
//! Three properties of that loop are each load-bearing and each measured:
//!
//! * **It is stable.** A function whose references are already satisfied does
//!   not move. `void g(){} void f(){g();}` stays `g, f`, and — the sharper
//!   control — `void g(){} void f(){g();} int h(int a){return a+1;}` stays
//!   `g, f, h`: `f` is a *caller* and does **not** get pushed to the end. That
//!   rules out "defer every caller", which fits the first four probes equally.
//! * **It takes as many passes as the chain is deep.** `a→b→c` written
//!   `a, b, c` comes out `c, b, a`, not `c, a, b`: one deferral pass is not
//!   enough. (Equivalently: it is not a DFS from each root either — a DFS of
//!   `f, h, g` with `f→g` gives `g, f, h` and c2 gives `h, g, f`.)
//! * **A self-reference is not an edge.** `void f(int n){if(n)f(n-1);}` beside a
//!   leaf stays in source order.
//!
//! # What counts as a reference — and why it is NOT the relocation list
//!
//! The edge that produced the original defect **emits no bytes at all**. In
//!
//! ```cpp
//! struct B { B(); ~B(); int x; };
//! struct D : B { D(); };
//! D::D() {}
//! B::~B() {}
//! ```
//!
//! `??0D`'s only relocation is a `bl` to `??0B`, which is *undefined* here;
//! there is no reference to `??1B` anywhere in the obj — no `bl`, no
//! relocation, no symbol. c2 emits `??1B` first anyway, because `??0D`'s IL
//! carries a `26` **unwind action** naming it
//! (`c2_il::IlFunction::eh_unwind_callees`). So the caller must pass the IL's
//! reference set, not the obj's; a planner fed the relocation list alone gets
//! this case wrong and looks right on every other probe in the grid.
//!
//! # A stall REFUSES
//!
//! A cycle (mutual recursion) stalls the loop, and c2 does something else
//! entirely there — it folds the recursion, and the three cycle probes in
//! `work/w-order/p/g*.cpp` do not agree with any single tie-break rule
//! (`a→b→c→a` written `a, b, c` comes out `b, a, c`, while `a↔b` beside a leaf
//! comes out unpermuted with the leaf **last**, which the loop above cannot
//! produce at all). So a stall returns `None` and the caller refuses. One
//! witness either way is not a rule.
//!
//! # Why this is not "fitted"
//!
//! With an empty edge set the loop is the identity permutation, which is
//! exactly what the port did before. A reference the caller fails to report can
//! therefore only degrade to the old behaviour; only a *spurious* edge can make
//! it worse, and edges come only from names the decoder actually read out of a
//! body. That asymmetry is why this is a correction and not a gamble.

/// Plan the order in which the TU's functions are emitted into `.text`.
///
/// `names[i]` is function `i`'s mangled name and `refs[i]` the mangled names
/// its IL references — callees **and** unwind-action names. Names not in
/// `names` (undefined externals) are ignored, as is a function's reference to
/// itself.
///
/// Returns the permutation of `0..names.len()` to emit, or `None` if the
/// reference graph has a cycle among the TU's own functions.
pub fn plan_text_order(names: &[&str], refs: &[Vec<&str>]) -> Option<Vec<usize>> {
    debug_assert_eq!(names.len(), refs.len());
    let n = names.len();

    // Resolve each reference to a function index once. A name that is not
    // defined in this TU, and a self-reference, contribute no edge.
    let mut deps: Vec<Vec<usize>> = Vec::with_capacity(n);
    for (i, r) in refs.iter().enumerate() {
        let mut d: Vec<usize> = Vec::new();
        for callee in r {
            if let Some(j) = names.iter().position(|nm| nm == callee) {
                if j != i && !d.contains(&j) {
                    d.push(j);
                }
            }
        }
        deps.push(d);
    }

    let mut emitted = vec![false; n];
    let mut order: Vec<usize> = Vec::with_capacity(n);
    while order.len() < n {
        let before = order.len();
        for i in 0..n {
            if emitted[i] {
                continue;
            }
            if deps[i].iter().all(|&j| emitted[j]) {
                emitted[i] = true;
                order.push(i);
            }
        }
        if order.len() == before {
            // Stalled: a cycle. Refuse rather than pick a tie-break with one
            // witness — see the module docs.
            return None;
        }
    }
    Some(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `plan_text_order` over a little adjacency description, so the cases below
    /// read like the probes they come from.
    fn plan(spec: &[(&str, &[&str])]) -> Option<Vec<String>> {
        let names: Vec<&str> = spec.iter().map(|(n, _)| *n).collect();
        let refs: Vec<Vec<&str>> = spec.iter().map(|(_, r)| r.to_vec()).collect();
        plan_text_order(&names, &refs)
            .map(|o| o.into_iter().map(|i| names[i].to_string()).collect())
    }

    /// **The identity property.** No edges => the `.ex` order, unchanged. This is
    /// the pin that says the planner cannot regress a TU it has no information
    /// about: every fixture that matched before this file existed has an empty
    /// edge set or an already-topological one.
    #[test]
    fn no_edges_is_the_identity() {
        assert_eq!(
            plan(&[("f", &[]), ("g", &[]), ("h", &[])]).unwrap(),
            ["f", "g", "h"]
        );
    }

    /// An edge to a name this TU does not define is not an edge. `a3_call_then_leaf`:
    /// `void v0(); void f(){v0();} int g(int){…}` stays in source order.
    #[test]
    fn undefined_callee_is_not_an_edge() {
        assert_eq!(
            plan(&[("f", &["v0"]), ("g", &[])]).unwrap(),
            ["f", "g"]
        );
    }

    /// `d2_plain_call_fwd` — the minimum: `void f(){g();} void g(){}` emits
    /// `g, f`.
    #[test]
    fn callee_defined_later_moves_ahead() {
        assert_eq!(plan(&[("f", &["g"]), ("g", &[])]).unwrap(), ["g", "f"]);
    }

    /// `e1_gfh` — **the stability control**. `f` calls `g`, `g` is already
    /// ahead of it, and `f` does NOT get pushed behind the unrelated `h`.
    /// "Defer every caller to the end" fits every other case here and fails this
    /// one.
    #[test]
    fn a_satisfied_caller_does_not_move() {
        assert_eq!(
            plan(&[("g", &[]), ("f", &["g"]), ("h", &[])]).unwrap(),
            ["g", "f", "h"]
        );
    }

    /// `e2_fhg` / `e3_hfg` — the deferred function goes to the END of the pass,
    /// behind functions that were after it in the `.ex` order.
    #[test]
    fn a_deferred_caller_lands_after_everything_ready() {
        assert_eq!(
            plan(&[("f", &["g"]), ("h", &[]), ("g", &[])]).unwrap(),
            ["h", "g", "f"]
        );
        assert_eq!(
            plan(&[("h", &[]), ("f", &["g"]), ("g", &[])]).unwrap(),
            ["h", "g", "f"]
        );
    }

    /// `e4_abc` — **the multi-pass pin**. A single deferral pass would produce
    /// `c, a, b`; c2 produces `c, b, a`, and so does this.
    #[test]
    fn a_chain_needs_one_pass_per_link() {
        assert_eq!(
            plan(&[("a", &["b"]), ("b", &["c"]), ("c", &[])]).unwrap(),
            ["c", "b", "a"]
        );
        // `e6_bac`: the same graph written in a third order lands the same way.
        assert_eq!(
            plan(&[("b", &["c"]), ("a", &["b"]), ("c", &[])]).unwrap(),
            ["c", "b", "a"]
        );
    }

    /// `g8_common` — two callers of one callee keep their relative `.ex` order.
    #[test]
    fn two_callers_of_one_callee_keep_their_order() {
        assert_eq!(
            plan(&[("f", &["h"]), ("g", &["h"]), ("h", &[])]).unwrap(),
            ["h", "f", "g"]
        );
    }

    /// `g7_two_calls` — the same callee named twice is one edge, not two.
    #[test]
    fn a_repeated_callee_is_one_edge() {
        assert_eq!(
            plan(&[("f", &["g", "g"]), ("g", &[])]).unwrap(),
            ["g", "f"]
        );
    }

    /// `g1_self_rec` — a self-reference is not an edge, so a recursive function
    /// does not deadlock the plan.
    #[test]
    fn self_reference_is_not_an_edge() {
        assert_eq!(
            plan(&[("f", &["f"]), ("z", &[])]).unwrap(),
            ["f", "z"]
        );
    }

    /// `g4_cycle_plus` / `g3_cycle3` — a cycle REFUSES. The caller must return
    /// `NotImplemented`, not guess a tie-break.
    #[test]
    fn a_cycle_refuses() {
        assert!(plan(&[("a", &["b"]), ("b", &["a"])]).is_none());
        assert!(plan(&[("a", &["b"]), ("b", &["c"]), ("c", &["a"])]).is_none());
        // …including when part of the TU is perfectly orderable. `g4` shows c2
        // does something this planner cannot express there (the unrelated leaf
        // comes out LAST), so the whole TU refuses.
        assert!(plan(&[("a", &["b"]), ("b", &["a"]), ("z", &[])]).is_none());
    }

    /// `c1_basedtor_here` — the edge that emits no bytes. `??0D`'s only
    /// relocation is to the *undefined* `??0B`; the ordering edge is the `26`
    /// unwind action naming `??1B`. Passing only the relocation list gives
    /// `??0D, ??1B` and the obj has `??1B, ??0D`.
    #[test]
    fn an_unwind_only_reference_is_an_edge() {
        // What the obj's relocations alone would say: no reordering.
        assert_eq!(
            plan(&[("??0D@@QAA@XZ", &["??0B@@QAA@XZ"]), ("??1B@@QAA@XZ", &[])]).unwrap(),
            ["??0D@@QAA@XZ", "??1B@@QAA@XZ"]
        );
        // What the IL says, and what c2 emits.
        assert_eq!(
            plan(&[
                ("??0D@@QAA@XZ", &["??0B@@QAA@XZ", "??1B@@QAA@XZ"]),
                ("??1B@@QAA@XZ", &[])
            ])
            .unwrap(),
            ["??1B@@QAA@XZ", "??0D@@QAA@XZ"]
        );
    }

    /// `f4_unwind_mid` — the three-function form of the same, registered as a
    /// prediction before it was compiled and confirmed by the oracle.
    #[test]
    fn unwind_edge_with_an_unrelated_function_between() {
        assert_eq!(
            plan(&[
                ("??0D@@QAA@XZ", &["??0B@@QAA@XZ", "??1B@@QAA@XZ"]),
                ("?k@@YAXXZ", &["?h@@YAXXZ"]),
                ("??1B@@QAA@XZ", &[]),
            ])
            .unwrap(),
            ["?k@@YAXXZ", "??1B@@QAA@XZ", "??0D@@QAA@XZ"]
        );
    }

    /// A one-function TU and an empty TU are both the identity, and neither
    /// stalls.
    #[test]
    fn degenerate_inputs() {
        assert_eq!(plan(&[]).unwrap(), Vec::<String>::new());
        assert_eq!(plan(&[("f", &["f"])]).unwrap(), ["f"]);
    }
}
