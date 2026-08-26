//! **W-ALIAS — the `.gl` tag-0x10 ALIAS record.**
//!
//! > **PROVENANCE — DISASSEMBLY-DERIVED.** The record grammar and the two bit
//! > positions below are transcribed from `c2.dll` 16.00.11886.00 (image base
//! > `0x10b00000`) and are **disclosed** in
//! > [`docs/whitebox/DISCLOSURE.md`](../../../../docs/whitebox/DISCLOSURE.md),
//! > rows **W-ALIAS-1** and **W-ALIAS-2**. They are the first adopted rows in
//! > that ledger. Everything else in this file is ordinary decode.
//!
//! # What this record is, and why six lanes did not have it
//!
//! Every model of c2's emit set built in this project has been a closure over
//! `U`, the gate-clean **tag-0x0E** `.gl` records — the ones that carry an `.ex`
//! body. The `.gl` stream also carries **tag-0x10** records. They have **no
//! body**, so no lane's `U` ever contained one, and in the same word that a
//! tag-0x0E record uses for its emit flags they carry a **token naming another
//! symbol**.
//!
//! **A vftable's initializer names the ALIAS; the symbol c2 emits is the alias's
//! TARGET.** `??_7FilePath@@6B@`'s initializer nodes name
//! `??_EFilePath@@UAAPAXI@Z` — a tag-0x10 record with no body — and the obj
//! emits `??_GFilePath@@UAAPAXI@Z`. That is the `??_G`/`??_E` deleting-destructor
//! class, **58.95 % of the residual** of the best previous model, which every
//! prior lane recorded as structurally unreachable on the grounds that c2
//! *synthesizes* those destructors so no initializer node could name them. The
//! initializer names the alias instead.
//!
//! # The grammar
//!
//! The `.gl` tag dispatch at `0x10b9b91f` sends tags `0x04` / `0x0E` / `0x10` to
//! one shared kind-4 handler at `0x10b9bdcf`, which splits on the tag only at
//! the very end:
//!
//! ```text
//!   10b9bf46  cmp  DWORD PTR [ebp-0x78],0xe      ; tag == 0x0E ?
//!   10b9bf4a  jne  0x10b9c01e
//!   10b9bf50  or   DWORD PTR [esi+0x37],0x200000 ; "has a body" = in U
//!   10b9bf57  call 0x10c1f9e9                    ; i32c -> +0x54  the .ex body
//!   10b9bf70  call 0x10c1f91b                    ; varU -> +0x4c  the MARK word
//! ---------------------------------------------------------------------------
//!   10b9c01e  cmp  DWORD PTR [ebp-0x78],0x10     ; tag == 0x10 ?
//!   10b9c022  jne  0x10b9c033
//!   10b9c024  or   DWORD PTR [esi+0x37],0x400000 ; THE ALIAS BIT
//!   10b9c02b  call 0x10c1f91b                    ; varU
//!   10b9c030  mov  DWORD PTR [esi+0x4c],eax      ; THE ALIAS TARGET TOKEN
//! ```
//!
//! So a tag-0x10 record is everything a tag-0x0E record has up to the `+0x54`
//! anchor, then **one `varU` and nothing else** — `0x10b9c033` falls straight
//! into the shared tail — and at that anchor `[sym+0x4c]` is **not** a flag word,
//! it is a **symbol token**.
//!
//! The reader of the bit is `0x10b99621`, which resolves that token and sets
//! `+0x20 |= 0x2000` at `0x10b99635` **on the target**.
//!
//! # The gate — and why it is NOT the terminus gate
//!
//! w-refs' reference-list decode asks that a record end exactly where the next
//! record's header begins. **That gate does not apply here and using it would
//! grade the neighbour**: it fails on 320 of 419 tag-0x10 records in
//! `src/App.cpp`, not because the field is wrong — every one of the 320 still
//! decodes to a `??_E<X>` → `??_G<X>` pair — but because the record *following*
//! an alias is usually a tag-0x0B undecorated-name record, whose header is not
//! the `<tag><varU><sep>` shape that gate models.
//!
//! The gate here is on the field itself, and it is two conditions:
//!
//! * **RT** — [`super::readers::read_token_var`] and [`var_u`] must consume the
//!   same number of bytes at the same position.
//! * **BIND** — the token must resolve in [`super::gl::gl_symbol_index`].
//!
//! Neither knows anything about `??_E` or `??_G`, which is what keeps the
//! measured shape a *result* rather than the gate restating itself.
//!
//! # The null, and it is shipped rather than described
//!
//! [`gl_alias_table_shifted`] takes the same read at `p−1` and `p+1`. Over 850
//! TUs those bind at 0.019 and 0.026 of the real read — and produce **zero**
//! `??_E` → `??_G` pairs. The count null is 40×; the shape null is infinite. It
//! is a public function because a field position claimed without its null is a
//! field position that was searched for rather than identified.
//!
//! # What a consumer must and must not do
//!
//! 1. **Apply the resolution ONCE, at the `in`-stream `02`-node resolution site
//!    only.** Not transitively: an alias never targets an alias
//!    (`dom(alias) ∩ U = 0`, and every bound target bar 2 of 95 820 is in `U`).
//! 2. **Do NOT apply it to the `.gl` reference list.** Measured: the model with
//!    the alias applied to the reference list is the alias-free incumbent **to
//!    the digit, `|P|` included**. `0x10b27f3c` keeps an edge only for a
//!    tag-0x0E target, so the same table worth +321 TUs through the `in` channel
//!    is worth **exactly zero** through the reference list.
//! 3. **Never emit a symbol whose name is in `dom(alias)`** — but see
//!    [`GlAliasStats::dom_with_body`] first, which is the count that makes that
//!    rule safe to apply and is why the rule is not hard-coded in here.
//!
//! **There is no consumer in `crates/` today.** `PortC2` has no emit-set model,
//! so this module is an input to Phase 7 and changes no obj byte. That is stated
//! rather than implied: the reader is additive, and if any obj number moves when
//! it lands, the reader is not additive and that is an alarm.

use std::collections::BTreeMap;

use super::gl::gl_symbol_index;
use super::readers::read_token_var;

/// The record tag this module reads. **Disclosed** — `0x10b9c01e`.
/// PROV[R] DISCLOSURE `W-ALIAS-1` — `0x10b9c01e`, the tag test in the shared kind-4 handler. Confirmed by lane `w-emitp` (15/15 interventional draws, 0/15 parity control) and reproduced by two implementations agreeing on 850 TUs; the black-box alternative was tried first and binds at 0.019/0.026 one byte either side, so the position is identified by the read and only GRADED by the corpus.
const ALIAS_TAG: u8 = 0x10;

/// The tags the shared kind-4 handler at `0x10b9bdcf` serves. A run's tag is
/// located by looking for one of these immediately before the operand token.
/// PROV[O] the three tags the shared kind-4 handler serves, named at `0x10b9bdcf` in the disassembly beside DISCLOSURE `W-ALIAS-1`'s own read at `0x10b9c01e`. [O] rather than [R] because the ADDRESS is what the doc cites for the handler, while the tag VALUES are located in the `.gl` byte stream; a lane that reads the handler's own comparisons can promote this to [R].
const KIND4_TAGS: [u8; 3] = [0x04, 0x0E, 0x10];

/// The two-byte-prefixed record kinds, which put their tag one byte further
/// back. Same locator as the `.gl` owner scan.
/// PROV[O] the two-byte-prefixed record kinds, located by the same `.gl` owner scan. Same promotion note as [`KIND4_TAGS`].
const KIND1_TAGS: [u8; 2] = [0x01, 0x02];

/// The bytes that delimit a `.gl` name run. Same enumerated pair
/// [`super::gl`] uses, and for the same measured reason.
/// PROV[F] the same enumerated delimiter SET as `gl::NAME_SEPARATORS` minus `25`, and [F] for the same reason: a separator byte outside the measured pair is the off-sample case.
const NAME_SEPARATORS: [u8; 2] = [0x00, 0x26];

// ---------------------------------------------------------------- primitives
//
// The four `.gl` scalar encodings, each named by the c2 routine that writes it.
// They are **navigation, not adoption**: the same four encodings are already
// re-derived from black-box IL in `super::readers`, and these are local copies
// only because the record walk below needs them at `.gl` positions rather than
// `.ex` ones. Every one fails closed on a truncated stream.

/// `0x10c1f8fc` — one raw byte.
#[inline]
fn get_byte(b: &[u8], p: usize) -> Option<(u8, usize)> {
    Some((*b.get(p)?, p + 1))
}

/// `0x10c1f91b` — the variable-width unsigned the flag word and the alias
/// target token are both written as. Two bytes unless the second carries `0x80`.
#[inline]
fn var_u(b: &[u8], p: usize) -> Option<(u32, usize)> {
    let b0 = *b.get(p)? as u32;
    let b1 = *b.get(p + 1)? as u32;
    if b1 & 0x80 == 0 {
        return Some((b0 | (b1 << 8), p + 2));
    }
    let b2 = *b.get(p + 2)? as u32;
    let b3 = *b.get(p + 3)? as u32;
    let lo = b0 | ((b1 & 0x7F) << 8);
    let hi = ((b2 << 16) | (b3 << 24)) >> 1;
    Some((lo | hi, p + 4))
}

/// `0x10c1f9a6` — a signed byte, or the `0x80` escape then a little-endian 16.
#[inline]
fn i16c(b: &[u8], p: usize) -> Option<(i32, usize)> {
    let v = *b.get(p)?;
    if v != 0x80 {
        return Some((if v >= 0x80 { v as i32 - 256 } else { v as i32 }, p + 1));
    }
    let lo = *b.get(p + 1)? as i32;
    let hi = *b.get(p + 2)? as i32;
    Some((lo | (hi << 8), p + 3))
}

/// `0x10c1f9e9` — a signed byte, or the `0x80` escape then a little-endian 32.
#[inline]
fn i32c(b: &[u8], p: usize) -> Option<(i32, usize)> {
    let v = *b.get(p)?;
    if v != 0x80 {
        return Some((if v >= 0x80 { v as i32 - 256 } else { v as i32 }, p + 1));
    }
    let mut w: u32 = 0;
    for k in 0..4 {
        w |= (*b.get(p + 1 + k)? as u32) << (8 * k);
    }
    Some((w as i32, p + 5))
}

/// `0x10c1fae7` — a signed byte, or the `0x80` escape then eight more. Only the
/// width is needed here, so only the width is returned.
#[inline]
fn i64c_skip(b: &[u8], p: usize) -> Option<usize> {
    if *b.get(p)? != 0x80 {
        Some(p + 1)
    } else if p + 9 <= b.len() {
        Some(p + 9)
    } else {
        None
    }
}

/// `0x10c1f90a` — consume bytes while the high bit is set, then one more.
#[inline]
fn skipvar(b: &[u8], mut p: usize) -> Option<usize> {
    while *b.get(p)? & 0x80 != 0 {
        p += 1;
    }
    Some(p + 1)
}

/// `0x10c1fcef` — an `i16c` length, then that many bytes.
#[inline]
fn blob_skip(b: &[u8], p: usize) -> Option<usize> {
    let (n, p) = i16c(b, p)?;
    let q = p.checked_add(n.max(0) as usize)?;
    if q <= b.len() {
        Some(q)
    } else {
        None
    }
}

/// Walk the **shared kind-4 record header** — `0x10b9bdcf` — from the byte just
/// past a record's name terminator to the `+0x54` anchor field.
///
/// Returns `(anchor position, storage-class byte)`, or `None` on any desync.
/// This is the same walk a tag-0x0E record needs to find its `.ex` body offset;
/// the tag-0x10 arm reuses it verbatim, which is the whole reason
/// `0x10b9b91f` routes both tags to one handler.
/// PROV[R] DISCLOSURE `W-ALIAS-1` — the shared kind-4 header walk `0x10b9bdcf`,
/// and the routing `0x10b9b91f` that sends tags `0x04`/`0x0E`/`0x10` to it. A
/// RULE marker, not a constant: what is adopted here is a record grammar.
fn record_head(b: &[u8], p: usize) -> Option<(usize, u8)> {
    let n = b.len();
    let (sc, mut p) = get_byte(b, p)?; // 0x10b9be0e — the storage class
    p = i32c(b, p)?.1; //                +0x40
    p = var_u(b, p)?.1; //               +0x20 flags
    p = var_u(b, p)?.1; //               +0x0c owner, unconditional
    let (optw, q) = i32c(b, p)?;
    p = q;
    if optw & 1 != 0 {
        p = i32c(b, p)?.1;
        let (cnt, q) = i32c(b, p)?;
        p = q;
        // A count wider than the stream cannot be a count. Bail rather than
        // spin: `skipvar` would fail on the first out-of-range read anyway, but
        // an unbounded loop bound taken from the data is a hazard in itself.
        if cnt.max(0) as usize > n {
            return None;
        }
        for _ in 0..cnt.max(0) {
            p = skipvar(b, p)?;
        }
    }
    if optw & 2 != 0 {
        p = var_u(b, p)?.1;
    }
    p = i32c(b, p)?.1; // +0x2c type index
    p = i32c(b, p)?.1; // debug
    let (m, q) = i32c(b, p)?;
    p = q;
    if m.max(0) as usize > n {
        return None;
    }
    for _ in 0..m.max(0) {
        p = i32c(b, p)?.1;
        p = skipvar(b, p)?;
        let (k, q) = i32c(b, p)?;
        p = q;
        if k.max(0) as usize > n {
            return None;
        }
        for _ in 0..k.max(0) {
            p = i32c(b, p)?.1;
            let (c, q) = get_byte(b, p)?;
            p = if c != 0 { blob_skip(b, q)? } else { i64c_skip(b, q)? };
        }
    }
    Some((p, sc))
}

/// The character set an MSVC symbol name is spelled in. Same test
/// [`super::gl`] applies, kept local so this module's run walk cannot drift
/// from its own documented alphabet.
#[inline]
fn is_symbol_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$' | b'?' | b'@')
}

/// A separator-delimited `.gl` run that could **be** a symbol name.
struct Run {
    /// First byte of the name.
    start: usize,
    /// Index of the separator that terminates it.
    end: usize,
    name: String,
}

/// Every separator-delimited run in `.gl` that passes the name test, in file
/// order.
///
/// Deliberately **not** [`super::gl::gl_symbol_index`]'s walk, which scans
/// printable runs and takes the *rightmost* separator-preceded start inside
/// one. This walk is anchored on the separators themselves, because a tag-0x10
/// record is located by the tag byte sitting a token's width behind its name and
/// that offset is only defined against the run's true start.
fn indexable_runs(gl: &[u8]) -> Vec<Run> {
    let n = gl.len();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < n {
        if !NAME_SEPARATORS.contains(&gl[i]) {
            i += 1;
            continue;
        }
        let s = i + 1;
        let mut e = s;
        while e < n && !NAME_SEPARATORS.contains(&gl[e]) {
            e += 1;
        }
        if e >= n || e == s {
            i += 1;
            continue;
        }
        let b = &gl[s..e];
        let head_ok = b[0] == b'?' || b[0].is_ascii_alphabetic() || b[0] == b'_';
        if b.len() >= 3 && head_ok && b.iter().all(|&c| is_symbol_char(c)) {
            out.push(Run {
                start: s,
                end: e,
                name: b.iter().map(|&c| c as char).collect(),
            });
            i = e;
        } else {
            i += 1;
        }
    }
    out
}

/// The record tag for the run whose name starts at `s`, with the record's own
/// operand token.
///
/// The separator at `s − 1` is the record's `+0x31` byte and the operand token
/// ends there, so the tag sits one token-width further back — or two, for the
/// [`KIND1_TAGS`] records that carry an extra byte.
fn tag_at(gl: &[u8], s: usize) -> Option<(u8, u32)> {
    for w in [4usize, 2] {
        if s < 1 + w + 1 {
            continue;
        }
        let p = s - 1 - w;
        let Some((tok, got)) = read_token_var(gl, p) else {
            continue;
        };
        if got != w {
            continue;
        }
        if KIND4_TAGS.contains(&gl[p - 1]) {
            return Some((gl[p - 1], tok));
        }
        if p >= 2 && KIND1_TAGS.contains(&gl[p - 2]) {
            return Some((gl[p - 2], tok));
        }
    }
    None
}

/// Read the alias target field at `p`: the **RT** gate, then the **BIND** gate.
///
/// RT is that [`read_token_var`] and [`var_u`] consume the same bytes. The two
/// readers take their width from the same bit of the same byte, so RT is in
/// practice a bounds check — measured `rt_fail 0` over 96 220 records — and it
/// is kept because a future width rule that split them must fail here rather
/// than silently disagree.
fn read_target(
    gl: &[u8],
    idx: &BTreeMap<u32, String>,
    p: usize,
) -> Option<(u32, Option<String>)> {
    let (tok, w) = read_token_var(gl, p)?;
    let (_raw, q) = var_u(gl, p)?;
    if q - p != w {
        return None;
    }
    Some((tok, idx.get(&tok).cloned()))
}

/// Everything the tag-0x10 scan counted, published beside the table.
///
/// **Every field is a count, never a status.** A decode reported as "ok" is a
/// decode nobody can grade (`docs/STATUS.md` trap 5).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GlAliasStats {
    /// Separator-delimited name runs examined.
    pub runs: usize,
    /// Runs whose record tag is `0x10`.
    pub tag10: usize,
    /// …of which the shared kind-4 header walk desynced before the anchor.
    pub head_fail: usize,
    /// …of which the two token readers disagreed on the field's width.
    pub rt_fail: usize,
    /// …of which the target token does not resolve in the `.gl` symbol index.
    pub unbound_target: usize,
    /// …of which the record aliases itself.
    pub self_alias: usize,
    /// …of which a second record gave the same name a *different* target. Both
    /// are dropped: an alias two records disagree about resolves to nothing,
    /// never to the first, for the same reason a token two names claim is
    /// dropped from the symbol index.
    pub dup: usize,
    /// Entries in the resulting table.
    pub bound: usize,
    /// Bound entries whose shape is `??_E<X>` → `??_G<X>`.
    ///
    /// **This is a result, not a gate.** Nothing in the RT/BIND gate mentions
    /// `??_E` or `??_G`; the shifted null passes the same gate and produces
    /// none of these.
    pub shape_e_to_g: usize,
    /// **Aliases whose own name ALSO carries a tag-0x0E record — i.e. has a
    /// body.**
    ///
    /// A **superset** of `dom(alias) ∩ U`, deliberately: `U` additionally
    /// requires the record's `80 <LE32>` field to land on a real `.ex` body
    /// start, and this count does not consult `.ex` at all. The superset is the
    /// safe direction for a check whose whole job is to refuse.
    ///
    /// This is the count that makes w-emitp §6 rule 4 ("never emit a name in
    /// `dom(alias)`") safe, and it is the reason the rule is **not** hard-coded
    /// into this module. Measured **0** over 850 TUs and 96 220 records; if it
    /// were ever nonzero, that rule would suppress a symbol that has a body and
    /// must be emitted, which is a wrong emit rather than a gap. A consumer
    /// applies the rule only with this number in hand.
    pub dom_with_body: usize,
}

/// `alias: Token → Token`, and the same relation by name.
///
/// Built by [`gl_alias_table`]. See the module docs for what a consumer may and
/// may not do with it — in particular that it is applied **once**, at the
/// `in`-stream `02`-node resolution site, and **never** to the `.gl` reference
/// list.
#[derive(Clone, Debug, Default)]
pub struct GlAliasTable {
    by_name: BTreeMap<String, String>,
    by_token: BTreeMap<u32, u32>,
    stats: GlAliasStats,
}

impl GlAliasTable {
    /// The decode's counts.
    pub fn stats(&self) -> &GlAliasStats {
        &self.stats
    }

    /// How many aliases bound.
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// Whether the table is empty. A TU with no vftable has no aliases and that
    /// is ordinary, not a failure.
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// `dom(alias) ∋ name`? **A name for which this is true is one c2 never
    /// emits** — 0 of 174 417 emitted names over 850 TUs, and 0 of 15
    /// interventional draws through the real `c2.dll`.
    ///
    /// Read [`GlAliasStats::dom_with_body`] before using this to suppress an
    /// emit.
    pub fn is_alias(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    /// Resolve one name through the table, **once**.
    ///
    /// Not transitive by construction, and that is the measured shape rather
    /// than an optimization: an alias has no body, every bound target bar 2 of
    /// 95 820 does, so an alias never targets an alias.
    pub fn resolve_name<'a>(&'a self, name: &'a str) -> &'a str {
        self.by_name.get(name).map(String::as_str).unwrap_or(name)
    }

    /// Resolve one operand token through the table, once. Tokens that are not
    /// aliases pass through unchanged.
    pub fn resolve_token(&self, tok: u32) -> u32 {
        self.by_token.get(&tok).copied().unwrap_or(tok)
    }

    /// The relation, by name, in name order.
    pub fn iter_names(&self) -> impl Iterator<Item = (&str, &str)> {
        self.by_name.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// Decode the `.gl` **tag-0x10 ALIAS** records of one translation unit.
///
/// See the module docs for the grammar, the gate, the null and the rules a
/// consumer is bound by.
pub fn gl_alias_table(gl: &[u8]) -> GlAliasTable {
    gl_alias_table_shifted(gl, 0)
}

/// [`gl_alias_table`] with the target field read `shift` bytes off its decoded
/// position — **the null control, and it is shipped rather than described.**
///
/// A field position claimed without a null is a field position that was
/// searched for. Over 850 TUs, `shift = −1` binds 1 795 targets and `shift =
/// +1` binds 2 449 — 0.019 and 0.026 of the real read — and **both produce zero
/// `??_E<X>` → `??_G<X>` pairs**. The count null is 40×; the shape null is
/// infinite. Callers other than a measurement should pass `0`, which is what
/// [`gl_alias_table`] does.
pub fn gl_alias_table_shifted(gl: &[u8], shift: i32) -> GlAliasTable {
    let runs = indexable_runs(gl);
    let idx = gl_symbol_index(gl);
    let mut st = GlAliasStats {
        runs: runs.len(),
        ..Default::default()
    };
    let mut by_name: BTreeMap<String, String> = BTreeMap::new();
    let mut by_token: BTreeMap<u32, u32> = BTreeMap::new();

    for run in &runs {
        let Some((tag, tok)) = tag_at(gl, run.start) else {
            continue;
        };
        if tag != ALIAS_TAG {
            continue;
        }
        st.tag10 += 1;
        let Some((anchor, _sc)) = record_head(gl, run.end + 1) else {
            st.head_fail += 1;
            continue;
        };
        let Some(p) = shifted(anchor, shift) else {
            st.rt_fail += 1;
            continue;
        };
        let Some((ttok, target)) = read_target(gl, &idx, p) else {
            st.rt_fail += 1;
            continue;
        };
        let Some(target) = target else {
            st.unbound_target += 1;
            continue;
        };
        if target == run.name {
            st.self_alias += 1;
            continue;
        }
        if let Some(prev) = by_name.get(&run.name) {
            if *prev != target {
                st.dup += 1;
                continue;
            }
        }
        by_name.insert(run.name.clone(), target);
        by_token.insert(tok, ttok);
    }

    st.bound = by_name.len();
    st.shape_e_to_g = by_name
        .iter()
        .filter(|(k, v)| {
            k.starts_with("??_E") && v.starts_with("??_G") && k[4..] == v[4..]
        })
        .count();
    // An alias whose own name also carries a tag-0x0E record. Computed from the
    // tag byte directly rather than from a `U` the caller would have to build,
    // so the invariant travels with the table — at the price of being a
    // superset of `dom(alias) ∩ U`, which is the direction that fails closed.
    let bodied: std::collections::BTreeSet<&str> = runs
        .iter()
        .filter(|r| matches!(tag_at(gl, r.start), Some((0x0E, _))))
        .map(|r| r.name.as_str())
        .collect();
    st.dom_with_body = by_name.keys().filter(|k| bodied.contains(k.as_str())).count();

    GlAliasTable {
        by_name,
        by_token,
        stats: st,
    }
}

/// `anchor + shift`, refusing to wrap past the start of the stream.
#[inline]
fn shifted(anchor: usize, shift: i32) -> Option<usize> {
    if shift >= 0 {
        anchor.checked_add(shift as usize)
    } else {
        anchor.checked_sub((-shift) as usize)
    }
}

#[cfg(test)]
mod tests;
