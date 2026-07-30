/// The int type encoding inline in the `.ex` body (`86 41 74`), per `IL_FORMAT`.
pub(crate) const INT_TYPE: [u8; 3] = [0x86, 0x41, 0x74];

/// Real token-width detector (ports `il_parser._detect_token_width`): find the
/// first `4F 02`, count the bytes to the next `4F`. That gap is the token
/// width. Defaults to 2 if the anchor is not found.
pub fn detect_token_width(ex: &[u8]) -> usize {
    let mut i = 0;
    while i + 1 < ex.len() {
        if ex[i] == 0x4F && ex[i + 1] == 0x02 {
            let mut j = i + 2;
            while j < ex.len() && ex[j] != 0x4F {
                j += 1;
            }
            let gap = j - (i + 2);
            if gap == 2 || gap == 4 {
                return gap;
            }
        }
        i += 1;
    }
    2
}

pub(crate) fn contains_subslice(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || hay.len() < needle.len() {
        return needle.is_empty();
    }
    hay.windows(needle.len()).any(|w| w == needle)
}

/// Owned `String` from a byte run the caller has already verified is ASCII
/// (graphic / space). `from_utf8` is then infallible and takes the fast
/// validated path; the lossy fallback is defensive only and never replaces
/// anything for such input, so the result is identical to
/// `String::from_utf8_lossy(bytes).into_owned()` — minus its chunk iterator,
/// which was measurable on the hot parse path.
pub(crate) fn ascii_string(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_owned(),
        Err(_) => String::from_utf8_lossy(bytes).into_owned(),
    }
}

/// Read one IL token, which is **2 or 4 bytes depending on the token itself**,
/// returning `(identity, width)`.
///
/// Token width is per-token, not a per-file constant: the second byte carries a
/// continuation flag in bit 7. Clear → the token is those 2 bytes; set → two
/// more bytes follow. Verified on a real capture of `system/world/Dir.cpp`,
/// where `4F 02` module markers appear as both `4f 02 e3 09` (2-byte) and
/// `4f 02 a4 96 03 00` (4-byte) in the same file, and where applying this rule
/// to every `B9` LOAD site lands on a valid 3-byte operand type at 21443 sites
/// (the residue being a third type class plus `B9` bytes occurring inside data).
///
/// [`detect_token_width`] — which returns a single width for the whole file —
/// is therefore wrong for real translation units, and its misalignment is what
/// produced the artifact census buckets `call-token-0x01…0x05` and
/// `expr-load-type-0N00A6`: a 2-byte read of a 4-byte token leaves the parse
/// standing on the token's own tail bytes. It is kept only for the K1/K2a codec
/// and the existing tests; the parser no longer consults it.
///
/// The returned identity is only ever compared for equality (token → parameter
/// register), so any injective encoding will do. The two widths cannot collide:
/// a 2-byte token's value is `< 0x10000` while a 4-byte token's byte 1 has bit 7
/// set, which lands in bits 23..16 of the result and forces it `>= 0x10000`.
pub(crate) fn read_token_var(ex: &[u8], p: usize) -> Option<(u32, usize)> {
    let b0 = *ex.get(p)? as u32;
    let b1 = *ex.get(p + 1)? as u32;
    if b1 & 0x80 == 0 {
        return Some(((b0 << 8) | b1, 2));
    }
    let b2 = *ex.get(p + 2)? as u32;
    let b3 = *ex.get(p + 3)? as u32;
    Some(((b0 << 24) | (b1 << 16) | (b2 << 8) | b3, 4))
}

/// Forward byte search, word-at-a-time (hand-rolled `memchr`; std-only, no
/// dependency). The `.ex` marker scans walk multi-KB streams whose prefix is
/// zero-fill, and the byte-at-a-time loops were the port's single hottest path
/// (~60% of a small compile). This finds the first candidate byte 8 bytes per
/// step with the classic SWAR zero-byte trick, then lets a plain scan finish
/// inside the matching word — same result as `iter().position`, just faster.
#[inline]
pub(crate) fn memchr_byte(needle: u8, hay: &[u8]) -> Option<usize> {
    const LO: u64 = 0x0101_0101_0101_0101;
    const HI: u64 = 0x8080_8080_8080_8080;
    #[inline(always)]
    fn has_zero_byte(w: u64) -> bool {
        w.wrapping_sub(LO) & !w & HI != 0
    }
    let bcast = LO.wrapping_mul(needle as u64);
    let mut i = 0;
    // 32 bytes per step while no word contains the needle…
    while i + 32 <= hay.len() {
        let a = u64::from_ne_bytes(hay[i..i + 8].try_into().unwrap()) ^ bcast;
        let b = u64::from_ne_bytes(hay[i + 8..i + 16].try_into().unwrap()) ^ bcast;
        let c = u64::from_ne_bytes(hay[i + 16..i + 24].try_into().unwrap()) ^ bcast;
        let d = u64::from_ne_bytes(hay[i + 24..i + 32].try_into().unwrap()) ^ bcast;
        if has_zero_byte(a) || has_zero_byte(b) || has_zero_byte(c) || has_zero_byte(d) {
            break;
        }
        i += 32;
    }
    // …then 8, then byte-at-a-time to pin the first occurrence.
    while i + 8 <= hay.len() {
        let w = u64::from_ne_bytes(hay[i..i + 8].try_into().unwrap()) ^ bcast;
        if has_zero_byte(w) {
            break;
        }
        i += 8;
    }
    hay[i..].iter().position(|&x| x == needle).map(|k| i + k)
}

// `find_byte` used to live here and had exactly one caller: the `this` lookup's
// "first `0x46` in the segment" anchor, which was a wrong-bytes emit (see
// `expr::formals_marker`). Nothing in the pre-body region is safely located by a
// bare byte search — a candidate has to be required to *end* somewhere known — so
// the helper is gone rather than left available for the next such anchor.

pub(crate) fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    // First-occurrence semantics identical to `windows().position`; the scan
    // just jumps between candidate first bytes word-at-a-time.
    let last_start = hay.len() - needle.len();
    let mut i = 0;
    while i <= last_start {
        let k = memchr_byte(needle[0], &hay[i..=last_start])?;
        let j = i + k;
        if &hay[j..j + needle.len()] == needle {
            return Some(j);
        }
        i = j + 1;
    }
    None
}

/// Read a `.ex` inline **type**: `<tag> <kind> <LEB128 id>`, returning
/// `(tag, kind, id, width)`.
///
/// This is a *third* encoding, distinct from both the operand token
/// ([`read_token_var`], 2 or 4 bytes) and the statement/literal varint
/// ([`read_varint`], 1 or 5 bytes). Total width is 3 bytes for an id below
/// `0x80`, 4 below `0x4000`, 5 below `0x200000`.
///
/// Getting this right matters more than it looks: across the 8628 well-formed
/// call sites of one real TU the return-type width splits 4157 / 3123 / 1358
/// between 3, 4 and 5 bytes, so a fixed-3 or even a "3 or 4" rule mis-parses
/// roughly one call in six. The boundaries are pinned by the fixed one-byte
/// markers that always bracket a type — the `41` result-type marker, the `55`
/// argument push, the `4C 4B` call end — against which a wrong width visibly
/// swallows the next marker.
///
/// The tag always has bit 7 set; its high bits are not understood (`0x86`,
/// `0xA6`, `0x96`, `0xC6` all occur and behave identically here), but a decoder
/// does not need them — the width rule is tag-independent. The `kind` byte is
/// treated as a fixed byte rather than a second LEB because `88 85 41`
/// (`double`) and `88 81 13` (`long long`) have bit 7 set there and would
/// otherwise run on.
pub(crate) fn read_type(seg: &[u8], p: usize) -> Option<(u8, u8, u32, usize)> {
    let tag = *seg.get(p)?;
    if tag & 0x80 == 0 {
        return None;
    }
    let kind = *seg.get(p + 1)?;
    let mut id: u32 = 0;
    let mut shift: u32 = 0;
    let mut i = p + 2;
    loop {
        let b = *seg.get(i)?;
        id |= ((b & 0x7F) as u32) << shift;
        i += 1;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 28 {
            return None; // malformed / not a type
        }
    }
    Some((tag, kind, id, i - p))
}

/// Advance `*p` past `pat` iff the stream matches it there; return whether it
/// did. The single primitive the positive parser is built on — every grammar
/// token is consumed through an `eat` (fixed pattern) or a typed read, so an
/// unrecognized byte anywhere fails the whole parse closed.
pub(crate) fn eat(seg: &[u8], p: &mut usize, pat: &[u8]) -> bool {
    if seg.len() >= *p + pat.len() && &seg[*p..*p + pat.len()] == pat {
        *p += pat.len();
        true
    } else {
        false
    }
}

pub(crate) fn eat_byte(seg: &[u8], p: &mut usize, x: u8) -> bool {
    if seg.get(*p) == Some(&x) {
        *p += 1;
        true
    } else {
        false
    }
}

/// Consume zero or more `4F 01 <varint>` **source-line markers**.
///
/// Two corrections over the previous reading, both from live captures:
///
/// * the payload is a [`read_varint`], not a fixed byte. A function at source line
///   200 emits `4f 01 80 c8 00 00 00` — the escaped four-byte form. Reading one
///   byte therefore desynchronizes the whole token stream for any TU whose
///   functions live past line 127, which is nearly all of them; the parse then
///   fails somewhere arbitrary downstream and the census attributes the block to
///   whatever byte it happened to land on. So this was not only costing coverage,
///   it was corrupting the blocking-feature histogram that the widening order is
///   chosen from.
/// * it is emitted on each line *change*, and two can appear in a row where a
///   declaration line generates no code (`int x;` followed by a statement), so
///   this loops instead of eating at most one.
///
/// Still specific to `4F 01` — it never eats the `4F 12` separator or the `4F 02`
/// module marker.
pub(crate) fn eat_opt_stmt_marker(seg: &[u8], p: &mut usize) {
    while seg.get(*p) == Some(&0x4F) && seg.get(*p + 1) == Some(&0x01) {
        let mut probe = *p + 2;
        if read_varint(seg, &mut probe).is_none() {
            return; // malformed payload: leave `p` put and let the caller block
        }
        *p = probe;
    }
}

/// Read an int literal varint `33 <int-type> <varint>` payload (the `33
/// <int-type>` prefix already consumed): a single byte if `< 0x80` (the value),
/// else `0x80` + a 4-byte LE i32. (Verified: 5→`05`, 42→`2a`, 200→`80 c8000000`,
/// 70000→`80 70110100`.) Any other lead byte → `None`.
pub(crate) fn read_varint(seg: &[u8], p: &mut usize) -> Option<i32> {
    let marker = *seg.get(*p)?;
    if marker == 0x80 {
        // Escape: `80` + a 4-byte LE i32. Used for anything that does not fit
        // the signed short form — including −128, whose byte encoding would
        // otherwise collide with this marker.
        //
        // NOTE: for tag-`0x88` types (`long long`) the escape payload is 8
        // bytes, not 4. Those types are rejected upstream, so this reads only
        // the 4-byte form; widening to 64-bit literals must fix this too.
        let v = i32::from_le_bytes([
            *seg.get(*p + 1)?,
            *seg.get(*p + 2)?,
            *seg.get(*p + 3)?,
            *seg.get(*p + 4)?,
        ]);
        *p += 5;
        Some(v)
    } else {
        // Short form: a **signed** byte, not an unsigned one. `-5` is `fb` and
        // `(char)200` is `c8`. An earlier revision accepted only `00..7F` and
        // rejected `81..FF` outright — fail-closed and safe, but it silently
        // blocked every negative literal in the corpus.
        *p += 1;
        Some(marker as i8 as i32)
    }
}

/// The `unsigned int` operand type encoding inline in the `.ex` body.
/// Distinguished from [`INT_TYPE`] only by its last two bytes; the relational
/// opcodes are sign-agnostic, so this triple is the *only* thing that says a
/// comparison is unsigned.
pub(crate) const UINT_TYPE: [u8; 3] = [0x86, 0x42, 0x75];

/// `long` (`86 41 12`) and `unsigned long` (`86 42 22`). On this target they are
/// 32-bit, and c2 emits **byte-identical** code for them and for `int`/`unsigned`
/// — see `docs/IL_TYPE_TAGS.md` §3.1.
const LONG_TYPE: [u8; 3] = [0x86, 0x41, 0x12];
const ULONG_TYPE: [u8; 3] = [0x86, 0x42, 0x22];

/// The 32-bit integer operand types that are interchangeable *for the operators
/// this parser accepts* (`+`, `-`, `*` and add-immediate folding).
///
/// Signedness is not a distinction PPC's `add`/`subf`/`mullw` make, and the
/// captures bear that out: `a+b+c`, `a-b`, `a*b*c` and `a+7` each produce exactly
/// the same words for `int`, `unsigned`, `long` and a mixed `int`/`unsigned`
/// expression (`docs/IL_TYPE_TAGS.md` §3.1). It is **not** a general licence to
/// ignore signedness — division and the shift-right family do differ, and both
/// are refused elsewhere — nor does it extend to the narrow types, whose
/// extension placement depends on the operator *and* the result type (§3.2).
const INT_LIKE_TYPES: [[u8; 3]; 4] = [INT_TYPE, UINT_TYPE, LONG_TYPE, ULONG_TYPE];

/// Consume any one of [`INT_LIKE_TYPES`] at `p`, reporting whether it matched.
pub(crate) fn eat_int_like(seg: &[u8], p: &mut usize) -> bool {
    INT_LIKE_TYPES.iter().any(|t| eat(seg, p, t))
}

/// The `float` operand type (`86 45 40`) and the `double` one (`88 85 41`).
/// Note the *literal* forms differ again ([`FLOAT_LIT_TYPE`] /
/// [`DOUBLE_LIT_TYPE`]).
pub(crate) const FLOAT_TYPE: [u8; 3] = [0x86, 0x45, 0x40];
pub(crate) const DOUBLE_TYPE: [u8; 3] = [0x88, 0x85, 0x41];

/// The *literal* FP type tags, which are distinct from the operand ones above.
/// A float literal carries `86 4a 40`, a double one `88 8a 41`.
pub(crate) const FLOAT_LIT_TYPE: [u8; 3] = [0x86, 0x4A, 0x40];
pub(crate) const DOUBLE_LIT_TYPE: [u8; 3] = [0x88, 0x8A, 0x41];

/// A TYPE's implied width in bytes. The tag's low nibble is
/// `2 * (log2(size) + 1)`, so `…2`→1, `…4`→2, `…6`→4, `…8`→8, and it is the same
/// in every observed tag family (`86`, `A6` const, `96` volatile, `82`/`84`/`88`).
/// Verified across every triple in `docs/IL_TYPE_TAGS.md` §2 and against the
/// pointee-width tags this document's `27` captures produced
/// (`27 82 43 f0 08` for a `char` member, `27 88 43 c1 08` for a `double` one).
fn type_width(tag: u8) -> Option<u32> {
    match tag & 0x0F {
        0x2 => Some(1),
        0x4 => Some(2),
        0x6 => Some(4),
        0x8 => Some(8),
        _ => None,
    }
}

/// True for a TYPE naming a **4-byte integer** value in any cv-qualification:
/// `kind`'s low nibble is 1 (signed) or 2 (unsigned) and the tag says 4 bytes.
/// Captured witnesses: `86 41 74` int, `86 42 75` unsigned, `86 41 12` long,
/// `86 42 22` unsigned long, `A6 41 84 20` const int, `96 41 86 20` volatile int.
///
/// This admits exactly the set over which a following `2C` to an int-like target
/// is *provably* a no-op (`docs/IL_CAST_CONVERT.md` §2.2/§4.2: int↔unsigned and
/// cv-strip at the same width emit nothing), which is what lets the cv-qualified
/// forms be accepted at all — a `const` member getter's load carries
/// `30 A6 41 …` followed by `2C 86 41 74 00`, and both captures
/// (`const int *`, `volatile int *`) emit a bare `lwz` with nothing added.
pub(crate) fn is_int4_type(tag: u8, kind: u8) -> bool {
    type_width(tag) == Some(4) && matches!(kind & 0x0F, 0x1 | 0x2)
}

/// True for a TYPE naming a **pointer to a 4-byte object**: `kind`'s low nibble
/// is 3 and the tag says 4. In a `B9` operand position a pointer's tag is the
/// *pointer's* own width (`86 43 f4 08` = `int *`); in the `27` byte-offset-add
/// position it is the **pointee's** width instead (`82 43 f0 08` for `char *`,
/// `88 43 c1 08` for `double *`), which is why this is applied to the `27` type
/// and not only to the base LOAD.
pub(crate) fn is_ptr_to_4(tag: u8, kind: u8) -> bool {
    type_width(tag) == Some(4) && (kind & 0x0F) == 0x3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_width_two_from_4f02_gap() {
        // `4F 02 20 00 4F` → gap 2.
        let ex = [0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01];
        assert_eq!(detect_token_width(&ex), 2);
    }

    // ---- variable-width tokens ----------------------------------------------

    #[test]
    fn token_is_two_bytes_when_the_continuation_bit_is_clear() {
        // Every fixture token is of this form (`e3 09`, `e5 09`, …) — bit 7 of
        // the second byte clear.
        assert_eq!(read_token_var(&[0xE3, 0x09, 0xFF], 0), Some((0xE309, 2)));
        assert_eq!(read_token_var(&[0x00, 0x7F], 0), Some((0x007F, 2)));
    }

    #[test]
    fn token_is_four_bytes_when_the_continuation_bit_is_set() {
        // Real-TU form, e.g. the module marker payload `a4 96 03 00`.
        assert_eq!(
            read_token_var(&[0xA4, 0x96, 0x03, 0x00], 0),
            Some((0xA496_0300, 4))
        );
        // Truncated 4-byte token → None, never a short read.
        assert_eq!(read_token_var(&[0xA4, 0x96, 0x03], 0), None);
    }

    #[test]
    fn token_identities_cannot_collide_across_widths() {
        // The parser compares token identities for equality (token → parameter
        // register), so a 2-byte and a 4-byte token must never produce the same
        // value. A 4-byte token's byte 1 has bit 7 set, which lands in bits
        // 23..16 and forces the value >= 0x10000; 2-byte values are < 0x10000.
        for b0 in [0x00u8, 0x7F, 0x80, 0xFF] {
            for b1 in [0x00u8, 0x7F, 0x80, 0xFF] {
                let (v, w) = read_token_var(&[b0, b1, 0x00, 0x00], 0).unwrap();
                if w == 2 {
                    assert!(v < 0x10000, "2-byte token {v:#X} must stay narrow");
                } else {
                    assert!(v >= 0x10000, "4-byte token {v:#X} must not alias a narrow one");
                }
            }
        }
    }

    #[test]
    fn varint_short_form_is_signed() {
        // `-5` is `fb`, not a rejected byte. An earlier revision accepted only
        // 00..7F, which was fail-closed but silently blocked every negative
        // literal in the corpus.
        let cases: &[(&[u8], i32, usize)] = &[
            (&[0x00], 0, 1),
            (&[0x05], 5, 1),
            (&[0x7F], 127, 1),
            (&[0xFB], -5, 1),
            (&[0xFF], -1, 1),
            (&[0x81], -127, 1),
            // Escape form: `80` + 4-byte LE i32.
            (&[0x80, 0x70, 0x11, 0x01, 0x00], 70000, 5),
            // -128 cannot use the short form (0x80 is the marker), so it is
            // forced to the escape.
            (&[0x80, 0x80, 0xFF, 0xFF, 0xFF], -128, 5),
        ];
        for (bytes, want, width) in cases {
            let mut p = 0usize;
            assert_eq!(read_varint(bytes, &mut p), Some(*want), "{bytes:02X?}");
            assert_eq!(p, *width, "width for {bytes:02X?}");
        }
    }

    // ---- inline type encoding (LEB128) --------------------------------------

    #[test]
    fn read_type_widths_match_the_captured_boundaries() {
        // Each of these is pinned in a live capture by the fixed one-byte marker
        // that follows it (`41` result-type, `55` arg push, `4C 4B` call end),
        // so a wrong width visibly swallows the next marker.
        let cases: &[(&[u8], u8, u8, u32, usize)] = &[
            (&[0x86, 0x41, 0x74], 0x86, 0x41, 116, 3),        // int
            (&[0x86, 0x42, 0x75], 0x86, 0x42, 117, 3),        // unsigned
            (&[0x82, 0x07, 0x03], 0x82, 0x07, 3, 3),          // void
            (&[0x86, 0x43, 0x83, 0x08], 0x86, 0x43, 1027, 4), // void*
            (&[0x86, 0x43, 0x82, 0x20], 0x86, 0x43, 4098, 4), // int**
            (&[0x88, 0x85, 0x41], 0x88, 0x85, 65, 3),         // double
            (&[0x88, 0x81, 0x13], 0x88, 0x81, 19, 3),         // long long
            (&[0x86, 0x43, 0x9B, 0xB9, 0x02], 0x86, 0x43, 40091, 5), // 5-byte id
        ];
        for (bytes, tag, kind, id, w) in cases {
            assert_eq!(
                read_type(bytes, 0),
                Some((*tag, *kind, *id, *w)),
                "type {bytes:02X?}"
            );
        }
        // A tag without bit 7 set is not a type.
        assert_eq!(read_type(&[0x41, 0x86, 0x41], 0), None);
    }

    #[test]
    fn call_token_is_decoded_not_anchor_matched() {
        // The old model hardcoded `00 80 01 10 00 00` as a fixed "callee anchor".
        // It is really flags=0 + varint(0x1001), so any TU whose callee function
        // type is not the first one created failed to parse. All three of these
        // are real captured CALL tokens with different fn-type ids and return
        // types; all must now decode.
        let cases: &[(&[u8], &str)] = &[
            (&[0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00], "void, id 0x1001"),
            (&[0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x80, 0x10, 0x00, 0x00], "void, id 0x1080"),
            (
                &[0xBD, 0x86, 0x43, 0x83, 0x08, 0x00, 0x80, 0xAE, 0x10, 0x00, 0x00],
                "void* (4-byte type), id 0x10AE",
            ),
        ];
        for (bytes, label) in cases {
            let mut p = 1; // past the BD
            let (_, _, _, w) = read_type(bytes, p).expect(label);
            p += w;
            assert_eq!(bytes.get(p), Some(&0x00), "{label}: cdecl flags byte");
            p += 1;
            assert!(read_varint(bytes, &mut p).is_some(), "{label}: fn-type id");
            assert_eq!(p, bytes.len(), "{label}: token must end exactly here");
        }
    }
}
