//! **The sub-object designator** — how a byte offset into an object is spelled
//! in IL, and how wide the thing at that offset is.
//!
//! ONE locator for one fact, with four named consumers: the indirect-load leaf
//! ([`super::leaf_load`], `lwz`/`lbz`/`lhz`/`ld`), the address leaf
//! ([`super::leaf_addr`], `addi`), the store leaf ([`super::leaf_store`],
//! `stb`/`sth`/`stw`/`std`) and the generated destructor's member receiver
//! ([`super::ctor_dtor`]). A recognizer that parses an offset chain without
//! importing this module is reinventing it — `docs/GAPS.md` §6's mis-emit #11
//! was one rule with two copies, each missing a gate the other had.
//!
//! `SIZED_PTEE`/`SIZED_PTR` are **literal whitelists**, not derived predicates,
//! and their tests assert exactly that: a type byte the corpus has not
//! separated must refuse, not be admitted by a plausible-looking rule.

use crate::func::body::mcall;
use crate::func::readers::{
    eat, eat_byte, eat_int_like, read_token_var, read_type, read_varint, value_class,
};


/// Pointee TYPEs admitted at the `30` indirect-load position **beyond** the
/// 4-byte integer class ([`is_int4_type`]), as `(tag, kind, width, signed)`.
///
/// Required **literally, as pairs**, rather than computed from the tag's
/// width nibble: the width is stated twice in a TYPE — the tag's low nibble and
/// the kind's high nibble — and demanding both is a free discriminator against a
/// misaligned read landing on a plausible-looking byte. Every pair below has a
/// capture in `fixtures/cpp/w12_narrow_getters.cpp` (see that file's header for
/// the per-case witness); a tag not listed — notably `volatile` (`92`/`94`/`98`),
/// which no probe produced — refuses rather than being assumed to behave like the
/// `const` one.
///
/// `signed` is the pointee's own signedness (kind's low nibble 1 vs 2), which
/// matters only when a `2C` widens the value to `int`: an unsigned narrow load is
/// already zero-extended by `lbz`/`lhz`, a signed one is not.
pub(crate) const SIZED_PTEE: &[(u8, u8, u8, bool)] = &[
    (0x82, 0x11, 1, true),  // char / signed char        `30 82 11 70` / `… 10`
    (0xA2, 0x11, 1, true),  // const char                `30 a2 11 8e 20`
    (0x82, 0x12, 1, false), // unsigned char / bool      `30 82 12 20` / `… 30`
    (0xA2, 0x12, 1, false), // const unsigned char/bool  `30 a2 12 95 20`
    (0x84, 0x21, 2, true),  // short                     `30 84 21 11`
    (0xA4, 0x21, 2, true),  // const short               `30 a4 21 99 20`
    (0x84, 0x22, 2, false), // unsigned short / wchar_t  `30 84 22 21` / `… 71`
    (0xA4, 0x22, 2, false), // const unsigned short/wchar_t `30 a4 22 9b 20`
    (0x88, 0x81, 8, true),  // long long                 `30 88 81 13`
    (0xA8, 0x81, 8, true),  // const long long           `30 a8 81 9f 20`
    (0x88, 0x82, 8, false), // unsigned long long        `30 88 82 23`
    (0xA8, 0x82, 8, false), // const unsigned long long  `30 a8 82 … 20`
];

/// `(tag, kind)` of a **pointer whose tag carries the pointee's width** — the
/// shape the `27` byte-offset-add position uses (`27 82 43 f0 08` for `char *`,
/// `27 a4 43 9a 20` for `const short *`, `27 a8 43 a0 20` for `const long long *`).
/// The tag's const bit here does **not** track the loaded type's: a *non*-const
/// member function's getter carries `27 a2 43 f0 08` over a `30 82 11 70`
/// (`D::n_c()`), so both tags are listed for each width and neither implies
/// anything about the load.
pub(crate) const SIZED_PTR: &[(u8, u8, u8)] = &[
    (0x82, 0x43, 1),
    (0xA2, 0x43, 1),
    (0x84, 0x43, 2),
    (0xA4, 0x43, 2),
    (0x88, 0x43, 8),
    (0xA8, 0x43, 8),
];

/// `(width, signed)` of a [`SIZED_PTEE`] pair, or `None` — which is a refusal,
/// never "assume 4".
pub(crate) fn sized_ptee(tag: u8, kind: u8) -> Option<(u8, bool)> {
    SIZED_PTEE
        .iter()
        .find(|&&(t, k, _, _)| t == tag && k == kind)
        .map(|&(_, _, w, s)| (w, s))
}

/// Pointee width of a [`SIZED_PTR`] pair, or `None`.
pub(crate) fn sized_ptr_width(tag: u8, kind: u8) -> Option<u8> {
    SIZED_PTR
        .iter()
        .find(|&&(t, k, _)| t == tag && k == kind)
        .map(|&(_, _, w)| w)
}

/// The intrinsic-2117 designator alone: `(summed byte offset, object token, end)`.
///
/// Split out of [`try_parse_base_member_load`] so the two consumers of the same
/// address — the LOAD leaf (`return b;`) and the ADDRESS leaf (`return &b;`) —
/// share one decoder. `GAPS.md` §6's "one fact, one locator": a second copy is a
/// second place for the two-literal sum, the `66` descriptor walk or the header
/// bound to drift.
///
/// `ptr_ok` is the caller's rule for the three pointer TYPEs the production
/// carries (the `40` result, the object `B9`, and its `55` push), and it is a
/// *parameter* rather than a fixed predicate because the two consumers are not
/// equally constrained and merging them would change what the load path accepts:
///
/// * the LOAD path passes [`is_ptr_to_4`] — pointer to a **4-byte** object — and
///   is byte-for-byte the rule it had before this split;
/// * the ADDRESS path passes [`is_ptr_any`], because the member's width does not
///   reach the emitted instruction at all. MEASURED (`work/bma/probes/p2.cpp`):
///   the inherited `char`, `short`, `int`, `long long`, `float` and `double`
///   members each emit the identical single `addi`, and their designators carry
///   `A6 43`, `A4 43`, `A6 43`, `A6 43`, `A6 43`, `A6 43` — so the tag's width
///   nibble is not even a reliable statement of the pointee width here, which is
///   the second reason not to gate on it.
pub(crate) fn parse_base_member_designator(
    seg: &[u8],
    start: usize,
    ptr_ok: fn(u8, u8) -> bool,
) -> Option<(i32, u32, usize)> {
    /// `33 <int-like> 80 45 08 00 00` — the selector literal, wide form.
    const SELECTOR_2117: [u8; 5] = [0x80, 0x45, 0x08, 0x00, 0x00];
    /// Longest argument-header type list accepted. Two witnesses (`n` = 2 and 3)
    /// bound what is understood; a deeper list is refused rather than skipped on
    /// the assumption that the shape keeps repeating.
    const MAX_HEADER_REFS: u8 = 3;

    let mut p = start;
    // The selector, pushed as an int literal.
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) || !eat(seg, &mut p, &SELECTOR_2117)
    {
        return None;
    }
    // The intrinsic-call marker; its result is the member's address.
    if !eat_byte(seg, &mut p, 0x40) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !ptr_ok(tag, kind) {
        return None;
    }
    p += tw;
    // The argument header: `66 <n>` then n type references, skipped structurally
    // so a second inheritance step (n = 3) parses like the first.
    //
    // The refs are **LEB128 ids**, not a fixed two bytes each — see
    // [`super::mcall::eat_class_descriptor`], which owns that encoding and carries
    // the witnesses. This code stepped `2 * n` and so landed inside the second ref
    // of any descriptor with a wide id, which is every large translation unit;
    // `src/App.cpp` and `src/lazer/game/Game.cpp` carry `fb 8a 01`, `ff ff 01`,
    // `d3 80 02`. The bound on `n` stays here rather than moving into the decoder,
    // because it is this shape's acceptance rule and not part of the encoding.
    let n_refs = mcall::eat_class_descriptor(seg, &mut p)?;
    if n_refs == 0 || n_refs > MAX_HEADER_REFS {
        return None;
    }
    // Each argument is `<value> 55 <its type>`.
    if !eat_byte(seg, &mut p, 0x55) || !eat_int_like(seg, &mut p) {
        return None;
    }
    // arg 1 — the member's offset within its base.
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return None;
    }
    let member_off = read_varint(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x55) || !eat_int_like(seg, &mut p) {
        return None;
    }
    // arg 2 — the base's offset within the object. The address is the sum.
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return None;
    }
    let base_off = read_varint(seg, &mut p)?;
    let off = member_off.checked_add(base_off)?;
    if !eat_byte(seg, &mut p, 0x55) || !eat_int_like(seg, &mut p) {
        return None;
    }
    // arg 3 — the object pointer.
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (base_tok, w) = read_token_var(seg, p)?;
    p += w;
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !ptr_ok(tag, kind) {
        return None;
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x55) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !ptr_ok(tag, kind) {
        return None;
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x4C) {
        return None;
    }
    Some((off, base_tok, p))
}

/// `(tag, kind)` of a **pointer TYPE, whatever it points at** — the rule the
/// *address* productions use, where the pointee's width never reaches the
/// emitted instruction.
///
/// The two existing pointer predicates each answer a narrower question and
/// neither fits: [`is_ptr_to_4`] demands a 4-byte pointee (it gates a `lwz`),
/// and [`is_ptr4_kind`] demands one of four exact tags (it gates a pointer
/// *value* in a register). An address leaf needs neither, because
/// `addi rD,rBase,k` is the same word for every pointee.
///
/// Spelled as a literal whitelist rather than as nibble arithmetic, for the
/// reason [`is_ptr4_kind`]'s own comment gives — and the whitelist is the cross
/// product of two axes each independently witnessed:
///
/// * the tag's **cv bits** `0x20` (const) and `0x10` (volatile), all four
///   combinations, exactly as [`is_ptr4_kind`] already admits. `0xC6` and every
///   other tag with bit `0x40` set is **refused**: `readers.rs` records that the
///   bit occurs and no probe produced it here.
/// * the tag's **width nibble**, which is 2/4/6/8. It is *not* a dependable
///   statement of the pointee width in this position and that is precisely why
///   it is admitted rather than checked: MEASURED (`work/bma/probes/p2.cpp`,
///   `p1.cpp`) `char*` carries `86 43`, `short*` carries `84 43`, and
///   `long long*`, `float*` and `double*` all carry `86 43` — while all six
///   emit the identical single `addi`. Witnessed tags are `84`, `86`, `A4`,
///   `A6`; the other twelve are the same two axes crossed and are admitted on
///   that basis, which is a HYPOTHESIS about the encoding and not a capture.
///
/// `kind` must be exactly `0x43` — width nibble 4 (the pointer's own size on
/// this target) and class nibble 3 (a **data** pointer). `0x44` (a function or
/// code pointer) is refused: no probe produced one at an address-leaf position,
/// and a code pointer is the one case where "the pointee width does not matter"
/// has not been checked.
pub(crate) fn is_ptr_any(tag: u8, kind: u8) -> bool {
    const PTR_TAGS: [u8; 16] = [
        0x82, 0x84, 0x86, 0x88, // plain, width nibble 2/4/6/8
        0x92, 0x94, 0x96, 0x98, // volatile
        0xA2, 0xA4, 0xA6, 0xA8, // const
        0xB2, 0xB4, 0xB6, 0xB8, // const volatile
    ];
    PTR_TAGS.contains(&tag) && kind == 0x43
}

/// Consume a run of **byte-offset adds** applied to an address, summing them.
///
/// ```text
///   33 <int-like> k   27 <PTR>        a member offset, re-typing the address
///   33 <int-like> k   28 00 00        a subscript offset, not re-typing it
/// ```
///
/// The load leaf ([`try_parse_indirect_load_leaf`]) admits **at most one** of
/// these, because a second one there means a chained subscript whose lowering
/// needs `slwi`/`lwzx`. An *address* has no such limit: every add is folded into
/// the one `addi`'s displacement, and the whole run costs nothing extra.
/// MEASURED — `int* DR::pt2()` (`&t[2]` on an inherited array) is
/// `LIT(0) 28 · LIT(8) 28` and emits `addi r3,r3,16`; `&s->arr[2]` on a plain
/// struct is `LIT(40) 27 · LIT(8) 28` and emits `addi r3,r3,48`.
///
/// The `28` payload must be exactly `00 00`, the same fail-closed rule
/// [`try_parse_indirect_load_leaf`] states: those two bytes are `00 00` at every
/// captured site and their meaning is UNKNOWN.
///
/// Returns `None` — cursor untouched — on an overflowing sum. Stops without
/// consuming at the first token that is not an offset add, which is not a
/// failure: zero adds is the legitimate `return &p->Base::m;`.
pub(crate) fn eat_addr_offset_adds(seg: &[u8], p: &mut usize) -> Option<i32> {
    eat_offset_adds(seg, p).map(|(total, _)| total)
}

/// [`eat_addr_offset_adds`] with the **last `27`'s TYPE** preserved, which is the
/// one thing the LOAD side needs and the address side does not.
///
/// The walk is the same walk — one locator, three consumers (the address leaf,
/// the store leaf and the indirect-load leaf) — because a second copy of a
/// summing loop is a second place for the overflow check, the `28` payload rule
/// and the stop condition to drift. `GAPS.md` §6's "one fact, one locator", and
/// this module's header states it as the module's whole purpose.
///
/// Why the extra return value exists at all: `27` **re-types** the address, so
/// its tag is a second, independent statement of the width the following `30`
/// load will announce, and [`super::leaf_load::try_parse_indirect_load_leaf`]
/// requires the two to agree. `addi` is the same word for every pointee width,
/// so the address and store leaves have nothing to cross-check and ignore it.
///
/// Only the **last** one is reported, and that is not an approximation. An
/// intermediate `27` in a run re-types the address to a pointer to the enclosing
/// sub-object — `p->c.b.a.e[3]` walks `27 C* · 27 B* · 27 A* · 28` — and an
/// aggregate pointer's tag width nibble is the POINTER's alignment, not the
/// aggregate's size (MEASURED: `work/w34/probe/p3.cpp` types a pointer to a
/// 24,004-byte struct `86 43`). So an intermediate `27`'s tag says nothing about
/// what is finally loaded, and only the last one is in a position to.
pub(crate) fn eat_offset_adds(seg: &[u8], p: &mut usize) -> Option<(i32, Option<(u8, u8)>)> {
    walk_offset_adds(seg, p, None)
}

/// [`eat_offset_adds`] with the offset-add literals themselves — **the LIST, not
/// its sum** — appended to `out` in the order the walk consumes them.
///
/// **Board #908.** The shipping reader returns only `total`, and the fact five
/// lanes have now measured is not a function of the sum: it is *one offset-add
/// chain being a byte-exact PREFIX of another*. `&t->mid` walks `[96]` and
/// `&t->mid.lo[N]` walks `[96, 4N]`; the first list is a prefix of the second,
/// and the sums — `96` and `96 + 4N` — say nothing at all about that. This is
/// board #644 applied to the IL: **not one contiguous field, and not one number
/// either.**
///
/// `total` and `last_retype` are returned unchanged beside the list, so a caller
/// that wants both pays for one walk. The walk is [`walk_offset_adds`], shared
/// with [`eat_offset_adds`] — the module header's "one fact, one locator", and
/// the reason this is a sibling rather than a second summing loop with its own
/// copy of the overflow check, the `28` payload rule and the stop condition.
///
/// On `Some`, `out` holds exactly the literals the cursor advanced past. On
/// `None` — the overflow case — the cursor is where the last successful add left
/// it and `out`'s contents are **unspecified**; the caller discards both, which
/// is what every caller of [`eat_offset_adds`] already does with its cursor.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn eat_offset_adds_list(
    seg: &[u8],
    p: &mut usize,
    out: &mut Vec<i32>,
) -> Option<(i32, Option<(u8, u8)>)> {
    walk_offset_adds(seg, p, Some(out))
}

/// The one walk. See [`eat_offset_adds`] for the grammar and every rule it
/// enforces; `sink`, when present, collects the literals [`eat_offset_adds`]
/// throws away.
fn walk_offset_adds(
    seg: &[u8],
    p: &mut usize,
    mut sink: Option<&mut Vec<i32>>,
) -> Option<(i32, Option<(u8, u8)>)> {
    let mut total: i32 = 0;
    let mut last_retype: Option<(u8, u8)> = None;
    macro_rules! done {
        () => {
            return Some((total, last_retype))
        };
    }
    loop {
        if seg.get(*p) != Some(&0x33) {
            done!();
        }
        let mut probe = *p + 1;
        if !eat_int_like(seg, &mut probe) {
            done!();
        }
        let k = match read_varint(seg, &mut probe) {
            Some(k) => k,
            None => done!(),
        };
        match seg.get(probe) {
            Some(&0x27) => {
                probe += 1;
                let (tag, kind, _, tw) = read_type(seg, probe)?;
                if !is_ptr_any(tag, kind) {
                    done!();
                }
                last_retype = Some((tag, kind));
                probe += tw;
            }
            Some(&0x28) => {
                probe += 1;
                if !eat(seg, &mut probe, &[0x00, 0x00]) {
                    done!();
                }
            }
            _ => done!(),
        }
        total = total.checked_add(k)?;
        if let Some(v) = sink.as_deref_mut() {
            v.push(k);
        }
        *p = probe;
    }
}

/// The **width and register file** of a stored value's TYPE, or `None` — which
/// is a refusal, never a guess.
///
/// One locator over the two predicates that already answer this question for the
/// *load* side, in the same order [`finish_indirect_load_of`] asks them:
/// [`value_class`] for the two 4-byte classes c2 keeps in a GPR (a 4-byte
/// integer and a pointer — the pair it lowers with one identical `stw`), then
/// [`sized_ptee`] for the captured 1-, 2- and 8-byte scalars.
///
/// Everything else refuses, and **the floating-point types are the reason this
/// is a function and not a width lookup**: `86 45 40` and `88 85 41` are 4 and 8
/// bytes wide and are stored with `stfs`/`stfd` from `f1`, not `stw`/`std` from
/// `r4` (MEASURED: `void s_f(S* s, float v){ s->f = v; }` is `d0230014`, and
/// `s_d` is `d8230018`). A width-only rule would emit `stw r4` for both — wrong
/// bytes inside an accepted class. The FP argument register is numbered over the
/// FP parameters *alone*, which is the fifth instance of `GAPS.md` §6's "two
/// facts sharing one field" and the live mis-emit `float_leaf_text`'s header
/// records; sizing that widening is a rung, not a line.
pub(crate) fn store_value_width(tag: u8, kind: u8) -> Option<u8> {
    if value_class(tag, kind).is_some() {
        return Some(4);
    }
    sized_ptee(tag, kind).map(|(w, _)| w)
}

/// The width of a **floating-point** stored value — 4 for `float`, 8 for
/// `double` — or `None` when the TYPE is not one.
///
/// Keyed on the kind's **class nibble** (5, "real") and the tag's width nibble,
/// the same two channels `sy::SyView::arg_classes` uses on the `.sy` side, so the
/// two layers agree about what a floating-point value is by construction rather
/// than by two independent whitelists.
pub(crate) fn store_fp_value_width(tag: u8, kind: u8) -> Option<u8> {
    if (kind & 0x0F) != 0x5 {
        return None;
    }
    match tag & 0x0F {
        0x6 => Some(4),
        0x8 => Some(8),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the globs keep that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::func::body::shapes::*;
    #[allow(unused_imports)]
    use crate::func::body::shapes::testutil::*;
    #[allow(unused_imports)]
    use crate::func::body::{parse_segment, parse_segment_detail};
    #[allow(unused_imports)]
    use crate::func::bundle::LO_MARKER;
    #[allow(unused_imports)]
    use crate::func::readers::{find_subslice, is_ptr4_kind, value_class, ValueClass};
    #[allow(unused_imports)]
    use crate::func::sy::{Formals, SyView};
    #[allow(unused_imports)]
    use crate::func::test_fixtures::*;
    #[test]
    fn the_offset_add_run_is_one_walk_with_two_readings() {
        // `27 · 28 · 27` — a member, a subscript, a member. Offsets 8, 4, 16.
        // The address/store reading and the load reading must be the SAME walk:
        // same cursor, same sum, and the load's extra return value is the LAST
        // `27`'s TYPE, never the first (`86 43` here, not `82 43`).
        let seg: &[u8] = &[
            0x33, 0x86, 0x41, 0x74, 0x08, 0x27, 0x82, 0x43, 0xF0, 0x08, // + 8  -> char *
            0x33, 0x86, 0x41, 0x12, 0x04, 0x28, 0x00, 0x00, //             + 4  (no retype)
            0x33, 0x86, 0x41, 0x74, 0x10, 0x27, 0x86, 0x43, 0xF4, 0x08, // + 16 -> int *
            0x30, 0x86, 0x41, 0x74, // the load, which is NOT part of the run
        ];
        let mut p_addr = 0usize;
        let total = eat_addr_offset_adds(seg, &mut p_addr).unwrap();
        let mut p_load = 0usize;
        let (total2, last) = eat_offset_adds(seg, &mut p_load).unwrap();
        assert_eq!(total, 28);
        assert_eq!((total2, last), (28, Some((0x86, 0x43))));
        // One walk: both readings stop at the same byte, on the `30`.
        assert_eq!(p_addr, p_load);
        assert_eq!(seg[p_load], 0x30);
    }

    /// **Board #1334 — the two predicates that guard the SAME byte `27`
    /// disagree on 16 of the 20 type pairs either of them admits, in BOTH
    /// directions, and nothing in the tree said so.**
    ///
    /// `27` is the typed byte-offset add. Five live productions read it and they
    /// do not agree on what TYPE may follow:
    ///
    /// | site | predicate |
    /// |---|---|
    /// | [`is_ptr_any`] — this file, `eat_offset_adds` | 16 tags x kind `43` |
    /// | `shapes::calls`'s data-address walk | [`is_ptr4_kind`] via `eat_operand_type` |
    /// | `shapes::ctor_dtor`'s `this`-adjust | [`is_ptr4_kind`] via `eat_value_type` |
    /// | `shapes::control_flow`'s `cf-offadd-type` | any well-formed TYPE |
    /// | `body::expr`'s sink arm (off by default) | any well-formed TYPE |
    ///
    /// **This is not a tidiness complaint — the divergence is WITNESSED by this
    /// file's own captured IL.** The sibling test above walks
    /// `27 82 43` (a `char *` retype) and
    /// `27 86 43` (an `int *` one) in a single run; [`is_ptr_any`] takes both
    /// and [`is_ptr4_kind`] takes only the second. So a `char *` member offset
    /// is admitted here and refused two productions over, on the same byte of
    /// the same construct.
    ///
    /// **Why a test and not a paragraph** (board #1299's rule): a lane that
    /// widens either predicate to close the gap will silently widen a *shared*
    /// one, and the failure mode this project keeps paying for is a sibling
    /// recognizer whose private limit is invisible to every gate — it emits
    /// nothing, so no byte compare catches it, and it agrees with the census by
    /// construction. This test cannot detect *which* answer is right; it fails
    /// the moment either predicate moves, which is the point.
    ///
    /// **It is NOT a claim that closing the gap converts anything.** Board
    /// **#1333** measured the whole `27` token end-to-end at **+8 emitted
    /// functions of 178,977 and +0 TU** over the 878-TU workload, and the sink
    /// that produced that number admits *any* well-formed TYPE — so every route
    /// through this divergence that surfaces as `expr-op-0x27` is bounded by
    /// that 8. What is NOT bounded by it is the route that falls through to some
    /// *other* key, and that is the open half of the row.
    #[test]
    fn the_two_pointer_predicates_guarding_byte_27_disagree_in_both_directions() {
        // Every (tag, kind) either predicate can see at this position. The tag
        // set is `is_ptr_any`'s own; the kinds are the two `is_ptr4_kind` names.
        const TAGS: [u8; 16] = [
            0x82, 0x84, 0x86, 0x88, 0x92, 0x94, 0x96, 0x98, //
            0xA2, 0xA4, 0xA6, 0xA8, 0xB2, 0xB4, 0xB6, 0xB8,
        ];
        let (mut both, mut any_only, mut ptr4_only) = (0usize, 0usize, 0usize);
        for tag in TAGS {
            for kind in [0x43u8, 0x44] {
                match (is_ptr_any(tag, kind), is_ptr4_kind(tag, kind)) {
                    (true, true) => both += 1,
                    (true, false) => any_only += 1,
                    (false, true) => ptr4_only += 1,
                    (false, false) => {}
                }
            }
        }
        // 4 agreed, 12 admitted only here, 4 admitted only over there.
        assert_eq!((both, any_only, ptr4_only), (4, 12, 4));
        // The two named witnesses, spelled out so the failure message names a
        // construct rather than a count. `82 43` is the `char *` of the sibling
        // test above; `86 44` is the code-pointer kind this file's doc comment
        // refuses by name and `is_ptr4_kind` takes.
        assert!(is_ptr_any(0x82, 0x43) && !is_ptr4_kind(0x82, 0x43), "char *");
        assert!(!is_ptr_any(0x86, 0x44) && is_ptr4_kind(0x86, 0x44), "code ptr");
    }

    /// **Board #908.** The list is not recoverable from the sum, and this is the
    /// board row's own example rather than a new one: `[96]` against `[96, 4]`.
    ///
    /// The two chains are `&t->mid` and `&t->mid.lo[1]`. The first list is a
    /// byte-exact PREFIX of the second — which is the fact `w-ilx`'s GRID I
    /// found — and the two sums, **96 and 100**, are simply two different
    /// numbers with no prefix relation between them to read. A rule stated over
    /// `eat_offset_adds`'s return value cannot express it; one stated over
    /// `eat_offset_adds_list`'s can.
    ///
    /// The walk is shared, so the sum and the retype are asserted to be
    /// **identical** to what `eat_offset_adds` reports on the same bytes. That
    /// is the property that makes this a sibling and not a second reader.
    #[test]
    fn the_offset_add_literals_are_a_list_and_the_sum_cannot_state_a_prefix() {
        // `33 <int> 96 27 <PTR>`                          -> [96]
        let short: &[u8] = &[
            0x33, 0x86, 0x41, 0x74, 0x60, 0x27, 0x86, 0x43, 0xF4, 0x08, //
            0x30, 0x86, 0x41, 0x74,
        ];
        // the same bytes, then `33 <int> 4 28 00 00`      -> [96, 4]
        let long: &[u8] = &[
            0x33, 0x86, 0x41, 0x74, 0x60, 0x27, 0x86, 0x43, 0xF4, 0x08, //
            0x33, 0x86, 0x41, 0x12, 0x04, 0x28, 0x00, 0x00, //
            0x30, 0x86, 0x41, 0x74,
        ];

        let mut a = Vec::new();
        let mut pa = 0usize;
        let ra = eat_offset_adds_list(short, &mut pa, &mut a).unwrap();
        let mut b = Vec::new();
        let mut pb = 0usize;
        let rb = eat_offset_adds_list(long, &mut pb, &mut b).unwrap();

        assert_eq!(a, vec![96]);
        assert_eq!(b, vec![96, 4]);

        // THE FACT, now sayable.
        assert!(b.starts_with(&a), "[96] is a byte-exact prefix of [96, 4]");

        // THE FACT'S ABSENCE from the sums, said as an assertion rather than as
        // a comment: 96 and 100 are unequal and neither divides or bounds the
        // other in any way that recovers `starts_with`.
        assert_eq!(ra.0, 96);
        assert_eq!(rb.0, 100);
        assert_ne!(ra.0, rb.0);

        // One walk: the sibling agrees with the shipping reader on both of its
        // return values and on the cursor, on the same bytes.
        let (mut qa, mut qb) = (0usize, 0usize);
        assert_eq!(eat_offset_adds(short, &mut qa), Some(ra));
        assert_eq!(eat_offset_adds(long, &mut qb), Some(rb));
        assert_eq!((qa, qb), (pa, pb));
        assert_eq!(ra.1, Some((0x86, 0x43)));
        assert_eq!(rb.1, Some((0x86, 0x43)));
    }

    #[test]
    fn an_empty_offset_add_run_reports_no_retype() {
        // Zero adds is the legitimate `return *p;` / `return &p->Base::m;`, and
        // it must report `None` rather than a width — the `30` type is then the
        // only evidence, and "assume 4" is exactly the guess that must not be
        // made here.
        let seg: &[u8] = &[0x30, 0x86, 0x41, 0x74];
        let mut p = 0usize;
        assert_eq!(eat_offset_adds(seg, &mut p), Some((0, None)));
        assert_eq!(p, 0, "the cursor is untouched when nothing is consumed");
    }

    #[test]
    fn the_any_pointee_pointer_gate_is_a_literal_whitelist() {
        // The address path admits every pointee width where the load path picks
        // its instruction from exactly that field — because `addi` is the same
        // word for all of them (MEASURED, `work/bma/probes/p2.cpp`). The tag is
        // still a whitelist: `0x80 | cv | width`, and the four tags with bit
        // `0x40` set are refused for the reason [`is_ptr4_kind`] gives.
        for tag in [0x82u8, 0x84, 0x86, 0x88, 0x92, 0x94, 0x96, 0x98, 0xA2, 0xA4, 0xA6, 0xA8,
                    0xB2, 0xB4, 0xB6, 0xB8]
        {
            assert!(is_ptr_any(tag, 0x43), "tag {tag:#02X}");
        }
        for tag in [0xC2u8, 0xC6, 0xD6, 0xE6, 0xF6, 0x80, 0x81, 0x8A, 0x7F] {
            assert!(!is_ptr_any(tag, 0x43), "tag {tag:#02X} is undetermined");
        }
        // Kind `0x44` — a function/code pointer — is refused here even though
        // [`is_ptr4_kind`] admits it as a loaded *value*: no probe produced one
        // at an address position, and "the pointee width does not matter" has
        // not been checked for code.
        for kind in [0x44u8, 0x41, 0x42, 0x45, 0x46, 0x47, 0x33, 0x53, 0x83] {
            assert!(!is_ptr_any(0x86, kind), "kind {kind:#02X}");
        }
    }

    #[test]
    fn the_ptr4_type_gate_is_a_literal_whitelist_on_both_bytes() {
        // Tags: `0x80 | cv | width-4`, with cv ⊆ {const 0x20, volatile 0x10}.
        for tag in [0x86u8, 0x96, 0xA6, 0xB6] {
            assert!(is_ptr4_kind(tag, 0x43), "tag {tag:#02X} data pointer");
            assert!(is_ptr4_kind(tag, 0x44), "tag {tag:#02X} function pointer");
        }
        // `0xC6` — bit 0x40 — is reported by `readers.rs` as occurring and was
        // produced by none of the `IL_LOAD_TYPES.md` probes. A field that never
        // varied across the probes is indistinguishable from a constant, so it
        // is required literally and refuses. Same for `0xD6`/`0xE6`/`0xF6`.
        for tag in [0xC6u8, 0xD6, 0xE6, 0xF6] {
            assert!(!is_ptr4_kind(tag, 0x43), "tag {tag:#02X} is undetermined");
        }
        // Other widths are other instructions: an 8-byte pointer does not exist
        // on this target and a 1/2-byte one is the `27` pointee-width spelling,
        // which is a different question ([`is_ptr_to_4`]).
        for tag in [0x82u8, 0x84, 0x88, 0xA2, 0xA8] {
            assert!(!is_ptr4_kind(tag, 0x43), "tag {tag:#02X} is not a 4-byte value");
        }
        // Kinds: only 0x43/0x44. Aggregates (class 6), reals (5), void (7) and
        // the integers are all excluded here — the integers have their own
        // predicate, and the rest are T2/T3 and later rungs.
        for kind in [0x41u8, 0x42, 0x45, 0x46, 0x47, 0x33, 0x53, 0x83, 0x84] {
            assert!(!is_ptr4_kind(0x86, kind), "kind {kind:#02X}");
        }
        // The two classes the leaf tail accepts are disjoint, which is what lets
        // `2C` and `41` be required to agree with the `30`.
        assert_eq!(value_class(0x86, 0x43), Some(ValueClass::Ptr4));
        assert_eq!(value_class(0x86, 0x41), Some(ValueClass::Int4));
        assert_eq!(value_class(0x86, 0x45), None, "float is not in either class");
    }

}
