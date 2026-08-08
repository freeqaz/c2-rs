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
pub(crate) fn gl_symbol_runs(gl: &[u8]) -> Vec<(usize, usize, String)> {
    symbol_runs(gl, false)
}

/// [`gl_symbol_runs`] **plus the `26` separator** — every name in `.gl`, not only
/// the NUL-introduced ones (W-VGL, board #151).
///
/// # The defect this exists for, measured
///
/// [`NAME_SEPARATORS`] records that a `.gl` record's name is introduced by `00`
/// **or** `26`, and lists what carries `26`: `??_G`/`??_E` deleting destructors,
/// `??_7` vftables, the `??_R*` RTTI records, `_CT`/`_TI` EH descriptors — and
/// **header-inline member functions**. [`gl_symbol_index`] already reads that
/// constant. [`gl_symbol_runs`] never did: it opens a run only after a `00`, so a
/// `26`-introduced name is not merely mis-framed, it is **never seen**.
///
/// The cost is not a missing name. It is a *wrong distance*: the record's
/// "nearest preceding run" becomes some unrelated symbol 85–194 bytes back, which
/// `bind::EMIT_MAX_NAME_TO_OFFSET` then correctly refuses — so the record lands
/// in `records_nameless` and its symbol is counted as *having no body at all*.
/// On `src/system/obj/TextFile.cpp`, 70 of 674 framed records:
///
/// ```text
/// ?_Copy_str@exception@std@@AAAXPBD@Z 00 <its own record> 0e ae 15
///   26 ??_Gexception@std@@UAAPAXI@Z 00 <the record the reader could not name>
/// ```
///
/// `ROADMAP.md` §9.18.3 read the same population as *virtual* and blamed the
/// framing and the 32-byte bound. Virtualness is a **correlate**: an out-of-line
/// virtual (`??1String@@UAA@XZ`) is `00`-separated and binds today, an *inline*
/// one is `26`-separated and vanishes. Measured under this scanner the maximum
/// name→offset distance is **27** across 676 records, so the 32-byte bound was
/// never the defect and must not be widened.
///
/// # Why a run also TERMINATES at `26`, and what that repairs
///
/// Terminating only at NUL is not enough: the run opened at the *previous* NUL
/// swallows the `26` and the scan resumes past it, so the name is still lost.
/// Terminating at `26` also fixes 14 names on `TextFile.cpp` that this scanner
/// was already emitting **corrupted** — two record bytes that happened to be
/// printable, glued to the front:
///
/// ```text
/// before   "H=&??_7FixedSizeAlloc@@6B@"      (`H=` is 0x48 0x3D, record bytes)
/// after    "??_7FixedSizeAlloc@@6B@"
/// ```
///
/// [`gl_symbol_index`] hit exactly this and solved it by taking the *rightmost*
/// separator inside a run; this solves it by not gluing them together at all.
///
/// # Scope — the gate does NOT use this
///
/// [`gl_defined_names`] and therefore `Bindings::per_record` keep
/// [`gl_symbol_runs`], deliberately, exactly as `bind::emit_offset_framed` is
/// kept separate from [`crate::codec::gl_offset_framed`]. This scanner widens
/// what the *instrument* can see; widening what the **gate** accepts moves the
/// emitted class and is a separately-gated decision (§9.20).
pub(crate) fn gl_symbol_runs_all_separators(gl: &[u8]) -> Vec<(usize, usize, String)> {
    symbol_runs(gl, true)
}

/// The shared scan. `sep26` adds `0x26` to both the opening and the terminating
/// separator set; see [`gl_symbol_runs_all_separators`] for why it must be both.
fn symbol_runs(gl: &[u8], sep26: bool) -> Vec<(usize, usize, String)> {
    let is_sep = |b: u8| b == 0 || (sep26 && b == NAME_SEPARATORS[1]);
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < gl.len() {
        if !is_sep(gl[i]) {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < gl.len() && !is_sep(gl[end]) {
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
///
/// **W-EXTDATA: the SECOND job no longer goes through this function.** The bound
/// record name is judged by [`INLINE_NAME_MAX`] instead, because that is what the
/// paragraph above is actually about and `@@` was only a proxy for it — see the
/// clause in [`gl_defined_names_framed`]. This predicate keeps the FIRST job
/// unchanged: which unclaimed runs the accounting rule must account for.
/// The largest name COFF stores **inline** in a symbol record's 8-byte name
/// field. A longer one goes to the string table, which is the encoding path
/// every mangled name — and `_vswprintf_s_l` — takes.
pub(crate) const INLINE_NAME_MAX: usize = 8;

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
///
/// # The separator (W-ADOPT, board #151)
///
/// This reads [`gl_symbol_runs_all_separators`], so a `26`-introduced name is
/// visible here. It was not, until W-ADOPT: `.gl` introduces a record's name
/// with `00` **or** `26` ([`NAME_SEPARATORS`]), the instrument-side scanner was
/// taught both in §9.20, and the gate kept the NUL-only one deliberately —
/// widening what a gate *accepts* is a differential decision, not a reader
/// repair, and it was gated separately.
///
/// What that cost, measured on `fixtures/cpp/il_gl_sep26.cpp`: the `26`-
/// introduced name was not mis-framed, it was **never seen**, so its record's
/// "nearest preceding run" was the previous record's name 63 bytes back. The
/// only thing standing between that and a body emitted under another symbol's
/// name was [`MAX_NAME_TO_OFFSET`] — a distance bound, refusing a record it
/// could not name. Right outcome, wrong reason, and on a TU where some
/// unrelated run happened to fall inside 32 bytes it would not have held.
pub(crate) fn gl_defined_names(gl: &[u8]) -> (Vec<(u32, String)>, Vec<String>) {
    gl_defined_names_with(gl, true)
}

/// **Which of `gl_defined_names`'s total-refusal clauses fired** — the name of
/// the byte that stopped the read, never a guess.
///
/// This exists so `vocab-gap` can be decomposed into causes with counts
/// (lane `w-vocab`). It is a *diagnostic* enum and nothing in the accept path
/// branches on it: [`gl_defined_names`] maps every variant to the same empty
/// pair it always returned, so the production reader is byte-for-byte the
/// incumbent. What changed is that the reader now says which clause it was
/// instead of discarding that fact at the `return`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GlBindStop {
    /// A framed record whose nearest preceding symbol run is absent or further
    /// than [`MAX_NAME_TO_OFFSET`] away.
    NameTooFar,
    /// A record name with no `@@` — an undecorated `extern "C"` symbol, whose
    /// COFF encoding path the port does not model.
    NameNotMangled,
    /// A run the widened scanner ended at `26`: the fields after it belong to
    /// the *next* record, so every fixed displacement downstream is wrong.
    RunEndsAt26,
    /// `__declspec(dllexport)` — c2 splices `/EXPORT:` into `.drectve`.
    DllexportLinkage,
    /// A `26`-**introduced** defined name: COMDAT-style linkage against a
    /// packed single-`.text` writer (board #232).
    Name26Introduced,
}

/// [`gl_defined_names`], but reporting the stop clause instead of swallowing it,
/// and with the record framing supplied by the caller.
///
/// `framed` is the record locator. Production passes
/// [`crate::codec::gl_offset_framed`]; lane `w-vocab`'s decomposition also
/// passes the window-free [`crate::func::bind::wide_offset_framed`] so that
/// "how many records can the gate's framing not SEE" is a measurement rather
/// than an inference. **Nothing in the accept path calls this with a widened
/// framing** — see `IlBundle::decode_causes`.
pub(crate) fn gl_defined_names_framed(
    gl: &[u8],
    sep26: bool,
    framed: fn(&[u8], usize) -> bool,
) -> Result<(Vec<(u32, String)>, Vec<String>), GlBindStop> {
    let runs = symbol_runs(gl, sep26);
    let mut claimed = vec![false; runs.len()];
    let mut bound: Vec<(u32, String)> = Vec::new();
    let mut p = 0usize;
    while p + 5 <= gl.len() {
        if framed(gl, p) {
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
                _ => return Err(GlBindStop::NameTooFar),
            };
            // Named positively, then judged: a record name the port cannot emit
            // refuses the TU.
            //
            // **W-EXTDATA — this clause was `!looks_mangled(name)` and it is the
            // 8-byte field it says it is about.** [`looks_mangled`]'s own doc
            // gives the ground: *"an undecorated `extern "C"` name is stored
            // **inline in the 8-byte COFF symbol name field** rather than in the
            // string table, which every mangled name uses — a different encoding
            // path, characterized by one capture."* That is a statement about the
            // name's LENGTH, and `@@` was a proxy for it: every MSVC-mangled name
            // is longer than eight bytes, so the two agreed on the whole shipped
            // population and nothing separated them.
            //
            // `_vswprintf_s_l` separates them. Fourteen bytes, `extern "C"`, and
            // therefore in the **string table** — the encoding path the clause
            // says is modeled — while `looks_mangled` refuses it for want of a
            // `@@` it can never have. `coff::symbol::emit_symbol` has branched on
            // `name.len() <= 8` since it was written; the reader was refusing a
            // shape the writer already builds.
            //
            // So the clause becomes the length test, and the refusal it keeps is
            // exactly the uncharacterized half: a bound record name that FITS
            // INLINE. That is a strictly smaller widening than "admit every
            // unmangled name", and it is the half no capture has graded.
            //
            // **The cost of the widening, stated rather than left to be found**:
            // for a TU with no mangled name anywhere, `Bindings::unclaimed` is
            // built by filtering runs through `looks_mangled` and therefore comes
            // back EMPTY — so `IlBundle::functions`' unclaimed-`.gl`-name gate has
            // nothing to check on such a TU and is vacuous rather than satisfied.
            // It is not widened here because the runs it would have to admit
            // include the source path, `__C1_11886`, `size_t` and `r` (measured on
            // `vswprnc.cpp`'s own `.gl`), and a filter separating those from a
            // real symbol is a rung of its own. Board **#1721**; the compensating
            // control is the 878-TU differential, which is what actually grades
            // every obj this widening lets through.
            if runs[k].2.len() <= INLINE_NAME_MAX {
                return Err(GlBindStop::NameNotMangled);
            }
            // **A run the widened scanner ended at `26` is not a record name this
            // reader understands.** Everything downstream of here reads the
            // record's fields at a fixed displacement from the name's terminator
            // — [`linkage_needs_a_directive`] takes the linkage byte at
            // `name_nul + 3` — and that arithmetic is measured against a NUL
            // terminator, which is what a *defined record's* name has: the TYPE
            // field follows it. A run that ends at `26` ends because the next
            // name began, so the three bytes after it are the next record's, and
            // reading them would be reading a field that is not there.
            //
            // Refuse rather than read. This is the only place the widened
            // separator set can change what a *bound* name is (it can shorten a
            // run that previously swallowed a `26`), so it is the one place the
            // widening could turn a refusal into wrong bytes, and it is closed
            // positively instead of being left to the distance bound — which is
            // the mistake this whole rung exists to correct.
            if gl.get(runs[k].1) != Some(&0) {
                return Err(GlBindStop::RunEndsAt26);
            }
            // …and a record whose *linkage* the port cannot emit refuses it too.
            // `__declspec(dllexport)` makes c2 splice `/EXPORT:<name>` into
            // `.drectve`, which the port emits as a constant, so the section grows
            // and every later offset shifts: `Port=Mismatch @ offset 8`
            // (`PointerToSymbolTable`), the same failure shape as the
            // `#pragma comment(lib, …)` case [`drectve_is_boilerplate`] already
            // refuses. It was a live wrong-bytes emit on a one-line getter.
            if linkage_needs_a_directive(gl, runs[k].1) {
                return Err(GlBindStop::DllexportLinkage);
            }
            // **…and a DEFINED record whose name is `26`-INTRODUCED refuses it
            // too, because `26` marks COMDAT-style linkage and the port's packed
            // writer has one `.text` for the whole TU** (board **#232**).
            //
            // This is the clause W-ADOPT was one step short of. That rung's own
            // message named the risk exactly — *"the one place the widening could
            // have produced wrong bytes instead of a refusal"* — and closed the
            // case where a run **ends** at `26` (above). The case where a run
            // **begins** at one was left open, and it is a different shape: an
            // `26`-introduced name is NUL-terminated like any other, so every
            // field arithmetic downstream is fine and the record binds happily.
            // Nothing was wrong with the *name*. What is wrong is the **obj
            // shape** it implies.
            //
            // Measured, `work/w-cross/alarm/`, at `/Ox /GS- /c`:
            //
            // ```cpp
            //   struct Bd { Bd(); ~Bd(); int b0; };
            //   struct M : Bd { M();  };
            //   struct D : M { D();  };
            //   D::D() {}
            // ```
            //
            // `.gl` introduces `??0D@@QAA@XZ` with `00` and `??1M@@QAA@XZ` — the
            // **implicitly generated** destructor of `M` — with `26`. And the
            // reference obj has **seven** sections with **two** `.text`s:
            // `??1M` in its own COMDAT (`chars 0x60401020`, `SELECT_ANY`) placed
            // **first**, then `??0D` in the ordinary packed `.text`
            // (`0x60400020`). The port emitted six sections, both symbols packed
            // into one `.text`, in the opposite order — `Port=Mismatch @ offset
            // 2`, `NumberOfSections`. **A refusal had become a wrong emit**,
            // which is the one direction `CLAUDE.md`'s correctness rule exists to
            // prevent.
            //
            // The correspondence is the whole content of the clause and it was
            // checked in both directions rather than assumed. Across the
            // generated sweep, **twelve** cases have more than one `.text` in
            // packed mode; the COMDAT is **not always first** (`51-dtor-member`
            // 0250 and 0255 put it second), so the section-ORDER rule is
            // unmeasured on top of the section-KIND one. Eleven of the twelve
            // already refused for unrelated reasons and exactly one — the case
            // above — reached the emitter.
            //
            // **Refused rather than emitted, deliberately.** Teaching the packed
            // writer to mint a per-function COMDAT is an emit-set / section-layout
            // model, which `docs/STATUS.md` puts in Phase 7 and says plainly is
            // not reachable by widening; and its ordering half has one witness
            // either way. `docs/GAPS.md` §6's rule applies to the separator too:
            // [`NAME_SEPARATORS`] declines to say what `26` *means*, and this
            // clause does not claim to know either — it keys on the byte, and the
            // byte is what the obj disagreed about.
            if runs[k].0 > 0 && gl[runs[k].0 - 1] == NAME_SEPARATORS[1] {
                return Err(GlBindStop::Name26Introduced);
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
    Ok((bound, unclaimed))
}

/// [`gl_defined_names`] with the separator set named, so the **incumbent stays
/// executable**.
///
/// `sep26 = true` is the only production path; `false` is the NUL-only reader
/// W-ADOPT replaced, kept callable so a test can state what the change did as a
/// pair of assertions on one input rather than as a claim about history. A
/// residue or a ceiling can move while the thing it proxies does not (§9.20.3),
/// and "this used to refuse" is exactly the sort of claim that rots into
/// folklore once the code it describes is gone.
fn gl_defined_names_with(gl: &[u8], sep26: bool) -> (Vec<(u32, String)>, Vec<String>) {
    // Every stop clause maps to the same empty pair this reader always
    // returned. The clause NAMES are new; the accepted class is not.
    gl_defined_names_framed(gl, sep26, crate::codec::gl_offset_framed).unwrap_or_default()
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

/// **WR1 — every `.gl` name that is an UNDEFINED-EXTERNAL DATA symbol**, i.e.
/// the only class of data symbol whose address this port may emit.
///
/// The distinction is invisible in the mangling — `extern int g;` and `int g;`
/// are both `?g@@3HA` — and it is the difference between an obj with five
/// sections and one with six: a **defined** global puts a `.data` section into
/// the middle of the section table (before the second `.XBLD$W`), and a
/// **static** one puts it after `.text`. Emitting the port's fixed shell against
/// either mismatches at file offset 2, the section count
/// (`docs/IL_CALL_IN_EXPR.md` §17.2 item 7).
///
/// MEASURED. A data record continues, immediately after its name's NUL, with the
/// object's TYPE, then the fixed pair `00 02`, then a linkage byte, then the
/// object's size:
///
/// ```text
///   ?gExt@@3HA\0             86 01    · 00 02 · 02 · 04 00   extern int      02
///   ?gExt2@@3PAHA\0          86 06    · 00 02 · 02 · 10 00   extern int[4]   02
///   ?gD@@3NA\0               88 05    · 00 02 · 02 · 08 00   extern double   02
///   ?gC@@3DA\0               82 01    · 00 02 · 02 · 01 00   extern char     02
///   ?TheDebug@@3VDebug@@A\0  88 06    · 00 02 · 02 · 18 00   extern class    02
///   ?gBig@@3VBig@@A\0        c6 81 06 · 00 02 · 02 · 2c 00   extern class    02
///   ?gDef@@3HA\0             86 01    · 00 02 · 01 · 04 80   int gDef = 3;   01
///   ?sm@C@@2HA\0             86 01    · 00 02 · 01 · 04 80   int C::sm = 9;  01
/// ```
///
/// **The TYPE is NOT a fixed two bytes** and `?gBig@@3VBig@@A` is the witness:
/// a class whose tag carries the WIDE bit (`0x40`) spells it `c6 81 06`, three
/// bytes, where `?TheDebug@@3VDebug@@A` — also a class, also polymorphic —
/// spells it `88 06`. `linkage_needs_a_directive`'s fixed `name_nul + 3` is
/// right for the *function* records it reads (fourteen return types, all two
/// bytes) and would be wrong here, so this one steps `<tag> [wide byte] <kind>`
/// with the same rule [`super::readers::read_type`] uses for that prefix.
/// Reading the linkage at a fixed offset refused `void b5(){ gvo(&gBig); }` — an
/// over-refusal, which is the direction that costs a rung rather than an obj, and
/// it is why the witness exists at all.
///
/// The *rest* of `read_type` is deliberately not reused: a data record ends its
/// TYPE at the kind byte, where an `.ex` TYPE continues with an aggregate size
/// and a per-TU id, and running the whole reader here consumes the `00` of the
/// frame below and then refuses every aggregate outright.
///
/// **The `00 02` frame is checked, not skipped**, and it is what makes this a
/// structural read rather than an offset guess: a *function* record has
/// `82 07 <05|04|03>` there and fails it, so a callee can never be mistaken for
/// an extern object. A record class this reader has not seen fails it too and is
/// therefore refused.
///
/// The set is returned rather than a per-name predicate because a name may occur
/// in `.gl` more than once, and a name that is a defined data symbol *anywhere*
/// in the file must not be admitted on the strength of some other record.
/// The linkage byte of the `.gl` **data** record whose name's NUL is at
/// `name_nul`, or `None` when the record is not one — the frame check.
///
/// `<tag> [wide byte] <kind> 00 02 <linkage>`. The wide prefix is the same rule
/// [`super::readers::read_type`] applies (tag bit `0x40`, and the byte after it
/// must carry `0x80`); the `00 02` pair is required literally, which is what
/// makes a *function* record — `82 07 <05|04|03>` — fail rather than yield its
/// third byte as a linkage.
fn data_linkage(gl: &[u8], name_nul: usize) -> Option<u8> {
    /// Tag bit that inserts one extra byte before the kind.
    const TAG_WIDE: u8 = 0x40;
    /// …and that byte must carry this.
    const WIDE_MARK: u8 = 0x80;
    let tag = *gl.get(name_nul + 1)?;
    if tag & 0x80 == 0 {
        return None;
    }
    let mut i = name_nul + 2;
    if tag & TAG_WIDE != 0 {
        if *gl.get(i)? & WIDE_MARK == 0 {
            return None;
        }
        i += 1;
    }
    i += 1; // the kind byte
    if gl.get(i) != Some(&0x00) || gl.get(i + 1) != Some(&0x02) {
        return None;
    }
    gl.get(i + 2).copied()
}

/// **W-R1c — a namespace-scope object THIS TU defines**, read off its `.gl` data
/// record: the COFF symbol name, `sizeof`, natural alignment, and linkage.
///
/// The three fields `c2_core::coff::BssObject` needs and nothing else. It exists
/// because [`gl_extern_data_names`] answers the opposite question — *is this an
/// UNDEFINED external whose address the port may reference without emitting a
/// section?* — and the dynamic-initializer class needs the records that one
/// refuses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlDataObject {
    /// The COFF symbol name, already in final form: undecorated for internal
    /// linkage, decorated for external (`docs/OBJ_DYNINIT_SHAPE.md` §3.1).
    pub(crate) coff_name: String,
    /// `sizeof` the object.
    pub(crate) size: u32,
    /// Natural alignment in bytes, from the TYPE tag — **not** the size.
    pub(crate) natural_align: u32,
    /// `true` => defined with external linkage (StorageClass 2 EXTERNAL);
    /// `false` => `static` (StorageClass 3 STATIC).
    pub(crate) external: bool,
    /// `true` => the object has a **static** initializer and goes to `.data`;
    /// `false` => it is uninitialized and goes to `.bss`.
    ///
    /// **This field is a gate, not a convenience** — see
    /// [`DATA_ATTR_INITIALIZED`]. `docs/OBJ_DATA_BSS_SHAPE.md` §2.2 (lane w-bss,
    /// 871 workload objs) refutes `OBJ_DYNINIT_SHAPE.md` §4.1's "`.bss` and
    /// `.CRT$XCU` are always exactly one each, always last": a dyninit TU that
    /// also declares one plain `char b1;` moves the single shared `.bss` out from
    /// behind `.text$yc` and **between the two `.XBLD$W` watermarks**, which is a
    /// different section order and a different obj. Counting the uninitialized
    /// objects is how a caller refuses that TU instead of emitting the
    /// dyninit-only layout for it.
    pub(crate) initialized: bool,
    /// `true` => the object is **its own COMDAT section** — `IMAGE_SCN_LNK_COMDAT`
    /// with aux `Selection = 2` (ANY) — rather than a member of the TU's shared
    /// `.data`/`.bss`. Over lane `w-cfg2`'s grid that is exactly a
    /// **function-local `static`**, and the section is placed **after** the code
    /// groups where a non-COMDAT one is placed before them.
    ///
    /// Read from [`DATA_ATTR_COMDAT`]; see that constant for the six graded
    /// cells and the three refuted rival readings. **A consumer that cannot
    /// place a COMDAT data section must refuse an object with this set**, which
    /// is what [`crate::IlBundle::data_tu`] does: emitting it into the shared
    /// section would be a wrong section count *and* a wrong symbol table.
    pub(crate) comdat: bool,
    /// **The attribute byte immediately AFTER the one [`Self::initialized`] is
    /// read from** — a bit field, published raw so each caller applies its own
    /// gate.
    ///
    /// **W-SECT found this field by finding the wrong emit it causes.**
    /// `__declspec(thread) int t1;` produces a `.gl` data record that is
    /// byte-identical to an ordinary uninitialized object *in every field this
    /// reader had*, and lands in `.tls$` rather than `.bss`:
    ///
    /// ```text
    ///   char b1;                     … 82 01 00 02 01 01 00 · 00 ·
    ///   __declspec(thread) int t1;   … 86 01 00 02 01 04 00 · 10 ·
    ///   __declspec(thread) int t2=4; … 86 01 00 02 01 04 80 · 10 ·
    /// ```
    ///
    /// So a reader that stopped at the attribute reported a `.tls$` object as a
    /// `.bss` one, and would have emitted the wrong section the moment a `.bss`
    /// writer had a caller. [`DATA_FLAG_THREAD_LOCAL`] is that bit.
    ///
    /// **It is deliberately NOT gated inside [`data_object_at`].** The bit field
    /// carries several meanings this lane measured but did not exhaust —
    ///
    /// ```text
    ///   00  char b1;                        (plain, unreferenced)
    ///   01  static int s1;  … referenced    (also ?ce, ?v2)
    ///   21  int gi;  with `int* gp = &gi;`  (0x20 = address taken)
    ///   61  the `??__E` fixtures' object    (0x40 unexplained)
    ///   83  `$sL$initializer$`              (0x80 unexplained)
    /// ```
    ///
    /// — and an allow-list of the values this lane could explain would have
    /// **refused the two `??__E` license TUs that are byte-exact matches today**
    /// (their object carries `0x61`), taking TU match 8 → 6. A gate in the shared
    /// reader is a gate on `dyninit_tu`, which is graded; so the byte is
    /// published and the *new* consumer applies the strict test.
    pub(crate) flags: u8,
}

/// The [`GlDataObject::flags`] bit that says the object is `__declspec(thread)`
/// and therefore lands in **`.tls$`**, not in `.bss`/`.data`.
///
/// MEASURED on four cells (initialized and uninitialized × external and
/// `static`); the bit is set in all four and clear in every non-thread-local
/// record this lane captured. `docs/OBJ_DATA_BSS_SHAPE.md` §5.8's Rule T1 is
/// fitted on ten probe cells and has **never been seen on a real TU**, so a
/// writer must **refuse** on this bit rather than emit a `.tls$`.
pub(crate) const DATA_FLAG_THREAD_LOCAL: u8 = 0x10;

/// The [`GlDataObject::flags`] bit that says **something references this
/// object**, and which decides whether an internal-linkage object is emitted at
/// all.
///
/// **MEASURED, and it is a rule `docs/OBJ_DATA_BSS_SHAPE.md` does not have.**
/// An `static` object that is **uninitialized** and **unreferenced** is
/// **dropped entirely** — no `.bss`, no symbol, and the obj is the bare
/// four-section shell:
///
/// ```text
///   static int za;                     04 04 00 · 00   DROPPED, obj is 720 B
///   static int zc; int r(){return zc;} 04 04 00 · 01   emitted to .bss
///   static int zb = 1;                 04 04 80 · 00   emitted to .data
///   int ze;                            01 04 00 · 00   emitted to .bss
/// ```
///
/// The three axes are separated one at a time: `za`→`zc` moves only the
/// reference, `za`→`zb` only the initializer, `za`→`ze` only the linkage. So
/// the predicate is the conjunction of all three and not any pair.
///
/// §5.2's static cells could not see this — every one of them is *"8 uninit
/// statics **and one function each**"*, so every object in them is referenced.
/// §4.4 records the same shape for `const` (*"dropped when every use was
/// folded"*); this is that rule for plain storage.
pub(crate) const DATA_FLAG_REFERENCED: u8 = 0x01;

/// The name separator that introduces an **internal-linkage** data symbol, whose
/// COFF name is the run that follows it **undecorated**.
///
/// [`NAME_SEPARATORS`] lists `00` and `26` and deliberately excludes `25`
/// (string literals). It never listed `24`, and that omission is precisely why
/// `TomCryptLicense.cpp` reported `data-sym-unresolved` while `ZlibLicense.cpp`
/// reported `data-sym-not-extern` from **byte-identical `.ex` files**: the only
/// difference between the two TUs is that one object is `static`.
///
/// MEASURED, four captures, `$`-introduced on the left and the COFF symbol
/// `docs/OBJ_DYNINIT_SHAPE.md` §3.1 records on the right:
///
/// ```text
///   $sL                 -> sL                    (fixture, static)
///   $sLicense           -> sLicense              (TomCryptLicense.cpp, static)
///   $sL$initializer$    -> sL$initializer$       (always static, either linkage)
///   ?sLicense@@3VLicenses@@A  (00-introduced) -> itself   (ZlibLicense.cpp, extern)
/// ```
///
/// The `$` is a *separator*, not part of the name — which is why
/// `$sL$initializer$` yields `sL$initializer$` and not `sL` — and it sits in
/// exactly the byte position `00` and `26` sit in, with the operand token
/// immediately before it.
///
/// **It is deliberately NOT added to [`NAME_SEPARATORS`].** That constant feeds
/// [`gl_symbol_index`], which binds every callee in the corpus; admitting a
/// fourth separator there would re-bind tokens globally, and this lane's whole
/// point is that the global data-symbol path must not move. This reader is
/// separate and its consumers are whole-TU-shaped.
const NAME_SEPARATOR_UNDECORATED: u8 = 0x24;

/// Linkage bytes a `.gl` data record can carry, at the fixed offset
/// [`data_linkage`] reads.
///
/// `02` (undefined external) is [`gl_extern_data_names`]'s whole population and
/// is **not** here: an object this TU does not define has no `.bss` to emit.
/// Anything unseen fails closed with it.
const LINKAGE_DEFINED_EXTERN: u8 = 0x01;

/// Tag bit that inserts one extra byte (the *mark*) before the kind.
///
/// `docs/IL_TYPE_WIDE_TAG.md` derived this for `.ex`/`.sy` on 2026-07-31; lane
/// `w-align` measured what the field UNDER it means in a `.gl` DATA record and
/// found it unchanged — see [`align_of_type_tag`].
const TAG_WIDE: u8 = 0x40;
const LINKAGE_STATIC: u8 = 0x04;

/// The attribute byte immediately after the size varint: `00` uninitialized
/// (`.bss`), `80` statically initialized (`.data`, or the `.CRT$XCU` slot).
///
/// MEASURED across every capture this lane took, and the two values separate
/// exactly the two section kinds:
///
/// ```text
///   $sL                       … 04 01 00   uninitialized -> .bss
///   $sLicense                 … 04 0c 00   uninitialized -> .bss
///   ?sLicense@@3VLicenses@@A  … 01 0c 00   uninitialized -> .bss
///   ?gExt@@3HA (extern int g) … 02 04 00   (refused earlier: linkage 02)
///   $sL$initializer$          … 04 04 80   initialized   -> .CRT$XCU slot
///   ?gDef@@3HA (int gDef = 3) … 01 04 80   initialized   -> .data
/// ```
///
/// A value that is neither fails closed: guessing which section an object lands
/// in is guessing the section *count*, which mismatches at file offset 2.
const DATA_ATTR_UNINITIALIZED: u8 = 0x00;
const DATA_ATTR_INITIALIZED: u8 = 0x80;

/// **The COMDAT bit of the same attribute byte** — the object is its own
/// section, `IMAGE_SCN_LNK_COMDAT` with aux `Selection = 2` (ANY), and its
/// symbol is a STATIC in it.
///
/// **MEASURED on a six-cell grid, both halves, lane `w-cfg2`**
/// (`work/w-cfg2/GRID_ATTR.md`). Each cell was read twice — the `.gl` attribute
/// byte, and independently the obj real `c2.dll` produced for it — and the two
/// readings agree on all six:
///
/// ```text
///   int f(){ static int p[4]={1,2,3,0}; … }   86 06 00 02 04 10 a0   .data  COMDAT sel 2  STATIC
///   int f(){ static int p=7; … }              86 01 00 02 04 04 a0   .data  COMDAT sel 2  STATIC
///   int f(int i){ static int p[4]; … }        86 06 00 02 04 10 20   .bss   COMDAT sel 2  STATIC
///   static int p[4]={1,2,3,0};                86 06 00 02 04 10 80   .data  plain      0  STATIC
///   int p[4]={1,2,3,0};                       86 06 00 02 01 10 80   .data  plain      0  EXTERNAL
///   int gDef=3;                               86 01 00 02 01 04 80   .data  plain      0  EXTERNAL
/// ```
///
/// The last row is the positive control: it is the value this function's own
/// doc already recorded for `?gDef@@3HA`, so the grid is reading the right byte.
/// Three rival readings were frozen with the grid and each dies at the cell
/// named for it — *aggregate* dies on the scalar `static int p=7` (`a0`) and on
/// the namespace-scope array (`80`); *the reader under-counted the initializer*
/// dies with it; *`a0` is a third enumerated value rather than a bit* dies on
/// the **uninitialized** function-local static, which reads `20` and which that
/// reading has no member for.
///
/// **`0x40` is deliberately NOT admitted.** This function's doc records `60`/`E0`
/// as the `__declspec(selectany)` forms — the same COMDAT bit plus `0x40` — and
/// no cell of this grid is a `selectany`. Admitting the bit this lane graded and
/// failing closed on the one it did not is the whole difference between a
/// measurement and a guess.
const DATA_ATTR_COMDAT: u8 = 0x20;

/// Every data object this TU defines, keyed by the operand token an `.ex` body
/// references it with.
///
/// A token two records disagree about is **dropped**, not resolved to the first
/// — the same third value [`gl_symbol_index`] gives an ambiguous token, and for
/// the same reason: a relocation against the wrong symbol is a mis-emit.
pub(crate) fn gl_data_objects(gl: &[u8]) -> std::collections::BTreeMap<u32, GlDataObject> {
    gl_data_objects_ordered(gl).into_iter().collect()
}

/// The same objects, **in `.gl` record order** rather than by token.
///
/// # Why the order is a value and not an implementation detail
///
/// `docs/OBJ_DATA_BSS_SHAPE.md` §5.2 (Rule A1) is the whole reason this exists:
///
/// > `.bss` ascending address = the IL `.gl` symbol-record order for objects
/// > **without** a dynamic initializer, and the **exact reverse** of it for
/// > objects **with** one; the two groups never interleave.
///
/// `OBJ_DYNINIT_SHAPE.md` §7.1 declined that permutation as "a name-keyed
/// ordering that would need the front end's hash reproduced". It does not: the
/// hash is `c1xx`'s, it runs before c2, and **its output is this record order**.
/// A reader that walks `.gl` in file order reproduces the permutation exactly and
/// never computes a hash.
///
/// [`gl_data_objects`] collects into a `BTreeMap` keyed by the operand token, so
/// it is sorted by *token* — which is a different order and has no reason to
/// agree. MEASURED on a six-object probe: `.gl` spells `$s2 $s1 $s5 $s3 $s4 $s6`
/// and the obj's `.bss` runs `s6@0 s4@1 s3@2 s5@3 s1@4 s2@5`, exactly the
/// reverse, reproducing `OBJ_DATA_BSS_SHAPE.md` §7.1's family-A row for N = 6.
/// Reading that order out of the `BTreeMap` would have given `s1 s2 s3 s4 s5 s6`
/// and six wrong `Value` fields.
///
/// The two functions share one body so they cannot drift: the map is built *from*
/// this vector, never beside it.
pub(crate) fn gl_data_objects_ordered(gl: &[u8]) -> Vec<(u32, GlDataObject)> {
    // Insertion-ordered, with the same poisoning rule [`gl_data_objects`]
    // documents: a token two records disagree about is dropped rather than
    // resolved to the first. `Option::None` marks a poisoned slot; the slot keeps
    // its position so a later re-bind cannot silently reorder the survivors.
    let mut out: Vec<(u32, Option<GlDataObject>)> = Vec::new();
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
        if i >= gl.len() {
            break;
        }
        // The run must be NUL-terminated; `end` indexes that NUL, which is where
        // the record's TYPE begins.
        let end = i;
        // Candidate name starts: every separator-preceded position in the run,
        // tried **rightmost first**, exactly the preference
        // `gl_symbol_index_checked` states — a record's own token bytes are often
        // printable and run together with the name, and the leftmost start binds
        // junk onto the front.
        //
        // Unlike that scanner this one cannot stop at the rightmost candidate,
        // because `24` is *both* a separator and a legal name character:
        // `$sL$initializer$` has three of them, and the rightmost two yield an
        // empty name and `initializer$`. So each candidate is validated whole —
        // token width, record-kind byte, and the data frame — and the first that
        // survives all three wins. Every rejected candidate is rejected on
        // structure, never on a guess about which `$` was meant.
        let mut q = end;
        while q > start {
            q -= 1;
            if q == 0 {
                break;
            }
            let sep = gl[q - 1];
            if !(NAME_SEPARATORS.contains(&sep) || sep == NAME_SEPARATOR_UNDECORATED) {
                continue;
            }
            if !is_object_name(&gl[q..end]) {
                continue;
            }
            // The operand token sits immediately before the separator, 4-byte form
            // first, and its own decoded width must land exactly on that separator.
            let mut bound = false;
            for w in [4usize, 2] {
                if q < w + 2 {
                    continue;
                }
                let p = q - 1 - w;
                let Some((tok, got)) = read_token_var(gl, p) else {
                    continue;
                };
                if got != w {
                    continue;
                }
                if !SYMBOL_RECORD_KINDS.contains(&gl[p - 1]) {
                    continue;
                }
                let Some(obj) = data_object_at(gl, end, &gl[q..end]) else {
                    continue;
                };
                match out.iter_mut().find(|(t, _)| *t == tok) {
                    None => out.push((tok, Some(obj))),
                    Some((_, slot @ Some(_))) if slot.as_ref() != Some(&obj) => *slot = None,
                    _ => {}
                }
                bound = true;
                break;
            }
            if bound {
                break;
            }
        }
    }
    out.into_iter().filter_map(|(t, o)| o.map(|o| (t, o))).collect()
}

/// The name separator that introduces a **string-literal** record. Named in
/// [`NAME_SEPARATORS`]'s doc as the value it deliberately excludes.
const NAME_SEPARATOR_STRING_LITERAL: u8 = 0x25;

/// Every `??_C@…` string-literal COMDAT name `.gl` carries, as a set.
///
/// **This is a fence, not a lookup.** `c2_core::coff::string_comdat_name`
/// computes the same name from the literal's bytes, and the two must agree
/// before anything is emitted. The reason is `/GF`, and it is the single most
/// likely way this class ships wrong bytes:
///
/// > `/GF` is implied by `/O1` and `/O2` but **not** by `/Ox`
/// > (`docs/OBJ_DYNINIT_SHAPE.md` §4.3). Without it the literal is a
/// > **non-COMDAT `$SG<n>` `.rdata` placed BEFORE `.text`**, with 5 relocations
/// > instead of 9 — a different obj entirely.
///
/// MEASURED: `fixtures/cpp/il_dyninit_static.cpp` captured at `/Ox` still
/// carries `abc\0` in `.in` (`ef 09 00 03 04 61 62 63 00 07`) and carries **no
/// `??_C@` record anywhere in `.gl`**. A reader that trusted `.in` alone would
/// compute `??_C@_03FIKCJHKP@abc?$AA@` and emit the `/O1` shape for a `/Ox`
/// TU. Requiring the computed name to be one `.gl` actually spells is what makes
/// that TU refuse, and it refuses on *structure* rather than on a flag test.
pub(crate) fn gl_string_comdat_names(gl: &[u8]) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let mut i = 0usize;
    while i < gl.len() {
        if gl[i] != NAME_SEPARATOR_STRING_LITERAL {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < gl.len() && gl[end].is_ascii_graphic() {
            end += 1;
        }
        // NUL-terminated, and spelled in the symbol alphabet. `??_C@` is required
        // literally: `25` is a common byte and only the mangled prefix makes a run
        // after one a string COMDAT rather than a coincidence.
        if end < gl.len() && gl[end] == 0 && gl[start..end].starts_with(b"??_C@") {
            let name = &gl[start..end];
            if name.iter().all(|&c| is_symbol_char(c)) {
                out.insert(ascii_string(name));
            }
        }
        i = end.max(start);
    }
    out
}

/// Whether a `.gl` run is an object name this reader may bind a token to.
///
/// Looser than [`is_indexable_name`] in exactly one way — **length 1 is
/// admitted** — because the reference cell's object is literally `sL`, and a
/// two-character name is not a weaker binding than a twenty-character one. The
/// structural work is done by [`data_object_at`]'s frame check, not by the shape
/// of the name.
fn is_object_name(b: &[u8]) -> bool {
    !b.is_empty()
        && (b[0] == b'?' || b[0].is_ascii_alphabetic() || b[0] == b'_')
        && b.iter().all(|&c| is_symbol_char(c))
}

/// Parse the DATA record whose name's terminating NUL is at `name_nul`, or
/// `None` when it is not one this port models.
///
/// The frame, MEASURED across the fixture, both workload TUs and three probes:
///
/// ```text
///   <tag> [wide] <kind> 00 <02|04> <linkage> <size varint> <attr>
///
///   $sL\0                       82 06 00 02 04 01 00     align 1  size 1    static
///   $sLicense\0                 86 06 00 02 04 0c 00     align 4  size 12   static
///   ?sLicense@@3VLicenses@@A\0  86 06 00 02 01 0c 00     align 4  size 12   extern
///   ?gL@@3UL@@A\0               86 06 00 02 01 04 00     align 4  size 4    extern
///   $sL\0  (char pad[200])      82 06 00 02 04 80c8000000  align 1 size 200 static
///   ??_C@_0BK@…\0  (a literal)  82 06 00 04 01 1a a0     REFUSED — `00 04`
/// ```
///
/// **The tag is the object's ALIGNMENT, not its size**, and the aggregate grid
/// is what separates the two readings — `docs/IL_TYPE_TAGS.md` §1 tabulates
/// 1→`82`, 2→`84`, 4→`86`, 8→`88` under the heading `size`, which is true only
/// because a scalar's size *is* its alignment. Three ints (`sizeof` 12) and one
/// int (`sizeof` 4) share tag `86`; two doubles (16) and one double (8) share
/// `88`. So the tag is read as alignment and the size is read from its own
/// field.
///
/// The `00 <02|04>` pair is required literally and is what makes this structural
/// rather than an offset guess: a *function* record spells `82 07 <05|04|03>`
/// there and fails, and a **string literal** spells `00 04` and fails — the
/// latter on purpose, since a literal is not a `.bss` object and is read from
/// `.in` instead.
fn data_object_at(gl: &[u8], name_nul: usize, name: &[u8]) -> Option<GlDataObject> {
    /// …and that byte must carry this.
    const WIDE_MARK: u8 = 0x80;
    let tag = *gl.get(name_nul + 1)?;
    if tag & 0x80 == 0 {
        return None;
    }
    let mut i = name_nul + 2;
    // The mark byte's VALUE is carried to [`align_of_type_tag`], not merely
    // stepped over: the width field's meaning under a mark this port has never
    // seen is unproven, and `docs/IL_TYPE_WIDE_TAG.md` §8 item 2 records that the
    // value set is larger than one (`84` occurs 106 times in `.ex`).
    let mut wide_mark = None;
    if tag & TAG_WIDE != 0 {
        let m = *gl.get(i)?;
        if m & WIDE_MARK == 0 {
            return None;
        }
        wide_mark = Some(m);
        i += 1;
    }
    i += 1; // the kind byte
    // The ORDINARY-DATA frame. `00 04` is a read-only (string-literal) record and
    // is refused here rather than admitted with a different meaning.
    if gl.get(i) != Some(&0x00) || gl.get(i + 1) != Some(&0x02) {
        return None;
    }
    let external = match *gl.get(i + 2)? {
        LINKAGE_DEFINED_EXTERN => true,
        LINKAGE_STATIC => false,
        // `02` (undefined external) and everything unseen: not an object this TU
        // defines, so there is no `.bss` for it and no size to believe.
        _ => return None,
    };
    // The size varint, in the SAME encoding `read_varint` reads — confirmed by
    // the `char pad[200]` probe, which spells `80 c8 00 00 00`. A negative or
    // zero size is not an object.
    let mut p = i + 3;
    let size = super::readers::read_varint(gl, &mut p)?;
    if size <= 0 {
        return None;
    }
    // The attribute is a **bit field of two independent bits**, and it is read
    // as one only for the two bits a grid has graded: `0x80` initialized
    // ([`DATA_ATTR_INITIALIZED`]) and `0x20` COMDAT ([`DATA_ATTR_COMDAT`]).
    // Everything else still fails closed — in particular `0x40`, which turns
    // `20`/`a0` into the `__declspec(selectany)` forms `60`/`E0` that this
    // reader has never been graded on. Board #172's "COMDAT-ness is NOT in
    // tag 9" is about the SECTION record and does not carry over to this one.
    let attr = *gl.get(p)?;
    let comdat = attr & DATA_ATTR_COMDAT != 0;
    let initialized = match attr & !DATA_ATTR_COMDAT {
        DATA_ATTR_UNINITIALIZED => false,
        DATA_ATTR_INITIALIZED => true,
        _ => return None,
    };
    let flags = *gl.get(p + 1)?;
    let natural_align = align_of_type_tag(tag, wide_mark)?;
    // A name introduced by `24` is spelled undecorated; one introduced by `00`
    // carries its own decoration. The separator has already selected the run, so
    // the name is used exactly as found.
    Some(GlDataObject {
        coff_name: ascii_string(name),
        size: size as u32,
        natural_align,
        external,
        initialized,
        comdat,
        flags,
    })
}

/// The natural alignment a `.gl` TYPE tag encodes, in bytes.
///
/// `docs/IL_TYPE_TAGS.md` §1's positional rule, read as **alignment** — see
/// [`data_object_at`] for the aggregate grid that separates that reading from
/// the size reading its heading implies. Fails closed on an unmodeled tag: a
/// wrong alignment nibble is a wrong `.bss` Characteristics word.
///
/// # The WIDE form (board #1110, lane `w-align`)
///
/// [`TAG_WIDE`] marks only the presence of the mark byte. **It is orthogonal to
/// the width field**, so the alignment is read off `tag & !TAG_WIDE` — `C6` is
/// `86` is 4 bytes. Board #1110 recorded this function refusing `0xC6` and
/// blocking a plain `.data` object with no RTTI involved; that is the arm below.
///
/// MEASURED on 23 frozen cells at the workload's own `/GR /O1 /Oi /EHsc`, each
/// one's tag read off `.gl` and its alignment read off **c2's own obj** (the
/// section `Characteristics` nibble c2 gave that symbol) — 23 of 23 agree:
///
/// ```text
///   tag  masked  align  c2   cells
///   82   82      1      1    char                                     (T12)
///   86   86      4      4    {int,int} · derived · nested · int[8]     (T10 T13 T17 T18)
///   88   88      8      8    double · poly+double · poly+long long     (T02 T05 T11 G03 G05)
///   C6   86      4      4    poly+int · poly+char · poly+char[64] ·
///                            poly+empty · poly[4] · vbase · vdtor      (T01 T03 T04 T06 T07 T14 T15 G01)
///   C8   88      8      8    __declspec(align(8)), poly AND plain      (T08 T16)
///   CA   8A      —      16   __declspec(align(16))  -> REFUSED here    (T09 G04)
/// ```
///
/// Three adjacent readings die on this grid, and each had a cell built to kill
/// it: the tag is **not the size** (`T03` is size 8 at `C6`, `T04` is size 68 at
/// `C6`), `0xC6` is **not an atomic "wide aggregate" tag pinned at 4** (`T02`
/// and `T05` are polymorphic and spell `88`), and it is **not the largest
/// member's width** (`T04`'s largest member is `char[64]`).
///
/// **`__declspec(align(N))` moves the tag** — `T08` and `T16` are naturally
/// 4-aligned and spell `C8`, and c2 gives them ALIGN_8. That was the prereg's
/// least-confident prediction (P5, 0.50) and it is the reason the grid exists: a
/// widening that read the *natural* layout instead would emit a wrong
/// `Characteristics` word for exactly those two cells.
///
/// `CA` (= wide, width 16) keeps refusing. c2 really does give `T09`/`G04`
/// ALIGN_16, and `super::super::…::placement_align` models only 1/2/4/8, so
/// reading it would produce a nibble the writer cannot honour. **Refusing an
/// alignment the writer cannot express is the whole point of this function.**
///
/// # Why the mark's VALUE is required
///
/// Only `0x81` was observed on all ten wide cells here. `IL_TYPE_WIDE_TAG.md`
/// §8 item 2 records `0x84` as a second value in `.ex` and registers "the value
/// set is unknown" as that work's residual risk. Accepting `C6 84 …` on the
/// strength of `C6 81 …` would be reading a field this port has never seen, so
/// the mark is matched and not merely stepped over.
fn align_of_type_tag(tag: u8, wide_mark: Option<u8>) -> Option<u32> {
    /// The only mark value any `.gl` DATA record in the frozen grid carries.
    const WIDE_MARK_MEASURED: u8 = 0x81;
    match wide_mark {
        None | Some(WIDE_MARK_MEASURED) => {}
        Some(_) => return None,
    }
    match tag & !TAG_WIDE {
        0x82 => Some(1),
        0x84 => Some(2),
        0x86 => Some(4),
        0x88 => Some(8),
        // Board #1120, lane `w-align16`. `8A` is 16 and the writer can now
        // express it — `placement_align`, `align_nibble` and `section_nibble`
        // all carry the 16 arm, and `bump_layout` rounds a `.bss` cursor to it.
        // Confirmed on 12 cells against c2's own obj, including the two-object
        // `.bss` whose second object lands at offset 16.
        //
        // **The wide form `CA` is what every 16-aligned cell actually spells;
        // bare `8A` has never been observed by this project.** It is accepted
        // here by the ORTHOGONALITY rule (`w-align`, 21 of 21) rather than by a
        // witness — see the note above this function.
        0x8A => Some(16),
        // `8C` (32) and `8E` (64) EXIST and c2 honours them — nibbles 6 and 7,
        // measured on `A09`/`A10`/`A18`. They stay refused: the grid varies
        // structure at 16 (scalar, aggregate, empty, array, polymorphic,
        // natural-via-member, static, `.data`, two-object) and varies nothing at
        // 32/64, so accepting them would be extending a table by `log2` past the
        // cells that constrain it. Price: one more grid of the same shape.
        _ => None,
    }
}

pub(crate) fn gl_extern_data_names(gl: &[u8]) -> std::collections::BTreeSet<String> {
    /// Undefined external. `01` (defined here) and `04` (static) are exactly the
    /// two this must refuse, and everything unseen refuses with them.
    const LINKAGE_UNDEF_EXTERN: u8 = 0x02;
    let mut out = std::collections::BTreeSet::new();
    let mut bad = std::collections::BTreeSet::new();
    for (_, end, name) in gl_symbol_runs(gl) {
        // `end` is the index of the run's terminating NUL; the TYPE begins at the
        // byte after it and its width is read, not assumed.
        let ok = data_linkage(gl, end) == Some(LINKAGE_UNDEF_EXTERN);
        if ok {
            out.insert(name);
        } else {
            bad.insert(name);
        }
    }
    // A name any record disagrees about is refused, not resolved to the record
    // that happened to be favourable — the same third value `gl_symbol_index`
    // gives an ambiguous token.
    out.retain(|n| !bad.contains(n));
    out
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
mod data_object_tests {
    use super::{gl_data_objects, GlDataObject, DATA_FLAG_THREAD_LOCAL};

    /// Build the `<kind byte> <token> <sep> <name> 00 <type…>` core of one `.gl`
    /// data record, the way every measured capture spells it.
    ///
    /// `ty` stops at the attribute byte and this helper appends the **flags**
    /// byte after it as `00`, because that is what every plain unreferenced
    /// object in a real capture spells and because [`super::data_object_at`]
    /// now requires the byte to exist. Use [`record_flags`] for a cell whose
    /// flags matter.
    fn record(tok: [u8; 2], sep: u8, name: &str, ty: &[u8]) -> Vec<u8> {
        record_flags(tok, sep, name, ty, 0x00)
    }

    /// [`record`] with the flags byte spelled explicitly.
    fn record_flags(tok: [u8; 2], sep: u8, name: &str, ty: &[u8], flags: u8) -> Vec<u8> {
        let mut v = vec![0x00]; // a SYMBOL_RECORD_KINDS byte
        v.extend_from_slice(&tok);
        v.push(sep);
        v.extend_from_slice(name.as_bytes());
        v.push(0x00);
        v.extend_from_slice(ty);
        v.push(flags);
        v
    }

    /// **The thread-local bit, and the wrong emit it prevents.**
    ///
    /// `__declspec(thread) int t1;` and `char b1;` differ in *no field this
    /// reader had* before W-SECT — same frame, same linkage, same attribute —
    /// and land in different sections (`.tls$` vs `.bss`). Both records still
    /// decode, on purpose: the bit is published rather than gated here, because
    /// gating it inside the shared reader is gating `dyninit_tu`, which is a
    /// graded path. The *caller* refuses.
    ///
    /// The four measured thread-local cells all carry `0x10`, across both
    /// linkages and both initialization states.
    #[test]
    fn the_thread_local_bit_is_published_not_swallowed() {
        // `__declspec(thread) int t1;` — MEASURED `86 01 00 02 01 04 00 · 10`.
        let gl = record_flags(
            [0xe3, 0x09],
            0x00,
            "?t1@@3HA",
            &[0x86, 0x01, 0x00, 0x02, 0x01, 0x04, 0x00],
            DATA_FLAG_THREAD_LOCAL,
        );
        let o = gl_data_objects(&gl);
        let o = o.get(&0xe309).expect("it still decodes — the caller refuses, not the reader");
        assert_eq!(o.flags & DATA_FLAG_THREAD_LOCAL, DATA_FLAG_THREAD_LOCAL);
        assert!(!o.initialized, "and it is indistinguishable from `.bss` in every OTHER field");

        // `__declspec(thread) int t2 = 4;` — MEASURED `… 04 80 · 10`.
        let gl = record_flags(
            [0xe3, 0x09],
            0x00,
            "?t2@@3HA",
            &[0x86, 0x01, 0x00, 0x02, 0x01, 0x04, 0x80],
            DATA_FLAG_THREAD_LOCAL,
        );
        assert_eq!(
            gl_data_objects(&gl).get(&0xe309).map(|o| o.flags & DATA_FLAG_THREAD_LOCAL),
            Some(DATA_FLAG_THREAD_LOCAL)
        );

        // …and the control: an ordinary `char b1;` carries the bit CLEAR, so the
        // test discriminates rather than asserting a constant.
        let gl = record([0xe3, 0x09], 0x00, "?b1@@3DA", &[0x82, 0x01, 0x00, 0x02, 0x01, 0x01, 0x00]);
        assert_eq!(
            gl_data_objects(&gl).get(&0xe309).map(|o| o.flags & DATA_FLAG_THREAD_LOCAL),
            Some(0)
        );
    }

    /// **`__declspec(selectany)` is refused by the ATTRIBUTE byte**, which this
    /// lane registered as prediction P2 and which is why no COMDAT gate is
    /// needed anywhere else.
    ///
    /// MEASURED, one axis, four cells: `int sa = 3;` spells `80` and
    /// `__declspec(selectany) int sa = 3;` spells `E0`; `int sb;` spells `00`
    /// and `__declspec(selectany) int sb;` spells `60`. A COMDAT `.data`/`.bss`
    /// is a different Characteristics word (`…1040`/`…1080`), a Selection of 2
    /// and its own section, so admitting one as ordinary data is a wrong
    /// section count.
    #[test]
    fn selectany_fails_closed_on_the_attribute_byte() {
        for attr in [0x60u8, 0xE0] {
            let gl = record([0xe3, 0x09], 0x00, "?sa@@3HA", &[0x86, 0x01, 0x00, 0x02, 0x01, 0x04, attr]);
            assert!(
                gl_data_objects(&gl).is_empty(),
                "selectany attribute {attr:#04x} must not decode as ordinary data"
            );
        }
        // The controls, same records with the ordinary attributes.
        for attr in [0x00u8, 0x80] {
            let gl = record([0xe3, 0x09], 0x00, "?sa@@3HA", &[0x86, 0x01, 0x00, 0x02, 0x01, 0x04, attr]);
            assert!(!gl_data_objects(&gl).is_empty(), "attribute {attr:#04x} is ordinary data");
        }
    }

    /// The flags byte is REQUIRED to exist: a record truncated immediately after
    /// its attribute byte refuses rather than decoding with a default.
    /// Fail-closed, because a missing byte is a record this reader is not
    /// looking at the whole of.
    #[test]
    fn a_record_truncated_before_the_flags_byte_refuses() {
        let mut v = vec![0x00, 0xe3, 0x09, 0x00];
        v.extend_from_slice(b"?b1@@3DA");
        v.push(0x00);
        v.extend_from_slice(&[0x82, 0x01, 0x00, 0x02, 0x01, 0x01, 0x00]);
        assert!(gl_data_objects(&v).is_empty(), "no flags byte, no object");
    }

    /// **The six measured rows**, from the fixture, both workload TUs and three
    /// probes. Every byte here was read off a real capture — see
    /// `docs/rungs/_2026-08-04-w-r1c-prereg.md` §6.
    #[test]
    fn the_measured_gl_data_records_decode() {
        // `static L sL("abc",0);` — the reference cell. sizeof 1, align 1.
        let gl = record([0xec, 0x09], 0x24, "sL", &[0x82, 0x06, 0x00, 0x02, 0x04, 0x01, 0x00]);
        assert_eq!(
            gl_data_objects(&gl).get(&0xec09),
            Some(&GlDataObject {
                coff_name: "sL".to_string(),
                size: 1,
                natural_align: 1,
                external: false,
                initialized: false,
                comdat: false,
                flags: 0,
            }),
            "the `$`-introduced name yields the UNDECORATED COFF symbol"
        );

        // TomCryptLicense.cpp — static, sizeof 12, align 4.
        let gl = record(
            [0xf9, 0x09],
            0x24,
            "sLicense",
            &[0x86, 0x06, 0x00, 0x02, 0x04, 0x0c, 0x00],
        );
        assert_eq!(
            gl_data_objects(&gl).get(&0xf909),
            Some(&GlDataObject {
                coff_name: "sLicense".to_string(),
                size: 12,
                natural_align: 4,
                external: false,
                initialized: false,
                comdat: false,
                flags: 0,
            })
        );

        // ZlibLicense.cpp — the SAME object, external, and therefore decorated.
        // Its `.ex` is byte-identical to TomCrypt's; this record is the only
        // structural difference between the two TUs.
        let gl = record(
            [0xf9, 0x09],
            0x00,
            "?sLicense@@3VLicenses@@A",
            &[0x86, 0x06, 0x00, 0x02, 0x01, 0x0c, 0x00],
        );
        assert_eq!(
            gl_data_objects(&gl).get(&0xf909),
            Some(&GlDataObject {
                coff_name: "?sLicense@@3VLicenses@@A".to_string(),
                size: 12,
                natural_align: 4,
                external: true,
                initialized: false,
                comdat: false,
                flags: 0,
            })
        );

        // `$sL$initializer$` — the `$` is a SEPARATOR, so the inner `$`s survive
        // and the name is `sL$initializer$`, not `sL`.
        let gl = record(
            [0x08, 0x0a],
            0x24,
            "sL$initializer$",
            &[0x86, 0x04, 0x00, 0x02, 0x04, 0x04, 0x80],
        );
        assert_eq!(
            gl_data_objects(&gl).get(&0x080a).map(|o| o.coff_name.as_str()),
            Some("sL$initializer$")
        );
    }

    /// **THE COMDAT BIT OF THE ATTRIBUTE BYTE — all six graded cells** (lane
    /// `w-cfg2`, board **#1680**; `work/w-cfg2/GRID_ATTR.md`).
    ///
    /// Each row is a real `.gl` record read off a real capture, and each one's
    /// verdict here is the one real `c2.dll`'s own obj gives for the same cell —
    /// COMDAT section with `Selection = 2` where `comdat` is true, a member of
    /// the TU's shared `.data`/`.bss` where it is false. The two halves were
    /// predicted separately and agree on all six, which is why the bit is read
    /// at all rather than left failing closed.
    ///
    /// The last row is the **positive control**: it is the value
    /// [`data_object_at`]'s own doc recorded for `?gDef@@3HA` before this lane
    /// existed, so a change that broke the reading would show here first.
    #[test]
    fn the_attribute_byte_is_two_independent_bits() {
        //  cell, frame,                                        init, comdat, extern
        let cells: [(&str, [u8; 7], bool, bool, bool); 6] = [
            // a1  int f(){ static int p[4]={1,2,3,0}; … }
            ("a1", [0x86, 0x06, 0x00, 0x02, 0x04, 0x10, 0xa0], true, true, false),
            // a2  int f(){ static int p=7; … }            — SCALAR, still COMDAT
            ("a2", [0x86, 0x01, 0x00, 0x02, 0x04, 0x04, 0xa0], true, true, false),
            // a3  int f(int i){ static int p[4]; … }      — UNINITIALIZED, COMDAT
            ("a3", [0x86, 0x06, 0x00, 0x02, 0x04, 0x10, 0x20], false, true, false),
            // a4  static int p[4]={1,2,3,0};              — AGGREGATE, NOT COMDAT
            ("a4", [0x86, 0x06, 0x00, 0x02, 0x04, 0x10, 0x80], true, false, false),
            // a5  int p[4]={1,2,3,0};
            ("a5", [0x86, 0x06, 0x00, 0x02, 0x01, 0x10, 0x80], true, false, true),
            // a6  int gDef=3;                             — THE POSITIVE CONTROL
            ("a6", [0x86, 0x01, 0x00, 0x02, 0x01, 0x04, 0x80], true, false, true),
        ];
        for (cell, ty, initialized, comdat, external) in cells {
            let gl = record([0xea, 0x09], 0x24, "p", &ty);
            let got = gl_data_objects(&gl);
            let o = got.get(&0xea09).unwrap_or_else(|| panic!("{cell}: refused"));
            assert_eq!(o.initialized, initialized, "{cell}: initialized");
            assert_eq!(o.comdat, comdat, "{cell}: comdat");
            assert_eq!(o.external, external, "{cell}: external");
        }
    }

    /// **`__declspec(selectany)` is NOT admitted, and that is the fence.**
    ///
    /// `60`/`E0` are the same COMDAT bit plus `0x40`, and no cell of w-cfg2's
    /// grid is a `selectany`. Widening the byte to a bit field would have made
    /// them fall out for free; they do not, because the mask names the one bit
    /// that was graded. Without this test the next reading of the attribute has
    /// no way to tell which bits carry evidence.
    #[test]
    fn the_selectany_forms_still_fail_closed() {
        for attr in [0x60u8, 0xe0] {
            let gl = record([0xea, 0x09], 0x24, "p", &[0x86, 0x06, 0x00, 0x02, 0x04, 0x10, attr]);
            assert!(
                gl_data_objects(&gl).is_empty(),
                "attr {attr:#04x} is a selectany form and has never been graded"
            );
        }
    }

    /// **The `size > 127` probe** — `char pad[200]` spells its size with
    /// `read_varint`'s escape. Reading one byte would have yielded 0x80 read as
    /// a signed byte, i.e. −128, and a size that is not a size.
    #[test]
    fn a_size_past_127_uses_the_varint_escape() {
        let gl = record(
            [0xec, 0x09],
            0x24,
            "sL",
            &[0x82, 0x06, 0x00, 0x02, 0x04, 0x80, 0xc8, 0x00, 0x00, 0x00, 0x00],
        );
        assert_eq!(
            gl_data_objects(&gl).get(&0xec09),
            Some(&GlDataObject {
                coff_name: "sL".to_string(),
                size: 200,
                natural_align: 1,
                external: false,
                initialized: false,
                comdat: false,
                flags: 0,
            })
        );
    }

    /// **The tag is ALIGNMENT, not size** — the reading `docs/IL_TYPE_TAGS.md`
    /// §1's `| size | tag |` heading implies is false for an aggregate, and only
    /// an aggregate can show it. Three ints (`sizeof` 12) hold tag `86` where one
    /// int (`sizeof` 4) also holds `86`; two doubles (16) and one double (8)
    /// share `88`. A scalar-only matrix cannot separate the two readings because
    /// a scalar's size IS its alignment.
    #[test]
    fn the_type_tag_encodes_alignment_and_not_size() {
        let cases: [(u8, u8, u32, u32); 6] = [
            // tag,  size byte, expect size, expect align
            (0x82, 0x01, 1, 1),   // char c;
            (0x84, 0x02, 2, 2),   // short c;
            (0x86, 0x04, 4, 4),   // int c;
            (0x88, 0x08, 8, 8),   // double c;
            (0x86, 0x0c, 12, 4),  // int a,b,c;   <- tag held, size moved
            (0x88, 0x10, 16, 8),  // double a,b;  <- tag held, size moved
        ];
        for (tag, sz, want_size, want_align) in cases {
            let gl = record(
                [0xec, 0x09],
                0x24,
                "sL",
                &[tag, 0x06, 0x00, 0x02, 0x04, sz, 0x00],
            );
            let got = gl_data_objects(&gl);
            let o = got.get(&0xec09).expect("in class");
            assert_eq!(
                (o.size, o.natural_align),
                (want_size, want_align),
                "tag {tag:#04x} size byte {sz:#04x}"
            );
        }
    }

    /// Four record classes that must refuse, each for its own structural reason.
    #[test]
    fn the_frame_check_refuses_everything_it_has_not_measured() {
        // A STRING LITERAL: `00 04`, not `00 02`. It is not a `.bss` object, and
        // its bytes come from `.in` instead.
        let gl = record(
            [0xfc, 0x09],
            0x25,
            "??_C@_0BK@PELMDOBM@x",
            &[0x82, 0x06, 0x00, 0x04, 0x01, 0x1a, 0xa0],
        );
        assert!(gl_data_objects(&gl).is_empty(), "a `00 04` record is not an object");

        // An UNDEFINED external (`extern int g;`): linkage 02. This TU defines no
        // storage for it, so there is no `.bss` and no size to believe.
        let gl = record([0xec, 0x09], 0x00, "?gExt@@3HA", &[0x86, 0x01, 0x00, 0x02, 0x02, 0x04, 0x00]);
        assert!(gl_data_objects(&gl).is_empty(), "linkage 02 is not defined here");

        // A FUNCTION record: `82 07 03` where the frame wants `00 02`.
        let gl = record([0xfa, 0x09], 0x00, "??__EsL@@YAXXZ", &[0x82, 0x07, 0x03, 0x00, 0x20, 0xa2]);
        assert!(gl_data_objects(&gl).is_empty(), "a function is not an object");

        // An UNMODELED alignment tag fails closed: a wrong alignment nibble is a
        // wrong `.bss` Characteristics word. **Board #1120 moved this witness
        // from `8a` to `8c`** — `8a` is 16 and is now modeled end to end, while
        // `8c` (32) is measured on cell `A09` (c2 emits nibble 6) and refused.
        // The row is kept rather than deleted because the *property* under test
        // is "an alignment the writer cannot express refuses the whole record",
        // and that property needs a live witness at all times.
        let gl = record([0xec, 0x09], 0x24, "sL", &[0x8c, 0x06, 0x00, 0x02, 0x04, 0x04, 0x00]);
        assert!(gl_data_objects(&gl).is_empty(), "tag 0x8c (align 32) is not modeled");
        // And the one below it now READS, which is what #1120 bought.
        let gl = record([0xec, 0x09], 0x24, "sL", &[0x8a, 0x06, 0x00, 0x02, 0x04, 0x04, 0x00]);
        assert_eq!(
            gl_data_objects(&gl).values().next().map(|o| o.natural_align),
            Some(16),
            "tag 0x8a is ALIGN_16 since #1120"
        );
    }

    /// **The `.bss` / `.data` discriminator, which lane w-bss made load-bearing.**
    ///
    /// `docs/OBJ_DATA_BSS_SHAPE.md` §2.2 refutes `OBJ_DYNINIT_SHAPE.md` §4.1's
    /// "`.bss` and `.CRT$XCU` are always exactly one each, always last". A
    /// dyninit TU that also declares one plain `char b1;` moves the shared `.bss`
    /// **between the two `.XBLD$W` watermarks** — a different section order and a
    /// different obj from the one `emit_dyninit_obj` builds.
    ///
    /// So a caller has to be able to count the *uninitialized* objects, and this
    /// is the field that lets it. The `$initializer$` slot parses as a data
    /// record too and must not be counted as a second `.bss` object.
    #[test]
    fn the_attr_byte_separates_bss_objects_from_data_objects() {
        // The object itself: uninitialized -> `.bss`.
        let gl = record([0xf9, 0x09], 0x24, "sLicense", &[0x86, 0x06, 0x00, 0x02, 0x04, 0x0c, 0x00]);
        assert_eq!(gl_data_objects(&gl).get(&0xf909).map(|o| o.initialized), Some(false));

        // Its `.CRT$XCU` slot: initialized -> NOT a `.bss` object.
        let gl = record(
            [0x15, 0x0a],
            0x24,
            "sLicense$initializer$",
            &[0x86, 0x04, 0x00, 0x02, 0x04, 0x04, 0x80],
        );
        assert_eq!(gl_data_objects(&gl).get(&0x150a).map(|o| o.initialized), Some(true));

        // `int gDef = 3;` — a real `.data` object, and the thing that would add a
        // ninth section if a caller mistook it for `.bss`.
        let gl = record([0xec, 0x09], 0x00, "?gDef@@3HA", &[0x86, 0x01, 0x00, 0x02, 0x01, 0x04, 0x80]);
        assert_eq!(gl_data_objects(&gl).get(&0xec09).map(|o| o.initialized), Some(true));

        // An unmodeled attribute fails closed — guessing which section an object
        // lands in is guessing the section COUNT.
        let gl = record([0xec, 0x09], 0x00, "?gX@@3HA", &[0x86, 0x01, 0x00, 0x02, 0x01, 0x04, 0x40]);
        assert!(gl_data_objects(&gl).is_empty());
    }

    /// A token two records disagree about is dropped, not resolved to the first.
    #[test]
    fn an_ambiguous_token_is_dropped() {
        let mut gl = record([0xec, 0x09], 0x24, "sL", &[0x82, 0x06, 0x00, 0x02, 0x04, 0x01, 0x00]);
        gl.extend_from_slice(&record(
            [0xec, 0x09],
            0x24,
            "sOther",
            &[0x86, 0x06, 0x00, 0x02, 0x04, 0x04, 0x00],
        ));
        assert_eq!(gl_data_objects(&gl).get(&0xec09), None);
    }

    /// **The global path must not move.** `gl_extern_data_names` is what
    /// `Bindings::resolve_data` gates on, and this lane's prereg clause 6
    /// declines if its acceptance changes. The two linkage classes this new
    /// reader admits are exactly the two that one refuses, and vice versa.
    #[test]
    fn the_new_reader_and_the_extern_gate_admit_disjoint_linkages() {
        let defined = record([0xec, 0x09], 0x00, "?gL@@3UL@@A", &[0x86, 0x06, 0x00, 0x02, 0x01, 0x04, 0x00]);
        let undefined = record([0xec, 0x09], 0x00, "?gExt@@3HA", &[0x86, 0x01, 0x00, 0x02, 0x02, 0x04, 0x00]);

        assert!(super::gl_extern_data_names(&defined).is_empty());
        assert!(!gl_data_objects(&defined).is_empty());

        assert!(!super::gl_extern_data_names(&undefined).is_empty());
        assert!(gl_data_objects(&undefined).is_empty());
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

    /// **W-VGL / board #151 — the reader's own fact: `.gl` introduces a name with
    /// `00` *or* `26`, and the two scanners must differ exactly there.**
    ///
    /// The layout is the one every real TU carries, transcribed from
    /// `src/system/obj/TextFile.cpp`: an out-of-line function's record, then its
    /// trailing binary, then a `26` introducing the deleting destructor's name.
    ///
    /// ```text
    /// ?_Copy_str@exception@std@@AAAXPBD@Z 00 <record> 0e ae 15
    ///   26 ??_Gexception@std@@UAAPAXI@Z 00 <record>
    /// ```
    ///
    /// Three claims, each of which can fail on its own:
    /// * the NUL-only scanner does **not** see the `26` name — if it did, the
    ///   whole repair would be measuring nothing;
    /// * the all-separator scanner **does**;
    /// * and it does not glue `0e ae 15`'s printable tail onto the front of it,
    ///   which is what terminating at `26` (and not only opening there) buys.
    #[test]
    fn a_26_introduced_name_is_invisible_to_the_nul_scanner_and_visible_to_the_other() {
        let mut gl = vec![0u8];
        gl.extend_from_slice(b"?out@@AAAXXZ");
        gl.push(0);
        // Record bytes, ending in two that happen to be printable ASCII.
        gl.extend_from_slice(&[0x86, 0x03, 0x05, 0x04, 0x0e, b'H', b'=']);
        gl.push(0x26);
        gl.extend_from_slice(b"??_Gexception@std@@UAAPAXI@Z");
        gl.push(0);

        let nul: Vec<String> = gl_symbol_runs(&gl).into_iter().map(|(_, _, n)| n).collect();
        let all: Vec<String> = gl_symbol_runs_all_separators(&gl)
            .into_iter()
            .map(|(_, _, n)| n)
            .collect();

        assert!(
            nul.iter().all(|n| n != "??_Gexception@std@@UAAPAXI@Z"),
            "control: the NUL-only scanner must NOT see a `26`-introduced name, or \
             this pair is not testing the separator: {nul:?}"
        );
        assert!(
            all.iter().any(|n| n == "??_Gexception@std@@UAAPAXI@Z"),
            "the all-separator scanner must see it: {all:?}"
        );
        assert!(
            all.iter().all(|n| !n.contains('&')),
            "and must not glue the record's printable tail onto it: {all:?}"
        );
        assert!(
            all.iter().any(|n| n == "?out@@AAAXXZ"),
            "the NUL-introduced name must still be found unchanged: {all:?}"
        );
    }

    /// The first two records of `fixtures/cpp/il_gl_sep26.cpp`, transcribed byte
    /// for byte from `.gl` offsets 125..213 of its capture, with **both** readers
    /// run over them.
    ///
    /// ```text
    /// 125  00 '??1R@@UAA@XZ' 00              name ends 138
    /// 139  82 07 05 00 20 20 03 00 00 03 00  TYPE
    /// 150  80 05 10 00 00 00 00              framing
    /// 157  80 54 0a 00 00                    body @ 2644     (distance 19)
    /// 162  00 15 c8 18 01 01 ec 09 01 0e ef 09   record tail
    /// 174  26                                the separator
    /// 175  '??_GR@@UAAPAXI@Z' 00             name ends 191
    /// 192  86 03 05 04 20 20 01 00 00        TYPE
    /// 201  80 0f 10 00 00 00 00              framing
    /// 208  80 d1 0a 00 00                    body @ 2769     (distance 17)
    /// ```
    ///
    /// The incumbent could not see the second name, so the second record's
    /// nearest preceding run was `??1R@@UAA@XZ` at **70** bytes —
    /// past [`MAX_NAME_TO_OFFSET`], hence a refusal. Both halves are asserted:
    /// a test that only showed the widened reader binding correctly would pass
    /// just as happily if the incumbent had bound it correctly too, and then it
    /// would be evidence of nothing.
    ///
    /// # REVISED by board #232 — and this revision is the finding
    ///
    /// This test used to assert that both records **bind**. That assertion was
    /// the wrong-bytes emit: a `26`-introduced *defined* record is a symbol c2
    /// gives its **own `.text` COMDAT** even in packed mode, and the port's
    /// packed writer has one `.text` for the whole TU. `scripts/expr_sweep.sh`
    /// found it at `checked=14484 mismatches=1`, on
    /// `struct Bd{Bd();~Bd();int b0;}; struct M:Bd{M();}; struct D:M{D();};
    /// D::D(){}` — where `??1M@@QAA@XZ`, the *implicitly generated* destructor,
    /// is the `26`-introduced name and the one c2 puts in its own COMDAT.
    ///
    /// So the reader half of W-ADOPT stands and the gate half did not: **seeing
    /// a name and being able to emit a body under it are different claims**, and
    /// this test conflated them. It now asserts the split directly — the widened
    /// *scanner* sees the name (which is what W-ADOPT bought, and it is what
    /// makes the record accountable rather than invisible) and the *gate*
    /// refuses the record.
    ///
    /// `il_gl_sep26.cpp` could not have caught this and the reason is the
    /// transferable part: its `??_GR` is a **deleting** destructor beside a
    /// vftable, so its whole TU is out of class for four other reasons and the
    /// verdict it asserts — `NotImplemented` — is right for the wrong cause. The
    /// axis it holds fixed is *implicit vs explicit*, and the defect is on that
    /// axis. `w232_gl_sep26_implicit.cpp` varies it (the prefix is the rung
    /// registry's doing, not a claim that it is a different family).
    #[test]
    fn a_26_introduced_record_is_SEEN_by_the_scanner_and_REFUSED_by_the_gate() {
        let mut gl = vec![0u8];
        gl.extend_from_slice(b"??1R@@UAA@XZ");
        gl.push(0);
        gl.extend_from_slice(&[0x82, 0x07, 0x05, 0x00, 0x20, 0x20, 0x03, 0x00, 0x00, 0x03, 0x00]);
        gl.extend_from_slice(&[0x80, 0x05, 0x10, 0x00, 0x00, 0x00, 0x00]);
        gl.push(0x80);
        gl.extend_from_slice(&2644u32.to_le_bytes());
        gl.extend_from_slice(&[0x00, 0x15, 0xc8, 0x18, 0x01, 0x01, 0xec, 0x09, 0x01, 0x0e, 0xef, 0x09]);
        gl.push(0x26);
        gl.extend_from_slice(b"??_GR@@UAAPAXI@Z");
        gl.push(0);
        gl.extend_from_slice(&[0x86, 0x03, 0x05, 0x04, 0x20, 0x20, 0x01, 0x00, 0x00]);
        gl.extend_from_slice(&[0x80, 0x0f, 0x10, 0x00, 0x00, 0x00, 0x00]);
        gl.push(0x80);
        gl.extend_from_slice(&2769u32.to_le_bytes());

        // The control, still executable: the NUL-only reader cannot name the
        // second record and refuses the whole TU.
        assert_eq!(
            gl_defined_names_with(&gl, false),
            (Vec::new(), Vec::new()),
            "control: the incumbent must REFUSE this shape — if it binds, this \
             test is not measuring the separator"
        );

        // …and the distance, not the name, is why. Stated separately so a future
        // change to `MAX_NAME_TO_OFFSET` cannot quietly turn the control green.
        let nul = gl_symbol_runs(&gl);
        assert!(
            nul.iter().all(|(_, _, n)| n != "??_GR@@UAAPAXI@Z"),
            "control: the name must be invisible to the NUL scanner: {nul:?}"
        );

        // **What W-ADOPT bought, and it still holds: the SCANNER sees the name.**
        // That is the half that made the record accountable instead of invisible
        // — before it, the second record's nearest run was 70 bytes back and the
        // only thing between that and a body emitted under `??1R`'s name was a
        // distance bound.
        let all = gl_symbol_runs_all_separators(&gl);
        assert!(
            all.iter().any(|(_, _, n)| n == "??_GR@@UAAPAXI@Z"),
            "the widened scanner must see the `26`-introduced name: {all:?}"
        );
        assert!(
            all.iter().any(|(_, _, n)| n == "??1R@@UAA@XZ"),
            "…and must not lose the NUL-introduced one: {all:?}"
        );

        // **What board #232 took back: the GATE refuses it.** A `26`-introduced
        // DEFINED record is a symbol c2 gives its own `.text` COMDAT even in
        // packed mode, and the port's packed writer has one `.text`. Binding it
        // is what produced `Port=Mismatch @ offset 2` — `NumberOfSections`.
        assert_eq!(
            gl_defined_names_with(&gl, true),
            (Vec::new(), Vec::new()),
            "a `26`-introduced DEFINED record must refuse the TU (board #232)"
        );

        // …and the refusal keys on the SEPARATOR and on nothing else about the
        // record: the same bytes with a `00` in that one position bind normally.
        // Without this the assertion above would pass just as well if the reader
        // had started refusing everything, which is how a refusal test goes
        // green while measuring nothing.
        let mut nul_sep = gl.clone();
        let at = nul_sep
            .iter()
            .position(|&b| b == 0x26)
            .expect("the separator under test");
        nul_sep[at] = 0x00;
        assert_eq!(
            gl_defined_names_with(&nul_sep, true),
            (
                vec![
                    (2644, "??1R@@UAA@XZ".to_string()),
                    (2769, "??_GR@@UAAPAXI@Z".to_string()),
                ],
                Vec::new()
            ),
            "with a `00` separator each record binds to its OWN name — so the \
             refusal above is the separator's doing and not the record's"
        );

        // The production path is the widened one.
        assert_eq!(gl_defined_names(&gl), gl_defined_names_with(&gl, true));
    }

    /// A bound name whose run ends at `26` refuses, because every field this
    /// reader takes after a name is at a fixed displacement from a **NUL**
    /// terminator — `linkage_needs_a_directive` reads `name_nul + 3`.
    ///
    /// This is the one shape the widening could have turned into wrong bytes
    /// rather than a refusal: terminating runs at `26` (which
    /// `gl_symbol_runs_all_separators` must do, or the name is still lost)
    /// *shortens* a run that previously swallowed the separator, so a record's
    /// bound name can change without any record moving.
    #[test]
    fn a_record_name_terminated_by_26_refuses_rather_than_reading_past_it() {
        let mut gl = vec![0u8];
        gl.extend_from_slice(b"?w_add@@YAHH@Z");
        gl.push(0x26); // …not the NUL the field arithmetic is measured against.
        gl.extend_from_slice(&[0x86, 0x01, 0x05, 0x04, 0x00, 0x00, 0x00]);
        gl.extend_from_slice(&[0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00]);
        gl.push(0x80);
        gl.extend_from_slice(&2644u32.to_le_bytes());

        // Seen — the run is found, and at a distance the bound accepts. So the
        // refusal below is the terminator check doing it, not the distance.
        let runs = gl_symbol_runs_all_separators(&gl);
        assert!(
            runs.iter().any(|(_, _, n)| n == "?w_add@@YAHH@Z"),
            "the widened scan must see the run: {runs:?}"
        );
        assert_eq!(gl_defined_names(&gl), (Vec::new(), Vec::new()));

        // Mirror: the identical record with a NUL terminator binds. Without this
        // the test above would pass on a reader that refused everything.
        let mut ok = vec![0u8];
        ok.extend_from_slice(b"?w_add@@YAHH@Z");
        ok.push(0x00);
        ok.extend_from_slice(&[0x86, 0x01, 0x05, 0x04, 0x00, 0x00, 0x00]);
        ok.extend_from_slice(&[0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00]);
        ok.push(0x80);
        ok.extend_from_slice(&2644u32.to_le_bytes());
        assert_eq!(
            gl_defined_names(&ok),
            (vec![(2644, "?w_add@@YAHH@Z".to_string())], Vec::new())
        );
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

    /// **Board #1110 — the WIDE tag's alignment table, pinned.**
    ///
    /// Every row is a frozen cell of lane `w-align`'s grid whose alignment was
    /// read off **c2's own obj** at the workload's `/GR /O1 /Oi /EHsc`; the
    /// cell name is the witness and `docs/rungs/2026-08-08-w-align.md` §2 is
    /// the table. This test exists because the arm is one line and the cells
    /// are the whole job: an alignment nibble guessed wrong is a wrong
    /// `Characteristics` word, and nothing in the portable lane would see it.
    #[test]
    fn the_wide_type_tag_reads_the_same_width_as_the_narrow_one() {
        // Narrow, unchanged — the incumbent's four.
        assert_eq!(align_of_type_tag(0x82, None), Some(1), "T12 char");
        assert_eq!(align_of_type_tag(0x84, None), Some(2));
        assert_eq!(align_of_type_tag(0x86, None), Some(4), "T10 two ints");
        assert_eq!(align_of_type_tag(0x88, None), Some(8), "T11 double");
        // Wide: TAG_WIDE is orthogonal to the width field.
        assert_eq!(align_of_type_tag(0xC2, Some(0x81)), Some(1));
        assert_eq!(align_of_type_tag(0xC4, Some(0x81)), Some(2));
        assert_eq!(align_of_type_tag(0xC6, Some(0x81)), Some(4), "G01 poly+int");
        assert_eq!(align_of_type_tag(0xC8, Some(0x81)), Some(8), "T16 declspec(8)");
        // **Board #1120, lane `w-align16`.** `8A`/`CA` is 16 and the writer can
        // now express it. `CA` is the form every 16-aligned cell actually
        // spells; bare `8A` is accepted by the ORTHOGONALITY rule and has zero
        // witnesses — see the doc comment and `work/w-align16/tagcensus.txt`,
        // which finds **0** `8A` records in 85,895 across all 878 workload TUs.
        assert_eq!(align_of_type_tag(0x8A, None), Some(16), "orthogonality, no witness");
        assert_eq!(align_of_type_tag(0xCA, Some(0x81)), Some(16), "A02 declspec(16)");
        // **32 and 64 EXIST and stay refused.** `A09`/`A10` are
        // `__declspec(align(32))` / `align(64)` and c2 gives them nibbles 6 and
        // 7 — so this is a deliberate stop, not an unexplored edge. The grid
        // varies structure nine ways at 16 and not at all at 32/64.
        assert_eq!(align_of_type_tag(0x8C, None), None, "A09 align(32) — measured, refused");
        assert_eq!(align_of_type_tag(0xCC, Some(0x81)), None, "A09 align(32)");
        assert_eq!(align_of_type_tag(0xCE, Some(0x81)), None, "A10 align(64)");
        // The mark's VALUE is matched: `IL_TYPE_WIDE_TAG.md` §8 item 2 records
        // `84` as a second value in `.ex`, and no cell says what the width
        // means under it.
        assert_eq!(align_of_type_tag(0xC6, Some(0x84)), None, "unmeasured mark");
        assert_eq!(align_of_type_tag(0xC6, Some(0xFF)), None);
    }
}
