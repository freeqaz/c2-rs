use super::readers::{
    ascii_string, contains_subslice, find_subslice, memchr_byte, read_token_var,
};

/// Extract the mangled name from `.gl`: the first `?`-prefixed, NUL-terminated
/// ASCII run whose second byte is alphabetic and which contains `@@` (the
/// `__cdecl`/global marker). Mirrors `ILGlobals._parse`.
pub fn mangled_name(gl: &[u8]) -> Option<String> {
    let mut i = 0;
    // Same walk as the old byte loop: jump to each `?`, read its NUL-terminated
    // run, and resume after the run whether or not it was accepted.
    while i < gl.len() {
        let Some(k) = memchr_byte(b'?', &gl[i..]) else {
            break;
        };
        let start = i + k;
        let end = start
            + memchr_byte(0, &gl[start..]).unwrap_or(gl.len() - start);
        let bytes = &gl[start..end];
        if bytes.len() >= 3
            && bytes[1].is_ascii_alphabetic()
            && contains_subslice(bytes, b"@@")
            && bytes.iter().all(|b| b.is_ascii_graphic())
        {
            return Some(ascii_string(bytes));
        }
        i = end + 1;
    }
    None
}

/// Extract **all** mangled names from `.gl`, in file order — one per function
/// in the translation unit. Same acceptance test as [`mangled_name`]; used for
/// multi-function TUs where `.gl` carries a name per function.
pub fn mangled_names(gl: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    // Same walk as `mangled_name`, collecting every accepted run.
    while i < gl.len() {
        let Some(k) = memchr_byte(b'?', &gl[i..]) else {
            break;
        };
        let start = i + k;
        let end = start
            + memchr_byte(0, &gl[start..]).unwrap_or(gl.len() - start);
        let bytes = &gl[start..end];
        if bytes.len() >= 3
            && bytes[1].is_ascii_alphabetic()
            && contains_subslice(bytes, b"@@")
            && bytes.iter().all(|b| b.is_ascii_graphic())
        {
            out.push(ascii_string(bytes));
        }
        i = end + 1;
    }
    out
}

/// Every NUL-delimited identifier-shaped run in `.gl`, as `(start, end, name)` in
/// file order. **Not** filtered to mangled names — see [`looks_mangled`].
///
/// Deliberately **broader** than [`mangled_names`], which requires the second
/// byte to be alphabetic and therefore silently drops every `??`-prefixed name:
/// constructors (`??0S@@QAA@XZ`) and the `??__E` dynamic-initializer thunks that
/// a namespace-scope object with a constructor makes c2 emit. Those are real
/// symbols in the obj, and dropping them is what made a positional pairing look
/// safe — `.gl` for `struct S{S();}; S gs; int f(int);` lists
/// `??__Egs@@YAXXZ`, `?f@@YAHH@Z`, `?gs@@3US@@A`, `??0S@@QAA@XZ`, of which
/// `mangled_names` sees only the second and *fourth*, so pairing two names to
/// two bodies named the second function after a **variable**.
///
/// Broader again than the first version of this function, which also required
/// `@@` and a length of 3. That made it blind to an undecorated `extern "C"` name
/// (`c1`), so such a record was skipped and **borrowed the previous record's
/// name** — two bodies under one symbol, wrong bytes at obj offset 804
/// (`fixtures/cpp/il_extern_c_name.cpp`). A record's name is established by
/// position, so this scan must be able to see *whatever* is there; deciding
/// whether a name is one the port can emit is a separate, later judgement.
///
/// A run is accepted if it is NUL-delimited, wholly printable, and starts like an
/// identifier. That admits the source path and `__C1_11886` too, which is why the
/// unclaimed-symbol accounting filters with [`looks_mangled`] rather than relying
/// on this scan to be selective.
fn gl_symbol_runs(gl: &[u8]) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < gl.len() {
        if gl[i] != 0 {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < gl.len() && gl[end] != 0 {
            end += 1;
        }
        if end >= gl.len() || end == start {
            i += 1;
            continue;
        }
        let bytes = &gl[start..end];
        let plausible = bytes.iter().all(|b| b.is_ascii_graphic())
            && (bytes[0] == b'?' || bytes[0].is_ascii_alphabetic() || bytes[0] == b'_');
        if plausible {
            out.push((start, end, ascii_string(bytes)));
        }
        i = end;
    }
    out
}

/// True iff `name` looks like a whole MSVC-mangled symbol — it contains `@@`.
///
/// Two jobs, both of which need the *name's contents* rather than its position:
///
/// * deciding which unclaimed `.gl` runs the port must account for. The source
///   path (`z:\…\t1.cpp`) and `__C1_11886` are NUL-delimited printable runs that
///   `gl_symbol_runs` accepts and that no function record claims; without this
///   filter the accounting rule in `IlBundle::functions` would refuse every TU.
/// * rejecting a bound record name the port cannot emit. An undecorated
///   `extern "C"` name is stored **inline in the 8-byte COFF symbol name field**
///   rather than in the string table, which every mangled name uses — a different
///   encoding path, characterized by one capture. Refused, positively.
pub(crate) fn looks_mangled(name: &str) -> bool {
    contains_subslice(name.as_bytes(), b"@@")
}

/// Bind each **defined** function's `.gl` name to the `.ex` offset of its body,
/// positively. Returns the `(body offset, name)` pairs in `.gl` record order,
/// plus every mangled run that no record claimed.
///
/// Each `.gl` function record carries a `80 <LE32>` body-start offset field,
/// located by its record framing ([`codec::gl_offset_framed`]) rather than by
/// what its value happens to be, and the record's name is the run immediately
/// preceding that field. So the binding is per record.
///
/// Records observed so far are uniform in shape:
///
/// ```text
/// 00 <name> 00  <TYPE>  80 01 10 00 00 00 00  80 <LE32 offset>
///                       \___ framing ______/
/// ```
///
/// which puts the name's terminating NUL 15 bytes before the offset field for an
/// `int(int)` and 19 for a `void()`, the difference being the TYPE width. The name
/// is taken as the nearest preceding run within [`MAX_NAME_TO_OFFSET`] — the bound
/// is what makes "nearest preceding" mean *this record's* name rather than
/// whatever happened to be last in the file.
///
/// This replaces "the Nth name belongs to the Nth body", an invariant `.gl` does
/// not promise. It happens to hold across the fixtures, and the four probes that
/// looked most likely to break it (`extern` data, static members, namespaces,
/// templates) all list definitions first — but nothing enforces it, `.gl`
/// interleaves data symbols and compiler-generated thunks into the same list,
/// and a shifted name is a `.text` symbol emitted under some other symbol's
/// name. That is a wrong-bytes emit, not a refusal, so it is not something to
/// leave resting on an unchecked ordering.
///
/// The unclaimed runs matter just as much: an unclaimed name is a symbol the
/// real obj carries and the port does not model. `int gv; int f(int a){…}` leaves
/// `?gv@@3HA` unclaimed and c2's obj has an extra section for it — the port used
/// to emit its fixed four-section shell and mismatch at file offset 2, the
/// section count. The caller must account for every unclaimed run or refuse.
pub(crate) fn gl_defined_names(gl: &[u8]) -> (Vec<(u32, String)>, Vec<String>) {
    let runs = gl_symbol_runs(gl);
    let mut claimed = vec![false; runs.len()];
    let mut bound: Vec<(u32, String)> = Vec::new();
    let mut p = 0usize;
    while p + 5 <= gl.len() {
        if crate::codec::gl_offset_framed(gl, p) {
            let off = u32::from_le_bytes([gl[p + 1], gl[p + 2], gl[p + 3], gl[p + 4]]);
            // The record's own name: the last run to END at or before this field,
            // and near enough to be part of the same record. Searched backwards so
            // a record cannot borrow the name of a *following* one.
            let k = match runs.iter().rposition(|&(_, end, _)| end <= p) {
                // A framed offset whose nearest preceding run is too far away, or
                // has none at all, is a record shape we do not understand. Refuse
                // the whole TU rather than emit a function under a name that
                // belongs to some other record — which is precisely the bug this
                // bound exists to stop.
                Some(k) if p - runs[k].1 <= MAX_NAME_TO_OFFSET => k,
                _ => return (Vec::new(), Vec::new()),
            };
            // Named positively, then judged: a record name the port cannot emit
            // refuses the TU. `extern "C"` lands here.
            if !looks_mangled(&runs[k].2) {
                return (Vec::new(), Vec::new());
            }
            claimed[k] = true;
            bound.push((off, runs[k].2.clone()));
            p += 5;
            continue;
        }
        p += 1;
    }
    // Only mangled runs need accounting for. The rest — the source path,
    // `__C1_11886` — are not symbols the port is responsible for emitting.
    let unclaimed = runs
        .iter()
        .zip(&claimed)
        .filter(|((_, _, n), &c)| !c && looks_mangled(n))
        .map(|((_, _, n), _)| n.clone())
        .collect();
    (bound, unclaimed)
}

/// True iff `.gl`'s linker-directive list is exactly the single-entry boilerplate
/// that the port's fixed `.drectve` reproduces.
///
/// `.drectve` was pure boilerplate in every capture until `#pragma comment(lib,
/// "somelib")`, which splices `/DEFAULTLIB:"somelib"` in and grows the section
/// from 132 to 154 bytes. Every later section's file offset shifts, so the first
/// divergence is at obj offset 8 — `PointerToSymbolTable` — and a byte-exact
/// function body never gets a chance to matter
/// (`fixtures/cpp/il_drectve_pragma.cpp`).
///
/// `.gl` carries the list, so this is decidable rather than invisible:
///
/// ```text
/// … 00 00 01 0a "/include:__C1_11886" 00                        boilerplate
/// … 00 00 02 0a "/include:__C1_11886" 00 04 "somelib" 00        one pragma
/// ```
///
/// The byte two before the `/include:__C1_11886` literal is an **entry count**.
/// Anchoring on that literal is sound because it is already a compile-time
/// constant of this toolchain — `c2-core`'s `coff.rs` hardcodes both it and
/// `__C2_11886` (the XDK build id `16.00.11886.00`) into the `.drectve` it emits,
/// so the port is only ever correct for the build whose id this is.
///
/// Fails closed: an absent anchor, or a truncated list, is refused. Entries beyond
/// the first are not decoded — the `04` kind byte is the only one seen and one
/// witness is not a production.
pub(crate) fn drectve_is_boilerplate(gl: &[u8]) -> bool {
    const ANCHOR: &[u8] = b"/include:__C1_11886\0";
    let Some(at) = find_subslice(gl, ANCHOR) else {
        return false;
    };
    // `<count> 0a` immediately precedes the literal.
    at >= 2 && gl[at - 1] == 0x0A && gl[at - 2] == 0x01
}

/// How far a record's name may sit from its body-start offset field. Observed
/// distances are 15 (an `int(int)` record) and 19 (a `void()` record), the
/// difference being the TYPE field's width; 32 leaves room for wider types
/// without letting "nearest preceding run" reach into a different record.
const MAX_NAME_TO_OFFSET: usize = 32;

/// Build the `.gl` **symbol index**: operand token → mangled name.
///
/// `.gl` records have the shape
/// `<kind byte> <operand token> 00 <NUL-terminated name> 00 <TYPE> …`, so a
/// record is located by its `00 <name> 00` core and the token read backwards
/// from it with the same variable-width rule the operand stream uses.
///
/// This is what binds a call to its callee. The CALL token does *not* name the
/// callee — three different callees sharing one signature produce byte-identical
/// CALL tokens — so the name comes from the `26 <tok>` symbol push that precedes
/// it, resolved through this index. Verified on a real TU: 2323 of 2323 direct
/// call sites resolve, and the complementary controlled fixtures show tokens are
/// assigned in *declaration* order but used in *call* order, and that a repeated
/// callee repeats its token.
///
/// `.sy` is deliberately not consulted: it holds local and parameter names, and
/// real callees (`?MemPushTemp@@YAXXZ`) are absent from it and present here.
///
/// Names are accepted only if they look like whole mangled identifiers, so a
/// stray NUL-delimited run inside binary payload cannot inject a false symbol.
pub fn gl_symbol_index(gl: &[u8]) -> std::collections::BTreeMap<u32, String> {
    let mut out = std::collections::BTreeMap::new();
    let mut i = 0usize;
    while i < gl.len() {
        // A record's name is a NUL-terminated printable run preceded by a NUL.
        if gl[i] != 0 {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < gl.len() && gl[end] != 0 {
            end += 1;
        }
        if end >= gl.len() || end == start {
            i += 1;
            continue;
        }
        let name_bytes = &gl[start..end];
        let plausible = name_bytes.len() >= 3
            && name_bytes.iter().all(|b| b.is_ascii_graphic())
            && (name_bytes[0] == b'?' || name_bytes[0].is_ascii_alphabetic() || name_bytes[0] == b'_');
        if !plausible {
            i = end.max(i + 1);
            continue;
        }
        // The operand token sits immediately before the leading NUL. Try the
        // 4-byte form first, then the 2-byte one, and keep whichever decodes to
        // a token whose own width lands exactly on that NUL.
        for w in [4usize, 2] {
            if i < w {
                continue;
            }
            let p = i - w;
            if let Some((tok, got)) = read_token_var(gl, p) {
                if got == w {
                    out.entry(tok)
                        .or_insert_with(|| ascii_string(name_bytes));
                    break;
                }
            }
        }
        i = end;
    }
    out
}

/// Extract the source path from `.gl`: a `<letter>:\…\<name>.cpp` NUL-terminated
/// ASCII run (case-insensitive drive + `.cpp` suffix). Provenance only.
pub fn source_path(gl: &[u8]) -> Option<String> {
    let mut i = 0;
    // A candidate must have `:` at its second byte, so scan for `:` and test
    // the byte on each side — the same candidates the old per-byte walk saw, in
    // the same order, with the same resume points (past the NUL run when the
    // `<x>:\` prefix matched, past the candidate start when it did not).
    while i + 2 < gl.len() {
        let Some(k) = memchr_byte(b':', &gl[i + 1..]) else {
            break;
        };
        let start = i + k; // gl[start + 1] == b':'
        if start + 2 >= gl.len() {
            break;
        }
        if gl[start].is_ascii_alphabetic() && gl[start + 2] == b'\\' {
            let end = start
                + memchr_byte(0, &gl[start..]).unwrap_or(gl.len() - start);
            let bytes = &gl[start..end];
            if bytes.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
                // Case-insensitive `.cpp` suffix, checked on the bytes — same
                // acceptance as lowercasing the whole string, without the two
                // String allocations that cost on the hot parse path.
                if bytes.len() >= 4 && bytes[bytes.len() - 4..].eq_ignore_ascii_case(b".cpp") {
                    return Some(ascii_string(bytes));
                }
            }
            i = end + 1;
        } else {
            i = start + 1;
        }
    }
    None
}

/// Lazily-built `.gl` symbol index (see [`gl_symbol_index`]) — same contents,
/// built on first use. Only the call productions consult it (callee-by-token
/// resolution), so a TU of straight-line leaves never pays for building it;
/// every consumer goes through [`GlIndex::map`], which always yields the full
/// real index, so laziness can never change what is accepted.
pub(crate) struct GlIndex<'a> {
    gl: &'a [u8],
    cell: std::cell::OnceCell<std::collections::BTreeMap<u32, String>>,
}

impl<'a> GlIndex<'a> {
    pub(crate) fn new(gl: &'a [u8]) -> Self {
        GlIndex {
            gl,
            cell: std::cell::OnceCell::new(),
        }
    }
    /// The token → name map, built on first use.
    pub(crate) fn map(&self) -> &std::collections::BTreeMap<u32, String> {
        self.cell.get_or_init(|| gl_symbol_index(self.gl))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mangled_name_from_gl_slice() {
        let gl = b"\x00\x00?add3@@YAHHHH@Z\x00trailing";
        assert_eq!(mangled_name(gl).as_deref(), Some("?add3@@YAHHHH@Z"));
    }

    #[test]
    fn mangled_name_rejects_stray_question_mark() {
        // `?` not followed by an alpha / no `@@`.
        assert_eq!(mangled_name(b"? not a name\x00"), None);
    }

    #[test]
    fn source_path_from_gl_slice() {
        let gl = b"\x12\x20\x00z:\\tmp\\ilcap\\mvp.cpp\x00\x10";
        assert_eq!(
            source_path(gl).as_deref(),
            Some("z:\\tmp\\ilcap\\mvp.cpp")
        );
    }

    #[test]
    fn mangled_names_collects_all_in_order() {
        let gl = b"\x00?add2@@YAHHH@Z\x00pad\x00?add4@@YAHHHHH@Z\x00";
        assert_eq!(
            mangled_names(gl),
            vec!["?add2@@YAHHH@Z".to_string(), "?add4@@YAHHHHH@Z".to_string()]
        );
    }

    // ---- `.gl` name → body binding ------------------------------------------

    /// One `.gl` function record: a name run, then the framing
    /// `80 XX 10 00 00 00 00` that `codec::gl_offset_framed` recognizes, then the
    /// `80 <LE32>` body-start offset.
    fn gl_record(name: &str, body_off: u32) -> Vec<u8> {
        let mut v = vec![0u8];
        v.extend_from_slice(name.as_bytes());
        v.push(0);
        v.extend_from_slice(&[0x80, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x80]);
        v.extend_from_slice(&body_off.to_le_bytes());
        v
    }

    #[test]
    fn gl_names_bind_to_their_own_record_not_their_position() {
        // The `il_gl_record_order.cpp` layout: a `??`-prefixed thunk first, then a
        // function, then a data symbol, then an external constructor. Positional
        // pairing over `mangled_names` (which cannot see either `??` name) would
        // pair `?w_add` with the thunk's body and the *variable* with the second.
        let mut gl = Vec::new();
        gl.extend_from_slice(&gl_record("??__Egs@@YAXXZ", 2644));
        gl.extend_from_slice(&gl_record("?w_add@@YAHH@Z", 2753));
        gl.push(0);
        gl.extend_from_slice(b"?gs@@3US@@A");
        gl.push(0);
        gl.push(0);
        gl.extend_from_slice(b"??0S@@QAA@XZ");
        gl.push(0);

        let (bound, unclaimed) = gl_defined_names(&gl);
        assert_eq!(
            bound,
            vec![
                (2644, "??__Egs@@YAXXZ".to_string()),
                (2753, "?w_add@@YAHH@Z".to_string()),
            ],
            "each name must come from the record carrying its own body offset"
        );
        // The data symbol and the external are unclaimed; the caller must account
        // for each as a resolved callee or refuse the TU.
        assert_eq!(
            unclaimed,
            vec!["?gs@@3US@@A".to_string(), "??0S@@QAA@XZ".to_string()]
        );
        // And the narrow scan is exactly what missed the two `??` names.
        assert_eq!(
            mangled_names(&gl),
            vec!["?w_add@@YAHH@Z".to_string(), "?gs@@3US@@A".to_string()],
            "regression guard: mangled_names drops ?? names, so it cannot bind bodies"
        );
    }

    #[test]
    fn gl_symbol_runs_ignore_non_symbol_strings() {
        // A source path is a NUL-delimited printable run too, so `gl_symbol_runs`
        // accepts it — `looks_mangled` is what keeps it out of the accounting set.
        // Without that filter the rule in `functions()` would refuse every TU.
        let mut gl = vec![0u8];
        gl.extend_from_slice(b"e:\\lazer_build_gmc1\\x.cpp");
        gl.push(0);
        gl.extend_from_slice(&gl_record("?w_add@@YAHH@Z", 2644));
        let (bound, unclaimed) = gl_defined_names(&gl);
        assert_eq!(bound, vec![(2644, "?w_add@@YAHH@Z".to_string())]);
        assert!(unclaimed.is_empty(), "got {unclaimed:?}");
    }

    #[test]
    fn gl_framed_offset_without_a_name_binds_nothing() {
        // Fail closed on a record shape we cannot name: binding nothing makes
        // `functions()` refuse, rather than emitting a nameless function or
        // borrowing the name of a following record.
        let mut gl = vec![0x80, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x80];
        gl.extend_from_slice(&2644u32.to_le_bytes());
        gl.extend_from_slice(&gl_record("?w_add@@YAHH@Z", 2753));
        assert_eq!(gl_defined_names(&gl), (Vec::new(), Vec::new()));
    }

    #[test]
    fn an_undecorated_record_name_is_seen_then_refused() {
        // `il_extern_c_name.cpp`. The regression this guards: the name scan used to
        // require `@@`, so the `extern "C"` record was invisible and the binding
        // fell back to the nearest *mangled* run — the previous record's name. Two
        // bodies under one symbol, wrong bytes at obj offset 804.
        let mut gl = Vec::new();
        gl.extend_from_slice(&gl_record("?w_mangled@@YAHH@Z", 2644));
        gl.extend_from_slice(&gl_record("c1", 2743));

        // Seen: the scan reaches the undecorated run rather than skipping it.
        let runs = gl_symbol_runs(&gl);
        assert!(
            runs.iter().any(|(_, _, n)| n == "c1"),
            "the run scan must see an undecorated name; got {runs:?}"
        );
        // Refused: it is bound to its own record, judged, and rejected — so the
        // whole TU refuses rather than emitting under a borrowed name.
        assert_eq!(gl_defined_names(&gl), (Vec::new(), Vec::new()));

        // The mirror order was the *clean refusal* before the fix, which is what
        // made the bug order-dependent. It must still refuse, for the same reason.
        let mut rev = Vec::new();
        rev.extend_from_slice(&gl_record("c1", 2644));
        rev.extend_from_slice(&gl_record("?w_mangled@@YAHH@Z", 2743));
        assert_eq!(gl_defined_names(&rev), (Vec::new(), Vec::new()));
    }

    #[test]
    fn a_distant_name_does_not_get_borrowed() {
        // The bound is what makes "nearest preceding run" mean *this record's*
        // name. Pad past MAX_NAME_TO_OFFSET between the name and its offset field
        // and the record must stop claiming it.
        let mut gl = vec![0u8];
        gl.extend_from_slice(b"?w_add@@YAHH@Z");
        gl.push(0);
        gl.extend_from_slice(&[0x11; MAX_NAME_TO_OFFSET + 1]);
        gl.extend_from_slice(&[0x80, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x80]);
        gl.extend_from_slice(&2644u32.to_le_bytes());
        assert_eq!(gl_defined_names(&gl), (Vec::new(), Vec::new()));
    }

    // ---- `.drectve` boilerplate --------------------------------------------

    /// The `.gl` directive list: `00 00 <count> 0A` then the `/include:` literal.
    fn gl_directives(count: u8, extra: &[u8]) -> Vec<u8> {
        let mut v = vec![0x00, 0x00, count, 0x0A];
        v.extend_from_slice(b"/include:__C1_11886\0");
        v.extend_from_slice(extra);
        v
    }

    #[test]
    fn drectve_boilerplate_is_one_entry() {
        // Transcribed from captures of `int f(int a){return a+1;}` with and without
        // `#pragma comment(lib, "somelib")`. The pragma bumps the count and appends
        // an entry; the port's `.drectve` is a constant, so only the first is in
        // class (`il_drectve_pragma.cpp`).
        assert!(drectve_is_boilerplate(&gl_directives(1, b"")));
        assert!(!drectve_is_boilerplate(&gl_directives(
            2,
            b"\x04somelib\0"
        )));
        // Fail closed when the anchor is absent entirely — a `.gl` we cannot read
        // the directive list out of is not a `.gl` whose `.drectve` we can assume.
        assert!(!drectve_is_boilerplate(b"no directives here"));
        assert!(!drectve_is_boilerplate(&[]));
    }

    // ---- `.gl` symbol index -------------------------------------------------

    #[test]
    fn gl_symbol_index_binds_tokens_to_names() {
        // A `.gl` record is `<kind> <token> 00 <name> 00 <TYPE> …`. Transcribed
        // from a controlled fixture with three externals declared a, b, c —
        // tokens are assigned in DECLARATION order (0x09E3, 0x09E4, 0x09E5),
        // which is what makes a positional pairing with call order wrong.
        let mut gl = Vec::new();
        for (tok, name) in [
            ([0xE3u8, 0x09], "?a@@YAXXZ"),
            ([0xE4, 0x09], "?b@@YAXXZ"),
            ([0xE5, 0x09], "?c@@YAXXZ"),
        ] {
            gl.push(0x04); // kind
            gl.extend_from_slice(&tok);
            gl.push(0x00);
            gl.extend_from_slice(name.as_bytes());
            gl.push(0x00);
            gl.extend_from_slice(&[0x82, 0x07, 0x04]); // TYPE
        }
        let idx = gl_symbol_index(&gl);
        assert_eq!(idx.get(&0xE309).map(String::as_str), Some("?a@@YAXXZ"));
        assert_eq!(idx.get(&0xE409).map(String::as_str), Some("?b@@YAXXZ"));
        assert_eq!(idx.get(&0xE509).map(String::as_str), Some("?c@@YAXXZ"));
        // An unknown token must not resolve — the caller rejects rather than
        // guessing, since a wrong callee is a relocation against a wrong symbol.
        assert!(idx.get(&0xFFFF).is_none());
    }

    #[test]
    fn gl_symbol_index_ignores_non_identifier_runs() {
        // Binary payload between NULs must not become a symbol.
        let gl = b"\x00\x01\x02\x03\x00\x04\xE3\x09\x00?ok@@YAXXZ\x00".to_vec();
        let idx = gl_symbol_index(&gl);
        assert_eq!(idx.get(&0xE309).map(String::as_str), Some("?ok@@YAXXZ"));
        assert_eq!(idx.len(), 1, "only the identifier-shaped run is indexed");
    }
}
