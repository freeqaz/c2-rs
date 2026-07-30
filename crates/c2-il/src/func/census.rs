use super::body::{parse_segment_detail, BodyShape};
use super::bundle::mangled_is_varargs;
use super::bundle::split_function_bodies;
use super::gl::mangled_names;
use super::sy::SyLocals;
use super::Block;
use crate::IlBundle;

/// Split the `.ex` stream into per-function byte segments at each `4F 1F`
/// function-start marker. Segment `k` runs from marker `k` to marker `k+1`
/// (the last to end-of-stream).
/// One function's census verdict (P2b). Either the modeled shape it parsed as,
/// or the first feature that blocked it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FnVerdict {
    /// Parsed as a modeled shape. The string is a stable shape label
    /// (`straight-line`, `void-tail-call`, `int-tail-call`, `framed-call`).
    InClass(&'static str),
    /// Blocked at the first unmodeled feature.
    Blocked(Block),
}

impl FnVerdict {
    /// The census bucket key: the shape label when in class, else the blocking
    /// feature (see [`Block::feature`]).
    pub fn key(&self) -> String {
        match self {
            FnVerdict::InClass(s) => (*s).to_string(),
            FnVerdict::Blocked(b) => b.feature(),
        }
    }
    pub fn in_class(&self) -> bool {
        matches!(self, FnVerdict::InClass(_))
    }
}

/// One census row: a function segment and how it classified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FnCensus {
    /// Index of the function within the TU (`.ex` segment order).
    pub index: usize,
    /// Mangled name, when `.gl` has one at this position.
    pub name: Option<String>,
    /// Segment length in bytes (a rough proxy for function size).
    pub seg_len: usize,
    pub verdict: FnVerdict,
    /// Raw bytes around the blocking site, for grammar work: the segment window
    /// `[off - CENSUS_HEX_BACK, off + CENSUS_HEX_FWD)` clamped to the segment,
    /// plus the index of the blocking byte within that window. Empty when the
    /// function is in class.
    pub hex: Vec<u8>,
    /// Index of the blocking byte inside [`FnCensus::hex`].
    pub hex_mark: usize,
}

/// Bytes of context kept before / after a blocking site.
pub const CENSUS_HEX_BACK: usize = 16;
pub const CENSUS_HEX_FWD: usize = 24;

impl IlBundle {
    /// **Function-level census (P2b).** Classify *every* function in the bundle
    /// independently, so a TU whose 700th function uses an unmodeled opcode
    /// still reports the other 699 as in-class.
    ///
    /// This is the measurement [`IlBundle::functions`] cannot give: that method
    /// is all-or-nothing per TU (correctly — the port must emit a whole obj or
    /// nothing), so over a real workload it reports one `vocab-gap` per TU and
    /// cannot rank the missing classes. The census runs the *same*
    /// [`parse_segment_detail`] per segment and keeps the first blocking
    /// feature, so the histogram of [`FnVerdict::key`] over a corpus is the
    /// widening order (docs/ROADMAP.md §G5).
    ///
    /// Diagnostic only — never a gate, and never consulted by the emitter.
    /// Returns `None` only when the bundle lacks the required files.
    pub fn function_census(&self) -> Option<Vec<FnCensus>> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;
        let names = mangled_names(gl);
        let segs = split_function_bodies(ex);
        // Names are paired positionally, which is only meaningful when `.gl`
        // yields exactly one name per body. On a real TU `mangled_names` finds
        // far fewer (it accepts only `?…@@…` forms, and `.gl` also lists
        // externals), so pairing there would attach wrong names to functions —
        // report none rather than a plausible-looking lie.
        let paired = names.len() == segs.len();
        // `.gl` is deliberately NOT threaded into the body parse. The assignment
        // class used to decide "is this destination a global?" by asking whether
        // `.gl` named it, and that was wrong (a file-scope `static` is `$sv`, which
        // the index does not accept as an identifier). The symbol view locals needed
        // turned out to be `.sy`, not `.gl`, so the vestigial `.gl` thread is gone
        // rather than left in place looking load-bearing. Do not restore the
        // absence test.
        //
        // Same `.sy` binding as the gate, built from the same segment list, so the
        // census cannot report a function in class that `IlBundle::functions` would
        // refuse for want of a local — or the reverse.
        let locals = SyLocals::new(self.get("sy"), &segs);
        Some(
            segs.iter()
                .enumerate()
                .map(|(i, seg)| {
                    // A variadic function is refused on its NAME, because its body
                    // IL is byte-identical to a non-variadic twin's — see
                    // [`super::bundle::mangled_is_varargs`], which is the same
                    // predicate `functions` applies, so the census and the gate
                    // cannot disagree.
                    //
                    // Only when the names are `paired`. Unpaired means the census
                    // has no name for this segment, and reporting the body's real
                    // blocker is better than inventing a reason: `functions`
                    // refuses that whole TU for want of names anyway, so nothing
                    // here can be emitted either way.
                    let varargs = paired
                        && names.get(i).is_some_and(|n| mangled_is_varargs(n));
                    let verdict = if varargs {
                        FnVerdict::Blocked(Block {
                            ctx: "fn-varargs",
                            byte: None,
                            off: 0,
                            aux: 0,
                        })
                    } else {
                        match parse_segment_detail(seg, locals.view(i)) {
                            Ok(BodyShape::StraightLine { .. }) => FnVerdict::InClass("straight-line"),
                            Ok(BodyShape::VoidTailCall { .. }) => FnVerdict::InClass("void-tail-call"),
                            // Emits exactly what a void tail call emits, but gets
                            // its own bucket so the movement out of
                            // `expr-call-in-expr` is attributable.
                            Ok(BodyShape::EmptyDtorDelegation { .. }) => {
                                FnVerdict::InClass("empty-dtor-delegation")
                            }
                            Ok(BodyShape::IntTailCall { .. }) => FnVerdict::InClass("int-tail-call"),
                            Ok(BodyShape::MultiArgTailCall { .. }) => {
                                FnVerdict::InClass("multiarg-tail-call")
                            }
                            Ok(BodyShape::FramedCall { .. }) => FnVerdict::InClass("framed-call"),
                            Ok(BodyShape::Compare(_)) => FnVerdict::InClass("compare-leaf"),
                            Ok(BodyShape::EmptyBody) => FnVerdict::InClass("empty-body"),
                            Ok(BodyShape::IndirectLoad { .. }) => {
                                FnVerdict::InClass("indirect-load-leaf")
                            }
                            Ok(BodyShape::FloatLeaf { double, .. }) => {
                                FnVerdict::InClass(if double { "double-leaf" } else { "float-leaf" })
                            }
                            Err(b) => FnVerdict::Blocked(b),
                        }
                    };
                    // Keep the raw bytes around the blocking site: decoding a new
                    // grammar production always starts by staring at exactly this
                    // window, and having it in the census means that work is a
                    // report away instead of a one-off script.
                    let (hex, hex_mark) = match &verdict {
                        FnVerdict::InClass(_) => (Vec::new(), 0),
                        FnVerdict::Blocked(b) => {
                            let start = b.off.saturating_sub(CENSUS_HEX_BACK);
                            let end = (b.off + CENSUS_HEX_FWD).min(seg.len());
                            let start = start.min(end);
                            (seg[start..end].to_vec(), b.off - start)
                        }
                    };
                    FnCensus {
                        index: i,
                        name: if paired { names.get(i).cloned() } else { None },
                        seg_len: seg.len(),
                        verdict,
                        hex,
                        hex_mark,
                    }
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::bundle::FN_START;
    use crate::func::test_fixtures::*;

    #[test]
    fn census_classifies_each_function_independently() {
        // The point of P2b: one blocked function does not hide the in-class
        // ones. `functions()` (the gate) is all-or-nothing and returns None.
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&FN_START);
        ex.extend_from_slice(MVP_CALL);
        ex.extend_from_slice(&FN_START);
        ex.extend_from_slice(TWO_CALLS);
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![
                ("ex".to_string(), ex),
                ("gl".to_string(), b"?f@@YAXXZ\x00?g@@YAXXZ\x00".to_vec()),
            ]
            .into_iter()
            .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 2);
        assert_eq!(census[0].verdict, FnVerdict::InClass("void-tail-call"));
        assert!(!census[1].verdict.in_class());
        assert_eq!(census[0].name.as_deref(), Some("?f@@YAXXZ"));
        // In-class functions carry no hex window; blocked ones point at the
        // offending byte inside theirs.
        assert!(census[0].hex.is_empty());
        let FnVerdict::Blocked(b) = census[1].verdict else {
            panic!("expected a block");
        };
        assert_eq!(census[1].hex[census[1].hex_mark], b.byte.unwrap());
    }

    #[test]
    fn census_hex_window_is_clamped_to_the_segment() {
        // A block at offset 0 must not underflow, and one near the end must not
        // run past it — the window is diagnostic and must never panic.
        let tiny: &[u8] = &[0x4C, 0x4F, 0x11, 0xFF];
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![
                ("ex".to_string(), tiny.to_vec()),
                ("gl".to_string(), b"?f@@YAXXZ\x00".to_vec()),
            ]
            .into_iter()
            .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 1);
        let c = &census[0];
        assert!(!c.verdict.in_class());
        assert!(c.hex_mark < c.hex.len().max(1));
        assert!(c.hex.len() <= CENSUS_HEX_BACK + CENSUS_HEX_FWD);
    }
}
