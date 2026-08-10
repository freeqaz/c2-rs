//! The `/EHsc` **scope-object** TU recognizer — one function, one destructible
//! local, one call while it is live.
//!
//! `src/Main.cpp` of the dc3 workload is
//! `int main(int argc, char **argv) { App app(argc, argv); app.Run(); }`, and
//! its obj is two code regions, two `.pdata` COMDATs and a 64-byte EH `.rdata`
//! (`docs/whitebox/WB_EH_FINDINGS.md`, `docs/EH_RECORDS.md`). None of that is
//! expressible per function, so — exactly as [`super::bundle::DynInitTu`] does
//! for the `??__E` shape — this is a **whole-TU** recognizer and the emitter is
//! `c2_core::coff::emit_eh_scope_obj`.
//!
//! # What this is, stated plainly
//!
//! **A transcription of one statement grammar, with holes.** It is not a
//! statement-layer parser and it does not widen one: `body::mcall`'s
//! `eat_dtor_stmt_trailer` still refuses this body at `op-0x5C`, the census is
//! unchanged, and every per-function instrument reads this TU exactly as it did
//! before. The template below is read off two independent captures — the target
//! TU and `work/w-main2/probe/m0.cpp`, which differ in every token, every type
//! index and in whether the statements carry line markers at all — and anything
//! that does not match it, byte for byte outside the holes, refuses.
//!
//! The holes are the part that is IL and not template: the four callee tokens,
//! the object's token and CodeView type index, and the two formals. Every one is
//! resolved through the ordinary binding, and the object's `sizeof` — which sets
//! the frame size and therefore one `stwu` immediate — is read out of `.db`.

use super::readers::read_token_var;

/// One TU that is a `/EHsc` scope-object function, as the emitter needs it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EhScopeTuIl {
    /// The function's COFF name.
    pub name: String,
    /// The constructor, the member function called while the object is live,
    /// and the destructor — in the order the emitter calls them.
    pub ctor: String,
    pub member: String,
    pub dtor: String,
    /// `sizeof` the scope object, from `.db`'s `LF_CLASS`/`LF_STRUCTURE`.
    pub object_size: u32,
    /// How many single-register formals the function declares.
    pub formals: u32,
    /// The `.gl` compiler-label counter.
    pub label_counter: u32,
}

// ---------------------------------------------------------------------------
// The template matcher
// ---------------------------------------------------------------------------

/// A hole's captured bytes, indexed by slot.
#[derive(Default)]
struct Caps {
    /// `Run` captures — a byte run up to a delimiter.
    runs: Vec<Vec<u8>>,
    /// `Name` captures — a token read by [`read_token_var`].
    names: Vec<u32>,
    /// `TypeIdx` captures — a CodeView type index from a `80 <LE32>` field.
    types: Vec<u32>,
}

struct M<'a> {
    b: &'a [u8],
    p: usize,
    c: Caps,
}

/// The longest a receiver/type run may be before this is not the shape that was
/// measured. Both captures give 5 and 4; 8 is the refusal boundary.
const MAX_RUN: usize = 8;

impl<'a> M<'a> {
    fn lit(&mut self, want: &[u8]) -> Option<()> {
        if self.b.get(self.p..self.p + want.len())? != want {
            return None;
        }
        self.p += want.len();
        Some(())
    }
    fn name(&mut self) -> Option<u32> {
        let (t, w) = read_token_var(self.b, self.p)?;
        self.p += w;
        self.c.names.push(t);
        Some(t)
    }
    /// `80 <LE32>` — a CodeView type index. Spelled out rather than read through
    /// [`read_token_var`], which would take `80 06` as a two-byte token and
    /// leave the parse standing on the index's own tail.
    fn type_idx(&mut self) -> Option<u32> {
        if *self.b.get(self.p)? != 0x80 {
            return None;
        }
        let v = u32::from_le_bytes([
            *self.b.get(self.p + 1)?,
            *self.b.get(self.p + 2)?,
            *self.b.get(self.p + 3)?,
            *self.b.get(self.p + 4)?,
        ]);
        self.p += 5;
        self.c.types.push(v);
        Some(v)
    }
    /// The bytes from here up to — not including — the next `delim`, capped at
    /// [`MAX_RUN`] and required to start with `first`.
    fn run(&mut self, first: u8, delim: u8) -> Option<usize> {
        if *self.b.get(self.p)? != first {
            return None;
        }
        let end = self.b[self.p..].iter().position(|&x| x == delim)? + self.p;
        if end == self.p || end - self.p > MAX_RUN {
            return None;
        }
        self.c.runs.push(self.b[self.p..end].to_vec());
        self.p = end;
        Some(self.c.runs.len() - 1)
    }
    /// The run captured in `slot`, again, byte for byte.
    fn same(&mut self, slot: usize) -> Option<()> {
        let want = self.c.runs.get(slot)?.clone();
        self.lit(&want)
    }
    /// An OPTIONAL `4F 01 <line>` marker. `m0` puts the whole function on one
    /// source line and carries none of these; the workload TU carries four.
    fn opt_line(&mut self) {
        while self.b.get(self.p..self.p + 2) == Some(&[0x4F, 0x01]) {
            let n = match self.b.get(self.p + 2) {
                Some(&v) if v < 0x80 => 3,
                Some(&0x80) => 7,
                _ => return,
            };
            if self.b.len() < self.p + n {
                return;
            }
            self.p += n;
        }
    }
    /// One `name(this, …)` call statement: the `26` pair, the `2C` spine, the
    /// receiver run, and whatever the trailer is.
    fn call_head(&mut self, obj: u32) -> Option<(u32, usize, usize)> {
        self.lit(&[0x26])?;
        let callee = self.name()?;
        self.lit(&[0x26])?;
        if self.name()? != obj {
            return None;
        }
        self.lit(&[0x2C])?;
        let recv = self.run(0xA6, 0x99)?;
        self.lit(&[0x99])?;
        let ty = self.run(0x86, 0xBD)?;
        self.lit(&[0xBD])?;
        Some((callee, recv, ty))
    }
}

/// Match the statement template. Returns `(fn, ctor, member, dtor, object,
/// formals, object type index)`.
///
/// The template, with `<…>` for the holes, exactly as
/// `work/w-main2/tpl.py` prints it from a real capture:
///
/// ```text
///   53 53 26 <fn> 46 2d <f0> 2d <f1> 4c 4f 11 53
///   26 <ctor> 26 <obj> 2c <recv> 99 <ty> bd <recv> <T:obj>
///        b9 <f0> <g0> 55 <g0>  b9 <f1> <g1> 55 <g1> 4c
///   26 <dtor> 26 <obj> 2c <recv> 99 <ty2> bd 82 07 03 00 <T:objref> 4c
///   5c <recv−1> 01 4b
///   26 <mem>  26 <obj> 2c <recv> 99 <ty2> bd 82 07 03 00 <T:objref> 4c 4b
///   5e 01 21 4b
///   54 02 29 <exit>
///   4f 12 47 54 01 54 00 4f 02 20 00 4d
/// ```
///
/// Three things in it are load-bearing and worth naming, because each is a
/// refusal for a shape that would otherwise look the same:
///
/// * the **`26 <dtor>` statement sits between the constructor and the member
///   call** and emits no code — it is the unwind ACTION, which is why the
///   destructor appears twice in the obj and only once here;
/// * **`5c … 01 4b`** is the state-1 transition, and the `01` is `maxState`.
///   Any other value is a state map this emitter does not build;
/// * **`5e 01 21 4b`** is the scope end — one sub-object — and it is what puts
///   the *second* `bl` to the destructor on the normal path.
#[allow(clippy::type_complexity)]
fn match_template(body: &[u8]) -> Option<(u32, u32, u32, u32, u32, [u32; 2], u32)> {
    let mut m = M { b: body, p: 0, c: Caps::default() };
    m.lit(&[0x53, 0x53])?;
    m.lit(&[0x26])?;
    let f = m.name()?;
    m.lit(&[0x46])?;
    m.lit(&[0x2D])?;
    let f0 = m.name()?;
    m.lit(&[0x2D])?;
    let f1 = m.name()?;
    m.lit(&[0x4C])?;
    m.lit(&[0x4F, 0x11, 0x53])?;
    m.opt_line();

    // ---- the constructor, with the function's own formals shifted up -------
    m.lit(&[0x26])?;
    let ctor = m.name()?;
    m.lit(&[0x26])?;
    let obj = m.name()?;
    m.lit(&[0x2C])?;
    let recv = m.run(0xA6, 0x99)?;
    m.lit(&[0x99])?;
    m.run(0x86, 0xBD)?;
    m.lit(&[0xBD])?;
    m.same(recv)?;
    let t_obj = m.type_idx()?;
    // Each formal, once as the value and once as its own type group, the two
    // separated by `55`. The pair must be byte-identical: an argument whose two
    // halves disagree is a conversion, which this class has none of.
    for want in [f0, f1] {
        m.lit(&[0xB9])?;
        if m.name()? != want {
            return None;
        }
        let g = m.run(0x86, 0x55)?;
        m.lit(&[0x55])?;
        m.same(g)?;
    }
    m.lit(&[0x4C])?;

    // ---- the unwind ACTION: the destructor, registered and not called ------
    let (dtor, recv2, ty2) = m.call_head(obj)?;
    if m.c.runs[recv2] != m.c.runs[recv] {
        return None;
    }
    m.lit(&[0x82, 0x07, 0x03, 0x00])?;
    m.type_idx()?;
    m.lit(&[0x4C])?;

    // ---- the state transition ---------------------------------------------
    //
    // `5C` carries the receiver run WITHOUT its trailing terminator byte, then
    // the state number. `01` and nothing else: `maxState` is the count of
    // `__unwindtable$` entries this emitter writes and it writes one.
    m.lit(&[0x5C])?;
    let head = m.c.runs[recv].clone();
    m.lit(&head[..head.len() - 1])?;
    m.lit(&[0x01, 0x4B])?;
    m.opt_line();

    // ---- the member call, while the object is live ------------------------
    let (member, recv3, ty3) = m.call_head(obj)?;
    if m.c.runs[recv3] != m.c.runs[recv] || m.c.runs[ty3] != m.c.runs[ty2] {
        return None;
    }
    m.lit(&[0x82, 0x07, 0x03, 0x00])?;
    m.type_idx()?;
    m.lit(&[0x4C, 0x4B])?;
    m.opt_line();

    // ---- the scope end, the exit label, the segment tail ------------------
    m.lit(&[0x5E, 0x01, 0x21, 0x4B])?;
    m.lit(&[0x54, 0x02, 0x29])?;
    m.name()?;
    m.lit(&[0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00])?;
    m.opt_line();
    m.lit(&[0x4D])?;
    if m.p != body.len() {
        return None;
    }
    Some((f, ctor, member, dtor, obj, [f0, f1], t_obj))
}

/// The statement opcodes this class accounts for. A byte from this set anywhere
/// in the segment PREFIX means a statement the template never saw.
const STATEMENT_BYTES: [u8; 7] = [0x26, 0x2C, 0x4B, 0x4C, 0x54, 0x5C, 0x5E];

/// **The one region of the segment the template does not cover, fenced rather
/// than assumed.**
///
/// [`match_template`] matches from the block open (`53 53`) to the last byte of
/// the segment, so everything it says is exact. What it says nothing about is
/// the segment PREFIX — the `4F 1F` header with its optimization word, the
/// `4F 20` and `4F 33` records, the `4F 02 20 00` module marker and the first
/// line marker. On both captures that prefix is 55 bytes of file and line
/// metadata and carries no statement, and the natural thing to do is to write
/// that down as an assumption. This checks it instead.
///
/// **With one exemption, and it is why this is a function and not a `contains`.**
/// The `4F 33` record is a content hash — 35 arbitrary bytes on both captures —
/// so scanning it for opcode bytes would refuse this class at random as the
/// source moves, which is a *refusal* and therefore safe, but also useless. The
/// record runs from its own `4F 33` to the `4F 02 20 00` that follows it, both
/// distinctive, and it is cut out of the scan. Everything on either side of it
/// must be statement-free.
///
/// A `4F 33` with no `4F 02 20 00` after it refuses: that is a prefix shaped
/// unlike either capture and there is nothing to exempt.
fn prefix_is_statement_free(prefix: &[u8]) -> bool {
    let clean = |r: &[u8]| !r.iter().any(|b| STATEMENT_BYTES.contains(b));
    let Some(h) = prefix.windows(2).position(|w| w == [0x4F, 0x33]) else {
        return clean(prefix);
    };
    let Some(rel) = prefix[h..].windows(4).position(|w| w == [0x4F, 0x02, 0x20, 0x00]) else {
        return false;
    };
    clean(&prefix[..h]) && clean(&prefix[h + rel..])
}

// ---------------------------------------------------------------------------
// `.sy` — the local's CodeView type index
// ---------------------------------------------------------------------------

/// The type index `.sy` gives the automatic named by `tok`.
///
/// One record shape, transcribed from the hand-parse in
/// `rungs/2026-08-09-w-main.md` §1.2 and re-read on both captures:
///
/// ```text
///   01 <depth> <tok> 00 <name> 00 86 <a> 00 <b> 04 04 00 <flags> 00 <type>
/// ```
///
/// where `<type>` is either a bare CodeView primitive (one byte, `74` = `int`)
/// or the `80 <LE32>` index form. Only the second can name a class, so a bare
/// primitive refuses: a scope object is never a primitive.
pub(crate) fn sy_local_type_index(sy: &[u8], tok: u32) -> Option<u32> {
    let mut p = 0usize;
    while p + 2 < sy.len() {
        if sy[p] != 0x01 {
            p += 1;
            continue;
        }
        let Some((t, w)) = read_token_var(sy, p + 2) else {
            p += 1;
            continue;
        };
        if t != tok {
            p += 1;
            continue;
        }
        let mut q = p + 2 + w;
        if sy.get(q) != Some(&0x00) {
            p += 1;
            continue;
        }
        q += 1;
        // The NUL-terminated source name.
        let Some(z) = sy[q..].iter().position(|&x| x == 0) else { return None };
        q += z + 1;
        // The fixed nine-byte descriptor, with two free bytes in it.
        if sy.get(q) != Some(&0x86) || sy.get(q + 2) != Some(&0x00) {
            return None;
        }
        if sy.get(q + 4..q + 7) != Some(&[0x04, 0x04, 0x00]) || sy.get(q + 8) != Some(&0x00) {
            return None;
        }
        q += 9;
        if sy.get(q) != Some(&0x80) {
            return None;
        }
        return Some(u32::from_le_bytes([
            *sy.get(q + 1)?,
            *sy.get(q + 2)?,
            *sy.get(q + 3)?,
            *sy.get(q + 4)?,
        ]));
    }
    None
}

// ---------------------------------------------------------------------------
// `.db` — the class's `sizeof`
// ---------------------------------------------------------------------------

/// CodeView `LF_CLASS` / `LF_STRUCTURE`.
const LF_CLASS: u16 = 0x1504;
const LF_STRUCTURE: u16 = 0x1505;

/// The `property` bit that marks a **forward reference** — a record with no
/// members and `size = 0`. c2 emits one of these for every class before the
/// complete definition, so the walk must skip them or it reads `sizeof` as 0.
const CV_FWDREF: u16 = 0x0080;

/// `sizeof` the class at CodeView type index `want`, from the `.db` type
/// stream, or `None`.
///
/// The stream is a run of records `0B <len> <body>` where `<len>` is one byte
/// below `0x80` and `80 <LE16>` above it; indices start at `0x1000` and count
/// **only** the `0B` records. Anything else in the stream — the `0C` header, the
/// padding runs — is skipped without consuming an index, which is the one thing
/// a naive walk gets wrong (it lands the target one record early).
///
/// The size is a CodeView **numeric leaf**: a value below `0x8000` is the
/// literal `u16`. Larger encodings exist and are refused rather than decoded —
/// a 32 KB scope object is not a shape anything here has graded.
pub(crate) fn db_class_size(db: &[u8], want: u32) -> Option<u32> {
    let mut p = 0usize;
    let mut idx: u32 = 0x1000;
    while p + 1 < db.len() {
        if db[p] != 0x0B {
            p += 1;
            continue;
        }
        let (len, hdr) = match db[p + 1] {
            0x80 => (
                (*db.get(p + 2)? as usize) | ((*db.get(p + 3)? as usize) << 8),
                4,
            ),
            b if b < 0x80 => (b as usize, 2),
            _ => return None,
        };
        let body = db.get(p + hdr..p + hdr + len)?;
        if idx == want {
            if body.len() < 20 {
                return None;
            }
            let leaf = u16::from_le_bytes([body[0], body[1]]);
            if leaf != LF_CLASS && leaf != LF_STRUCTURE {
                return None;
            }
            let property = u16::from_le_bytes([body[4], body[5]]);
            if property & CV_FWDREF != 0 {
                return None;
            }
            let size = u16::from_le_bytes([body[18], body[19]]);
            if size == 0 || size >= 0x8000 {
                return None;
            }
            return Some(size as u32);
        }
        p += hdr + len;
        idx += 1;
    }
    None
}

// ---------------------------------------------------------------------------
// The recognizer
// ---------------------------------------------------------------------------

/// Recognize a `/EHsc` scope-object TU, or `None`.
///
/// The gates, each one a refusal for a shape nothing here was graded on:
///
/// * exactly **one** `.ex` function segment. This is not a convenience — the
///   `__unwind$N` funclet label is the one number of the ten that is **not** at
///   a fixed offset from the `plan_labels` cursor, and the six probe cells of
///   `work/w-main2/LABELS.md` do not separate the two readings that fit them. At
///   one function only the `B−2` branch can fire, and it is measured on three
///   distinct `.gl` seeds;
/// * `Bindings::per_record` binds it, so the emit binding is the one the gate
///   uses and never the positional one;
/// * the `.drectve` is the boilerplate the shell writer emits;
/// * every one of the three callees resolves to a name `.gl` actually spells,
///   and the constructor and destructor are **not defined in this TU** — a
///   locally defined callee is `comdat::fenced_inlined_callee`'s question and
///   this emitter does not ask it;
/// * the object's `sizeof` is readable and small enough that the frame is one
///   `stwu` immediate.
impl crate::IlBundle {
    /// Recognize a `/EHsc` scope-object TU — see [`eh_scope_tu`].
    ///
    /// Tried by `PortC2::build` **before** `functions()`, like
    /// [`crate::IlBundle::dyninit_tu`] and for the same reason: this is a
    /// whole-TU shape and `functions()` correctly refuses it.
    pub fn eh_scope_tu(&self) -> Option<EhScopeTuIl> {
        eh_scope_tu(self)
    }
}

pub(crate) fn eh_scope_tu(b: &crate::IlBundle) -> Option<EhScopeTuIl> {
    let gl = b.get("gl")?;
    let ex = b.ex()?;
    let sy = b.get("sy")?;
    let db = b.get("db")?;

    // The cheap early-out. `PortC2::build` runs this on EVERY TU, so a bundle
    // that cannot be this shape must not pay for the segment split, the binding
    // or the template walk — the same performance-as-correctness gate
    // `IlBundle::dyninit_tu` opens with. `5C` is the state transition and every
    // member of this class has exactly one.
    if !ex.contains(&0x5C) {
        return None;
    }
    if !super::gl::drectve_is_boilerplate(gl) {
        return None;
    }
    let (starts, segs) = super::bundle::split_functions_at(ex);
    if segs.len() != 1 {
        return None;
    }
    let bind = super::bind::Bindings::per_record(gl, b.get("in").unwrap_or(&[]), Some(sy), &segs, &starts)?;
    let name = bind.names().first()?.clone();

    let seg = segs[0];
    // Exactly one block open, and the template covers everything after it.
    let anchor = seg.windows(2).position(|w| w == [0x53, 0x53])?;
    if seg.windows(2).filter(|w| *w == [0x53, 0x53]).count() != 1 {
        return None;
    }
    if seg.get(..2) != Some(&[0x4F, 0x1F]) {
        return None;
    }
    if !prefix_is_statement_free(&seg[..anchor]) {
        return None;
    }
    let (fn_tok, ctor_tok, member_tok, dtor_tok, obj_tok, _formals, _t_obj) =
        match_template(&seg[anchor..])?;

    // The template's own function token must be the record that bound.
    if bind.resolve(fn_tok).is_some_and(|n| n != name) {
        return None;
    }
    let ctor = bind.resolve(ctor_tok)?;
    let member = bind.resolve(member_tok)?;
    let dtor = bind.resolve(dtor_tok)?;
    if ctor == member || member == dtor || ctor == dtor {
        return None;
    }
    // A callee this TU also DEFINES is the inline fence's question
    // (`CEILING.md` §11's NC-5), and this emitter always writes the `bl`. All
    // three must be undefined externals.
    for c in [&ctor, &member, &dtor] {
        if bind.names().iter().any(|n| n == c) {
            return None;
        }
    }

    let ty = sy_local_type_index(sy, obj_tok)?;
    let object_size = db_class_size(db, ty)?;
    if object_size > 4096 {
        return None;
    }
    let label_counter = super::gl::label_counter(gl)?;

    Some(EhScopeTuIl {
        name,
        ctor,
        member,
        dtor,
        object_size,
        formals: 2,
        label_counter,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `src/Main.cpp`'s own `.ex` function segment, all 222 bytes, captured at
    /// the workload's flags — the same way `body::shapes::control_flow`'s
    /// `EH_ST_*` constants are captured. Reproduce with
    /// `python3 work/w-main2/tpl.py work/w-main2/il/<bundle>.ex`.
    const MAIN_SEG: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x01, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x53, 0x53, 0x26, 0x0C, 0x0A,
        0x46, 0x2D, 0x0B, 0x0A, 0x2D, 0x0A, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x04, 0x26,
        0xFB, 0x09, 0x26, 0x0E, 0x0A, 0x2C, 0xA6, 0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x8B,
        0x20, 0x00, 0xBD, 0xA6, 0x43, 0x81, 0x20, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0xB9, 0x0B,
        0x0A, 0x86, 0x43, 0x84, 0x20, 0x55, 0x86, 0x43, 0x84, 0x20, 0xB9, 0x0A, 0x0A, 0x86, 0x41,
        0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x26, 0xFD, 0x09, 0x26, 0x0E, 0x0A, 0x2C, 0xA6, 0x43,
        0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x83, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80,
        0x03, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0xA6, 0x43, 0x81, 0x20, 0x01, 0x4B, 0x4F, 0x01, 0x05,
        0x26, 0x01, 0x0A, 0x26, 0x0E, 0x0A, 0x2C, 0xA6, 0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43,
        0x83, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x03, 0x10, 0x00, 0x00, 0x4C, 0x4B,
        0x4F, 0x01, 0x06, 0x5E, 0x01, 0x21, 0x4B, 0x54, 0x02, 0x29, 0x0D, 0x0A, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07, 0x4D,
    ];

    /// `work/w-main2/probe/m0.cpp`'s segment — the SAME class written on one
    /// source line, so it carries **no** inter-statement `4F 01` markers and a
    /// different token and type index in every hole. This is the cell that says
    /// the template is a template and not a transcription of one file.
    const M0_SEG: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x01, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xF4, 0x09,
        0x46, 0x2D, 0xF3, 0x09, 0x2D, 0xF2, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE7, 0x09, 0x26,
        0xF6, 0x09, 0x2C, 0xA6, 0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x8B, 0x20, 0x00, 0xBD,
        0xA6, 0x43, 0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0xB9, 0xF3, 0x09, 0x86, 0x43,
        0x82, 0x20, 0x55, 0x86, 0x43, 0x82, 0x20, 0xB9, 0xF2, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86,
        0x41, 0x74, 0x4C, 0x26, 0xE8, 0x09, 0x26, 0xF6, 0x09, 0x2C, 0xA6, 0x43, 0x81, 0x20, 0x00,
        0x99, 0x86, 0x43, 0x86, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x06, 0x10, 0x00,
        0x00, 0x4C, 0x5C, 0xA6, 0x43, 0x81, 0x20, 0x01, 0x4B, 0x26, 0xE9, 0x09, 0x26, 0xF6, 0x09,
        0x2C, 0xA6, 0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x86, 0x20, 0x00, 0xBD, 0x82, 0x07,
        0x03, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x5E, 0x01, 0x21, 0x4B, 0x54, 0x02,
        0x29, 0xF5, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F,
        0x01, 0x03, 0x4D,
    ];

    fn anchor(seg: &[u8]) -> &[u8] {
        let a = seg.windows(2).position(|w| w == [0x53, 0x53]).unwrap();
        &seg[a..]
    }

    /// The prefix fence, on both captures and on the two shapes it exists for.
    #[test]
    fn the_segment_prefix_is_fenced_with_the_hash_record_exempt() {
        let pre = |seg: &[u8]| {
            let a = seg.windows(2).position(|w| w == [0x53, 0x53]).unwrap();
            seg[..a].to_vec()
        };
        // Both real prefixes pass, and both CONTAIN the `4F 33` hash record.
        for seg in [MAIN_SEG, M0_SEG] {
            let p = pre(seg);
            assert!(p.windows(2).any(|w| w == [0x4F, 0x33]));
            assert!(prefix_is_statement_free(&p));
        }
        // A statement byte OUTSIDE the hash record refuses — injected into the
        // `4F 1F` header, ahead of the `4F 33`.
        let mut p = pre(MAIN_SEG);
        p[3] = 0x26;
        assert!(!prefix_is_statement_free(&p));
        // …and one INSIDE the hash record does not, which is the exemption.
        let mut p = pre(MAIN_SEG);
        let h = p.windows(2).position(|w| w == [0x4F, 0x33]).unwrap();
        p[h + 4] = 0x26;
        assert!(prefix_is_statement_free(&p));
        // A `4F 33` with no `4F 02 20 00` after it has nothing to exempt.
        let mut p = pre(MAIN_SEG);
        let e = p.windows(4).rposition(|w| w == [0x4F, 0x02, 0x20, 0x00]).unwrap();
        p[e] = 0x00;
        assert!(!prefix_is_statement_free(&p));
    }

    #[test]
    fn the_template_matches_the_workload_tu_and_names_its_four_tokens() {
        let (f, ctor, member, dtor, obj, formals, t) =
            match_template(anchor(MAIN_SEG)).expect("in class");
        // `read_token_var` is big-endian two-byte here, so `0C 0A` is 0x0C0A.
        assert_eq!(f, 0x0C0A);
        assert_eq!(ctor, 0xFB09);
        assert_eq!(dtor, 0xFD09);
        assert_eq!(member, 0x010A);
        assert_eq!(obj, 0x0E0A);
        assert_eq!(formals, [0x0B0A, 0x0A0A]);
        assert_eq!(t, 0x1006);
    }

    /// The second capture: different tokens, different type indices, and NO
    /// inter-statement line markers at all.
    #[test]
    fn the_template_matches_a_second_independent_capture() {
        let (f, ctor, member, dtor, obj, formals, t) =
            match_template(anchor(M0_SEG)).expect("in class");
        assert_eq!(f, 0xF409);
        assert_eq!(ctor, 0xE709);
        assert_eq!(dtor, 0xE809);
        assert_eq!(member, 0xE909);
        assert_eq!(obj, 0xF609);
        assert_eq!(formals, [0xF309, 0xF209]);
        assert_eq!(t, 0x1004);
    }

    /// The three statements are ORDERED, and the order is what says which callee
    /// is the destructor. Swapping the unwind action with the member call is a
    /// different obj — the `bl` in the funclet would name the wrong symbol — so
    /// the template must not accept it.
    #[test]
    fn a_reordered_body_refuses() {
        let mut seg = MAIN_SEG.to_vec();
        // Move the `5C` state transition ahead of the unwind action by deleting
        // it from its place; the template's `5C` arm then lands on `26`.
        let p = seg.windows(2).position(|w| w == [0x5C, 0xA6]).unwrap();
        seg.drain(p..p + 7);
        assert!(match_template(anchor(&seg)).is_none());
    }

    /// A body with a SECOND sub-object (`5E 02 21`) is a two-entry unwind map
    /// and a different `.rdata`; it must not be read as this one.
    #[test]
    fn a_two_object_scope_end_refuses() {
        let mut seg = MAIN_SEG.to_vec();
        let p = seg.windows(4).position(|w| w == [0x5E, 0x01, 0x21, 0x4B]).unwrap();
        seg[p + 1] = 0x02;
        assert!(match_template(anchor(&seg)).is_none());
    }

    /// `maxState` other than 1 is a state map this emitter does not build.
    #[test]
    fn a_deeper_state_refuses() {
        let mut seg = MAIN_SEG.to_vec();
        let p = seg.windows(2).position(|w| w == [0x5C, 0xA6]).unwrap();
        seg[p + 5] = 0x02;
        assert!(match_template(anchor(&seg)).is_none());
    }

    /// A trailing byte the template does not consume refuses — the match is over
    /// the WHOLE body, not a prefix of it.
    #[test]
    fn a_trailing_statement_refuses() {
        let mut seg = MAIN_SEG.to_vec();
        seg.push(0x4B);
        assert!(match_template(anchor(&seg)).is_none());
    }

    /// `src/Main.cpp`'s own `.sy`, all 80 bytes, and its `.db` class record.
    const MAIN_SY: &[u8] = &[
        0x03, 0x01, 0x0D, 0x0A, 0x1F, 0x00, 0x00, 0x01, 0x0D, 0x01, 0x01, 0x01, 0x0B, 0x0A, 0x00,
        0x61, 0x72, 0x67, 0x76, 0x00, 0x86, 0x03, 0x00, 0x03, 0x04, 0x04, 0x00, 0x01, 0x00, 0x80,
        0x04, 0x10, 0x00, 0x00, 0x01, 0x01, 0x0A, 0x0A, 0x00, 0x61, 0x72, 0x67, 0x63, 0x00, 0x86,
        0x01, 0x00, 0x03, 0x04, 0x04, 0x00, 0x01, 0x00, 0x74, 0x0D, 0x02, 0x01, 0x02, 0x0E, 0x0A,
        0x00, 0x61, 0x70, 0x70, 0x00, 0x86, 0x06, 0x00, 0x01, 0x04, 0x04, 0x00, 0x21, 0x00, 0x80,
        0x0A, 0x10, 0x00, 0x00, 0x06,
    ];

    #[test]
    fn the_locals_type_index_is_read_and_a_primitive_refuses() {
        assert_eq!(sy_local_type_index(MAIN_SY, 0x0E0A), Some(0x100A));
        // `argc` is a bare `74` (`int`) — no index form, so no class.
        assert_eq!(sy_local_type_index(MAIN_SY, 0x0A0A), None);
        assert_eq!(sy_local_type_index(MAIN_SY, 0x9999), None);
    }

    /// The `.db` walk, on a stream built to the measured framing: the leading
    /// `0C` header (which must not consume an index), a forward reference at
    /// `0x1000`, a wide-length record at `0x1001`, and the complete class at
    /// `0x1002`.
    #[test]
    fn the_class_size_walk_skips_non_type_records_and_forward_references() {
        let mut db: Vec<u8> = vec![0x0C, 0x01, 0x0A];
        let mut rec = |property: u16, size: u16, pad: usize| {
            let mut body: Vec<u8> = Vec::new();
            body.extend_from_slice(&LF_CLASS.to_le_bytes());
            body.extend_from_slice(&0u16.to_le_bytes()); // count
            body.extend_from_slice(&property.to_le_bytes());
            body.extend_from_slice(&[0; 12]); // field, derived, vshape
            body.extend_from_slice(&size.to_le_bytes());
            body.extend_from_slice(&vec![0xF1; pad]);
            db.push(0x0B);
            if body.len() < 0x80 {
                db.push(body.len() as u8);
            } else {
                db.push(0x80);
                db.extend_from_slice(&(body.len() as u16).to_le_bytes());
            }
            db.extend_from_slice(&body);
        };
        rec(CV_FWDREF, 0, 0);
        rec(0x0202, 8, 200); // wide length
        rec(0x0202, 4, 0);
        assert_eq!(db_class_size(&db, 0x1000), None, "a forward reference has no size");
        assert_eq!(db_class_size(&db, 0x1001), Some(8));
        assert_eq!(db_class_size(&db, 0x1002), Some(4), "the wide record consumed ONE index");
        assert_eq!(db_class_size(&db, 0x1003), None);
    }
}
