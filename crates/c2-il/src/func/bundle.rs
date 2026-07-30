use super::body::{parse_segment, BodyShape};
use super::gl::{gl_defined_names, source_path, GlIndex};
use super::readers::{find_subslice, memchr_byte};
use super::{FramedCall, IlFunction};
use crate::IlBundle;

/// The `.ex` per-function start marker (`4F 1F`). The module stream is a
/// sequence of function bodies, each introduced by this marker; the header /
/// index region before the first one is opaque zero-fill for this class.
pub(crate) const FN_START: [u8; 2] = [0x4F, 0x1F];

/// The `.ex` body marker `4C 4F 11` (`LO`) that opens every function body.
pub(crate) const LO_MARKER: [u8; 3] = [0x4C, 0x4F, 0x11];

/// Split `.ex` into one segment per **function body**, anchored on the `LO`
/// marker rather than the `4F 1F` function-start marker (P2b).
///
/// `4F 1F` is only two bytes and also occurs inside token and varint payloads,
/// so a raw marker scan over a real translation unit over-counts: measured on
/// `system/world/Dir.cpp` (1.5 MB `.ex`), 5340 `4F 1F` against 5239 `LO` body
/// markers and 5243 function tails (`4F 12 47 54 01 54 00`) — the latter two
/// agree to 0.08%, the first is ~2% high. Anchoring on `LO` keeps the count
/// honest without inventing a denominator.
///
/// Each segment starts at the `4F 1F` immediately preceding its `LO` (so the
/// formals region stays inside the segment, where [`parse_formals`] looks for
/// it) and runs to the next segment's start. Two bodies sharing one preceding
/// `4F 1F` would collide; the later one then starts at its own `LO`, which
/// simply blocks it at `formals-marker` — an honest miss, never a merge that
/// would silently drop a function from the denominator.
pub(crate) fn split_function_bodies(ex: &[u8]) -> Vec<&[u8]> {
    // Body markers, in file order. Same walk as the old byte loop (a match
    // consumes 3 bytes, a miss 1); candidates are found word-at-a-time.
    let mut los: Vec<usize> = Vec::new();
    let mut i = 0;
    while i + 3 <= ex.len() {
        let Some(k) = memchr_byte(LO_MARKER[0], &ex[i..ex.len() - 2]) else {
            break;
        };
        let j = i + k;
        if ex[j + 1] == LO_MARKER[1] && ex[j + 2] == LO_MARKER[2] {
            los.push(j);
            i = j + 3;
        } else {
            i = j + 1;
        }
    }
    if los.is_empty() {
        return Vec::new();
    }
    // Function-start markers, in file order, for the "nearest preceding" lookup.
    let mut starts: Vec<usize> = Vec::new();
    let mut i = 0;
    while i + 2 <= ex.len() {
        let Some(k) = memchr_byte(FN_START[0], &ex[i..ex.len() - 1]) else {
            break;
        };
        let j = i + k;
        if ex[j + 1] == FN_START[1] {
            starts.push(j);
            i = j + 2;
        } else {
            i = j + 1;
        }
    }

    let mut segs_start: Vec<usize> = Vec::with_capacity(los.len());
    for &lo in &los {
        // Greatest `4F 1F` offset strictly below this body marker.
        let cand = match starts.partition_point(|&s| s < lo) {
            0 => lo,
            k => starts[k - 1],
        };
        // Never reuse a start (would merge two bodies into one segment).
        let cand = if segs_start.last() == Some(&cand) { lo } else { cand };
        segs_start.push(cand);
    }
    (0..segs_start.len())
        .map(|k| {
            let start = segs_start[k];
            let end = segs_start.get(k + 1).copied().unwrap_or(ex.len());
            &ex[start..end.max(start)]
        })
        .collect()
}

/// True iff `.ex` positively declares a module with **no function bodies**
/// (R1): it carries neither a body marker (`4C 4F 11`) nor a function-start
/// marker (`4F 1F`).
///
/// Both signals are required. `4F 1F` alone is two bytes and collides inside
/// payloads (so its *absence* is meaningful but its presence is not), while
/// `LO` is the marker every real body opens with — on a 1.5 MB real `.ex` the
/// `LO` count tracked the function-tail count to 0.08%. A capture with zero of
/// each has nothing that could be a function.
///
/// Verified against the live toolchain: a TU containing only a typedef captures
/// a 2691-byte `.ex` that is entirely zero-fill apart from a 4-byte head and a
/// 46-byte module-metadata tail, with 0 `LO` and 0 `4F 1F`, and c2 emits a
/// 720-byte four-section obj for it.
pub fn is_empty_module(ex: &[u8]) -> bool {
    let has_lo = find_subslice(ex, &LO_MARKER).is_some();
    let has_fn_start = find_subslice(ex, &FN_START).is_some();
    !has_lo && !has_fn_start
}

fn split_functions(ex: &[u8]) -> Vec<&[u8]> {
    split_functions_at(ex).1
}

/// [`split_functions`], keeping the `4F 1F` offsets alongside the segments. The
/// offsets are what `.gl`'s framed body-start fields are matched against, so the
/// name binding is per record rather than per position (see
/// [`gl_defined_names`]).
fn split_functions_at(ex: &[u8]) -> (Vec<usize>, Vec<&[u8]>) {
    let mut starts = Vec::new();
    let mut i = 0;
    // Same walk as the old byte loop (a match consumes 2 bytes, a miss 1);
    // candidates are found word-at-a-time.
    while i + 1 < ex.len() {
        let Some(k) = memchr_byte(FN_START[0], &ex[i..ex.len() - 1]) else {
            break;
        };
        let j = i + k;
        if ex[j + 1] == FN_START[1] {
            starts.push(j);
            i = j + 2;
        } else {
            i = j + 1;
        }
    }
    let mut segs = Vec::with_capacity(starts.len());
    for k in 0..starts.len() {
        let end = if k + 1 < starts.len() { starts[k + 1] } else { ex.len() };
        segs.push(&ex[starts[k]..end]);
    }
    (starts, segs)
}

impl IlBundle {
    /// Parse this bundle as a sequence of straight-line add-chain functions
    /// (the MVP class, generalized to a multi-function TU). Returns `None` if
    /// the required files are absent, or if the `.gl` name count does not match
    /// the `.ex` function count, or if ANY function body is outside the class
    /// (the caller — `PortC2` — then reports `NotImplemented` for the whole TU).
    ///
    /// Bodies come from `.ex` split at each `4F 1F`; each body's name comes from
    /// the `.gl` record whose framed body-start offset **is** that split point
    /// ([`gl_defined_names`]) — a per-record binding, not a positional one. Any
    /// `.gl` symbol no record claimed must be a resolved callee, or the TU is
    /// refused: an unclaimed symbol is one the real obj defines and the port does
    /// not model.
    pub fn functions(&self) -> Option<Vec<IlFunction>> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;
        let src = source_path(gl);

        // R1: a TU that defines no functions is in class, and its obj is the
        // fixed four-section shell with no `.text`. Recognized **positively**
        // (no body markers AND no function-start markers), never as "the split
        // returned nothing" — the latter would also fire on a bundle we merely
        // failed to split, and emitting an empty obj for a TU that really has
        // code is precisely the mis-emit the fail-closed rule forbids.
        //
        // Evaluated in one pass over `.ex` instead of calling
        // [`is_empty_module`] up front: the split already proves whether any
        // `4F 1F` exists, so only the no-start case still needs the body-marker
        // probe. The predicate is unchanged — all four (LO?, 4F1F?) cases land
        // exactly where they did:
        //   neither        → empty module (was: is_empty_module → Some([]))
        //   LO only        → None         (was: not empty; split empty → None)
        //   4F 1F, any LO  → parse        (was: not empty; split non-empty)
        let (starts, segs) = split_functions_at(ex);
        if segs.is_empty() {
            return if find_subslice(ex, &LO_MARKER).is_none() {
                Some(Vec::new())
            } else {
                None
            };
        }
        // Per-record name binding, gated fail-closed: the `.gl` records' framed
        // body-start offsets must be exactly the `.ex` split points, in order and
        // 1:1. A disagreement means either `.gl` has a record shape we cannot
        // frame or the splitter miscounted bodies, and in both cases every name
        // after the divergence would be wrong — so bind none of them.
        //
        // A *defined* function's own name comes from here. Callee names do NOT:
        // they are resolved by token through the `.gl` symbol index, because the
        // CALL token carries only a function-type id and cannot distinguish two
        // callees with the same signature.
        let (bound, unclaimed) = gl_defined_names(gl);
        if bound.len() != segs.len()
            || bound
                .iter()
                .zip(&starts)
                .any(|(&(off, _), &s)| off as usize != s)
        {
            return None;
        }
        let names: Vec<String> = bound.into_iter().map(|(_, n)| n).collect();
        let n_defined = segs.len();
        // Lazily built: only the call productions resolve through it, so a TU
        // of straight-line leaves never constructs the index at all.
        let symbols = GlIndex::new(gl);
        let resolve = |tok: u32| -> Option<String> { symbols.map().get(&tok).cloned() };

        let mut funcs = Vec::with_capacity(n_defined);
        for (name, seg) in names.iter().take(n_defined).zip(segs) {
            match parse_segment(seg, &symbols)? {
                // An indirect-load leaf reaches the ordinary integer selector,
                // which pattern-matches its exact two-op stream; `params` carries
                // a member function's `this` at index 0 so the base register comes
                // out right.
                BodyShape::IndirectLoad { params, ops } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops,
                        tail_call: None,
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                BodyShape::StraightLine { params, ops } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops,
                        tail_call: None,
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                // Tail calls: the callee is resolved BY TOKEN through the `.gl`
                // symbol index. An unresolvable token rejects the whole TU
                // rather than falling back to a positional guess — a wrong
                // callee name is a relocation against the wrong symbol, which is
                // a mis-emit, not a gap.
                BodyShape::VoidTailCall { callee_tok } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: Vec::new(),
                        ops: Vec::new(),
                        tail_call: Some(resolve(callee_tok)?),
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                BodyShape::IntTailCall { params, arg_ops, callee_tok } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops: arg_ops,
                        tail_call: Some(resolve(callee_tok)?),
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                // A multi-argument tail call is still a tail call — same resolved
                // callee, same `b <callee>` — but its argument setup is a register
                // permutation rather than an operand stream, so `ops` stays empty
                // and `arg_sources` carries the mapping.
                BodyShape::MultiArgTailCall { params, arg_sources, callee_tok } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops: Vec::new(),
                        tail_call: Some(resolve(callee_tok)?),
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: Some(arg_sources),
                    });
                }
                // The framed non-leaf path stays SINGLE-FUNCTION. Its obj carries
                // `.pdata` with compiler label symbols ($M2545/$M2546/$T2547)
                // whose counters are a fixed toolchain seed for the first
                // function and shift once preceding functions consume slots
                // (W-UNW-1, docs/CODEGEN_PPC_MVP.md), so a multi-function TU
                // containing one would be mis-numbered.
                BodyShape::FramedCall { add_k, callee_tok } => {
                    if n_defined != 1 {
                        return None;
                    }
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: Vec::new(),
                        ops: Vec::new(),
                        tail_call: None,
                        framed_call: Some(FramedCall {
                            callee: resolve(callee_tok)?,
                            add_k,
                        }),
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                // W6: a comparison leaf carries no op stream — codegen emits its
                // spine from the decoded relation instead.
                BodyShape::EmptyBody => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: Vec::new(),
                        ops: Vec::new(),
                        tail_call: None,
                        framed_call: None,
                        compare: None,
                        empty_body: true,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                BodyShape::FloatLeaf { params, ops, double } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops,
                        tail_call: None,
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: Some(double),
                        arg_sources: None,
                    });
                }
                BodyShape::Compare(cmp) => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: vec![cmp.param],
                        ops: Vec::new(),
                        tail_call: None,
                        framed_call: None,
                        compare: Some(cmp),
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
            }
        }
        // Account for every `.gl` symbol no record claimed. The port emits
        // exactly the `n_defined` bodies plus an external symbol per resolved
        // callee, so an unclaimed name is a symbol the real obj has and this obj
        // would not — and for a *data* definition it is a whole extra section.
        // `int gv; int f(int a){return a+1;}` mismatched at file offset 2, the
        // section count, for exactly this reason: `?gv@@3HA` was invisible to the
        // emitter. A defined static member (`?sm@S@@2HA`) did the same.
        //
        // Extern data cannot be told from defined data by mangling — `extern int
        // g;` and `int g;` both appear as `?g@@3HA` — so this refuses both. That
        // costs nothing today: reading a global is already out of class, so an
        // extern that is never referenced is one c2 would not have listed.
        let mut accounted: Vec<&str> = names.iter().map(String::as_str).collect();
        for f in &funcs {
            if let Some(c) = &f.tail_call {
                accounted.push(c);
            }
            if let Some(fc) = &f.framed_call {
                accounted.push(&fc.callee);
            }
        }
        if unclaimed.iter().any(|n| !accounted.contains(&n.as_str())) {
            return None;
        }
        // A callee that is also DEFINED here is out of class: c2 may inline it,
        // and the port cannot. `int f(int); int use(int a){return f(a);}
        // int f(int a){return a+1;}` gets a `.text` of *two* copies of
        // `addi r3,r3,1 ; blr` and **no relocations** — c2 cloned `f` into `use`
        // rather than branching to it. The port emitted `b ?f` against an
        // undefined external and mismatched at file offset 8.
        //
        // Refused wholesale rather than by callee size, because what makes c2
        // inline (and what it does to the symbol table and `.pdata` when it does)
        // is uncharacterized. Calls to true externals are unaffected — those are
        // the tail calls the class was built on.
        if funcs.iter().any(|f| {
            let callee = f
                .tail_call
                .as_deref()
                .or(f.framed_call.as_ref().map(|c| c.callee.as_str()));
            callee.is_some_and(|c| names.iter().any(|n| n == c))
        }) {
            return None;
        }
        Some(funcs)
    }

    /// Parse this bundle as a SINGLE MVP function. Convenience wrapper over
    /// [`IlBundle::functions`]; returns `None` unless the TU has exactly one
    /// in-class function.
    pub fn mvp_function(&self) -> Option<IlFunction> {
        let mut fs = self.functions()?;
        if fs.len() == 1 {
            fs.pop()
        } else {
            None
        }
    }
}
