//! Test-only `IlFunction` builders shared by the per-module test modules.
//!
//! `func_with` was one helper inside the single 1,735-line `mod tests`; the
//! straight-line, tail-call, FP and mode tests all build their inputs with it.
//! It is here, once, rather than copied into each — the split is the easiest
//! moment in this project to commit the "two copies of one fact" defect
//! (`docs/ARCHITECTURE_SEAMS.md` §4.3).

#![cfg(test)]

#[allow(unused_imports)]
use c2_il::{IlFunction, IlOp};

pub(crate) fn func_with(params: Vec<u32>, ops: Vec<IlOp>) -> IlFunction {
    IlFunction {
            inlinable: None,
            mangled_name: "?f@@YAHH@Z".into(),
            source_path: None,
            data_syms: Vec::new(),
            fn_addr_sym: None,
            data_def: None,
            params,
            ops,
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
            body: c2_il::BodyShape::Plain,
        }
}
