#!/usr/bin/env python3
"""w-inlfence — replace the depth-1 exemption with the fixpoint `elide` uses.

The depth-1 set was refuted by four standing tests in `dead_temp_elision.rs`
whose chains close one and two links deeper. Under-exempting there is not a
"safe direction": it refuses bodies the judge grades byte-exact.
"""
p = "crates/c2-il/src/func/census.rs"
s = open(p).read()

helper = '''
/// **W-INLFENCE — the same-TU callees that reduce to NOTHING**, restated over
/// the census's own vocabulary.
///
/// # Why this exists, and the cost of it existing
///
/// The inline fence ([`super::bind::callee_defined_here`]) refuses a body whose
/// callee this TU defines, because c2 may inline it. **The port is not silent
/// about every inline**: mechanism E (`c2_core::elide`) says a call to a callee
/// that emits nothing costs no branch at all, and the judge grades that
/// **1,877 of 1,877 byte-exact** on the 878-TU workload. Refusing those in the
/// census would refuse bodies the port provably gets right.
///
/// `c2_core::elide::TuEmptyCallees` is the owner of that reduction and this is a
/// **second implementation of it**, which is a real cost and is stated rather
/// than hidden: `c2-core` depends on `c2-il` and not the other way round, so the
/// census cannot call it. What keeps the two in agreement is that
/// `crates/c2-harness/tests/dead_temp_elision.rs`'s four chain cells and
/// `crates/c2-harness/tests/call_targets.rs`'s locator cell fail loudly if this
/// one is narrower — a **depth-1** version of this function was written first
/// and all five of them caught it.
///
/// # The rule, mirrored clause for clause from `elide.rs`
///
/// * **seeds** — [`IlFunction::empty_body`], and a refused row whose grammar
///   proves it emits nothing at all (`no_effect_nothing`, board #1053).
/// * **links** — a refused row that emits nothing but one call
///   (`no_effect_call` / `no_effect_loop`), and a parsed body that is a bare
///   tail call: no data symbol, no `framed_call`, no `call_seq`, no `cond_pair`
///   (`elide::elidable_step`, whose doc gives the graded reason each of those
///   four disqualifies).
/// * **a name two segments disagree about contributes neither**, exactly as
///   `TuContext::of_rows` drops it.
///
/// Keyed on [`EmitBinding::name`], which is the key
/// `c2_harness::gap::fnbytes::tu_empty_callees` feeds the real context with, so
/// the two cannot key one function two ways.
#[allow(clippy::too_many_arguments)]
fn tu_reduces_to_nothing(
    segs: &[&[u8]],
    bind: &Bindings,
    emit: &EmitBinding,
    src: &Option<String>,
    resolve: &dyn Fn(u32) -> Option<String>,
    resolve_data: &dyn Fn(u32) -> Option<String>,
    resolve_data_def: &dyn Fn(u32) -> Option<crate::func::IlDataDef>,
) -> std::collections::BTreeSet<String> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut seed: BTreeSet<String> = BTreeSet::new();
    let mut link: BTreeMap<String, String> = BTreeMap::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut conflict: BTreeSet<String> = BTreeSet::new();
    for (j, s2) in segs.iter().enumerate() {
        let Some(n) = emit.name(j) else { continue };
        if !seen.insert(n.to_string()) {
            conflict.insert(n.to_string());
            continue;
        }
        if body::shapes::no_effect::no_effect_nothing(s2) {
            seed.insert(n.to_string());
            continue;
        }
        if let Some(c) = body::shapes::no_effect::no_effect_call(s2)
            .or_else(|| body::shapes::no_effect::no_effect_loop(s2))
            .and_then(resolve)
        {
            link.insert(n.to_string(), c);
            continue;
        }
        let Ok(sh) = parse_segment_detail(s2, bind.locals(j)) else {
            continue;
        };
        let Some(f) = shape_to_function(
            sh,
            &bind.name_for_shape(j),
            src,
            resolve,
            resolve_data,
            resolve_data_def,
        ) else {
            continue;
        };
        if f.empty_body {
            seed.insert(n.to_string());
        } else if f.data_syms.is_empty()
            && f.framed_call.is_none()
            && f.call_seq.is_none()
            && f.cond_pair.is_none()
        {
            if let Some(c) = f.tail_call.as_deref() {
                link.insert(n.to_string(), c.to_string());
            }
        }
    }
    for n in &conflict {
        seed.remove(n);
        link.remove(n);
    }
    // The closure. `seed` only grows and is bounded by `link`, so this
    // terminates on a cycle instead of chasing it — `elide.rs`'s own cycle
    // re-derivation, and `a_cycle_of_dead_temporary_bodies_is_never_admitted`
    // is the cell that grades it.
    loop {
        let step: Vec<String> = link
            .iter()
            .filter(|(n, c)| !seed.contains(*n) && seed.contains(*c))
            .map(|(n, _)| n.clone())
            .collect();
        if step.is_empty() {
            break;
        }
        seed.extend(step);
    }
    seed
}
'''

anchor = "\nimpl IlBundle {"
assert anchor in s, "impl IlBundle anchor"
s = s.replace(anchor, helper + anchor, 1)

# swap the inline depth-1 builder for the call
old_start = s.index("                                            empty_here.get_or_init(|| {")
old_end = s.index("                                        )\n                                        .is_some() =>")
s = (
    s[:old_start]
    + """                                            empty_here.get_or_init(|| {
                                                tu_reduces_to_nothing(
                                                    &segs,
                                                    &bind,
                                                    &emit,
                                                    &src,
                                                    &resolve,
                                                    &resolve_data,
                                                    &resolve_data_def,
                                                )
                                            }),
"""
    + s[old_end:]
)
open(p, "w").write(s)
print("ok")
