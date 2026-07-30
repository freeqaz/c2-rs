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
            // …and a record whose *linkage* the port cannot emit refuses it too.
            // `__declspec(dllexport)` makes c2 splice `/EXPORT:<name>` into
            // `.drectve`, which the port emits as a constant, so the section grows
            // and every later offset shifts: `Port=Mismatch @ offset 8`
            // (`PointerToSymbolTable`), the same failure shape as the
            // `#pragma comment(lib, …)` case [`drectve_is_boilerplate`] already
            // refuses. It was a live wrong-bytes emit on a one-line getter.
            if linkage_needs_a_directive(gl, runs[k].1) {
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

/// Whether the record whose name ends at `name_nul` declares a linkage the port's
/// **constant `.drectve`** cannot represent — today, `__declspec(dllexport)`.
///
/// MEASURED. A defined function's `.gl` record continues, immediately after its
/// name's NUL, with a **two-byte** `<tag> <kind>` return type, then a linkage byte,
/// then the return type's size. The two-byte width held across fourteen return
/// types, including the ones most likely to widen it — a 20-byte aggregate
/// (`86 06 05 14`), a reference (`86 03 05 04`), an enum, a class, and a function
/// pointer (`86 04 05 04`) — so the linkage byte is at a fixed `name_nul + 3` and
/// not behind a variable-width field, which is what an earlier reading of a single
/// `int`-returning probe had assumed.
///
/// ```text
/// ?de@@YAHPAUH@@@Z\0  86 01 09 04 …   __declspec(dllexport)   bit 0x08 SET
/// ?glob@@YAHPAUH@@@Z\0 86 01 05 04 …  external                05
/// ?stat_fn@@YAHPAUH@@@Z\0 86 01 03 04 … internal (`static`)   03
/// ```
///
/// **The gate is a known-bad bit test, not a known-good allowlist, and that is a
/// deliberate weakening.** Over the same byte in every `?`-mangled run of six real
/// translation units the values are `{0, 3, 4, 5, 6}` — but those runs include
/// externals, callees and vtable symbols, not only the framed-offset records this
/// function binds, and the *defined-record* value set could not be separated from
/// them without replicating this function's own framing. Requiring `{03, 05}` might
/// therefore refuse a real defined function carrying a fourth value and regress a
/// translation unit that matches today. Refusing on bit `0x08` cannot: it turns one
/// measured mis-emit into a refusal and leaves every unmeasured value exactly where
/// it was.
///
/// The cost of that choice, stated so it is not mistaken for completeness: a linkage
/// that needs some *other* directive, and does not set this bit, still mis-emits.
/// Closing that needs the value set for defined records measured properly.
///
/// One witness for the bit (`05` -> `09`), so reading `0x08` as "export" rather than
/// as the literal value `09` is an inference — it is taken because the direction of
/// error is over-refusal.
fn linkage_needs_a_directive(gl: &[u8], name_nul: usize) -> bool {
    /// `05` external becomes `09` under `__declspec(dllexport)`.
    const LINKAGE_EXPORT: u8 = 0x08;
    gl.get(name_nul + 3).is_some_and(|b| b & LINKAGE_EXPORT != 0)
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

/// The byte that separates a `.gl` record's operand token from the record's
/// name. **MEASURED, and it is not always `00`.**
///
/// A record is
/// `80 <LE32 type id> <2 bytes> <kind> <operand token> <SEP> <name> 00 <TYPE> …`,
/// and the two forms differ in exactly one byte. Transcribed from
/// `src/system/jpeg/Jpeg.cpp`'s `.gl`, two records of the *same* class, with the
/// same `04` kind byte and byte-identical framing on both sides:
///
/// ```text
/// 80 75 14 00 00  00 00  04  84 30  00  ??YString@@QAAAAV0@PBD@Z  00 86 03 04 04 …
/// 80 85 14 00 00  00 00  04  c2 30  26  ??_GString@@UAAPAXI@Z     00 86 03 04 04 …
///                            \_tok/ \sep/
/// ```
///
/// That identity is what licenses reading the same two bytes as the operand token
/// in both: the field's *position* is fixed by the record framing, not inferred
/// from the value that follows it.
///
/// Measured over eight real translation units (33,059 `?`-mangled `.gl` names,
/// 20,336 + 12,505 of them), the byte before the name takes exactly these two
/// values and no others. The remaining candidates in that scan are all a name
/// whose own first character is not `?` (`_TI4?AV…`, `_CT??_R0…`, `$?$S1@…`), so
/// they are names, not separators.
///
/// **What `26` MEANS is not claimed here.** Every witness carrying it is a symbol
/// with COMDAT-style linkage — `??_G`/`??_E` deleting destructors, `??_7`
/// vftables, the `??_R*` RTTI records, `_CT`/`_TI` EH descriptors, and
/// header-inline member functions such as `??1logic_error@stlpmtx_std@@UAA@XZ` —
/// while `??1String@@UAA@XZ`, defined out of line, carries `00`. That is a
/// correlation over one corpus and it is deliberately not turned into a name:
/// `docs/GAPS.md` §6 ("a guessed name is worse than a hex bucket"). Nothing in
/// this file branches on the value; both are simply records.
///
/// A third value, `25`, introduces a string-literal (`??_C@…`) record; it is
/// **not** admitted, because nothing calls a string literal and admitting a record
/// class is a licence to bind tokens from it.
const NAME_SEPARATORS: [u8; 2] = [0x00, 0x26];

/// The record-kind byte, immediately before the operand token. **MEASURED as
/// exactly this set** over 32,898 `?`-mangled records in eight real translation
/// units: `0E` (18,770), `00` (10,385), `04` (2,540), `10` (1,203), and nothing
/// else.
///
/// It is required because `.gl` also carries a **type table** whose records have a
/// different layout — `RndLight`, `MetaMaterial`, `stlpmtx_std::_List_node_base`,
/// and the source paths — in which the two bytes at the operand token's offset are
/// part of a type id instead. Those reads are the only thing that ever produced a
/// token two names disagree about (105 in `system/world/Dir.cpp`, every one a path
/// or a type name), and the kind byte is what tells the two record classes apart
/// structurally rather than by guessing at their contents.
///
/// It does **not** remove all of them, and the residue is stated rather than
/// implied: a type record whose id bytes happen to be printable reads as a junk run
/// under a junk token (`k0String`), because the misparse lands on kind `00`, which
/// is a real symbol kind. Measured on the 878-TU workload, that leaves **1,750**
/// junk tokens, of which **44** collide with a real symbol's token — and those 44
/// are decided by rank in [`gl_symbol_index`] rather than dropped, because a bare
/// run is never a callee.
///
/// Fails **closed**: a fifth kind is not indexed, so its callees refuse.
const SYMBOL_RECORD_KINDS: [u8; 4] = [0x00, 0x04, 0x0E, 0x10];

/// The character set an MSVC symbol name is spelled in: identifiers, plus the
/// four mangling punctuation characters `$ ? @`.
///
/// This is a *name* test, not a plausibility heuristic. `.gl` runs that fail it are
/// paths (`z:\…\joypad.h`) and qualified or templated type names
/// (`BoxLightArray<BoxMapLighting::LightParams_Directional,50>`) — records whose
/// token field this reader does not model — and the junk a `<token> <sep> <name>`
/// run leaves in front of a name when the token's own bytes are printable
/// (`b[&??_R0?AVFixedString@@@8`).
fn is_symbol_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$' | b'?' | b'@')
}

/// Build the `.gl` **symbol index**: operand token → mangled name.
///
/// A record is located by its `<operand token> <SEP> <name> 00` core, with the
/// token read backwards from the separator using the same variable-width rule the
/// operand stream uses. See [`NAME_SEPARATORS`] for the record framing and for
/// why the separator is an enumerated *pair* of bytes rather than `00`.
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
/// Four rules keep this a *binding* rather than a plausible guess, because the
/// differential cannot grade a correspondence (`docs/GAPS.md` §6, the `.sy`
/// bullet):
///
/// * **The name is the RIGHTMOST separator-preceded start of its run, not the
///   leftmost.** A record's token bytes are frequently printable, so the
///   `<token> 26 <name>` form runs together into one graphic run — `c2 30 26
///   ??_GString@@…` reads as `0&??_GString@@…`. Taking the leftmost start binds a
///   token to a name with junk glued on its front, which is exactly what the
///   NUL-anchored scan this replaces did whenever a record's kind byte was `00`:
///   `b[&??_R0?AVFixedString@@@8` and four more like it were live index entries in
///   `src/Memory_Xbox.cpp` alone.
/// * **A record must carry a symbol [`SYMBOL_RECORD_KINDS`] kind byte**, which is
///   what keeps `.gl`'s type table and its source-path records — whose bytes at the
///   token's offset are a type id — out of a symbol index.
/// * **A name must be spelled in the symbol alphabet** ([`is_symbol_char`]).
/// * **A token claimed by two different names is dropped, not resolved to the
///   first.** A wrong callee is a relocation against the wrong symbol — a
///   mis-emit, not a gap — so an ambiguous token gets the third value that
///   refuses (`docs/GAPS.md` §6, "a failed search is not evidence of absence").
///   [`gl_symbol_conflicts`] counts them and every scan reports the count.
///
/// **What the widening was measured to cost, since a binding cannot be graded by
/// the oracle** (eight real TUs, 24,281 index entries before / 34,208 after):
/// **zero** `?`-mangled bindings change name, **zero** are lost except one that was
/// itself a junk read (`?6%??_C@_0BM@…`, a string-literal record whose own token
/// bytes are printable), and **zero** token conflicts involve a mangled name. What
/// is lost is entirely the type-table pollution the old scan carried: `size_t`,
/// `_PMD`, `RndBone`, `GfxMode`, `CODE`.
pub fn gl_symbol_index(gl: &[u8]) -> std::collections::BTreeMap<u32, String> {
    let (index, _) = gl_symbol_index_checked(gl);
    index
}

/// How many operand tokens `.gl` claims for **two different names**, and which are
/// therefore dropped from [`gl_symbol_index`] rather than bound to the first.
///
/// Returns `(dropped, of-which-mangled)`. The **second** number is the invariant
/// with a known answer: `.gl` assigns one operand token per symbol, so a `?`-mangled
/// name can never be in a disagreement. The first is expected to be small and
/// nonzero — `.gl`'s type table shares this reader's record shape closely enough
/// that two bare type names occasionally land on one token, and dropping them costs
/// nothing because nothing calls a type.
///
/// Reported by `c2rs gap` / `c2rs census` next to the numerator, because a binding
/// change cannot be graded by the oracle and this is one of the invariants that
/// *can* grade it.
pub fn gl_symbol_conflicts(gl: &[u8]) -> (usize, usize) {
    let (_, conflicts) = gl_symbol_index_checked(gl);
    conflicts
}

fn gl_symbol_index_checked(
    gl: &[u8],
) -> (std::collections::BTreeMap<u32, String>, (usize, usize)) {
    // `None` is the third value: a token two records disagree about, which must
    // refuse rather than pick one.
    // The value is `(rank, name)`, rank 1 for a whole mangled name; `None` is the
    // third value — a token two records of EQUAL rank disagree about, which must
    // refuse rather than pick one.
    let mut out: std::collections::BTreeMap<u32, Option<(usize, String)>> =
        std::collections::BTreeMap::new();
    let mut conflicts = 0usize;
    let mut mangled_conflicts = 0usize;
    let mut i = 0usize;
    while i < gl.len() {
        if !gl[i].is_ascii_graphic() {
            i += 1;
            continue;
        }
        let start = i;
        while i < gl.len() && gl[i].is_ascii_graphic() {
            i += 1;
        }
        // A record's name is NUL-terminated; a run that hits end-of-file is not
        // one, and neither is one that ends on some other non-printable byte —
        // which cannot happen, since the run ends where `is_ascii_graphic` does.
        if i >= gl.len() {
            break;
        }
        let end = i;
        // The rightmost start in this run that a separator precedes. See the doc
        // comment: leftmost is what glued a record's own token bytes onto the
        // front of its name.
        let mut name_at: Option<usize> = None;
        for p in start..end {
            if p == 0 {
                continue;
            }
            if NAME_SEPARATORS.contains(&gl[p - 1]) && is_indexable_name(&gl[p..end]) {
                name_at = Some(p);
            }
        }
        let Some(q) = name_at else { continue };
        // The operand token sits immediately before the separator. Try the 4-byte
        // form first, then the 2-byte one, and keep whichever decodes to a token
        // whose own width lands exactly on that separator.
        for w in [4usize, 2] {
            if q < w + 2 {
                continue;
            }
            let p = q - 1 - w;
            if let Some((tok, got)) = read_token_var(gl, p) {
                if got != w {
                    continue;
                }
                // …and the record must be a SYMBOL record. `.gl`'s type table puts
                // a type id where the token would be, and that is the whole source
                // of the ambiguity this index used to carry.
                if !SYMBOL_RECORD_KINDS.contains(&gl[p - 1]) {
                    break;
                }
                let name = ascii_string(&gl[q..end]);
                let rank = usize::from(looks_mangled(&name));
                match out.get(&tok) {
                    None => {
                        out.insert(tok, Some((rank, name)));
                    }
                    // A WHOLE mangled name outranks a bare one. `.gl`'s type
                    // table is the only thing that ever collides with a symbol
                    // record here, its names are bare, and a bare name is never a
                    // callee — so the tie-break is decided by what the two
                    // records ARE, not by which came first. Measured: it is the
                    // difference between 44 dropped mangled tokens on the
                    // workload and 0.
                    Some(Some((prev_rank, _))) if *prev_rank < rank => {
                        out.insert(tok, Some((rank, name)));
                    }
                    Some(Some((prev_rank, _))) if *prev_rank > rank => {}
                    Some(Some((_, prev))) if *prev != name => {
                        conflicts += 1;
                        if rank == 1 {
                            mangled_conflicts += 1;
                        }
                        out.insert(tok, None);
                    }
                    _ => {}
                }
                break;
            }
        }
    }
    (
        out.into_iter()
            .filter_map(|(t, n)| n.map(|(_, n)| (t, n)))
            .collect(),
        (conflicts, mangled_conflicts),
    )
}

/// Whether a `.gl` run is a symbol name this index may bind a token to.
///
/// Deliberately **not** [`looks_mangled`]: `??2@YAPAXI@Z` (`operator new`),
/// `_purecall`, `malloc` and `XMemAlloc` are real callees with no `@@` in them, and
/// requiring one dropped five resolving tail calls in `src/system/jpeg/Jpeg.cpp`
/// alone. The alphabet is what separates a symbol from a path or a template-id.
fn is_indexable_name(b: &[u8]) -> bool {
    b.len() >= 3
        && (b[0] == b'?' || b[0].is_ascii_alphabetic() || b[0] == b'_')
        && b.iter().all(|&c| is_symbol_char(c))
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

/// The fixed `.gl` header prefix that precedes the label counter:
/// `11 02 06 '1' 'j' '2' 01`. Required literally — the counter is read at a
/// fixed offset, so a `.gl` that does not start exactly like this is
/// **undetermined**, never guessed at.
const GL_HEADER_PREFIX: [u8; 7] = [0x11, 0x02, 0x06, b'1', b'j', b'2', 0x01];

/// The **compiler label counter** — the u32 at `.gl` offset 7, immediately
/// after [`GL_HEADER_PREFIX`].
///
/// This is the seed of c2's `$M…`/`$T…` compiler-generated label names, which
/// every framed function's obj carries (two `$M` labels marking the prologue end
/// and the function end, one `$T` label on its `.pdata` record). Without it the
/// port can emit a framed function only for a TU whose counter was pinned by
/// capture — which is exactly the state `docs/OBJ_GY_SHAPES.md` §3.5 recorded as
/// "not determined … the port cannot emit any framed function beyond TUs where B
/// is pinned by capture", and why `emit_framed_obj` hardcoded `2545/2546/2547`.
///
/// The first label c2 allocates for a TU is `counter + 9`
/// (`c2_core::coff::LabelPlan`). Established over 25 TUs whose `.gl` and obj were
/// captured together: `mvp_framed` 2536 → `$M2545`, `mvp_call_twice` 2534 →
/// `$M2543`, `il_call_return` 2578 → `$M2587`, and so on across TUs whose
/// counters span 2534..2683. An earlier attempt read this field as a LEB128 and
/// got a plausible-looking value (1256 for `mvp_framed`) that fitted a constant
/// gap on every single-byte-continuation TU in the fixture set and broke as soon
/// as the low byte fell below 0x80 — the field is a fixed-width u32.
///
/// `None` (undetermined, so the caller refuses) if the header does not match or
/// the file is short. A missing counter must never be defaulted: an emitted
/// `$M` with the wrong number is a wrong-bytes obj.
pub fn label_counter(gl: &[u8]) -> Option<u32> {
    if gl.len() < GL_HEADER_PREFIX.len() + 4 || gl[..7] != GL_HEADER_PREFIX {
        return None;
    }
    Some(u32::from_le_bytes([gl[7], gl[8], gl[9], gl[10]]))
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
    #[test]
    fn label_counter_is_a_fixed_width_u32_behind_the_header() {
        // `mvp_framed`'s real `.gl` prefix. 0x9E8 = 2536, and the TU's first
        // compiler label is $M2545 = 2536 + 9.
        let gl = [0x11, 0x02, 0x06, b'1', b'j', b'2', 0x01, 0xE8, 0x09, 0x00, 0x00, 0x00];
        assert_eq!(super::label_counter(&gl), Some(2536));
        // Read as a LEB128 the same bytes give 1256, which fits a constant gap
        // of 1289 across most of the corpus and breaks the moment the low byte
        // drops below 0x80 — hence the fixed-width read and this control:
        let low = [0x11, 0x02, 0x06, b'1', b'j', b'2', 0x01, 0x7F, 0x09, 0x00, 0x00];
        assert_eq!(super::label_counter(&low), Some(0x0000_097F));
        // Undetermined, never guessed: wrong header, or too short.
        let bad = [0x11, 0x02, 0x06, b'1', b'j', b'3', 0x01, 0xE8, 0x09, 0x00, 0x00];
        assert_eq!(super::label_counter(&bad), None);
        assert_eq!(super::label_counter(&gl[..9]), None);
        assert_eq!(super::label_counter(&[]), None);
    }

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

    /// The two `.gl` record forms, **transcribed verbatim** from
    /// `src/system/jpeg/Jpeg.cpp`'s capture, adjacent records of the same class.
    /// Their framing is identical byte for byte and the separator is the only
    /// difference — which is the whole argument that the two bytes before it are
    /// the operand token in both. See [`NAME_SEPARATORS`].
    #[test]
    fn gl_symbol_index_reads_both_separator_forms() {
        let mut gl = Vec::new();
        // 80 <LE32 type id> 00 00 <kind 04> <tok 8430> <sep 00> <name> 00 <TYPE…>
        gl.extend_from_slice(&[
            0x01, 0x80, 0x75, 0x14, 0x00, 0x00, 0x00, 0x00, 0x04, 0x84, 0x30, 0x00,
        ]);
        gl.extend_from_slice(b"??YString@@QAAAAV0@PBD@Z\x00\x86\x03\x04\x04\x00\x00\x00");
        // …and the same record shape with the `26` separator.
        gl.extend_from_slice(&[
            0x00, 0x80, 0x85, 0x14, 0x00, 0x00, 0x00, 0x00, 0x04, 0xC2, 0x30, 0x26,
        ]);
        gl.extend_from_slice(b"??_GString@@UAAPAXI@Z\x00\x86\x03\x04\x04\x00\x20\x01");
        let idx = gl_symbol_index(&gl);
        assert_eq!(
            idx.get(&0x8430).map(String::as_str),
            Some("??YString@@QAAAAV0@PBD@Z")
        );
        assert_eq!(
            idx.get(&0xC230).map(String::as_str),
            Some("??_GString@@UAAPAXI@Z"),
            "the `26`-separated record is the one 9,028 generated destructors \
             resolve their callee through"
        );
    }

    /// A record whose kind byte is `00` and whose token bytes are both printable
    /// runs together with its name. Transcribed from `src/Memory_Xbox.cpp`, where
    /// the NUL-anchored scan this replaces bound `b[&??_R0?AVFixedString@@@8` —
    /// the name with the record's own token glued on the front, under a token read
    /// from the *previous* record's tail.
    #[test]
    fn gl_symbol_index_does_not_glue_a_records_own_token_onto_its_name() {
        let mut gl = vec![
            0x1C, 0xA0, 0xA3, 0x00, 0x80, 0x8F, 0x28, 0x00, 0x80, 0x10, 0x1F, 0x00, 0x00, 0x01,
            0x00, 0x62, 0x5B, 0x26,
        ];
        gl.extend_from_slice(b"??_R0?AVFixedString@@@8\x00\x86\x06");
        let idx = gl_symbol_index(&gl);
        assert_eq!(
            idx.get(&0x625B).map(String::as_str),
            Some("??_R0?AVFixedString@@@8")
        );
        assert!(
            !idx.values().any(|n| n.starts_with("b[&")),
            "the token bytes must not become part of the name: {idx:?}"
        );
    }

    /// `.gl`'s **type table** uses the same neighbourhood with a different layout —
    /// no separator at all, and a type id where the operand token would be.
    /// Transcribed shape, from `src/system/jpeg/Jpeg.cpp`:
    /// `80 <LE32> 00 00 0B 00 <id> <name> 00`.
    ///
    /// [`SYMBOL_RECORD_KINDS`] removes most of these, but **not this one**: reading
    /// it as if it had a separator lands on kind `00`, which is a real symbol kind,
    /// so what gets bound is a junk run (`k0String`) under a junk token. That is the
    /// honest residue, and it is what this test states — the *type's* name is never
    /// bound as a symbol, and the junk costs nothing until it collides with a real
    /// token, which is what the rank tie-break below settles.
    #[test]
    fn a_type_table_record_never_binds_the_type_name() {
        let mut gl = vec![0x80, 0x9E, 0x14, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x6B, 0x30];
        gl.extend_from_slice(b"String\x00\x00");
        let idx = gl_symbol_index(&gl);
        assert!(
            !idx.values().any(|n| n == "String"),
            "a type is not a symbol: {idx:?}"
        );
    }

    #[test]
    fn a_token_two_symbols_claim_is_dropped_rather_than_guessed() {
        // Both records are whole mangled names, so neither outranks the other and
        // the token has no answer. A wrong callee is a relocation against the wrong
        // symbol, so the third value refuses (`docs/GAPS.md` §6).
        let mut gl = vec![0x00, 0x04, 0xE3, 0x09, 0x00];
        gl.extend_from_slice(b"?a@@YAXXZ\x00");
        gl.extend_from_slice(&[0x04, 0xE3, 0x09, 0x00]);
        gl.extend_from_slice(b"?b@@YAXXZ\x00");
        let idx = gl_symbol_index(&gl);
        assert_eq!(idx.get(&0xE309), None);
        assert_eq!(gl_symbol_conflicts(&gl), (1, 1));
    }

    #[test]
    fn a_whole_mangled_name_outranks_a_bare_one_on_the_same_token() {
        // The residue the type table leaves: a bare run landing on a real symbol's
        // token. `?MemFree@@YAXPAXPBDH1@Z` and `O6FileStream` both read as token
        // 0x000B in `src/system/utl/UTF8.cpp`; dropping the pair cost the mangled
        // one its binding, so rank decides instead of order.
        let mut gl = vec![0x00, 0x00, 0x00, 0x0B, 0x00];
        gl.extend_from_slice(b"O6FileStream\x00");
        gl.extend_from_slice(&[0x04, 0x00, 0x0B, 0x00]);
        gl.extend_from_slice(b"?MemFree@@YAXPAXPBDH1@Z\x00");
        assert_eq!(
            gl_symbol_index(&gl).get(&0x000B).map(String::as_str),
            Some("?MemFree@@YAXPAXPBDH1@Z")
        );
        assert_eq!(gl_symbol_conflicts(&gl), (0, 0));
    }

    #[test]
    fn an_undecorated_callee_is_still_indexed() {
        // `??2@YAPAXI@Z` (operator new), `_purecall` and `malloc` are real callees
        // with no `@@` in them. Requiring one dropped five resolving tail calls in
        // `src/system/jpeg/Jpeg.cpp` alone, which is why the name test is the symbol
        // ALPHABET and not [`looks_mangled`].
        let mut gl = vec![0x00, 0x04, 0xE3, 0x09, 0x00];
        gl.extend_from_slice(b"??2@YAPAXI@Z\x00");
        gl.extend_from_slice(&[0x04, 0xE4, 0x09, 0x00]);
        gl.extend_from_slice(b"_purecall\x00");
        let idx = gl_symbol_index(&gl);
        assert_eq!(idx.get(&0xE309).map(String::as_str), Some("??2@YAPAXI@Z"));
        assert_eq!(idx.get(&0xE409).map(String::as_str), Some("_purecall"));
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
