//! MSVC name mangling for string-literal COMDATs: the `A`..`P` base-16
//! alphabet, the length mangling, the escaped text, and the assembled
//! `??_C@…` name.

use super::*;

/// One base-16 digit in MSVC's `A`..`P` alphabet (`A` = 0 … `P` = 15).
pub(crate) fn base16_ap_digit(nibble: u32) -> char {
    (b'A' + nibble as u8) as char
}

/// A `u32` in base 16, digits `A`..`P`, **most-significant first with leading
/// zeros suppressed**.
///
/// The suppression is the rule the 101-byte held-out literal bought
/// (`docs/OBJ_DYNINIT_SHAPE.md` §5): its JamCRC is `0x0B7B9BC4`, the obj carries
/// the **7**-digit `LHLJLME`, and a fixed-width-8 renderer would have written
/// `ALHLJLME` — right on ~15 of 16 literals and silently wrong on the rest.
///
/// `0` renders as the empty string, which no caller may emit; both callers
/// reject it explicitly rather than inventing a spelling for it.
pub(crate) fn base16_ap(v: u32) -> String {
    let mut out = String::new();
    let mut started = false;
    for shift in (0..8).rev() {
        let d = (v >> (shift * 4)) & 0xF;
        if d == 0 && !started {
            continue;
        }
        started = true;
        out.push(base16_ap_digit(d));
    }
    out
}

/// The `<L>` field of a `??_C@_0…` name: `n`, the literal's byte length
/// **including the NUL**, as an MSVC-mangled number.
///
/// `1..=10` → the single character `'0' + (n - 1)`; anything larger →
/// [`base16_ap`] followed by `@`. Verified: 4→`3`, 10→`9`, 11→`L@`, 14→`O@`,
/// 16→`BA@`, 26→`BK@`, 31→`BP@`, 32→`CA@`, 33→`CB@`, 49→`DB@`, 101→`GF@`.
///
/// **CORRECTION to §5.** The doc's decomposition line writes the template as
/// `??_C@` `_0` `<L>` `@` `<H>` `@` `<text>` `@`, i.e. with an `@` between the
/// length and the hash. There is none: the obj carries
/// `??_C@_03FIKCJHKP@abc?$AA@`, where `3` is the whole length field and the
/// next character is the hash's first digit. The `@` visible in the long form
/// `_0BK@` is the **trailing `@` of this mangling**, present only for `n > 10`.
/// Coding the doc's line literally produces `??_C@_03@FIKCJHKP@abc?$AA@`.
/// Cross-checked three ways, on the string-table *sizes* of three reference
/// objs (which no part of this rule was fitted to): the fixture's table is 100
/// bytes, TomCrypt's 161 and Zlib's 175, and each is reproduced to the byte
/// only by the template as written here.
pub(crate) fn mangle_len(n: u32) -> String {
    if (1..=10).contains(&n) {
        ((b'0' + (n - 1) as u8) as char).to_string()
    } else {
        format!("{}@", base16_ap(n))
    }
}

/// Append one literal byte in its `??_C@…` escaped form, or return `false` if
/// its escape has **not been measured**.
///
/// Three classes, all measured (`docs/OBJ_DYNINIT_SHAPE.md` §5 plus this lane's
/// probes):
///
/// * `[A-Za-z0-9_$]` pass through literally — uppercase and `$` included.
/// * six single-`?` escapes: `?0`=`,` `?1`=`/` `?3`=`:` `?4`=`.` `?5`=space
///   `?9`=`-`.
/// * `?$` + two `A`..`P` nibble digits, MSB first, fixed width 2: NUL→`?$AA`,
///   `!`(0x21)→`?$CB`, `+`(0x2B)→`?$CL`.
///
/// **Everything else is refused, and the refusal is the point.** `?2`, `?6`,
/// `?7` and `?8` are single-`?` escape slots that this lane never observed a
/// character in. Some byte claims each of them, and it is *not* discoverable
/// from the three `?$XX` cells above which one — a byte that takes a single-`?`
/// escape in real c2 would be rendered here as a two-digit `?$XX` and the whole
/// COMDAT name, its length field and the obj's string table would all be wrong,
/// with nothing to flag it. Guessing the four unmeasured slots to widen coverage
/// is strictly worse than declining: a synthesized name that links is the
/// failure mode this project's one correctness rule exists to prevent. Only `/`
/// and `?$AA` are needed for the #158 target class.
pub(crate) fn escape_literal_byte(byte: u8, out: &mut String) -> bool {
    match byte {
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$' => {
            out.push(byte as char);
            true
        }
        b',' => {
            out.push_str("?0");
            true
        }
        b'/' => {
            out.push_str("?1");
            true
        }
        b':' => {
            out.push_str("?3");
            true
        }
        b'.' => {
            out.push_str("?4");
            true
        }
        b' ' => {
            out.push_str("?5");
            true
        }
        b'-' => {
            out.push_str("?9");
            true
        }
        // The three `?$XX` cells that were actually captured.
        0x00 | b'!' | b'+' => {
            out.push_str("?$");
            out.push(base16_ap_digit((byte >> 4) as u32));
            out.push(base16_ap_digit((byte & 0xF) as u32));
            true
        }
        _ => false,
    }
}

/// How many of a literal's bytes the escaped-text field of a `??_C@…` name
/// renders before it is cut off.
///
/// **CORRECTION to §5.** The doc says the text is "truncated at 32 characters",
/// which reads as a limit on the *escaped output*. It is not: the limit is on
/// the **source bytes of `literal + NUL`**. Measured on this lane's probes —
/// a 31-character literal (32 bytes with its NUL) renders the `?$AA`, a
/// 32-character one (33 bytes) drops it, and a 30-character all-`/` literal
/// produces 54 escaped characters with nothing cut. Reading the limit as an
/// output-character budget truncates the second of those in the middle.
/// PROV[O] the doc above is the grade, and it is a PINNED boundary rather than a fitted one: probe cells sit at 31 and 32 source bytes, one on each side, so no off-sample value is consistent with the measurement. Contrast `MAX_OBJECTS_PER_SECTION`, which is `[F]`.
pub(crate) const LITERAL_TEXT_BYTE_LIMIT: usize = 32;

/// `??_C@_0<len><hash>@<escaped text>@` — the COMDAT symbol name c2 gives a
/// narrow (`char`) string literal under `/GF`.
///
/// `bytes` is the literal **including its trailing NUL**; that NUL is part of
/// the length, part of the hash and (unless cut by
/// [`LITERAL_TEXT_BYTE_LIMIT`]) part of the escaped text. Returns `None` when
/// any byte's escape is outside the measured set — see [`escape_literal_byte`].
///
/// Byte evidence, every literal this lane or the characterization measured:
///
/// | literal | n | JamCRC | `<H>` |
/// |---|---:|---|---|
/// | `abc` | 4 | `0x58A297AF` | `FIKCJHKP` |
/// | `defg` | 5 | `0x3F7194AC` | `DPHBJEKM` |
/// | *(empty)* | 1 | `0x2DFD1072` | `CNPNBAHC` |
/// | `Hello, world!` | 14 | `0x647FB1F9` | `GEHPLBPJ` |
/// | `xyzzy` | 6 | `0xFE973C8F` | `POJHDMIP` |
/// | `q`×100 | 101 | `0x0B7B9BC4` | `LHLJLME` |
/// | `system/src/synth/tomcrypt` | 26 | `0xF4BC3E1C` | `PELMDOBM` |
/// | `system/src/zlib` | 16 | `0x55C0A74D` | `FFMAKHEN` |
pub fn string_comdat_name(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() || *bytes.last().unwrap() != 0 {
        // The NUL is load-bearing in all three fields; a caller that dropped it
        // would get a name that is wrong everywhere and looks right nowhere.
        return None;
    }
    let hash = jamcrc(bytes);
    if hash == 0 {
        // `base16_ap(0)` is the empty string and no cell measured what c2 writes
        // for a literal whose JamCRC is zero. Refuse rather than pick between
        // "" and "A".
        return None;
    }
    let mut text = String::new();
    for &b in bytes.iter().take(LITERAL_TEXT_BYTE_LIMIT) {
        if !escape_literal_byte(b, &mut text) {
            return None;
        }
    }
    Some(format!(
        "??_C@_0{}{}@{}@",
        mangle_len(bytes.len() as u32),
        base16_ap(hash),
        text
    ))
}

/// **SURFACE[mangle.string_comdat]** — the registered decision surface's domain
/// (`crate::surface`, board **#3762**, lane `w-inlbudget`, closing one of the
/// four rows board **#3746** named as *"a real refusal boundary … not
/// enumerated yet"*).
///
/// Three families, and the third is the one the registry exists for:
///
/// * **length**, from 0 to 44 source bytes, which walks
///   [`LITERAL_TEXT_BYTE_LIMIT`] from both sides. The `?$AA` for the trailing
///   NUL is what the boundary decides, and §5's CORRECTION is that the limit
///   counts **source** bytes and not escaped output — so a second family of
///   literals whose every byte escapes to *two* characters is enumerated
///   beside it, because a limit read as an output budget would cut those in
///   the middle and this domain would show it;
/// * **escape coverage**, one row per byte outside the measured escape set,
///   each of which must stay a refusal. `escape_literal_byte`'s set is a
///   *measured* set, not a derived one, so widening it is exactly the kind of
///   quiet accept `#3723` is about;
/// * the two **structural** refusals — an empty slice and a slice with no
///   trailing NUL — which are not length boundaries at all and which a domain
///   over lengths alone would never reach.
///
/// The corpus reaches almost none of this: the workload's literals are paths
/// and format strings, and no fixture compiles a 44-character all-`/` string.
pub(crate) fn surface_rows() -> Vec<crate::surface::Row> {
    fn row(point: String, bytes: &[u8]) -> crate::surface::Row {
        let outcome = match string_comdat_name(bytes) {
            Some(n) => n,
            None => crate::surface::REFUSE.to_string(),
        };
        crate::surface::Row::new(point, outcome)
    }

    let mut rows = Vec::new();
    // Family 1 — one source byte per escaped character.
    for n in 0..=44usize {
        let mut b = vec![b'a'; n];
        b.push(0);
        rows.push(row(format!("len-1x n={n:02}"), &b));
    }
    // Family 2 — TWO escaped characters per source byte (`/` is `?1`). §5's
    // CORRECTION is precisely that this family is cut at the same *source*
    // count as family 1 and not at half of it.
    for n in 24..=40usize {
        let mut b = vec![b'/'; n];
        b.push(0);
        rows.push(row(format!("len-2x n={n:02}"), &b));
    }
    // Family 3 — a mixed stream, so the boundary is not being read off a
    // single repeated byte.
    for n in 24..=40usize {
        let mut b: Vec<u8> = (0..n).map(|i| [b'a', b'/', b'.'][i % 3]).collect();
        b.push(0);
        rows.push(row(format!("len-mix n={n:02}"), &b));
    }
    // Family 4 — every byte outside the measured escape set refuses.
    for &c in &[b'#', b'%', b'&', b'*', b'@', b'^', b'~', b'"', b'?', b'\\', 0x7Fu8, 0x80, 0xFF] {
        let b = [b'a', b'b', c, 0];
        rows.push(row(format!("escape byte=0x{c:02x}"), &b));
    }
    // Family 5 — the structural refusals.
    rows.push(row("structural empty".into(), &[]));
    rows.push(row("structural no-nul".into(), b"abc"));
    rows
}
