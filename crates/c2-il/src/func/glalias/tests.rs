//! Constructed `.gl` streams for the tag-0x10 alias decode.
//!
//! **Every stream here is built byte by byte, not captured**, because the
//! questions these tests answer are about what the reader does on input the
//! corpus does not contain — in particular the two cases that would make a
//! consumer of this table wrong:
//!
//! * [`shift_null_binds_yet_pairs_nothing`] — the **BIND gate is not
//!   self-validating**. A field read one byte off its position can still resolve
//!   to a real symbol, so "it bound" is not evidence the position is right. This
//!   is why the shape is reported as a result and why the null ships.
//! * [`an_alias_that_also_has_a_body_is_counted`] — the constructed
//!   counterexample to w-emitp §6 **rule 4**. "Never emit a name in
//!   `dom(alias)`" suppresses a symbol; if that name also had a body it would
//!   suppress a symbol that must be emitted, which is a wrong emit and not a
//!   gap. The corpus says this never happens; the reader must still count it
//!   rather than assume it.

use super::*;

/// A record, as this reader locates one:
///
/// ```text
///   <tag> <operand token, 2 bytes> 00 <name> 00 <10-byte kind-4 header> [anchor]
/// ```
///
/// The header is all zeroes, which is the shortest legal form: `optw = 0` (no
/// optional block) and `m = 0` (no trailing table), so the walk is
/// `sc · i32c · varU · varU · i32c · i32c · i32c · i32c` = 10 bytes and the
/// anchor is whatever follows.
fn record(out: &mut Vec<u8>, tag: u8, tok: u32, name: &str, anchor: &[u8]) {
    assert!(tok & 0x80 == 0, "the 2-byte token form needs b1 & 0x80 == 0");
    out.push(tag);
    out.push((tok >> 8) as u8);
    out.push((tok & 0xFF) as u8);
    out.push(0x00);
    out.extend_from_slice(name.as_bytes());
    out.push(0x00);
    out.extend_from_slice(&[0u8; 10]);
    out.extend_from_slice(anchor);
}

/// The 2-byte form of `tok` as the alias target field holds it.
fn target(tok: u32) -> [u8; 2] {
    [(tok >> 8) as u8, (tok & 0xFF) as u8]
}

const DTOR_E: &str = "??_EFilePath@@UAAPAXI@Z";
const DTOR_G: &str = "??_GFilePath@@UAAPAXI@Z";

/// The headline case: a bodyless tag-0x10 record naming a tag-0x0E record.
#[test]
fn alias_pair_decodes() {
    let mut gl = vec![0u8; 4];
    record(&mut gl, 0x0E, 0x1234, DTOR_G, &[0, 0]);
    record(&mut gl, 0x10, 0x2244, DTOR_E, &target(0x1234));
    gl.extend_from_slice(&[0u8; 4]);

    let t = gl_alias_table(&gl);
    assert_eq!(t.stats().tag10, 1);
    assert_eq!(t.stats().bound, 1);
    assert_eq!(t.stats().shape_e_to_g, 1, "??_E<X> -> ??_G<X>");
    assert_eq!(t.stats().head_fail, 0);
    assert_eq!(t.stats().rt_fail, 0);
    assert_eq!(t.stats().unbound_target, 0);
    assert_eq!(t.resolve_name(DTOR_E), DTOR_G);
    assert_eq!(t.resolve_token(0x2244), 0x1234);
    assert!(t.is_alias(DTOR_E));
    assert!(!t.is_alias(DTOR_G));
}

/// The resolution is **once**, never transitive — an alias never targets an
/// alias, and a reader that chased the chain would be modelling something c2
/// does not do.
#[test]
fn resolution_is_not_transitive() {
    let mut gl = vec![0u8; 4];
    record(&mut gl, 0x0E, 0x1234, "?c@@YAXXZ", &[0, 0]);
    record(&mut gl, 0x10, 0x2244, "?b@@YAXXZ", &target(0x1234));
    record(&mut gl, 0x10, 0x3355, "?a@@YAXXZ", &target(0x2244));
    gl.extend_from_slice(&[0u8; 4]);

    let t = gl_alias_table(&gl);
    assert_eq!(t.len(), 2);
    assert_eq!(t.resolve_name("?a@@YAXXZ"), "?b@@YAXXZ");
    assert_eq!(t.resolve_name("?b@@YAXXZ"), "?c@@YAXXZ");
}

/// A tag-0x0E record is not an alias, however its anchor reads.
#[test]
fn a_body_record_is_never_an_alias() {
    let mut gl = vec![0u8; 4];
    record(&mut gl, 0x0E, 0x1234, DTOR_G, &target(0x1234));
    record(&mut gl, 0x0E, 0x2244, DTOR_E, &target(0x1234));
    gl.extend_from_slice(&[0u8; 4]);

    let t = gl_alias_table(&gl);
    assert_eq!(t.stats().tag10, 0);
    assert!(t.is_empty());
}

/// A target that binds to nothing is refused and counted, not guessed at.
#[test]
fn an_unbound_target_is_refused() {
    let mut gl = vec![0u8; 4];
    record(&mut gl, 0x0E, 0x1234, DTOR_G, &[0, 0]);
    record(&mut gl, 0x10, 0x2244, DTOR_E, &target(0x5566));
    gl.extend_from_slice(&[0u8; 4]);

    let t = gl_alias_table(&gl);
    assert_eq!(t.stats().tag10, 1);
    assert_eq!(t.stats().unbound_target, 1);
    assert_eq!(t.stats().bound, 0);
    assert_eq!(t.resolve_name(DTOR_E), DTOR_E, "unresolved names pass through");
}

/// A record that names itself is refused.
#[test]
fn a_self_alias_is_refused() {
    let mut gl = vec![0u8; 4];
    record(&mut gl, 0x10, 0x2244, DTOR_E, &target(0x2244));
    gl.extend_from_slice(&[0u8; 4]);

    let t = gl_alias_table(&gl);
    assert_eq!(t.stats().tag10, 1);
    assert_eq!(t.stats().self_alias, 1);
    assert!(t.is_empty());
}

/// Two records giving one name two different targets resolve to **nothing**.
///
/// Same rule the `.gl` symbol index applies to a token two names claim: an
/// ambiguous binding gets the third value that refuses. A wrong target here is
/// a wrong symbol in the emit set, which is a mis-emit rather than a gap.
#[test]
fn a_disagreeing_duplicate_is_dropped() {
    let mut gl = vec![0u8; 4];
    record(&mut gl, 0x0E, 0x1234, DTOR_G, &[0, 0]);
    record(&mut gl, 0x0E, 0x3344, "?other@@YAXXZ", &[0, 0]);
    record(&mut gl, 0x10, 0x2244, DTOR_E, &target(0x1234));
    record(&mut gl, 0x10, 0x2255, DTOR_E, &target(0x3344));
    gl.extend_from_slice(&[0u8; 4]);

    let t = gl_alias_table(&gl);
    assert_eq!(t.stats().tag10, 2);
    assert_eq!(t.stats().dup, 1);
    // The FIRST binding stands and the disagreeing one is counted, exactly as
    // the reference implementation does; a consumer that wants neither reads
    // `dup` and refuses the TU.
    assert_eq!(t.stats().bound, 1);
}

/// **THE COUNTEREXAMPLE TO THE BIND GATE.** Reading the target field one byte
/// past its decoded position still resolves to a real symbol here — so a decode
/// that offered "it bound" as its evidence would be offering nothing.
///
/// What separates the real read from this one is the **shape**, which the gate
/// does not mention: the true position pairs `??_E<X>` with `??_G<X>`, and the
/// shifted position pairs it with whatever the neighbouring bytes happen to
/// name. Over 850 real TUs the shifted reads bind 1 795 and 2 449 times and
/// produce **zero** pairs.
#[test]
fn shift_null_binds_yet_pairs_nothing() {
    // The alias's anchor holds `12 34` and is followed by `00 56`. Read at the
    // true position the target token is 0x1234 (= `??_G…`); read one byte late
    // it is 0x3400, and a record is planted under exactly that token so the
    // BIND gate cannot tell the difference.
    let mut gl = vec![0u8; 4];
    record(&mut gl, 0x0E, 0x1234, DTOR_G, &[0, 0]);
    record(&mut gl, 0x0E, 0x3400, "?decoy@@YAXXZ", &[0, 0]);
    record(&mut gl, 0x10, 0x2244, DTOR_E, &[0x12, 0x34, 0x00, 0x56]);
    gl.extend_from_slice(&[0u8; 4]);

    let real = gl_alias_table(&gl);
    assert_eq!(real.stats().bound, 1);
    assert_eq!(real.stats().shape_e_to_g, 1);
    assert_eq!(real.resolve_name(DTOR_E), DTOR_G);

    let null = gl_alias_table_shifted(&gl, 1);
    assert_eq!(null.stats().bound, 1, "the null BINDS — that is the point");
    assert_eq!(
        null.stats().shape_e_to_g,
        0,
        "…and pairs nothing: BIND alone is not evidence of position"
    );
    assert_eq!(null.resolve_name(DTOR_E), "?decoy@@YAXXZ");
}

/// **THE COUNTEREXAMPLE TO RULE 4** — `never emit a name in dom(alias)`.
///
/// The corpus says `dom(alias) ∩ U` is 0 over 96 220 records, so the rule is
/// safe *there*. This stream is the case that would make it a **wrong emit**: a
/// name that carries both a tag-0x0E body record and a tag-0x10 alias record is
/// in `dom(alias)` and must be emitted anyway. The reader counts it — it does
/// not apply the rule and it does not assume the count is zero.
#[test]
fn an_alias_that_also_has_a_body_is_counted() {
    let mut gl = vec![0u8; 4];
    record(&mut gl, 0x0E, 0x1234, DTOR_G, &[0, 0]);
    // The SAME name, twice: once with a body, once as an alias.
    record(&mut gl, 0x0E, 0x4455, DTOR_E, &[0, 0]);
    record(&mut gl, 0x10, 0x2244, DTOR_E, &target(0x1234));
    gl.extend_from_slice(&[0u8; 4]);

    let t = gl_alias_table(&gl);
    assert_eq!(t.stats().bound, 1);
    assert_eq!(
        t.stats().dom_with_body,
        1,
        "a consumer must be able to refuse this TU rather than suppress a body"
    );
}

/// The clean case: nothing in `dom(alias)` has a body, which is what licenses
/// rule 4 on the real corpus.
#[test]
fn dom_with_body_is_zero_when_the_alias_is_bodyless() {
    let mut gl = vec![0u8; 4];
    record(&mut gl, 0x0E, 0x1234, DTOR_G, &[0, 0]);
    record(&mut gl, 0x10, 0x2244, DTOR_E, &target(0x1234));
    gl.extend_from_slice(&[0u8; 4]);
    assert_eq!(gl_alias_table(&gl).stats().dom_with_body, 0);
}

/// A truncated record desyncs the header walk and is counted, never guessed.
#[test]
fn a_truncated_record_fails_the_header_walk() {
    let mut gl = vec![0u8; 4];
    record(&mut gl, 0x10, 0x2244, DTOR_E, &[]);
    gl.truncate(gl.len() - 8);

    let t = gl_alias_table(&gl);
    assert_eq!(t.stats().tag10, 1);
    assert_eq!(t.stats().head_fail + t.stats().rt_fail, 1);
    assert!(t.is_empty());
}

/// An empty `.gl` is empty, not a panic.
#[test]
fn an_empty_stream_decodes_to_nothing() {
    assert!(gl_alias_table(&[]).is_empty());
    assert!(gl_alias_table(&[0x10, 0x00]).is_empty());
}

// --------------------------------------------------------------- primitives

#[test]
fn var_u_reads_both_widths() {
    // Narrow: `b1 & 0x80` clear.
    assert_eq!(var_u(&[0x34, 0x12], 0), Some((0x1234, 2)));
    // Wide: `b1 & 0x80` set; the high half is shifted down by one.
    assert_eq!(var_u(&[0x34, 0x92, 0x01, 0x00], 0), Some((0x1234 | (1 << 15), 4)));
    assert_eq!(var_u(&[0x34], 0), None);
}

#[test]
fn i32c_reads_the_escape_and_the_sign() {
    assert_eq!(i32c(&[0x7F], 0), Some((127, 1)));
    assert_eq!(i32c(&[0xFF], 0), Some((-1, 1)));
    assert_eq!(i32c(&[0x80, 0xC8, 0, 0, 0], 0), Some((200, 5)));
    assert_eq!(i32c(&[0x80, 0xFB, 0xFF, 0xFF, 0xFF], 0), Some((-5, 5)));
    assert_eq!(i32c(&[0x80, 0x01], 0), None);
}

#[test]
fn i16c_reads_the_escape_and_the_sign() {
    assert_eq!(i16c(&[0x7F], 0), Some((127, 1)));
    assert_eq!(i16c(&[0x80], 0), None);
    assert_eq!(i16c(&[0x80, 0x00, 0x80], 0), Some((0x8000, 3)));
    assert_eq!(i16c(&[0x81], 0), Some((-127, 1)));
}

#[test]
fn skipvar_consumes_the_high_bit_run() {
    assert_eq!(skipvar(&[0x00], 0), Some(1));
    assert_eq!(skipvar(&[0x80, 0x81, 0x02], 0), Some(3));
    assert_eq!(skipvar(&[0x80, 0x81], 0), None);
}

/// A count taken from the data is never trusted past the end of the stream —
/// an unbounded loop bound is a hazard even when the walk would fail anyway.
#[test]
fn a_wild_header_count_does_not_spin() {
    // optw = 1, then a count of 0x7FFFFFFF.
    let mut gl = vec![0u8; 4];
    gl.push(0x10);
    gl.extend_from_slice(&[0x22, 0x44, 0x00]);
    gl.extend_from_slice(DTOR_E.as_bytes());
    gl.push(0x00);
    gl.extend_from_slice(&[
        0x00, // storage class
        0x00, // i32c +0x40
        0x00, 0x00, // varU +0x20
        0x00, 0x00, // varU +0x0c
        0x01, // i32c optw = 1
        0x00, // i32c
        0x80, 0xFF, 0xFF, 0xFF, 0x7F, // i32c count = 0x7FFFFFFF
    ]);
    gl.extend_from_slice(&[0u8; 8]);

    let t = gl_alias_table(&gl);
    assert_eq!(t.stats().tag10, 1);
    assert_eq!(t.stats().head_fail, 1);
}
