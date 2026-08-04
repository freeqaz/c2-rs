use std::collections::BTreeSet;

use c2_il::{ExToken, IlModel};

// ===========================================================================
// Move set — the K3a-licensed neighborhood
// ===========================================================================

/// Which K3a edit families the neighborhood enumerates. The default is the full
/// licensed family; [`MoveSet::length_only`] restricts to the pure length moves
/// (widen/narrow + term add/delete) that P0.6a proved re-optimize byte-exact.
#[derive(Clone, Debug)]
pub struct MoveSet {
    /// Widen/narrow the varint form of each int literal (same value; P0.6a A/B).
    pub widen_narrow: bool,
    /// Nudge each int literal by each delta in `value_nudges` (relative, so a
    /// value perturbation of magnitude ≤ the nudge range is recoverable). The
    /// emitted immediate is a flat field, so this is search-by-trial, not a
    /// smooth gradient — kept to a small local window.
    pub literal_value: bool,
    /// Relative deltas tried by `literal_value` (`current + delta`).
    pub value_nudges: Vec<i32>,
    /// Delete a trailing `<operand> <op>` term (`a+b+c` → `a+b`; P0.6a F).
    pub term_delete: bool,
    /// Insert a `<operand> <op>` term after an existing value token (`a+5` →
    /// `(a+5)+5`; P0.6a E). Ops from `insert_ops`; the operand vocabulary is the
    /// body's own operands **plus**, when `insert_from_scope`, generated operands
    /// (params in scope + `insert_literals`) so a *vanished* operand can be
    /// regenerated (the drop-term lossy-seed case).
    pub term_insert: bool,
    /// Binary ops used when inserting a term.
    pub insert_ops: Vec<ExToken>,
    /// Generative insert vocabulary: also enumerate insert operands NOT present in
    /// the body — every formal parameter in scope (as a `Load`) and each literal
    /// in `insert_literals` — so a term whose operand vanished from the seed (a
    /// dropped `+param` or `+k`) can be reconstructed. Bounded (params ≤ arity,
    /// literals a small fixed set) to keep the branching factor sane.
    pub insert_from_scope: bool,
    /// The small literal vocabulary generated for insert when `insert_from_scope`
    /// (the "vanished literal" set — a dropped `+k` is only recoverable if `k` is
    /// here or already elsewhere in the body).
    pub insert_literals: Vec<i32>,
    /// **Opt-in, OFF by default** ([`MoveSet::default`]/[`MoveSet::length_only`]
    /// both leave it `false`; enable via [`MoveSet::with_mul_reorder`]). When set,
    /// [`MoveSet::neighbors`] emits, for each `MUL` (`04`) node whose two immediate
    /// operands are single-token leaves, the operand-swapped ordering as one d=1
    /// neighbor — the primitive for the commutative-order class (the real
    /// `Box::Volume` regalloc floor is a commutative reorder).
    ///
    /// **Why it is gated (CLAUDE.md rule 1).** The move is licensed ONLY because
    /// `04` is **commutative**, so the swap stays inside the licensed action space.
    /// It is guarded strictly to `04`: an operand swap of `03` (SUB), `-`, `/`,
    /// `%`, `<<`, `>>`, or a function/argument/comparison swap is a
    /// *non-commutative silent corruption* (the DC3 `SetupCharacter` flip of
    /// `z0 - aspect*size` matched flat at 83.8% and made all game text invisible),
    /// so this generator NEVER emits one. It is off by default because the binding
    /// seam is the **adopt-as-truth / floor-certification** path: a generator that
    /// can emit swaps must be opt-in there, since `fuzzy%` cannot see a
    /// match-neutral behavioral corruption in machine-minted source trusted without
    /// a byte-exact witness. On the *search* path the byte-exact terminal (the sole
    /// judge) makes any accepted candidate behavior-safe as a crack regardless, so
    /// enabling the move only widens which candidates are compiled — it can never
    /// promote a non-byte-exact result.
    pub mul_reorder: bool,
}

impl Default for MoveSet {
    fn default() -> Self {
        MoveSet {
            widen_narrow: true,
            literal_value: true,
            value_nudges: vec![-10, -5, -3, -2, -1, 1, 2, 3, 5, 10],
            term_delete: true,
            term_insert: true,
            insert_ops: vec![ExToken::Add, ExToken::Sub, ExToken::Mul],
            insert_from_scope: true,
            insert_literals: vec![1, 2, 5],
            mul_reorder: false, // opt-in only (see the field docstring)
        }
    }
}

impl MoveSet {
    /// The pure length-move family: widen/narrow + term add/delete, no literal
    /// value enumeration. This is the family K3a/P0.6a licensed as
    /// re-optimize-to-byte-exact, and the one whose recovery the climber guides
    /// structurally rather than by trial.
    pub fn length_only() -> Self {
        MoveSet {
            widen_narrow: true,
            literal_value: false,
            value_nudges: Vec::new(),
            term_delete: true,
            term_insert: true,
            insert_ops: vec![ExToken::Add, ExToken::Sub, ExToken::Mul],
            insert_from_scope: true,
            insert_literals: vec![1, 2, 5],
            mul_reorder: false, // opt-in only (see the field docstring)
        }
    }

    /// Enable the opt-in commutative **MUL-factor reorder** move (OFF by default).
    /// See the [`MoveSet::mul_reorder`] field docstring for the guard rationale
    /// (MUL/`04`-only, licensed by commutativity; the adopt-as-truth /
    /// floor-certification seam is why it is opt-in). Chains on any constructor,
    /// e.g. `MoveSet::default().with_mul_reorder()`.
    pub fn with_mul_reorder(mut self) -> Self {
        self.mul_reorder = true;
        self
    }

    /// Enumerate the bounded neighborhood of `model`: every in-scope K3a edit,
    /// applied to a fresh clone. A refused edit ([`c2_il::EditError`]) is skipped
    /// (fail-closed — the model is left untouched by a failed splice). Candidates
    /// are deduplicated by their `.ex` bytes and returned in a deterministic
    /// order, each labelled for the readout/log.
    pub fn neighbors(&self, model: &IlModel) -> Vec<(String, IlModel)> {
        let mut out: Vec<(String, IlModel)> = Vec::new();
        let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
        // The seed's own `.ex` is not a neighbor of itself.
        if let Some(ex) = model.encode().get("ex") {
            seen.insert(ex.to_vec());
        }

        let nfns = model.ex_function_count();
        for fi in 0..nfns {
            let Ok(tokens) = model.function_tokens(fi) else {
                continue; // opaque body — not token-addressable
            };

            // ---- widen / narrow each literal -------------------------------
            if self.widen_narrow {
                for (ti, tok) in tokens.iter().enumerate() {
                    if let ExToken::Lit { wide, .. } = tok {
                        let mut cand = model.clone();
                        if cand.set_literal_wide(fi, ti, !wide).is_ok() {
                            let label = format!(
                                "fn{fi} lit@{ti} {}",
                                if *wide { "narrow" } else { "widen" }
                            );
                            push(&mut out, &mut seen, label, cand);
                        }
                    }
                }
            }

            // ---- nudge each literal by a local delta ------------------------
            if self.literal_value {
                for (ti, tok) in tokens.iter().enumerate() {
                    if let ExToken::Lit { value, wide } = *tok {
                        for &d in &self.value_nudges {
                            let Some(v) = value.checked_add(d) else {
                                continue;
                            };
                            if v == value {
                                continue;
                            }
                            let mut cand = model.clone();
                            // Keep the same varint width where the value permits;
                            // a narrow slot with a wide value must widen.
                            let want_wide = wide || !(0..=0x7F).contains(&v);
                            let repl = vec![ExToken::Lit { value: v, wide: want_wide }];
                            if cand
                                .splice_function_tokens(fi, ti..ti + 1, repl)
                                .is_ok()
                            {
                                push(&mut out, &mut seen, format!("fn{fi} lit@{ti} {v:+}"), cand);
                            }
                        }
                    }
                }
            }

            // ---- delete a trailing `<operand> <op>` term -------------------
            if self.term_delete {
                for i in 0..tokens.len().saturating_sub(1) {
                    if is_operand(&tokens[i]) && is_binop(&tokens[i + 1]) {
                        let mut cand = model.clone();
                        if cand.splice_function_tokens(fi, i..i + 2, vec![]).is_ok() {
                            push(&mut out, &mut seen, format!("fn{fi} del term@{i}"), cand);
                        }
                    }
                }
            }

            // ---- insert a `<operand> <op>` term after a value token ---------
            // A value-producing token (operand OR binop) leaves exactly one net
            // value on the stack, so appending `<operand> <op>` after it is
            // always valid postfix (`… V` → `… (V op W)`). Anchoring after
            // operands too — not only ops — lets insert reconstruct a term even
            // when the seed body has no remaining binop (e.g. a fully-dropped
            // single term), the direction P0.6a E exercised.
            if self.term_insert {
                let operands: Vec<ExToken> = if self.insert_from_scope {
                    generative_operands(&tokens, &self.insert_literals)
                } else {
                    distinct_operands(&tokens)
                };
                for (i, tok) in tokens.iter().enumerate() {
                    if !is_operand(tok) && !is_binop(tok) {
                        continue;
                    }
                    for operand in &operands {
                        for op in &self.insert_ops {
                            let mut cand = model.clone();
                            let repl = vec![operand.clone(), op.clone()];
                            if cand
                                .splice_function_tokens(fi, i + 1..i + 1, repl)
                                .is_ok()
                            {
                                let label = format!("fn{fi} ins@{i} {}", op_name(op));
                                push(&mut out, &mut seen, label, cand);
                            }
                        }
                    }
                }
            }

            // ---- commutative MUL-factor reorder (opt-in; MUL `04` ONLY) ------
            // For a `… A B MUL` node whose two immediate operands `A`,`B` are
            // single-token leaves, emit the operand-swapped ordering `… B A MUL`
            // as one d=1 neighbor. Guarded strictly to `ExToken::Mul` (opcode
            // `04`) — the ONLY commutative binop here; SUB/`03` (and `-` `/` `%`
            // `<<` `>>`, argument/comparison swaps) are non-commutative silent
            // corruptions and are never swapped (CLAUDE.md rule 1; the guard's
            // full rationale + the adopt-as-truth/floor-cert seam are on the
            // `mul_reorder` field docstring). Identical operands (`a*a`) are
            // skipped — the swap is a no-op the `.ex` dedup would drop anyway.
            if self.mul_reorder {
                for i in 2..tokens.len() {
                    if !matches!(tokens[i], ExToken::Mul) {
                        continue;
                    }
                    let (a, b) = (&tokens[i - 2], &tokens[i - 1]);
                    if is_operand(a) && is_operand(b) && a != b {
                        let mut cand = model.clone();
                        let repl = vec![b.clone(), a.clone()];
                        if cand.splice_function_tokens(fi, i - 2..i, repl).is_ok() {
                            push(&mut out, &mut seen, format!("fn{fi} mul-swap@{i}"), cand);
                        }
                    }
                }
            }
        }
        out
    }
}

fn push(
    out: &mut Vec<(String, IlModel)>,
    seen: &mut BTreeSet<Vec<u8>>,
    label: String,
    cand: IlModel,
) {
    let ex = cand
        .encode()
        .get("ex")
        .map(|b| b.to_vec())
        .unwrap_or_default();
    if seen.insert(ex) {
        out.push((label, cand));
    }
}

pub(super) fn is_operand(t: &ExToken) -> bool {
    // `FloatLoad` is a leaf operand exactly as `Load` is — the float-leaf codec
    // widening (Box::Volume class) types the float-arith stream, and its member
    // diffs/products push a single float value per `FloatLoad`, so a `FloatLoad`
    // is a delete/insert anchor for the length moves (and a swap anchor for the
    // MUL reorder below), just like the int `Load`. Without this a float leaf has
    // zero operand anchors, i.e. an empty action space on every real dc3 target.
    matches!(
        t,
        ExToken::Load(_) | ExToken::FloatLoad(_) | ExToken::Lit { .. }
    )
}

pub(super) fn is_binop(t: &ExToken) -> bool {
    matches!(t, ExToken::Add | ExToken::Sub | ExToken::Mul)
}

fn op_name(t: &ExToken) -> &'static str {
    match t {
        ExToken::Add => "add",
        ExToken::Sub => "sub",
        ExToken::Mul => "mul",
        _ => "op",
    }
}

/// The distinct operand tokens in a body (each `Load`/`Lit`), in first-seen
/// order — the operands a term-insert reuses.
pub(super) fn distinct_operands(tokens: &[ExToken]) -> Vec<ExToken> {
    let mut out: Vec<ExToken> = Vec::new();
    for t in tokens {
        if is_operand(t) && !out.contains(t) {
            out.push(t.clone());
        }
    }
    out
}

/// The **generative** insert-operand vocabulary for a function body: its distinct
/// body operands (as [`distinct_operands`]) **plus** operands that may have
/// *vanished* from the body —
/// 1. every **formal parameter** in scope, as a `Load(t)` (a function's
///    `Formal(t)` header token and its `Load(t)` share the token id `t`, so a
///    param used nowhere in the current body is still loadable); and
/// 2. each literal in `literals`, as a narrow `Lit`.
///
/// This lets a term-insert reconstruct a dropped `+param` or `+k` even though the
/// operand no longer appears in the seed (the drop-term lossy-seed case a
/// reuse-only insert cannot solve). Deduplicated, first-seen order; bounded by
/// arity + `literals.len()` so the branching factor stays sane.
pub(super) fn generative_operands(tokens: &[ExToken], literals: &[i32]) -> Vec<ExToken> {
    let mut out = distinct_operands(tokens);
    let mut add = |t: ExToken| {
        if !out.contains(&t) {
            out.push(t);
        }
    };
    // Params in scope: each Formal(id) is loadable as Load(id).
    for t in tokens {
        if let ExToken::Formal(id) = t {
            add(ExToken::Load(*id));
        }
    }
    // The small "vanished literal" set.
    for &v in literals {
        add(ExToken::Lit {
            value: v,
            wide: !(0..=0x7F).contains(&v),
        });
    }
    out
}
