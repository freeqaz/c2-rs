/// The int type encoding inline in the `.ex` body (`86 41 74`), per `IL_FORMAT`.
/// PROV[O] `86 41 74`, the int type triple, read off `.ex` captures and cross-checked against `il_parser.py`'s `KNOWN_TYPES`.
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
    // PROV[S] the SWAR has-zero-byte constant `0x0101…01`; standard bit-twiddling (Mycroft's test, in Hacker's Delight), nothing to do with c2 and nothing to do with `bundle::LO`. Name collision only.
    const LO: u64 = 0x0101_0101_0101_0101;
    // PROV[S] the SWAR high-bit mask `0x8080…80`, the other half of the same published test.
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

/// The `kind` byte's low nibble is a **class**: 1 signed int, 2 unsigned int,
/// 3 data pointer, 4 function pointer, 5 real, **6 aggregate**, 7 void, A real
/// literal (`docs/IL_LOAD_TYPES.md` §1). Class 6 is the only one whose payload is
/// not simply the LEB id, so it is the only one [`read_type`] special-cases.
///
/// An aggregate's **size** is a 5-bit field spread across `tag` bit 0 (as bit 4)
/// and `kind`'s high nibble (as bits 3..0); the tag's remaining low bits are an
/// alignment class, not the value's width. When the size does not fit in 5 bits
/// the field reads 0 and a *statement* varint ([`read_varint`]) carrying the real
/// size is inserted **between the kind and the LEB id**.
///
/// MEASURED, from the struct-copy (`*d = *s;`) size ladder in
/// `docs/IL_LOAD_TYPES.md` §1a — each row is a captured TYPE in a `30`/`32`
/// position, and the table's whole point is that it is a *ladder*, so no single
/// row carries the rule alone:
///
/// ```text
///   S4  size 4  align 4   86 46 80 20        kind hi = 4,  tag bit0 = 0
///   S8  size 8  align 4   86 86 86 20        kind hi = 8
///   B12 size 12 align 4   86 C6 80 20        kind hi = C
///   S15 size 15 align 1   82 F6 8C 20        kind hi = F,  tag 82
///   S16 size 16 align 4   87 06 93 20        kind hi = 0,  tag bit0 = 1  → 16
///   S20 size 20 align 4   87 46 99 20        16 + 4
///   SD16 size 16 align 8  89 06 95 20        tag 88|1 — align moved, bit0 stayed
///   S31 size 31 align 1   83 F6 80 20        16 + F, the top of the field
///   S32 size 32 align 1   82 06 20 87 20     field 0 → varint 0x20, then id
///   S33 size 33 align 1   82 06 21 8e 20     varint 0x21
///   T40 size 40 align 4   86 06 28 a0 20     varint 0x28
/// ```
///
/// Two plausible wrong rules this ladder rules out, because a rule believed
/// without its discriminating neighbour is how this project has emitted wrong
/// bytes before:
///
/// * **"the bytes after the kind are a fixed class token / suffix, not a size."**
///   Then growing the struct by one byte could not move them. `S32`→`S33` moves
///   `20`→`21` while nothing else about the source changes — and the id moves
///   independently (0x1007 → 0x100E), so the two fields are separate. This is
///   the pair the rule stands on.
/// * **"the size is only the kind's high nibble (4 bits)."** Then `S16`'s kind
///   `06` reads size 0 and the parse would look for a varint at `93` — which is
///   a negative short-form varint and refuses here. So the 4-bit reading is
///   *distinguishable*: it fails closed on S16/S20/SD16 where the 5-bit reading
///   steps 4 bytes. `S15` (`82 F6`) vs `S31` (`83 F6`) shows the same bit
///   carrying the +16, with the alignment nibble held constant.
///
/// Wild witness, aligned by a bracketing marker rather than by a probe:
/// `src/system/meta/Sorting.cpp` at `.ex` 0xc7e3 carries
/// `… 55 86 41 74 4C | 30 86 06 80 14 10 00 00 a5 29 | 4B …` — a 4,116-byte
/// (0x1014) object. The type is 9 bytes wide and the statement-end `4B` sits
/// exactly at its end; the old 4-byte read (`86 06 80 14`, the varint escape
/// swallowed as LEB continuation bytes) left the parse standing on `10`.
///
/// The size itself is decoded and then **dropped**: aggregates are refused for
/// acceptance by the class gates ([`is_int4_type`] / [`is_ptr_to_4`] are false
/// for class 6), so nothing downstream needs it, and a second parser for it
/// elsewhere is exactly the "one fact, two locators" mistake `find_byte` was
/// deleted for. If a future rung needs the size, widen this return — do not
/// re-read the field.
///
/// ## This branch is currently unreachable, and that is measured, not assumed
///
/// MEASURED 2026-07-30: with an `eprintln!` in this branch, a full 878-TU /
/// 2,462,571-function workload scan enters it **zero** times, and so does a
/// single-TU census of `Sorting.cpp` — the TU that contains the wild witness
/// above. (The instrument was validated by watching it fire 16 times under the
/// unit tests, so this is a positive measurement rather than a silent probe.)
/// The reason is that every position where a `read_type` call could see class 6
/// sits behind an earlier refusal: `fixtures/cpp/w12_aggr_type.cpp`'s struct
/// copies stop at the base pointer LOAD, a by-value struct return stops at the
/// `9B` sret bind, and the blocker *names* come from three raw bytes
/// (`blk_type`) rather than from this width, so attribution never depended on it
/// either. The before/after workload scan is identical in every per-TU field:
/// same census numerator (110,366), same 900-odd blocker buckets, zero moves.
///
/// So the honest value of this fix is **entirely latent**: it removes a desync
/// that would fire the moment any of those earlier gates widens — and the next
/// rungs on the board widen exactly the `30`/`41`/`2C`/`27` leaf positions where
/// an aggregate TYPE would first arrive. It buys no coverage and improves no
/// bucket today. Anyone re-ranking work from this function's existence should
/// read that paragraph before assuming it did.
/// PROV[O] the type-kind byte for an aggregate, read off `.ex`/`.sy` captures.
const AGGREGATE_CLASS: u8 = 0x6;

/// **A type tag with this bit set carries one extra byte — the WIDE MARK —
/// between the tag and the kind, displacing every field after it.**
///
/// This is the same rule [`super::sy`]'s `read_type_prefix` has enforced since
/// the `.sy` layer first bound on a real translation unit, where getting it wrong
/// was measured as the single largest cause of `.sy` never binding: 197 of 200
/// workload TUs contain such a record. **The `.ex` inline reader did not have
/// it**, and the consequence is a width, not an opinion:
///
/// ```text
///   30 c6 81 46 9a 3a          load, kind 46, type id 0x1D1A — FIVE bytes
///   30 c6 81 46 | 9a 3a        what this function used to read: tag c6,
///                              "kind" 81, "id" 0x46 — THREE bytes, and the
///                              walk resumes two bytes early, on `9a`
/// ```
///
/// `9A` is the vtable-slot bind, so the desynchronized walk met a plausible
/// opcode and refused at *its* operand. That is how 129 workload bodies came to
/// be filed under `cf-vbind-type-cflow-jump` — a row named after virtual dispatch
/// containing none of it — and how a further ~200 `cf-expr-0xNN` rows, including
/// the 23,254-body `cf-expr-0x82` that ranked **second** on the control-flow
/// axis, came to be named after bytes that are the *second byte of a type id*.
/// See `docs/IL_TYPE_WIDE_TAG.md`.
///
/// **Bracketed by the grammar, not by this function's arithmetic.** The same type
/// appears at a `5C` EH-live marker in the same bodies, where the production
/// `5C <TYPE> <varint state>` is closed by a `4B`:
///
/// ```text
///   5c c6 81 46 9a 3a 01 4b    TYPE, state 1, end of statement — exact
///   5c c6 81 46 | 9a …         state = `9a`, which is not a legal varint
/// ```
///
/// The wide reading lands on the `4B`; the narrow one cannot.
/// PROV[O] `40` — the same wide-tag bit `docs/IL_TYPE_WIDE_TAG.md` derived on 2026-07-31, here in the `.ex` reader.
const TYPE_TAG_WIDE_BIT: u8 = 0x40;

/// **The wide mark's discriminator: bit 7, and it is a bit test rather than the
/// literal `81` for a measured reason.**
///
/// `.sy`'s reader requires the literal `81` because all three of its witnesses
/// have it. `.ex` has a second value: `C6 84 43 <id>` occurs 106 times on the
/// 878-TU workload, bracketed exactly — `2C c6 84 43 bf 82 01 00  55 c6 84 43 bf
/// 82 01` is a CONVERT and a push of the *same six-byte type*, and no other
/// width closes both. Requiring `81` literally refuses 36 bodies that are really
/// there; that was measured as `cf-load-type-0xC6` (18) + `cf-convert-type-0xC6`
/// (18) before this constant existed, which is the same shape as `CA 81 0D`
/// refuting the literal `C6 81` prefix one container over.
///
/// Bit 7 is the discriminator and not merely a convenience: this reader is also
/// called speculatively at positions that are **not** types (a blocker's own
/// naming, `mcall`'s lookahead), and a bit-6 tag met there is the middle of some
/// other field's LEB. Instrumented over the whole workload, the byte here is
/// `81` (213,140 calls, on three tags), `84` (106, on one), or one of `01`…`07`
/// (60,819, spread thinly across some fifty tags) — and **the settling
/// measurement is not that description but a scan**: stepping this byte
/// unchecked decodes 2,394,338 bodies and the bit-7 test decodes 2,394,338, the
/// same number to the function, so the low group never contributed a decode. Bit
/// 7 is what a LEB continuation bit would do if "tag + mark + kind" is really
/// "tag + two-byte kind". The value is otherwise NOT interpreted, and a third
/// value with bit 7 clear would be refused rather than read — see
/// `docs/IL_TYPE_WIDE_TAG.md` §8.2.
/// PROV[O] `80`, the mark byte a wide tag's extra byte carries, read off captures.
const TYPE_WIDE_MARK_BIT: u8 = 0x80;

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
/// does not need them for a scalar — the width rule is tag-independent *there*.
/// The `kind` byte is treated as a fixed byte rather than a second LEB because
/// `88 85 41` (`double`) and `88 81 13` (`long long`) have bit 7 set there and
/// would otherwise run on.
///
/// **One exception, and it is the whole reason this function is not three lines:
/// an aggregate carries its size inline, and above 31 bytes the size moves out
/// of the tag/kind pair and in front of the id.** See
/// [`AGGREGATE_CLASS`] and `docs/IL_LOAD_TYPES.md` §1a. Before that was decoded
/// this function LEB-read straight past the size, which is a **parse desync
/// inside the positive parser** — the worst failure mode this project has, since
/// a stream read at the wrong alignment can land on a valid accepted shape by
/// chance and emit bytes for a body it never actually understood, and (measured
/// below) it mis-attributes the census bucket even when it does refuse.
pub(crate) fn read_type(seg: &[u8], p: usize) -> Option<(u8, u8, u32, usize)> {
    let tag = *seg.get(p)?;
    if tag & 0x80 == 0 {
        return None;
    }
    let mut i = p + 1;
    // The WIDE prefix — see [`TYPE_TAG_WIDE_BIT`] and [`TYPE_WIDE_MARK_BIT`].
    if tag & TYPE_TAG_WIDE_BIT != 0 {
        if *seg.get(i)? & TYPE_WIDE_MARK_BIT == 0 {
            return None;
        }
        i += 1;
    }
    let kind = *seg.get(i)?;
    i += 1;
    if kind & 0x0F == AGGREGATE_CLASS {
        // The 5-bit inline size (see `AGGREGATE_CLASS`). Zero is not a legal
        // struct size — C++ has no zero-sized object — so it is free to mean
        // "the size did not fit; it follows as a statement varint".
        let size5 = ((tag & 0x01) << 4) | (kind >> 4);
        if size5 == 0 {
            let size = read_varint(seg, &mut i)?;
            // Fail closed on anything the 5-bit field could itself have carried
            // (and on the negative values a mis-aligned byte produces): under
            // this rule the escape is only reachable at >= 32, so a smaller
            // value means we are not looking at an aggregate size and must not
            // pretend to know where the type ends. This is the three-valued
            // answer — "undetermined" refuses rather than guessing a width.
            if size < 32 {
                return None;
            }
        }
    }
    let mut id: u32 = 0;
    let mut shift: u32 = 0;
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
/// PROV[O] `86 42 75`, read off `.ex` captures beside [`INT_TYPE`].
pub(crate) const UINT_TYPE: [u8; 3] = [0x86, 0x42, 0x75];

/// `long` (`86 41 12`) and `unsigned long` (`86 42 22`). On this target they are
/// 32-bit, and c2 emits **byte-identical** code for them and for `int`/`unsigned`
/// — see `docs/IL_TYPE_TAGS.md` §3.1.
/// PROV[O] `86 41 12`, read off `.ex` captures.
pub(crate) const LONG_TYPE: [u8; 3] = [0x86, 0x41, 0x12];
// PROV[O] `86 42 22`, read off `.ex` captures.
pub(crate) const ULONG_TYPE: [u8; 3] = [0x86, 0x42, 0x22];

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
/// PROV[N] derived from four already-marked constants and nothing else — a grouping, not an observation.
const INT_LIKE_TYPES: [[u8; 3]; 4] = [INT_TYPE, UINT_TYPE, LONG_TYPE, ULONG_TYPE];

/// Consume a **width-4 integer TYPE in any spelling** at `p`, reporting whether
/// it matched: one of the four bare [`INT_LIKE_TYPES`] triples, or any TYPE
/// [`is_int4_type`] admits on its tag/kind nibbles.
///
/// The second arm is what makes an `enum`, a `typedef`, a `const int` or a
/// `volatile int` an int-like operand. Those carry a **per-TU type id** in place
/// of the fixed third byte, so an exact-triple whitelist cannot see them however
/// ordinary the code is — `int get(S* s){ return s->e; }` for an `enum` member
/// emits the identical `lwz r3,off(r3) ; blr` as for an `int` one, and refused.
/// Measured by counterfactual over the 878-TU workload: the whitelist was
/// over-refusing by **15,924 functions**, against a 5,684 estimate attributed
/// from three census key names — `docs/ROADMAP.md` §6d, and the §6 rule it is
/// the third instance of (estimate the *fix*, not the finding: `eat_int_like`
/// has five call sites and the key-name estimate covered one attribution).
///
/// [`is_int4_type`] is the same predicate the `2C` conversion target, the `41`
/// result annotation and the `30` load already agree through, so this is one
/// locator gaining a call site rather than a new rule. It requires the tag's
/// width nibble to say **4-byte alignment** *and* the kind's high nibble to say
/// **4-byte size**, so the narrow types, `long long`, and a 4-byte int under
/// `#pragma pack(1)` all still refuse — `fixtures/cpp/w22_int_spelling_neg.cpp`
/// holds them, and the packed pair is the fixture for `docs/GAPS.md` §6's third
/// wrong-bytes emit.
pub(crate) fn eat_int_like(seg: &[u8], p: &mut usize) -> bool {
    if INT_LIKE_TYPES.iter().any(|t| eat(seg, p, t)) {
        return true;
    }
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_int4_type(tag, kind) => {
            *p += w;
            true
        }
        _ => false,
    }
}

/// The `float` operand type (`86 45 40`) and the `double` one (`88 85 41`).
/// Note the *literal* forms differ again ([`FLOAT_LIT_TYPE`] /
/// [`DOUBLE_LIT_TYPE`]).
/// PROV[O] `86 45 40`, from `il_parser.py`'s `KNOWN_TYPES` and a live 16.00.11886.00 float-leaf capture.
pub(crate) const FLOAT_TYPE: [u8; 3] = [0x86, 0x45, 0x40];
// PROV[O] `88 85 41`, read off `.ex` captures.
pub(crate) const DOUBLE_TYPE: [u8; 3] = [0x88, 0x85, 0x41];

/// The *literal* FP type tags, which are distinct from the operand ones above.
/// A float literal carries `86 4a 40`, a double one `88 8a 41`.
/// PROV[O] `86 4A 40`, the literal-position float type triple, read off captures.
pub(crate) const FLOAT_LIT_TYPE: [u8; 3] = [0x86, 0x4A, 0x40];
// PROV[O] `88 8A 41`, read off captures beside [`FLOAT_LIT_TYPE`].
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
/// The tag's width nibble is the value's **alignment**, and `kind`'s high nibble is
/// its **size**. Those agree for every naturally-aligned type, which is why reading
/// only the tag survived — until `#pragma pack(4)`, where an 8-byte `long long`
/// member carries `86 81 …`: tag says align 4, kind says size 8. That passed this
/// predicate and lowered `(int)s->q` to a single `lwz` at the wrong offset —
/// `Port=Mismatch @ offset 8`. So the size is checked too, and it is checked from
/// the field that actually carries it.
///
/// Only the size check is added: a 4-byte int at a *smaller* alignment (`pack(1)`)
/// still refuses on the tag, as before. Whether an unaligned `lwz` is even what c2
/// emits there is unprobed, and admitting it on the strength of this decode would be
/// widening on an assumption.
pub(crate) fn is_int4_type(tag: u8, kind: u8) -> bool {
    type_width(tag) == Some(4) && (kind >> 4) == 4 && matches!(kind & 0x0F, 0x1 | 0x2)
}

/// True for a TYPE naming a **pointer to a 4-byte object**: `kind`'s low nibble
/// is 3 and the tag says 4. In a `B9` operand position a pointer's tag is the
/// *pointer's* own width (`86 43 f4 08` = `int *`); in the `27` byte-offset-add
/// position it is the **pointee's** width instead (`82 43 f0 08` for `char *`,
/// `88 43 c1 08` for `double *`), which is why this is applied to the `27` type
/// and not only to the base LOAD.
pub(crate) fn is_ptr_to_4(tag: u8, kind: u8) -> bool {
    // `kind`'s high nibble is the size of the *pointer*, which is 4 on this target
    // in every witness (`82 43 …` char *, `88 43 …` double *). Checked for the same
    // reason as [`is_int4_type`]: the tag carries alignment, not size.
    type_width(tag) == Some(4) && (kind >> 4) == 4 && (kind & 0x0F) == 0x3
}

/// True for a TYPE tag carrying the **`volatile`** qualifier (bit `0x10`), which
/// at a `B9` operand LOAD is a whole stack frame and nowhere else is anything.
///
/// The thirteenth live wrong-bytes emit, found by this rung's neighbour grid and
/// **pre-existing on mainline across five shapes at once** — the straight-line
/// leaf, the integer tail call, the framed call, the discarded statement call and
/// the multi-argument permutation. A `volatile`-qualified *parameter* is a
/// volatile object, so c2 homes the incoming register in the frame and reads it
/// back from memory at every use:
///
/// ```text
///   int   f(int x, volatile int y)   { return y; }        stw r4,124(r1) ; lwz r3,124(r1)
///   float f(float x, volatile float y) { return gf(y); }  <96-byte frame>
///     mflr r12 · stw r12,-8(r1) · stwu r1,-96(r1)
///     d041007c  stfs f2,124(r1)      <- homed
///     c021007c  lfs  f1,124(r1)      <- read back
///     4bffffed  bl ?gf               <- and therefore NOT a tail call
///     addi r1,r1,96 · lwz r12,-8(r1) · mtlr r12 · blr
/// ```
///
/// against the port's `fmr f1,f2 ; b ?gf`. `Port=Mismatch @ offset 2` — the
/// section count, because the reference obj has a `.pdata` the port never
/// emitted.
///
/// **The bit is only load-bearing at the `B9` LOAD**, and that is measured, not
/// assumed. Three positions carry a volatile tag and two of them are free:
///
/// ```text
///   int f(volatile int y)      { return y; }    b9 <y> 96 41 …   REFUSE  (spills)
///   int f(int* volatile p)     { return *p; }   b9 <p> 96 43 …   REFUSE  (spills)
///   int f(volatile S* p)       { return p->i; } b9 <p> 86 43 …   free — the POINTER
///                                               30     96 41 …   is not volatile and
///   struct S { volatile int i; };                                 the load-through is
///   int f(S* p)                { return p->i; } 30     96 41 …   one `lwz` either way
/// ```
///
/// So the gate goes on the operand LOAD and **not** on [`eat_int_like`],
/// [`eat_value_type`] or the `27`/`30` designator readers, where the same two
/// bytes appear and cost nothing. A `volatile` formal the body never reads is
/// also free (`int f(int x, volatile int y){ return x; }` is a bare `blr`), and
/// it stays in class for the same reason: there is no LOAD to refuse.
///
/// `const` (bit `0x20`) is genuinely free everywhere and is untouched —
/// `A6 41 84 20` is admitted exactly as before. It is the *pair* that makes this
/// a measurement rather than a guess: `const float` and `volatile float` differ
/// in one bit of one byte and in a whole stack frame.
pub(crate) fn is_volatile_tag(tag: u8) -> bool {
    tag & TAG_VOLATILE != 0
}

/// The TYPE tag's `volatile` bit. `86` is plain, `A6` adds `const` (`0x20`),
/// `96` adds `volatile`, `B6` is both — the four combinations `is_ptr4_kind`
/// whitelists, read as bits here because only one of them changes the emission.
/// PROV[O] the `volatile` qualifier bit in a type tag, read off captures.
const TAG_VOLATILE: u8 = 0x10;

/// True for a TYPE naming a **floating-point value**, returning `true` for the
/// 8-byte (`double`) one and `false` for the 4-byte (`float`) one.
///
/// The same two-channel test [`is_int4_type`] uses, with the class nibble moved
/// from "signed or unsigned" to **`5` = real**: the tag's low nibble is the
/// value's alignment and the kind's high nibble is its *size*, and both must say
/// the same width. Captured witnesses are the two bare triples
/// ([`FLOAT_TYPE`] `86 45 40`, [`DOUBLE_TYPE`] `88 85 41`); reading the nibbles
/// rather than whitelisting the triples is what admits a `const float` parameter
/// (`A6 45 …`) and a `typedef float Real` — which carry a **per-TU type id** in
/// place of the fixed third byte and are therefore invisible to an exact-triple
/// compare, however ordinary the code. That is `docs/ROADMAP.md` §6d's lesson
/// (`eat_int_like`'s whitelist was over-refusing by 15,924 functions) applied to
/// the FP file rather than re-learned on it.
///
/// The width is **not** a licence to ignore it: `float` and `double` occupy one
/// FP register each (`docs/CODEGEN_FP_ARGS.md` §1, the `t8` capture), but a
/// conversion *between* them is free in one direction and an `frsp` in the other,
/// so every consumer is handed the bit rather than a bare bool.
pub(crate) fn is_fp_type(tag: u8, kind: u8) -> Option<bool> {
    // `volatile float` is a memory object and its LOAD is a spill, not a move —
    // see [`is_volatile_tag`], and `float f(float x, volatile float y)
    // { return gf(y); }` is a 40-byte framed body where this rung emits 8.
    if is_volatile_tag(tag) {
        return None;
    }
    let w = type_width(tag)?;
    if u32::from(kind >> 4) != w || (kind & 0x0F) != TYPE_KIND_REAL_CLASS {
        return None;
    }
    match w {
        4 => Some(false),
        8 => Some(true),
        // A "real" that is neither 4 nor 8 bytes — `long double` under some other
        // flag, or a misread record. Never guessed; the same refusal
        // `sy::SyView::arg_classes` makes on the `.sy` side of the same fact.
        _ => None,
    }
}

/// The TYPE kind's **class nibble** for a floating-point ("real") type. The `.sy`
/// side of the same fact is `sy::TYPE_KIND_REAL`; both read the low nibble of the
/// kind byte, and a union being `16` where a struct is `06` is why neither may
/// compare the whole byte.
/// PROV[O] the type-kind byte for a real (non-forward-reference) class, read off captures.
pub(crate) const TYPE_KIND_REAL_CLASS: u8 = 0x5;

/// Consume a floating-point TYPE at `p` ([`is_fp_type`]), reporting `true` for a
/// `double`. `p` is untouched when the type is not an FP one.
pub(crate) fn eat_fp_type(seg: &[u8], p: &mut usize) -> Option<bool> {
    let (tag, kind, _, w) = read_type(seg, *p)?;
    let double = is_fp_type(tag, kind)?;
    *p += w;
    Some(double)
}

/// A TYPE naming a **width-4 pointer value**: a data pointer (kind class 3) or a
/// function/code pointer (kind class 4), in any cv-qualification.
///
/// Spelled as a literal tag/kind whitelist rather than as nibble arithmetic,
/// because the two bytes are not equally well understood and the honest gate
/// says so:
///
/// * `tag` — `0x80` plus the cv bits (`0x20` const, `0x10` volatile) and the
///   width nibble `6` (= 4 bytes). All four combinations occur and are
///   captured: `86 43` a plain pointer, `A6 43` a const one (the type of
///   `this`, and of a member read through a const `this`), `96`/`B6` the
///   volatile pair. **`0xC6` is refused.** `readers.rs` records that bit 0x40
///   occurs, and none of the `IL_LOAD_TYPES.md` probes produced it — a field
///   that never varied across the probes is indistinguishable from a constant,
///   so it is required literally and fails closed. Odd tags (bit 0 set) are the
///   aggregate size-bit-4 encoding and are not pointers at all.
/// * `kind` — required to be exactly `0x43` or `0x44`, i.e. width nibble 4 with
///   class nibble 3 or 4. Class 3 is a data pointer and class 4 a function/code
///   pointer (`IL_LOAD_TYPES.md` §1: `int (*)()` literal 0 is `33 86 44 8d 20
///   00`). Both load with the same `lwz`, so gating them together keeps one
///   instruction behind one predicate.
///
/// Deliberately **not** [`is_ptr_to_4`], which is the *other* question: that one
/// is applied to the base LOAD and to the `27` byte-offset-add, where the tag
/// carries the **pointee's** width and only the kind class is meaningful. Two
/// predicates because two facts; one locator each.
pub(crate) fn is_ptr4_kind(tag: u8, kind: u8) -> bool {
    matches!(tag, 0x86 | 0x96 | 0xA6 | 0xB6) && matches!(kind, 0x43 | 0x44)
}

/// Consume the operand TYPE of a `parse_expr` LOAD/LIT position: an int-like
/// triple ([`eat_int_like`]) **or** a width-4 pointer value ([`is_ptr4_kind`]).
/// Returns `Some(true)` when the type consumed was the pointer one, `Some(false)`
/// for the int-like one, and `None` — with `p` untouched — for anything else.
///
/// The two are one *position* but not one *class*, and the caller is told which
/// it got because they are not interchangeable under arithmetic: see
/// `super::body::expr::parse_expr`'s pointer-arithmetic guard and
/// `docs/IL_CALL_IN_EXPR.md` §21.
/// The two value classes the modeled shapes lower identically over: a 4-byte
/// integer and a 4-byte pointer. Kept as a class rather than a bare bool so that a
/// `2C` conversion target, a `41` result and a `30` load must **agree** with each
/// other rather than merely each be "some 4-byte thing" — see
/// [`super::body::shapes::finish_indirect_load`] and
/// [`super::body::expr::parse_expr`]'s `2C` arm.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ValueClass {
    /// `int`/`unsigned`/`long`, in any cv-qualification ([`is_int4_type`]).
    Int4,
    /// A width-4 data or function pointer ([`is_ptr4_kind`]).
    Ptr4,
    /// **`bool` / `unsigned char`** — the one-byte unsigned class ([`is_int1u_type`]).
    ///
    /// Its own class, not a spelling of [`ValueClass::Int4`], and the distinction
    /// is a captured instruction rather than tidiness: a value **inside** the
    /// class is free in a register (`bool f(bool b){ return b; }` is a bare
    /// `blr`, `bool f(){ return false; }` is `li r3,0`, `bool f(int k,bool b)
    /// { return b; }` is the same `mr r3,r4` the int identity emits), while a
    /// conversion **out of** it is a real mask — `unsigned u(bool b)
    /// { return b; }` is `5463063e`, `rlwinm r3,r3,0,24,31`. Keeping it apart is
    /// what makes `eat_value_type` refuse that `2C` instead of dropping it.
    ///
    /// `bool` and `unsigned char` are one class because their TYPE `<tag><kind>`
    /// is one thing (`82 12`) and only the per-TU id differs; no capture
    /// separates them in any position this parser reaches.
    Int1u,
}

/// True for a TYPE naming the **one-byte unsigned** class — `bool`,
/// `unsigned char`, and the `unsigned char` typedefs — `82 12 <id>`.
///
/// Required as the literal pair rather than as nibble arithmetic, for the reason
/// [`is_ptr4_kind`] is: the tag carries the *alignment* class and the kind's high
/// nibble the *size*, and both say 1 here, so the pair is a free double check
/// against a misaligned read. The cv-qualified spellings (`A2 12` const,
/// `92 12` volatile) are **not** admitted — neither occurs as an operand in any
/// capture taken for this rung, and a tag that never varied is indistinguishable
/// from a constant (`GAPS.md` §6).
///
/// `82 11` — `char`/`signed char` — is deliberately absent: it is the same width
/// and a different class, and a *signed* narrow value widened to `int` costs an
/// `extsb` where this one costs a `rlwinm` or nothing. One predicate per fact.
pub(crate) fn is_int1u_type(tag: u8, kind: u8) -> bool {
    (tag, kind) == (0x82, 0x12)
}

pub(crate) fn value_class(tag: u8, kind: u8) -> Option<ValueClass> {
    if is_int4_type(tag, kind) {
        Some(ValueClass::Int4)
    } else if is_ptr4_kind(tag, kind) {
        Some(ValueClass::Ptr4)
    } else {
        None
    }
}

/// Consume the operand TYPE of a `parse_expr` LOAD/LIT position, widened to the
/// **one-byte unsigned** class ([`is_int1u_type`]) beside the two width-4 ones.
///
/// A separate entry point rather than a widening of [`eat_int_like_or_ptr4`],
/// which has five call sites and gates three byte-graded shapes: `ROADMAP.md`
/// §6d is the record of what changing a shared locator costs, and only the two
/// `parse_expr` operand positions have been graded for this class. The caller
/// gets the class back and is required to act on it — a `bool` value may not
/// enter arithmetic, may not be converted, and its `41` result annotation must
/// restate the class.
pub(crate) fn eat_operand_type(seg: &[u8], p: &mut usize) -> Option<ValueClass> {
    // **A `volatile` operand is a memory object, and reading it is a memory
    // access.** See [`is_volatile_tag`]: at THIS position the qualifier costs a
    // whole frame, and the port emitted the register move — a live wrong-bytes
    // emit across five shapes at once. The gate is here, at the operand type,
    // rather than inside [`eat_int_like_or_ptr4`], because that locator also
    // serves the `55` call-end, the `41` result and the `2C` target, where the
    // same qualifier is free and where refusing it would cost coverage for
    // nothing.
    if read_type(seg, *p).is_some_and(|(tag, _, _, _)| is_volatile_tag(tag)) {
        return None;
    }
    if let Some(is_ptr) = eat_int_like_or_ptr4(seg, p) {
        return Some(if is_ptr { ValueClass::Ptr4 } else { ValueClass::Int4 });
    }
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_int1u_type(tag, kind) => {
            *p += w;
            Some(ValueClass::Int1u)
        }
        _ => None,
    }
}

/// Consume a TYPE at `p` iff it belongs to `class`, reporting whether it did.
pub(crate) fn eat_value_type(seg: &[u8], p: &mut usize, class: ValueClass) -> bool {
    match class {
        // Both sides now consume by tag/kind, so a `2C` whose target is a
        // width-4 integer carrying a per-TU type id (an enum, a typedef, a
        // `const int`) is admitted exactly where `is_int4_type` says it is a
        // no-op — the same predicate `value_class` classifies with.
        ValueClass::Int4 => eat_int_like(seg, p),
        ValueClass::Ptr4 => match read_type(seg, *p) {
            Some((tag, kind, _, w)) if is_ptr4_kind(tag, kind) => {
                *p += w;
                true
            }
            _ => false,
        },
        ValueClass::Int1u => match read_type(seg, *p) {
            Some((tag, kind, _, w)) if is_int1u_type(tag, kind) => {
                *p += w;
                true
            }
            _ => false,
        },
    }
}

/// Consume a `2C` target TYPE at `p` that names **the other width-4
/// [`ValueClass`]** than the one the value already has, returning the class it
/// names. `None` — with `p` untouched — for anything else.
///
/// This is the **width-4 reinterpret**, and it is a separate entry point from
/// [`eat_value_type`] rather than a widening of it for the reason that locator's
/// own comment gives: `eat_value_type` gates three byte-graded shapes through
/// five call sites, and only [`super::body::expr::parse_expr_classed`]'s `2C` arm
/// has been graded for the cross-class case. A caller reaching this has *already*
/// been refused by `eat_value_type`, so the class returned is necessarily
/// different from the one passed in.
///
/// ## The 3×3, measured — `lane w-convert`, board **#700**
///
/// c2's `.text` for `T g(U x){ return (T)x; }` at `/Ox` and at the workload's
/// `/O1 /Oi /EHsc /GR`, both identical (`work/w-convert/probe/m1.cpp`,
/// `m4.cpp`). "free" is a bare `blr` — the conversion emits nothing at all:
///
/// ```text
///   source \ target |  Int4          Ptr4          Int1u
///   ----------------+------------------------------------------
///   Int4            |  free  (old)   free  (NEW)   clrlwi 24
///   Ptr4            |  free  (NEW)   free  (old)   clrlwi 24
///   Int1u           |  clrlwi 24     clrlwi 24     free  (old)
/// ```
///
/// **Four of the nine cells cost an instruction and this predicate admits none
/// of them.** `Int1u` is barred on *both* sides by an explicit arm rather than
/// by falling off the end, because the enum makes it look like a peer of the
/// other two and it is not: `unsigned u(bool b){ return b; }` is
/// `rlwinm r3,r3,0,24,31`, and so is `void *p(bool b){ return (void *)b; }` —
/// the pointer direction is *not* the free one people expect from the fact that
/// a pointer is one register. The one free cell in that row
/// (`bool`→`char`, `82 11`) names a class this parser does not model, so there
/// would be nothing to track the result as; it refuses with the rest.
///
/// ## The widening this admits is graded across the axes it could hide in
///
/// 31 cells at both profiles, all free: every integer spelling (`int`,
/// `unsigned`, `long`, `unsigned long`, a typedef, an enum) against every
/// pointee (`void`, `int`, `char`, `const char`, `S`, `S*`, a **function**
/// pointer, `const S`, `volatile S`), both directions, and at every position —
/// leaf return, each of four call-argument slots, permuted slots, a repeated
/// operand, a nested call, and `this` (`A6 43`). Signedness is the axis this
/// target could plausibly have broken on — the GPRs are 64-bit and the pointers
/// 32-bit, so a *signed* `int`→pointer is where an `extsw`/`rldicl` would appear
/// if one appeared anywhere. It does not: `void *f(int a){ return (void *)a; }`
/// is a bare `blr`.
///
/// ## What the caller still owes
///
/// **A convert that produces [`ValueClass::Ptr4`] must indict the value for the
/// pointer-arithmetic guard exactly as a pointer LOAD does.** `(S *)a + 1` is
/// `addi r3,r3,8` — c2 **scales** — and `(S *)a + k` is `slwi r11,r4,3 ; add`.
/// A chain that added 1 unscaled would be a wrong emit, not a gap. This
/// predicate cannot enforce that itself; `parse_expr_classed` sets `saw_ptr` on
/// the accepting arm and `scripts/sweep.d/77-reinterpret-2c.py` is the corpus
/// that can express the failure if it is ever dropped.
pub(crate) fn eat_reinterpret_type(
    seg: &[u8],
    p: &mut usize,
    class: ValueClass,
) -> Option<ValueClass> {
    let (tag, kind, _, w) = read_type(seg, *p)?;
    let got = match class {
        ValueClass::Int4 if is_ptr4_kind(tag, kind) => ValueClass::Ptr4,
        ValueClass::Ptr4 if is_int4_type(tag, kind) => ValueClass::Int4,
        // Barred on both sides, and by its own arm. See the table above.
        ValueClass::Int1u => return None,
        _ => return None,
    };
    *p += w;
    Some(got)
}

pub(crate) fn eat_int_like_or_ptr4(seg: &[u8], p: &mut usize) -> Option<bool> {
    if eat_int_like(seg, p) {
        return Some(false);
    }
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_ptr4_kind(tag, kind) => {
            *p += w;
            Some(true)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tag carries ALIGNMENT and the kind carries SIZE, and they diverge under
    /// `#pragma pack`. Every captured witness has them equal, which is exactly why
    /// reading only the tag looked correct.
    #[test]
    fn a_packed_eight_byte_int_is_not_a_four_byte_int() {
        // `int`, `unsigned`, `long`, `unsigned long`, `const int` — size 4 in the
        // kind's high nibble, and all admitted.
        for (tag, kind) in [(0x86, 0x41), (0x86, 0x42), (0xA6, 0x41), (0x96, 0x41)] {
            assert!(is_int4_type(tag, kind), "{tag:02X} {kind:02X}");
        }
        // `#pragma pack(4)` `long long`: tag says align 4, kind says size 8. This
        // was admitted and lowered to one `lwz` at the wrong offset.
        assert!(!is_int4_type(0x86, 0x81));
        // A `double` at 4-byte alignment is the same trap in the FP family.
        assert!(!is_int4_type(0x86, 0x85));
        // `is_ptr_to_4` is "pointer to a 4-byte OBJECT", and in the `27` position the
        // tag carries the POINTEE's width — so `int *` passes and `char *` / `double *`
        // are refused, which is the existing intent and not a size bug.
        assert!(is_ptr_to_4(0x86, 0x43)); //  int *
        assert!(!is_ptr_to_4(0x82, 0x43)); // char *   — 1-byte pointee
        assert!(!is_ptr_to_4(0x88, 0x43)); // double * — 8-byte pointee
        // The pointer's own size sits in the kind's high nibble and is 4 in every
        // witness; a kind claiming 8 is not a pointer this target emits.
        assert!(!is_ptr_to_4(0x86, 0x83));
    }

    /// The `volatile` bit is `0x10` on the tag, `const` is `0x20`, and only the
    /// first of them reaches the emitted bytes — W32, the thirteenth live
    /// wrong-bytes emit. Every tag below is transcribed from a live capture; the
    /// four rows of the `const`/`volatile` cross product occur and are named.
    #[test]
    fn the_volatile_qualifier_is_a_tag_bit_and_const_is_a_different_one() {
        for tag in [0x96u8, 0xB6] {
            assert!(is_volatile_tag(tag), "tag {tag:02X} is volatile");
        }
        for tag in [0x86u8, 0xA6, 0x82, 0x88, 0x84] {
            assert!(!is_volatile_tag(tag), "tag {tag:02X} is not volatile");
        }
        // …and the operand LOAD refuses it in every class it would otherwise
        // admit, while `const` is admitted in all of them. Captured spellings:
        //   96 41 80 20  volatile int      A6 41 84 20  const int
        //   96 43 80 20  int* volatile     A6 43 8F 20  int* const (and `this`)
        //   96 45 82 20  volatile float    A6 45 …      const float
        for (kind, what) in [(0x41u8, "int"), (0x43, "pointer"), (0x45, "float")] {
            let mut p = 0usize;
            assert!(
                eat_operand_type(&[0x96, kind, 0x80, 0x20], &mut p).is_none(),
                "volatile {what} must refuse at the operand LOAD"
            );
            assert_eq!(p, 0, "a refusal must not advance the cursor");
        }
        // The FP predicate answers the same way, and keeps the width.
        assert_eq!(is_fp_type(0x86, 0x45), Some(false));
        assert_eq!(is_fp_type(0x88, 0x85), Some(true));
        assert_eq!(is_fp_type(0xA6, 0x45), Some(false)); // `const float` is free
        assert_eq!(is_fp_type(0x96, 0x45), None); // `volatile float` is a spill
        assert_eq!(is_fp_type(0xB6, 0x85), None);
        // …and the non-FP classes are not real types however wide they are.
        assert_eq!(is_fp_type(0x86, 0x41), None); // int
        assert_eq!(is_fp_type(0x86, 0x43), None); // pointer
        assert_eq!(is_fp_type(0x86, 0x85), None); // tag says 4, kind says 8
        assert_eq!(is_fp_type(0x82, 0x15), None); // a 1-byte "real"
    }

    /// The widened operand-type reader. Both halves are required to be exactly
    /// what they say: the int side keeps its four-triple whitelist, the pointer
    /// side is [`is_ptr4_kind`], and the caller is told which it got because the
    /// two are NOT interchangeable under arithmetic (`body::expr::parse_expr`).
    #[test]
    fn operand_type_takes_int_like_or_a_four_byte_pointer_and_says_which() {
        // Captured pointer operand types, all from live `.ex` captures:
        //   `86 43 f4 08` int*      `86 43 f0 08` char*    `86 43 c1 08` double*
        //   `A6 43 8f 20` int* const (and `this`)          `86 44 8d 20` int(*)()
        let ptr: &[&[u8]] = &[
            &[0x86, 0x43, 0xF4, 0x08],
            &[0x86, 0x43, 0xF0, 0x08],
            &[0x86, 0x43, 0xC1, 0x08],
            &[0xA6, 0x43, 0x8F, 0x20],
            &[0x96, 0x43, 0x8F, 0x20],
            &[0xB6, 0x43, 0x8F, 0x20],
            &[0x86, 0x44, 0x8D, 0x20],
            &[0x86, 0x43, 0x9B, 0xB9, 0x02], // 5-byte id
        ];
        for bytes in ptr {
            let mut p = 0usize;
            assert_eq!(eat_int_like_or_ptr4(bytes, &mut p), Some(true), "{bytes:02X?}");
            assert_eq!(p, bytes.len(), "consumed the whole type {bytes:02X?}");
        }
        for bytes in [INT_TYPE, UINT_TYPE, LONG_TYPE, ULONG_TYPE] {
            let mut p = 0usize;
            assert_eq!(eat_int_like_or_ptr4(&bytes, &mut p), Some(false));
            assert_eq!(p, 3);
        }
        // W22: a width-4 integer carrying a per-TU type id — an enum, a typedef,
        // a `const`/`volatile` qualification — is the same operand and the same
        // instruction. The exact-triple whitelist could not see any of them, and
        // that cost 15,924 functions on the workload (`ROADMAP.md` §6d). Admitted
        // through `is_int4_type`, which is the predicate the `2C` target and the
        // `41` result already agree through.
        let int_spellings: &[(&[u8], &str)] = &[
            (&[0x86, 0x42, 0x76], "unsigned with a per-TU id (an enum, a typedef)"),
            (&[0xA6, 0x41, 0x84, 0x20], "const int"),
            (&[0x96, 0x41, 0x86, 0x20], "volatile int"),
        ];
        for (bytes, label) in int_spellings {
            let mut p = 0usize;
            assert_eq!(eat_int_like_or_ptr4(bytes, &mut p), Some(false), "{label}");
            assert_eq!(p, bytes.len(), "{label}: consumed the whole type");
        }
        // Everything else refuses, and refuses WITHOUT moving the cursor — the
        // caller reports the census key from the untouched position.
        let no: &[(&[u8], &str)] = &[
            (&[0x86, 0x45, 0x40], "float"),
            (&[0x88, 0x85, 0x41], "double"),
            (&[0x88, 0x81, 0x13], "long long"),
            (&[0x82, 0x07, 0x03], "void"),
            (&[0x86, 0x46, 0x80, 0x20], "aggregate"),
            (&[0xC6, 0x43, 0x8F, 0x20], "tag bit 0x40 — never captured, fails closed"),
            (&[0x87, 0x43, 0x8F, 0x20], "odd tag is the aggregate size bit"),
            (&[0x86, 0x83, 0x8F, 0x20], "kind claims an 8-byte pointer"),
            (&[0x41, 0x86, 0x41], "not a type at all"),
        ];
        for (bytes, label) in no {
            let mut p = 0usize;
            assert_eq!(eat_int_like_or_ptr4(bytes, &mut p), None, "{label}");
            assert_eq!(p, 0, "{label}: the cursor must not move on a refusal");
        }
    }

    /// The two pointer predicates answer two different questions and must not be
    /// merged: in a `B9`/`33` operand position the tag is the POINTER's own width
    /// (so `char*` and `double*` are both admitted as values), while in the `27`
    /// byte-offset-add position it is the POINTEE's (so [`is_ptr_to_4`] refuses
    /// them). Both readings are captured — `86 43 f0 08` is `char*` at a LOAD and
    /// `82 43 f0 08` is the same `char*` at a `27`.
    #[test]
    fn the_operand_position_reads_the_pointers_own_width() {
        assert!(is_ptr4_kind(0x86, 0x43) && !is_ptr_to_4(0x82, 0x43));
        let mut p = 0usize;
        assert_eq!(eat_int_like_or_ptr4(&[0x86, 0x43, 0xF0, 0x08], &mut p), Some(true));
        // The `27` spelling of that same `char*` is NOT a 4-byte pointer value and
        // must not be admitted here — the tag says width 1.
        let mut q = 0usize;
        assert_eq!(eat_int_like_or_ptr4(&[0x82, 0x43, 0xF0, 0x08], &mut q), None);
    }

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

    // ---- aggregates: the size ladder (docs/IL_LOAD_TYPES.md §1a) -------------

    /// Every row of the §1a struct-copy ladder, verbatim, with the width the
    /// bracketing marker in the capture pins. The rows below 32 bytes are the
    /// ones the old reader got right *by accident* (opaque kind byte + LEB id);
    /// they are here so a future change to the aggregate branch cannot quietly
    /// break them.
    #[test]
    fn read_type_walks_the_aggregate_size_ladder() {
        //           bytes                          tag    kind   id      width
        let cases: &[(&[u8], u8, u8, u32, usize, &str)] = &[
            (&[0x86, 0x46, 0x80, 0x20], 0x86, 0x46, 0x1000, 4, "S4 size 4 align 4"),
            (&[0x86, 0x86, 0x86, 0x20], 0x86, 0x86, 0x1006, 4, "S8 size 8"),
            (&[0x86, 0xC6, 0x80, 0x20], 0x86, 0xC6, 0x1000, 4, "B12 size 12"),
            (&[0x82, 0xF6, 0x8C, 0x20], 0x82, 0xF6, 0x100C, 4, "S15 size 15 align 1"),
            // tag bit 0 set → size bit 4. `type_width(0x87)` is None, so these
            // used to refuse *before* stepping; now they step correctly and are
            // refused by the class gate instead. Both are refusals.
            (&[0x87, 0x06, 0x93, 0x20], 0x87, 0x06, 0x1013, 4, "S16 = 16 + 0"),
            (&[0x87, 0x46, 0x99, 0x20], 0x87, 0x46, 0x1019, 4, "S20 = 16 + 4"),
            (&[0x89, 0x06, 0x95, 0x20], 0x89, 0x06, 0x1015, 4, "SD16 align 8"),
            (&[0x83, 0xF6, 0x80, 0x20], 0x83, 0xF6, 0x1000, 4, "S31 = 16 + F, field full"),
            // …and above 31 the field is 0 and a statement varint precedes the id.
            (&[0x82, 0x06, 0x20, 0x87, 0x20], 0x82, 0x06, 0x1007, 5, "S32 varint 0x20"),
            (&[0x82, 0x06, 0x21, 0x8E, 0x20], 0x82, 0x06, 0x100E, 5, "S33 varint 0x21"),
            (&[0x86, 0x06, 0x28, 0xA0, 0x20], 0x86, 0x06, 0x1020, 5, "T40 varint 0x28"),
            // The original task-list witness, now readable: a 32-byte aggregate
            // with id 0x106C. Five bytes, where the old reader said three.
            (&[0x86, 0x06, 0x20, 0xEC, 0x20], 0x86, 0x06, 0x106C, 5, "32 B, id 0x106C"),
        ];
        for (bytes, tag, kind, id, w, label) in cases {
            assert_eq!(
                read_type(bytes, 0),
                Some((*tag, *kind, *id, *w)),
                "{label}: {bytes:02X?}"
            );
            // The point of the whole exercise: an aggregate is never accepted.
            // Stepping it correctly must not turn into admitting it.
            assert!(!is_int4_type(*tag, *kind), "{label} must not read as int4");
            assert!(!is_ptr_to_4(*tag, *kind), "{label} must not read as ptr-to-4");
        }
    }

    // ---- the WIDE tag (WVB, docs/IL_TYPE_WIDE_TAG.md) -----------------------

    /// **`work/WVB/probe/p3.cpp`, four lines, captured at the workload's own
    /// flags — the whole separation in one function.**
    ///
    /// ```cpp
    /// struct P { virtual void V(); int q; };   // 8 bytes, polymorphic
    /// struct N { int a, b; N(); };             // 8 bytes, NOT polymorphic
    /// struct D : P, N { D(); };
    /// D::D() {}
    /// ```
    ///
    /// `D`'s constructor builds both bases in two adjacent statements of the same
    /// production — `26 <ctor> 33 int 2113 40 <T> 66 02 <pair> … BD … 4C  30 <T> 4B`
    /// — so everything except `virtual` is held fixed, including the *kind byte*
    /// `86` (aggregate, size 8) and the closing `4B`:
    ///
    /// ```text
    ///   30 c6 81 86 82 20 4b        P — WIDE: tag C6, mark 81, kind 86, id 0x1002
    ///   30 86    86 93 20 4b        N — narrow: tag 86,         kind 86, id 0x1013
    /// ```
    ///
    /// **One `virtual` in the source; one byte in the type.** A class with a
    /// vtable spells its type with tag bit 6 set and one extra byte, and nothing
    /// else in this pair differs. The whole segment is scanned end to end in
    /// [`super::super::body::shapes::control_flow`]'s `the_polymorphic_base_...`
    /// test; these two are its discriminating bytes.
    const P_LOAD: &[u8] = &[0x4C, 0x30, 0xC6, 0x81, 0x86, 0x82, 0x20, 0x4B];
    const N_LOAD: &[u8] = &[0x4C, 0x30, 0x86, 0x86, 0x93, 0x20, 0x4B];

    #[test]
    fn the_polymorphic_base_takes_one_more_byte_than_its_plain_neighbour() {
        assert_eq!(read_type(P_LOAD, 2), Some((0xC6, 0x86, 0x1002, 5)));
        assert_eq!(read_type(N_LOAD, 2), Some((0x86, 0x86, 0x1013, 4)));
        // The bracket is what pins the width: both statements are closed by `4B`,
        // and only these readings land on it.
        assert_eq!(P_LOAD[2 + 5], 0x4B);
        assert_eq!(N_LOAD[2 + 4], 0x4B);
        // The kind byte is the SAME in both, so the extra byte is not a wider
        // kind value — it is a field the narrow form does not have at all.
        assert_eq!(P_LOAD[4], N_LOAD[3]);
        // …and the narrow reading of the wide one stops two bytes early, on `82`,
        // which is what the workload reported as a 23,254-body `cf-expr-0x82`.
        assert_eq!(P_LOAD[2 + 3], 0x82);
    }

    #[test]
    fn the_wide_tag_is_a_bit_test_and_the_mark_is_not_the_literal_81() {
        // `C6 84 43 bf 82 01` — the second mark value, from the wild
        // `2C <TYPE> 00  55 <TYPE>` pair that brackets it twice over. A reader
        // requiring the literal `81` (as `.sy`'s does) refuses this.
        assert_eq!(
            read_type(&[0xC6, 0x84, 0x43, 0xBF, 0x82, 0x01], 0),
            Some((0xC6, 0x43, 16703, 6))
        );
        // …and the mark's bit 7 is the discriminator against a misaligned read:
        // a bit-6 tag met in the middle of some other field's LEB has it clear.
        for m in [0x00u8, 0x01, 0x02, 0x05, 0x07, 0x7F] {
            assert_eq!(read_type(&[0xC6, m, 0x43, 0x74], 0), None, "mark {m:#04X}");
        }
        // A tag WITHOUT bit 6 keeps the three-byte reading, `81` or not — this is
        // `long long`, and treating its `81` as a mark would swallow the id.
        assert_eq!(read_type(&[0x88, 0x81, 0x13], 0), Some((0x88, 0x81, 19, 3)));
        // Truncation is a refusal, never a short read.
        assert_eq!(read_type(&[0xC6], 0), None);
        assert_eq!(read_type(&[0xC6, 0x81], 0), None);
    }

    #[test]
    fn a_wide_aggregate_is_still_never_accepted() {
        // The width fix must not become an admission: the wide types on the
        // workload are classes (class nibble 6) and pointers (3), and the gates
        // have to keep refusing both under their NEW (tag, kind) pair.
        for (bytes, tag, kind) in [
            (&[0xC6u8, 0x81, 0x86, 0x82, 0x20][..], 0xC6u8, 0x86u8),
            (&[0xC6, 0x81, 0x46, 0x9A, 0x3A][..], 0xC6, 0x46),
            (&[0xC7, 0x81, 0x46, 0xB6, 0x3B][..], 0xC7, 0x46),
        ] {
            let (t, k, _, _) = read_type(bytes, 0).expect("wide type reads");
            assert_eq!((t, k), (tag, kind));
            assert!(!is_int4_type(t, k), "{bytes:02X?} must not read as int4");
            assert!(!is_ptr_to_4(t, k), "{bytes:02X?} must not read as ptr-to-4");
        }
    }

    /// The S32/S33 pair is the *discriminating* capture, so assert the thing that
    /// discriminates rather than only the two decodings: one byte of struct
    /// growth moves the third byte. Under the "trailing bytes are a fixed class
    /// token" reading it could not move, and under a "the third byte is part of
    /// the id" reading the id would jump by 0x100 rather than the ids differing
    /// independently.
    #[test]
    fn aggregate_size_field_moves_with_the_struct_and_the_id_moves_separately() {
        let s32 = read_type(&[0x82, 0x06, 0x20, 0x87, 0x20], 0).unwrap();
        let s33 = read_type(&[0x82, 0x06, 0x21, 0x8E, 0x20], 0).unwrap();
        assert_eq!((s32.0, s32.1), (s33.0, s33.1), "tag/kind identical");
        assert_eq!((s32.3, s33.3), (5, 5), "both five bytes wide");
        assert_ne!(s32.2, s33.2, "the ids are independent of the size byte");
        // A 4-bit-only size rule (kind's high nibble alone) would read S16's
        // field as 0 and look for a varint at `93` — a negative short form — so
        // it fails closed exactly where the 5-bit rule steps four bytes. That is
        // what makes the two rules distinguishable rather than merely different.
        assert_eq!(read_type(&[0x87, 0x06, 0x93, 0x20], 0).unwrap().3, 4);
        let mut q = 0usize;
        assert_eq!(read_varint(&[0x93], &mut q), Some(-109));
    }

    /// The wild witness, pinned the way the capture pins it: by the statement-end
    /// `4B` that must sit exactly at the type's end. `src/system/meta/Sorting.cpp`,
    /// `.ex` 0xc7e3 — a 4,116-byte object, the escape form of the size varint.
    #[test]
    fn aggregate_varint_size_escape_is_pinned_by_the_next_marker() {
        // 4C (call end) 30 (indirect load) <TYPE> 4B (statement end)
        let seg: &[u8] = &[
            0x4C, 0x30, 0x86, 0x06, 0x80, 0x14, 0x10, 0x00, 0x00, 0xA5, 0x29, 0x4B,
        ];
        let (tag, kind, id, w) = read_type(seg, 2).expect("Sorting.cpp aggregate");
        assert_eq!((tag, kind, id), (0x86, 0x06, 0x14A5));
        assert_eq!(w, 9, "2 header + 5 varint escape + 2 LEB id");
        assert_eq!(seg.get(2 + w), Some(&0x4B), "the marker must land exactly here");
        // The old reading consumed `86 06 80 14` (the escape's lead byte taken
        // for a LEB continuation) and left the parse standing on `10` — a desync,
        // not a refusal at the right place.
        assert_ne!(w, 4);
    }

    /// Everything that violates the rule refuses. `size < 32` is the interesting
    /// one: a size the 5-bit field could have carried cannot legally appear in the
    /// escape, so seeing one means we are not looking at an aggregate size and do
    /// not know where the type ends — undetermined, therefore refused.
    #[test]
    fn malformed_aggregates_fail_closed() {
        let cases: &[(&[u8], &str)] = &[
            (&[0x82, 0x06, 0x1F, 0x87, 0x20], "size 31 fits the field → not the escape"),
            (&[0x82, 0x06, 0x00, 0x87, 0x20], "size 0 is not a struct size"),
            (&[0x82, 0x06, 0xFF, 0x87, 0x20], "negative short-form varint"),
            (&[0x82, 0x06, 0x80, 0x1F, 0x00, 0x00, 0x00, 0x87, 0x20], "escape carrying 31"),
            (
                &[0x82, 0x06, 0x80, 0xFF, 0xFF, 0xFF, 0xFF, 0x87, 0x20],
                "escape carrying -1",
            ),
            (&[0x82, 0x06, 0x20], "size read, id truncated"),
            (&[0x82, 0x06, 0x80, 0x14, 0x10], "escape truncated"),
            (&[0x82, 0x06], "nothing after the kind"),
            // LEB overflow after a valid size still refuses.
            (
                &[0x82, 0x06, 0x28, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80],
                "id LEB runs away",
            ),
        ];
        for (bytes, label) in cases {
            assert_eq!(read_type(bytes, 0), None, "{label}: {bytes:02X?}");
        }
    }

    /// Non-aggregate classes must be untouched by the aggregate branch — including
    /// the neighbouring kinds that differ from `6` in one bit (`5` real, `7`
    /// void) and the `86 06`-shaped bytes' scalar cousins. If the branch keyed on
    /// the wrong nibble this is where it would show.
    #[test]
    fn the_aggregate_branch_does_not_touch_other_classes() {
        let cases: &[(&[u8], usize, &str)] = &[
            (&[0x86, 0x45, 0x40], 3, "float (class 5)"),
            (&[0x88, 0x85, 0x41], 3, "double (class 5)"),
            (&[0x82, 0x07, 0x03], 3, "void (class 7)"),
            (&[0x86, 0x41, 0x74], 3, "int (class 1)"),
            (&[0x86, 0x43, 0x83, 0x08], 4, "void* (class 3)"),
            // Class 6 in the *high* nibble is not an aggregate: `86 65 …` is
            // width-6-of-class-5 nonsense in this grammar, but it must at least
            // not be re-parsed as one.
            (&[0x86, 0x65, 0x20, 0x87, 0x20], 3, "class 5, high nibble 6"),
        ];
        for (bytes, w, label) in cases {
            assert_eq!(read_type(bytes, 0).map(|t| t.3), Some(*w), "{label}");
        }
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

    /// The whole 3x3 of [`ValueClass`] pairs, as the table on
    /// [`eat_reinterpret_type`] measured it. Written as the *complete* matrix and
    /// not as the accepting cells, because the failure this guards against is a
    /// future widening that reads "a value class is a value class" — and four of
    /// these nine cells are `rlwinm r3,r3,0,24,31` on the real compiler.
    #[test]
    fn the_width4_reinterpret_admits_four_of_nine_class_pairs_and_no_others() {
        use ValueClass::*;
        // One TYPE per class, all from live captures.
        let ty = |c: ValueClass| -> &'static [u8] {
            match c {
                Int4 => &[0x86, 0x41, 0x74],
                Ptr4 => &[0x86, 0x43, 0x83, 0x08],
                Int1u => &[0x82, 0x12, 0x20],
            }
        };
        for src in [Int4, Ptr4, Int1u] {
            for dst in [Int4, Ptr4, Int1u] {
                let bytes = ty(dst);
                let mut p = 0;
                let got = eat_reinterpret_type(bytes, &mut p, src);
                // Accepted exactly when both ends are width-4 AND they differ:
                // the same-class cells belong to `eat_value_type`, which is
                // consulted first and which this must not duplicate.
                let want = matches!((src, dst), (Int4, Ptr4) | (Ptr4, Int4));
                assert_eq!(
                    got.is_some(),
                    want,
                    "{src:?} -> {dst:?}: c2 emits nothing only for the width-4 \
                     cross pair; every `Int1u` cell but the identity is a mask"
                );
                if want {
                    assert_eq!(got, Some(dst), "the class returned is the TARGET's");
                    assert_eq!(p, bytes.len(), "the whole TYPE is consumed");
                } else {
                    assert_eq!(p, 0, "a refused reinterpret must not move `p`");
                }
            }
        }
        // `Int1u` is barred by its own arm rather than by falling off the end, so
        // it stays barred even against a target its *width* would not exclude.
        let mut p = 0;
        assert_eq!(eat_reinterpret_type(&[0x82, 0x11, 0x70], &mut p, Int1u), None);
        // And a target that is neither class — `char`, `short`, `long long`,
        // `float`, `double` — is refused from a width-4 source too.
        for t in [
            &[0x82u8, 0x11, 0x70][..],
            &[0x84, 0x21, 0x11][..],
            &[0x88, 0x81, 0x13][..],
            &[0x86, 0x45, 0x40][..],
            &[0x88, 0x85, 0x41][..],
        ] {
            for src in [Int4, Ptr4] {
                let mut p = 0;
                assert_eq!(eat_reinterpret_type(t, &mut p, src), None, "{t:02X?}");
                assert_eq!(p, 0);
            }
        }
    }
}
