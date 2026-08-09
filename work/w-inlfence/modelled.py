#!/usr/bin/env python3
"""w-inlfence — widen the exemption from "reduces to nothing" to "the port has a
MODEL of this callee at all".

The fixpoint version was refuted by `empty_elision.rs`'s c19 cell: mechanism I
(`c2_core::splice`, SPLICE-0) splices a LOWERABLE callee's body into its caller
and is graded 723 of 723 byte-exact, and the fence was refusing it. Elide's
population is REFUSED callees and splice's is IN-CLASS ones, so no single one of
the two exemptions covers both.
"""
p = "crates/c2-il/src/func/census.rs"
s = open(p).read()

s = s.replace(
    """/// **W-INLFENCE — the same-TU callees that reduce to NOTHING**, restated over
/// the census's own vocabulary.""",
    """/// **W-INLFENCE — the same-TU callees the port has a MODEL of**, so the inline
/// fence can refuse only the ones it does not.""",
    1,
)

s = s.replace(
    """/// `c2_core::elide::TuEmptyCallees` is the owner of that reduction and this is a
/// **second implementation of it**, which is a real cost and is stated rather
/// than hidden: `c2-core` depends on `c2-il` and not the other way round, so the
/// census cannot call it. What keeps the two in agreement is that
/// `crates/c2-harness/tests/dead_temp_elision.rs`'s four chain cells and
/// `crates/c2-harness/tests/call_targets.rs`'s locator cell fail loudly if this
/// one is narrower — a **depth-1** version of this function was written first
/// and all five of them caught it.""",
    """**And mechanism I** (`c2_core::splice`, SPLICE-0) says a call to a callee this
/// TU defines and that the port can LOWER is replaced by that callee's own
/// emitted body, graded **723 of 723 byte-exact**. The two populations are
/// disjoint in the worst way for a single rule: E's callees are rows the parser
/// REFUSED and I's are rows it ACCEPTED, so an exemption that covers one covers
/// neither.
///
/// So the set below is the union — *reduces to nothing* **or** *the port can
/// lower it* — and the fence refuses only a callee the port has **no** model of,
/// which is the honest statement of what it is for: c2 may inline, and here
/// nobody knows what that produces.
///
/// `c2_core::elide::TuEmptyCallees` is the owner of the first half and this is a
/// **second implementation of it**, which is a real cost and is stated rather
/// than hidden: `c2-core` depends on `c2-il` and not the other way round, so the
/// census cannot call it. What keeps the two in agreement is that six standing
/// integration cells fail loudly if this one is narrower —
/// `dead_temp_elision.rs`'s four chains, `call_targets.rs`'s locator and
/// `empty_elision.rs`'s c19. A **depth-1** version of this function was written
/// first and five of them caught it; a *reduces-to-nothing only* version was
/// written second and c19 caught that.
///
/// The second half is deliberately **not** a re-statement of SPLICE-0's own
/// refusals (`splice.rs`'s S1–S6). It is *"the callee's body is one the port
/// lowers"*, which is broader, so a callee the splice declines is exempted here
/// and the port keeps its `bl`. That is the pre-existing behaviour and it is a
/// named residue, not a claim (`docs/rungs/2026-08-09-w-inlfence.md` §6).""",
    1,
)

s = s.replace(
    """/// * **a name two segments disagree about contributes neither**, exactly as
///   `TuContext::of_rows` drops it.""",
    """/// * **lowerable** — the segment parses whole, `shape_to_function` resolves
///   every token in it, and its optimization-settings word is one the port
///   emits under. Asked WITHOUT the inline fence, which is what keeps this
///   non-recursive: it is a statement about the callee's own body, not about
///   whether the callee would itself be admitted.
/// * **a name two segments disagree about contributes neither**, exactly as
///   `TuContext::of_rows` drops it.""",
    1,
)

s = s.replace("fn tu_reduces_to_nothing(", "fn tu_modelled_callees(", 1)
s = s.replace("tu_reduces_to_nothing(\n", "tu_modelled_callees(\n", 1)

old = """        let Some(f) = shape_to_function(
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
        } else if f.data_syms.is_empty()"""
new = """        let Some(f) = shape_to_function(
            sh,
            &bind.name_for_shape(j),
            src,
            resolve,
            resolve_data,
            resolve_data_def,
        ) else {
            continue;
        };
        // **Mechanism I's half.** The body parses whole, every token in it
        // resolves, and the mode is one the port emits under: the splice has a
        // body to substitute, so the caller is not guessing.
        if opt_word_mode(opt_word_at(s2)).is_some() {
            lowerable.insert(n.to_string());
        }
        if f.empty_body {
            seed.insert(n.to_string());
        } else if f.data_syms.is_empty()"""
assert old in s
s = s.replace(old, new, 1)

s = s.replace(
    """    let mut seed: BTreeSet<String> = BTreeSet::new();
    let mut link: BTreeMap<String, String> = BTreeMap::new();""",
    """    let mut seed: BTreeSet<String> = BTreeSet::new();
    let mut lowerable: BTreeSet<String> = BTreeSet::new();
    let mut link: BTreeMap<String, String> = BTreeMap::new();""",
    1,
)
s = s.replace(
    """    for n in &conflict {
        seed.remove(n);
        link.remove(n);
    }""",
    """    for n in &conflict {
        seed.remove(n);
        link.remove(n);
        lowerable.remove(n);
    }""",
    1,
)
s = s.replace(
    """        seed.extend(step);
    }
    seed
}""",
    """        seed.extend(step);
    }
    seed.extend(lowerable);
    seed
}""",
    1,
)
open(p, "w").write(s)
print("ok")
