use c2_il::{ExToken, IlModel};

use super::moves::{distinct_operands, is_binop, is_operand};

// ===========================================================================
// Solvable-instance harness — the honest solve-rate
// ===========================================================================

/// One perturbation family used to build a solvable instance from a solution IL.
///
/// Each family's inverse is in the [`MoveSet`], so a byte-exact IL is reachable
/// by construction — a failure is a real *search* failure. Note that `WidenLit`
/// is **obj-invisible** on the real toolchain (P0.6a A: c2 re-optimizes a
/// widened literal to byte-identical code), so it is a valid perturbation only in
/// `.ex`-space (the mock scorer / unit tests); the real solve-rate roster uses
/// the obj-changing families (`AddTerm`, `LitNudge`, `DropTerm`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Perturb {
    /// Widen a narrow literal — `.ex`-visible but obj-invisible (see above).
    WidenLit,
    /// Insert `d` redundant `+<operand>` terms (seed longer; recover by `d`
    /// deletes — a genuine obj change, gradient-guided for `d ≥ 2`).
    AddTerm,
    /// Nudge a literal by `+3d` (seed's immediate differs; recover by a value
    /// move — flat gradient, so recovery is enumeration within the nudge window).
    LitNudge,
    /// Delete a trailing term (seed shorter; recover by insert — only where the
    /// dropped operand survives elsewhere in the body).
    DropTerm,
}

impl Perturb {
    /// A short name for logs/readouts.
    pub fn label(&self) -> &'static str {
        match self {
            Perturb::WidenLit => "widen-lit",
            Perturb::AddTerm => "add-term",
            Perturb::LitNudge => "lit-nudge",
            Perturb::DropTerm => "drop-term",
        }
    }
}

/// Apply a `d`-step perturbation to `solution`, returning the seed model, or
/// `None` if there is no site (the instance is skipped, never faked). `d` is the
/// perturbation distance: `d` stacked edits, whose `d`-move inverse the climber
/// must find.
pub fn perturb(solution: &IlModel, kind: Perturb, d: usize) -> Option<IlModel> {
    let mut m = solution.clone();
    for _ in 0..d.max(1) {
        m = perturb_step(&m, kind)?;
    }
    // A perturbation that produced no net `.ex` change is not a real instance.
    if m.encode().get("ex") == solution.encode().get("ex") {
        return None;
    }
    Some(m)
}

/// A d=1 perturbation (one edit). Retained for the unit tests.
pub fn perturb_once(solution: &IlModel, kind: Perturb) -> Option<IlModel> {
    perturb(solution, kind, 1)
}

fn perturb_step(model: &IlModel, kind: Perturb) -> Option<IlModel> {
    let nfns = model.ex_function_count();
    for fi in 0..nfns {
        let Ok(tokens) = model.function_tokens(fi) else {
            continue;
        };
        match kind {
            Perturb::WidenLit => {
                for (ti, tok) in tokens.iter().enumerate() {
                    if matches!(tok, ExToken::Lit { wide: false, .. }) {
                        let mut m = model.clone();
                        if m.set_literal_wide(fi, ti, true).is_ok() {
                            return Some(m);
                        }
                    }
                }
            }
            Perturb::AddTerm => {
                // Duplicate the first operand as a `+<operand>` term after the
                // first binop — a redundant term the climber removes by delete.
                let operands = distinct_operands(&tokens);
                let first_op = tokens.iter().position(is_binop);
                if let (Some(operand), Some(i)) = (operands.first(), first_op) {
                    let mut m = model.clone();
                    let repl = vec![operand.clone(), ExToken::Add];
                    if m.splice_function_tokens(fi, i + 1..i + 1, repl).is_ok() {
                        return Some(m);
                    }
                }
            }
            Perturb::LitNudge => {
                for (ti, tok) in tokens.iter().enumerate() {
                    if let ExToken::Lit { value, wide } = *tok {
                        let v = value.wrapping_add(3);
                        let mut m = model.clone();
                        let want_wide = wide || !(0..=0x7F).contains(&v);
                        let repl = vec![ExToken::Lit { value: v, wide: want_wide }];
                        if m.splice_function_tokens(fi, ti..ti + 1, repl).is_ok() {
                            return Some(m);
                        }
                    }
                }
            }
            Perturb::DropTerm => {
                for i in 0..tokens.len().saturating_sub(1) {
                    if is_operand(&tokens[i]) && is_binop(&tokens[i + 1]) {
                        let mut m = model.clone();
                        if m.splice_function_tokens(fi, i..i + 2, vec![]).is_ok() {
                            return Some(m);
                        }
                    }
                }
            }
        }
    }
    None
}
