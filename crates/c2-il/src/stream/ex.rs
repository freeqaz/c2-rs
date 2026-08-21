//! IR0's `.ex` framer and the two views over it.
//!
//! See [`super`] for what this layer is and, more importantly, for what it
//! deliberately is not.

use super::{Extent, Record, RecordKind};
use crate::func::bundle::{bare_body_start, FN_START, LO_MARKER};
use crate::func::readers::memchr_byte;
use crate::IlBundle;

/// The framing of one file.
///
/// # The lifetime rule, stated up front because getting it wrong costs a day
///
/// `bytes` is borrowed from the **bundle** (`'a`), and the views return
/// `&'a [u8]` tied to `'a` and **not** to `&self`. `IlBundle::functions` builds
/// a `Vec<&[u8]>` of segments that outlives the local framing; if the views
/// borrowed from `&self`, every call site would fight the borrow checker and
/// the tempting fix — cloning the segments — would reintroduce exactly the
/// per-obj copy this type exists to avoid.
#[derive(Clone, Debug)]
pub struct FileFraming<'a> {
    /// The file's suffix, e.g. `"ex"`.
    pub suffix: &'a str,
    /// The file's bytes, borrowed from the bundle.
    pub bytes: &'a [u8],
    /// The framing. Tiles `[0, bytes.len())` exactly — see [`Ir0Framing`].
    pub records: Vec<Record>,
    /// **`.ex` only:** every `4F 1F` function-start marker, ascending — the
    /// offsets the `ExFnSegment` records were built from, kept beside them so a
    /// view is a pure function of the index rather than a re-scan.
    pub(crate) starts: Vec<usize>,
}

// **WHY THERE IS NO `los` FIELD HERE, and why the obvious design was WRONG.**
//
// The first version of this type held the `4C 4F 11` body-marker index beside
// `starts`, framed eagerly, on the argument that "one index serves both views"
// — today `split_functions_at` and `split_function_bodies_at` each run their
// own byte walk, and the latter runs *both*.
//
// **That is a performance REGRESSION at seven of the nine gate call sites, and
// it was measured, not reasoned about.** Every gate-side reader
// (`ex_segment_count`, `gl_body_start_coverage`, `selective_bind_coverage`,
// `opt_words`, `functions`, `dyninit_tu`, `provide_data_tu`) wants the `4F 1F`
// split and nothing else, and does exactly ONE scan for it today. An eager
// `los` makes each of them do TWO. `functions()` is on `PortC2::build`'s path
// and paying two scans where one is wanted is a cost with no buyer.
// (This clause used to read "and the port's whole thesis is throughput". That
// thesis was retired on 2026-08-21 — `docs/GOAL_DECISION_2026-08-21.md`. The
// engineering argument is unchanged and never needed it: the sharing bought
// nothing, because **no call site wants both segmentations**, which is the
// sentence this comment closes on.)
//
// So `los` is computed inside [`FileFraming::body_segments`], where it is
// needed and where the incumbent already computes it. Each view then costs
// exactly what its incumbent costs — one scan for the gate, two for the census
// — and the sharing that looked free was buying nothing, because **no call
// site wants both segmentations.**
//
// The lane's plan flagged framing cost as risk F5 and predicted it as "the same
// scans that already happen". It is not: the gate sites do one scan and the
// eager framing does two. Recorded as a plan defect rather than silently fixed.

/// The whole bundle, framed. One [`FileFraming`] per present file, in the
/// bundle's own suffix order.
#[derive(Clone, Debug)]
pub struct Ir0<'a> {
    /// Bundle base, suffix-free, e.g. `_CL_dfd7b253`.
    pub base_name: &'a str,
    /// One framing per present file.
    pub files: Vec<FileFraming<'a>>,
}

impl<'a> Ir0<'a> {
    /// Frame a whole bundle. **Infallible by signature** — there is no refusal
    /// predicate at this layer to get wrong.
    ///
    /// `.gl`, `.sy`, `.in` and `.db` are one [`RecordKind::Opaque`] record each
    /// (IR0 v1's scope; `.gl` framing is IR1). `.ex` is framed by
    /// [`Ir0::frame_ex`].
    pub fn frame(bundle: &'a IlBundle) -> Ir0<'a> {
        let mut files = Vec::with_capacity(bundle.files.len());
        for (suffix, bytes) in &bundle.files {
            files.push(match suffix.as_str() {
                "ex" => Ir0::frame_ex(bytes),
                _ => FileFraming {
                    suffix,
                    bytes,
                    records: whole_file_opaque(bytes),
                    starts: Vec::new(),
                },
            });
            // `frame_ex` cannot know the suffix string's lifetime source, so
            // fix it up here where the map entry is in hand.
            files.last_mut().expect("just pushed").suffix = suffix;
        }
        Ir0 {
            base_name: &bundle.base_name,
            files,
        }
    }

    /// Frame one `.ex` stream. **Infallible.**
    ///
    /// The framing is: an [`RecordKind::Opaque`] head covering `[0, first
    /// 4F 1F)` when that range is non-empty, then one
    /// [`RecordKind::ExFnSegment`] per `4F 1F` running to the next one or to
    /// end of file. A stream with no `4F 1F` at all is one `Opaque` record; an
    /// empty stream is **no records**, which is the one case I1 admits an empty
    /// `records` for.
    ///
    /// The head record is the part neither incumbent splitter owns:
    /// `split_functions_at` covers only `[starts[0], len)` and
    /// `split_function_bodies_at` only `[segs_start[0], len)`. Owning it turns
    /// a scan into a framing.
    pub fn frame_ex(ex: &'a [u8]) -> FileFraming<'a> {
        let starts = fn_start_offsets(ex);

        let mut records = Vec::with_capacity(starts.len() + 1);
        match starts.first() {
            None => records.extend(whole_file_opaque(ex)),
            Some(&first) => {
                if first > 0 {
                    records.push(Record {
                        extent: Extent {
                            start: 0,
                            end: first,
                        },
                        kind: RecordKind::Opaque,
                    });
                }
                for k in 0..starts.len() {
                    let end = starts.get(k + 1).copied().unwrap_or(ex.len());
                    records.push(Record {
                        extent: Extent {
                            start: starts[k],
                            end,
                        },
                        kind: RecordKind::ExFnSegment,
                    });
                }
            }
        }

        FileFraming {
            suffix: "ex",
            bytes: ex,
            records,
            starts,
        }
    }
}

/// One record covering the whole file, or none at all when the file is empty
/// (I1 forbids an empty extent, so an empty file has an empty framing).
fn whole_file_opaque(bytes: &[u8]) -> Vec<Record> {
    if bytes.is_empty() {
        Vec::new()
    } else {
        vec![Record {
            extent: Extent {
                start: 0,
                end: bytes.len(),
            },
            kind: RecordKind::Opaque,
        }]
    }
}

/// Every `4F 1F` offset, ascending. **The same walk the incumbent
/// `split_functions_at` runs** — a match consumes 2 bytes, a miss 1 — because
/// the overlapping-match behaviour is part of the segmentation, not an
/// incidental detail of how it was written.
fn fn_start_offsets(ex: &[u8]) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut i = 0;
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
    starts
}

/// Every `4C 4F 11` offset, ascending — same walk as the incumbent, a match
/// consuming 3 bytes and a miss 1.
fn lo_marker_offsets(ex: &[u8]) -> Vec<usize> {
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
    los
}

impl<'a> FileFraming<'a> {
    /// Whether this framing is of an `.ex` stream, and therefore whether the
    /// two segmentation views mean anything on it.
    ///
    /// **The views are `Option` because of this predicate, not for tidiness.**
    /// See [`Self::gate_segments`].
    pub fn is_ex(&self) -> bool {
        self.suffix == "ex"
    }

    /// **The GATE segmentation** — `Some((offsets, segments))`, byte-identical
    /// to `func::bundle::split_functions_at`. **`None` on any file that is not
    /// `.ex`.**
    ///
    /// One entry per `4F 1F`. The head `Opaque` record is deliberately NOT
    /// included: this view reproduces the incumbent exactly, and the incumbent
    /// has never covered the head. The head is visible in `records` and in the
    /// opaque denominators, which is where a fact about the input belongs.
    ///
    /// # Why this returns `Option` and not a tuple
    ///
    /// [`Ir0::frame`] frames `.gl`/`.sy`/`.in`/`.db` as a single
    /// [`RecordKind::Opaque`] record with an empty `starts` (IR0 v1's scope).
    /// A view that read only `records`/`starts` would hand such a file back
    /// `(vec![], vec![])` — a well-typed, silent **"this file has no
    /// functions"**, indistinguishable from a genuinely function-free `.ex`.
    ///
    /// That is this repo's signature *absence-reads-as-success* shape
    /// (#3219/#3231's family), and the first IR1 caller that iterates
    /// `ir0.files` and asks each one for segments would have got a correct
    /// answer for one file in five and a silent zero for the other four. The
    /// totality controls cannot see it — they check extents, not view
    /// semantics — so the type carries it instead. Found by review of lane
    /// `ir0`, fixed in its fix round.
    pub fn gate_segments(&self) -> Option<(Vec<usize>, Vec<&'a [u8]>)> {
        if !self.is_ex() {
            return None;
        }
        let mut segs = Vec::with_capacity(self.starts.len());
        for r in &self.records {
            if r.kind == RecordKind::ExFnSegment {
                segs.push(&self.bytes[r.extent.start..r.extent.end]);
            }
        }
        Some((self.starts.clone(), segs))
    }

    /// **The CENSUS segmentation** — `Some((offsets, segments))`,
    /// byte-identical to `func::bundle::split_function_bodies_at`. **`None` on
    /// any file that is not `.ex`**, for the reason spelled out on
    /// [`Self::gate_segments`].
    ///
    /// Anchored on the `LO` body marker, not on `4F 1F`, plus the strictly
    /// additive bare-`4C` second pass. **It is not a refinement of
    /// [`Self::gate_segments`] and its boundaries need not be `4F 1F`
    /// offsets** — when a `4F 1F` start would be reused by two bodies, the
    /// later body starts at its own `LO`. That is why the two views are two,
    /// and why IR0 holds a marker index rather than trying to express one
    /// segmentation in terms of the other.
    pub fn body_segments(&self) -> Option<(Vec<usize>, Vec<&'a [u8]>)> {
        if !self.is_ex() {
            return None;
        }
        let ex = self.bytes;
        // Scanned HERE, not in `frame_ex`: see the note beside `FileFraming`.
        // The gate view must not pay for an index only the census view reads.
        let mut los = lo_marker_offsets(ex);
        let starts = &self.starts;

        // The strictly-additive bare-`4C` pass, per `4F 1F` region that holds
        // no composed marker (ROADMAP §10.12). Both lists are ascending, so
        // "does this region already hold a body marker" is a binary search.
        let mut extra: Vec<usize> = Vec::new();
        for (k, &s) in starts.iter().enumerate() {
            let e = starts.get(k + 1).copied().unwrap_or(ex.len());
            let idx = los.partition_point(|&l| l < s);
            if los.get(idx).is_some_and(|&l| l < e) {
                continue;
            }
            if let Some(p) = bare_body_start(&ex[s..e]) {
                extra.push(s + p);
            }
        }
        if !extra.is_empty() {
            los.extend(extra);
            los.sort_unstable();
        }

        if los.is_empty() {
            // An `.ex` with no body marker genuinely has no census segments.
            // This empty is a MEASURED zero about an `.ex`, which is exactly
            // what `None` above is reserved to distinguish it from.
            return Some((Vec::new(), Vec::new()));
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
        let segs = (0..segs_start.len())
            .map(|k| {
                let start = segs_start[k];
                let end = segs_start.get(k + 1).copied().unwrap_or(ex.len());
                &ex[start..end.max(start)]
            })
            .collect();
        Some((segs_start, segs))
    }

    /// `(framed, opaque)` byte counts. `framed + opaque == bytes.len()` in a
    /// well-formed framing; the identity is the `ir0-accounting-broken`
    /// control's whole content.
    pub fn byte_split(&self) -> (usize, usize) {
        let mut opaque = 0usize;
        let mut framed = 0usize;
        for r in &self.records {
            match r.kind {
                RecordKind::Opaque => opaque += r.extent.len(),
                RecordKind::ExFnSegment => framed += r.extent.len(),
            }
        }
        (framed, opaque)
    }
}

/// Why a framing is not well formed. Every variant is an **IR0 defect**, never
/// a statement about the input: totality is by construction, so a value of this
/// type means the framer is wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ir0Broken {
    /// I1: the first record does not start at 0.
    HeadNotAtZero { first_start: usize },
    /// I1: a gap or an overlap between two adjacent records.
    NotContiguous {
        index: usize,
        prev_end: usize,
        next_start: usize,
    },
    /// I1: the last record does not reach end of file.
    TailShort { last_end: usize, len: usize },
    /// I1: an empty extent. A framing must not contain records covering nothing.
    EmptyExtent { index: usize },
    /// I1: a non-empty file framed into no records at all.
    NoRecords { len: usize },
    /// I2: re-serialization differs. Cannot happen when I1 holds, and is
    /// computed anyway as the #3288 second derivation.
    Reserialized { first_offset: usize, got_len: usize, len: usize },
}

/// The invariant checker. **For tests and instruments only — never on the emit
/// path.**
pub trait Ir0Framing {
    /// Check I1 (indices) and I2 (bytes), in that order, as two separately
    /// built computations. See [`super`] for why that is one invariant twice
    /// and not two invariants.
    fn verify(&self) -> Result<(), Ir0Broken>;
}

impl Ir0Framing for FileFraming<'_> {
    fn verify(&self) -> Result<(), Ir0Broken> {
        // ---- I1: an INDEX claim. Nothing here reads a byte of `self.bytes`.
        if self.records.is_empty() {
            if self.bytes.is_empty() {
                return Ok(());
            }
            return Err(Ir0Broken::NoRecords {
                len: self.bytes.len(),
            });
        }
        if self.records[0].extent.start != 0 {
            return Err(Ir0Broken::HeadNotAtZero {
                first_start: self.records[0].extent.start,
            });
        }
        for (i, r) in self.records.iter().enumerate() {
            if r.extent.is_empty() {
                return Err(Ir0Broken::EmptyExtent { index: i });
            }
            if let Some(next) = self.records.get(i + 1) {
                if r.extent.end != next.extent.start {
                    return Err(Ir0Broken::NotContiguous {
                        index: i,
                        prev_end: r.extent.end,
                        next_start: next.extent.start,
                    });
                }
            }
        }
        let last_end = self.records[self.records.len() - 1].extent.end;
        if last_end != self.bytes.len() {
            return Err(Ir0Broken::TailShort {
                last_end,
                len: self.bytes.len(),
            });
        }

        // ---- I2: a BYTE claim, built from the bytes rather than the indices.
        let mut out: Vec<u8> = Vec::with_capacity(self.bytes.len());
        for r in &self.records {
            out.extend_from_slice(&self.bytes[r.extent.start..r.extent.end]);
        }
        if out != self.bytes {
            let first_offset = out
                .iter()
                .zip(self.bytes.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(out.len().min(self.bytes.len()));
            return Err(Ir0Broken::Reserialized {
                first_offset,
                got_len: out.len(),
                len: self.bytes.len(),
            });
        }
        Ok(())
    }
}

impl Ir0Framing for Ir0<'_> {
    fn verify(&self) -> Result<(), Ir0Broken> {
        for f in &self.files {
            f.verify()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::bundle::{split_function_bodies_at, split_functions_at};

    /// Both views equal the incumbent splitters, and the framing is total.
    /// Called by every case below, so a case is three assertions and not one.
    fn check(ex: &[u8]) -> FileFraming<'_> {
        let f = Ir0::frame_ex(ex);
        f.verify().expect("IR0 framing must be total");
        let (gs, gsegs) = f.gate_segments().expect("an .ex framing has a gate view");
        let (is, isegs) = split_functions_at(ex);
        assert_eq!(gs, is, "gate offsets differ");
        assert_eq!(gsegs, isegs, "gate segments differ");
        let (bs, bsegs) = f.body_segments().expect("an .ex framing has a body view");
        let (ibs, ibsegs) = split_function_bodies_at(ex);
        assert_eq!(bs, ibs, "body offsets differ");
        assert_eq!(bsegs, ibsegs, "body segments differ");
        f
    }

    #[test]
    fn empty_file_frames_to_no_records_and_verifies() {
        let f = check(&[]);
        assert!(f.records.is_empty());
        assert_eq!(f.byte_split(), (0, 0));
    }

    #[test]
    fn a_file_with_no_marker_is_one_opaque_record() {
        let ex = [0u8, 1, 2, 3, 4, 5, 6, 7];
        let f = check(&ex);
        assert_eq!(f.records.len(), 1);
        assert_eq!(f.records[0].kind, RecordKind::Opaque);
        assert_eq!(f.records[0].extent, Extent { start: 0, end: 8 });
        // Every byte is opaque, and NOT ONE of them is a refusal: the incumbent
        // splitter returns an empty vec here and never says so.
        assert_eq!(f.byte_split(), (0, 8));
        assert!(f.gate_segments().expect(".ex").1.is_empty());
    }

    #[test]
    fn a_marker_at_offset_zero_leaves_no_opaque_head() {
        let ex = [0x4F, 0x1F, 0xAA, 0xBB];
        let f = check(&ex);
        assert_eq!(f.records.len(), 1);
        assert_eq!(f.records[0].kind, RecordKind::ExFnSegment);
        assert_eq!(f.byte_split(), (4, 0));
    }

    #[test]
    fn a_marker_after_a_head_owns_the_head_as_opaque() {
        let ex = [0x11, 0x22, 0x33, 0x4F, 0x1F, 0xAA];
        let f = check(&ex);
        assert_eq!(f.records.len(), 2);
        assert_eq!(f.records[0].kind, RecordKind::Opaque);
        assert_eq!(f.records[0].extent, Extent { start: 0, end: 3 });
        assert_eq!(f.records[1].kind, RecordKind::ExFnSegment);
        assert_eq!(f.byte_split(), (3, 3));
    }

    /// The `i + 1 < ex.len()` boundary in the incumbent walk: a lone `4F` in
    /// the last byte position is not a marker, so the whole file is opaque.
    #[test]
    fn a_truncated_marker_at_end_of_file_is_not_a_marker() {
        let ex = [0x11, 0x22, 0x4F];
        let f = check(&ex);
        assert_eq!(f.records.len(), 1);
        assert_eq!(f.records[0].kind, RecordKind::Opaque);
        assert_eq!(f.byte_split(), (0, 3));
    }

    /// `4F 1F` at `len - 2` IS a marker and opens a two-byte segment.
    #[test]
    fn a_marker_in_the_last_two_bytes_opens_a_segment() {
        let ex = [0x11, 0x4F, 0x1F];
        let f = check(&ex);
        assert_eq!(f.records.len(), 2);
        assert_eq!(f.records[1].extent, Extent { start: 1, end: 3 });
    }

    /// **Two `4F 1F` markers — the only portable case with an ADJACENCY to
    /// check.** Found by the fix round's mutation run: every other case here
    /// frames to at most two records, so `Ir0Broken::NotContiguous` — the I1
    /// clause about the seam BETWEEN two records — was unreachable by the
    /// portable set, and a mutant that corrupts a span could only ever be
    /// caught as `TailShort`. A clause no case can reach is a clause nobody
    /// has tested.
    #[test]
    fn two_markers_put_a_seam_between_two_framed_records() {
        let ex = [0x11, 0x4F, 0x1F, 0xAA, 0xBB, 0x4F, 0x1F, 0xCC];
        let f = check(&ex);
        assert_eq!(f.records.len(), 3, "opaque head + two segments");
        assert_eq!(f.records[0].extent, Extent { start: 0, end: 1 });
        assert_eq!(f.records[1].extent, Extent { start: 1, end: 5 });
        assert_eq!(f.records[2].extent, Extent { start: 5, end: 8 });
        // The seam itself: record 1 ends exactly where record 2 begins. This is
        // what `NotContiguous` guards and what nothing else here exercises.
        assert_eq!(f.records[1].extent.end, f.records[2].extent.start);
        assert_eq!(f.byte_split(), (7, 1));
        assert_eq!(f.gate_segments().expect(".ex").0, vec![1, 5]);
    }

    /// Overlapping candidates: `4F 4F 1F`. The incumbent walk consumes 1 byte
    /// on a miss, so the marker at offset 1 is found. The framing must agree,
    /// because which of the two is the marker changes every downstream offset.
    #[test]
    fn an_overlapping_candidate_resolves_the_way_the_incumbent_walk_does() {
        let ex = [0x4F, 0x4F, 0x1F, 0x00];
        let f = check(&ex);
        assert_eq!(f.starts, vec![1]);
        assert_eq!(f.records[0].extent, Extent { start: 0, end: 1 });
    }

    /// Two body markers under ONE `4F 1F`. The census view must put the second
    /// body at its own `LO` rather than reusing the start — the rule at
    /// `bundle.rs:571`, which is the single likeliest thing a re-expression
    /// breaks.
    #[test]
    fn two_bodies_sharing_one_start_do_not_reuse_it() {
        let mut ex = vec![0x4F, 0x1F, 0x00, 0x00];
        ex.extend_from_slice(&LO_MARKER);
        ex.extend_from_slice(&[0x00, 0x00]);
        ex.extend_from_slice(&LO_MARKER);
        ex.extend_from_slice(&[0x00]);
        let f = check(&ex);
        let (bs, _) = f.body_segments().expect(".ex");
        assert_eq!(bs.len(), 2, "two bodies");
        assert_eq!(bs[0], 0, "first body keeps the 4F 1F start");
        assert_eq!(bs[1], 9, "second body starts at its own LO, not at 0");
        // And the GATE view still sees exactly one segment. This is the
        // disagreement the two splitters are pinned apart on; a refactor that
        // made these agree would be the flattering failure.
        assert_eq!(f.gate_segments().expect(".ex").0.len(), 1);
    }

    /// The two views disagreeing is a PROPERTY, asserted here so a future
    /// unification cannot land quietly (C1's shape, local to this module).
    #[test]
    fn the_two_views_are_not_the_same_segmentation() {
        let mut ex = vec![0x4F, 0x1F, 0x00, 0x00];
        ex.extend_from_slice(&LO_MARKER);
        ex.extend_from_slice(&[0x00, 0x00]);
        ex.extend_from_slice(&LO_MARKER);
        ex.extend_from_slice(&[0x00]);
        let f = Ir0::frame_ex(&ex);
        assert_ne!(
            f.gate_segments().expect(".ex").0.len(),
            f.body_segments().expect(".ex").0.len(),
            "IR0's two views must remain two segmentations"
        );
    }

    /// A `4F 1F` region carrying no composed `4C 4F 11` but a grammar-gated
    /// bare `4C` — the `??__E`/`??__F` thunk class. The additive second pass
    /// must find it through the view exactly as the incumbent does.
    #[test]
    fn a_bare_4c_thunk_region_is_found_by_the_additive_pass() {
        // `4F 1F` .. `53 53` `46` `4C 53` — the prefix grammar
        // `bare_lo_after_prefix` walks.
        let ex = vec![0x4F, 0x1F, 0x53, 0x53, 0x46, 0x4C, 0x53, 0x00];
        let f = check(&ex);
        let (bs, _) = f.body_segments().expect(".ex");
        assert_eq!(bs, vec![0], "the thunk's segment starts at its 4F 1F");
        // The incumbent sees it too — `check` already asserted equality, so
        // this pins that the case is NOT vacuous.
        assert_eq!(split_function_bodies_at(&ex).0.len(), 1);
    }

    /// **A view on a non-`.ex` file must be `None`, not an empty answer.**
    ///
    /// The bytes below are a `.gl` that CONTAINS a `4F 1F` and a `4C 4F 11`,
    /// so the failure this pins is not "there was nothing to find": IR0 v1
    /// frames `.gl` as one `Opaque` record on purpose, and a view that read
    /// only `records`/`starts` would answer `(vec![], vec![])` — the silent
    /// zero. `None` is the only answer that a caller cannot mistake for a
    /// measurement.
    ///
    /// Review finding of lane `ir0`; this test is the criterion, and it is red
    /// against the pre-fix views.
    #[test]
    fn the_views_refuse_a_file_that_is_not_ex() {
        let mut bundle = crate::IlBundle::new("_CL_test");
        let gl = vec![0x4F, 0x1F, 0x00, 0x4C, 0x4F, 0x11, 0x00];
        bundle.files.insert("gl".to_string(), gl.clone());
        bundle.files.insert("ex".to_string(), gl.clone());

        let ir0 = Ir0::frame(&bundle);
        assert_eq!(ir0.files.len(), 2);
        for f in &ir0.files {
            // Totality holds on BOTH files — this is not a framing failure.
            f.verify().expect("every file is framed totally");
            if f.suffix == "ex" {
                assert!(f.is_ex());
                assert_eq!(
                    f.gate_segments().expect("the .ex has a gate view").0,
                    vec![0],
                    "the same bytes DO segment when they are the .ex"
                );
                assert!(f.body_segments().is_some());
            } else {
                assert!(!f.is_ex());
                assert_eq!(f.records.len(), 1, ".gl is one opaque record in IR0 v1");
                assert!(
                    f.gate_segments().is_none(),
                    "a .gl must not answer the gate view with a silent empty"
                );
                assert!(
                    f.body_segments().is_none(),
                    "a .gl must not answer the body view with a silent empty"
                );
            }
        }
    }

    /// A file whose only `LO` precedes every `4F 1F`: `partition_point == 0`,
    /// so the census segment starts at the `LO` itself.
    #[test]
    fn a_body_marker_before_any_start_anchors_on_itself() {
        let mut ex = Vec::new();
        ex.extend_from_slice(&LO_MARKER);
        ex.extend_from_slice(&[0x00, 0x4F, 0x1F, 0x00]);
        let f = check(&ex);
        assert_eq!(f.body_segments().expect(".ex").0, vec![0]);
        assert_eq!(f.gate_segments().expect(".ex").0, vec![4]);
    }
}
