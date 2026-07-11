//! P1.3 (angle F) — the obj→IL **retrieval baseline**.
//!
//! Given a query `.obj`, rank the corpus's other objs by an obj-derived
//! similarity feature and check how often the *true* IL — or an IL that
//! compiles to a behaviorally identical obj — sits in the top-k. This is the
//! reference number every learned obj→IL approach (angle B LoRA, angle C
//! IL-space search) must beat before its verdict means anything.
//!
//! ## Honesty notes (why the ground truth is `.text`, not `obj_sha256_norm`)
//!
//! The task keys "correct retrieval" on obj-equivalence: credit *any* corpus IL
//! whose obj is byte-identical to the query's, not only the exact source
//! (commutative / reassociation variants compile to the same code and must not
//! be scored as misses). The manifest's `obj_sha256_norm` cannot serve as that
//! key here: the captured obj embeds its per-triple `/Fo` output path in
//! `S_OBJNAME` (`…\triples\tNNNNN\_work\out.obj`), so **every** triple's full
//! obj is unique *by path*, even two byte-identical programs — `obj_sha256_norm`
//! collapses to all-singletons and would score every behavioral twin as a miss
//! (the exact per-root-path pollution flagged in ROADMAP P1.3). The behavioral
//! signal that is path-free is the COFF **`.text` section** (the emitted code).
//! So obj-equivalence is keyed on `sha256(.text)`; the strict full-obj
//! (`obj_sha256_norm`) recall is reported too, and is 0 by construction.
//!
//! ## Feature (obj-derived only — the query's IL is never read)
//!
//! An L1-normalized 256-bin histogram of the `.text` bytes, ranked by cosine
//! similarity. Bag-of-bytes NN: a classic, honest baseline — no training. Two
//! objs with identical `.text` have identical histograms (cosine 1.0), so a
//! behavioral twin present in the index is retrieved at rank 1; the number the
//! baseline reports is therefore essentially the corpus's **code-collision
//! coverage**, which is the honest ceiling of lookup-based retrieval.

use std::path::Path;

use crate::corpus::{self, sha256_hex};

/// One corpus row reduced to its obj-derived feature + ground-truth keys.
pub struct Item {
    pub id: String,
    /// `source_sha256` — exact-source ground truth.
    pub src_key: String,
    /// `sha256(.text)` — behavioral (path-free) obj-equivalence ground truth.
    pub text_key: String,
    /// `obj_sha256_norm` — strict full-obj ground truth (path-polluted here).
    pub full_key: String,
    /// L1-normalized 256-bin `.text` byte histogram (the NN feature).
    pub hist: Vec<f32>,
    /// L2 norm of `hist`, cached for cosine.
    pub norm: f32,
    pub text_len: usize,
    pub nsym: u32,
    pub obj_len: usize,
}

/// Which ground-truth relation counts a candidate as a correct retrieval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroundTruth {
    /// Candidate is the same source (`source_sha256`).
    ExactSource,
    /// Candidate emits byte-identical `.text` (behavioral obj-equivalence).
    ObjText,
    /// Candidate has the same `obj_sha256_norm` (strict full obj).
    ObjFull,
}

impl Item {
    fn key(&self, gt: GroundTruth) -> &str {
        match gt {
            GroundTruth::ExactSource => &self.src_key,
            GroundTruth::ObjText => &self.text_key,
            GroundTruth::ObjFull => &self.full_key,
        }
    }
}

// ------------------------------------------------------------------------
// Feature extraction (minimal COFF `.text` reader — c2-obj stays read-only)
// ------------------------------------------------------------------------

/// The `.text` section bytes and the COFF symbol count of a (normalized) obj.
/// Returns `(&[], 0)` on anything too short/malformed to parse — a degenerate
/// item that cosine-scores 0 against everything (never a false twin).
pub fn text_section(obj: &[u8]) -> (&[u8], u32) {
    if obj.len() < 20 {
        return (&[], 0);
    }
    let nsec = u16::from_le_bytes([obj[2], obj[3]]) as usize;
    let nsym = u32::from_le_bytes([obj[12], obj[13], obj[14], obj[15]]);
    let opt = u16::from_le_bytes([obj[16], obj[17]]) as usize;
    let mut off = 20 + opt;
    for _ in 0..nsec {
        if off + 40 > obj.len() {
            break;
        }
        let name = &obj[off..off + 8];
        let trimmed = match name.iter().position(|&b| b == 0) {
            Some(p) => &name[..p],
            None => name,
        };
        let rawsize = u32::from_le_bytes([obj[off + 16], obj[off + 17], obj[off + 18], obj[off + 19]])
            as usize;
        let rawptr = u32::from_le_bytes([obj[off + 20], obj[off + 21], obj[off + 22], obj[off + 23]])
            as usize;
        if trimmed == b".text" && rawptr != 0 && rawptr + rawsize <= obj.len() {
            return (&obj[rawptr..rawptr + rawsize], nsym);
        }
        off += 40;
    }
    (&[], nsym)
}

/// L1-normalized 256-bin byte histogram of `bytes` and its L2 norm.
pub fn byte_histogram(bytes: &[u8]) -> (Vec<f32>, f32) {
    let mut counts = [0u32; 256];
    for &b in bytes {
        counts[b as usize] += 1;
    }
    let total: f32 = bytes.len().max(1) as f32;
    let hist: Vec<f32> = counts.iter().map(|&c| c as f32 / total).collect();
    let norm = hist.iter().map(|x| x * x).sum::<f32>().sqrt();
    (hist, norm)
}

/// Build one [`Item`] from an obj's (normalized) bytes and its ground-truth keys.
pub fn item_from_obj(
    id: String,
    obj: &[u8],
    src_key: String,
    full_key: String,
) -> Item {
    let (text, nsym) = text_section(obj);
    let text_key = sha256_hex(text);
    let (hist, norm) = byte_histogram(text);
    Item {
        id,
        src_key,
        text_key,
        full_key,
        hist,
        norm,
        text_len: text.len(),
        nsym,
        obj_len: obj.len(),
    }
}

/// Load every `ok` triple in `root` as a retrieval [`Item`].
pub fn load_items(root: &Path) -> std::io::Result<Vec<Item>> {
    let rows = corpus::load_manifest(root)?;
    let mut items = Vec::new();
    for r in rows {
        if r.status != "ok" {
            continue;
        }
        let (Some(obj_rel), Some(src_rel), Some(full_key)) =
            (r.obj_rel, r.source_rel, r.obj_sha256_norm)
        else {
            continue;
        };
        let obj = std::fs::read(root.join(&obj_rel))?;
        let src = std::fs::read(root.join(&src_rel))?;
        let src_key = sha256_hex(&src);
        items.push(item_from_obj(r.id, &obj, src_key, full_key));
    }
    Ok(items)
}

// ------------------------------------------------------------------------
// Ranking (pure — unit-tested without a corpus)
// ------------------------------------------------------------------------

/// Cosine similarity of two histograms given their cached L2 norms.
pub fn cosine(a: &[f32], na: f32, b: &[f32], nb: f32) -> f32 {
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    dot / (na * nb)
}

/// Index positions of `index`, ranked by descending cosine to `query`, ties
/// broken by ascending `id` (deterministic — no wall-clock, no RNG).
pub fn rank(query: &Item, index: &[Item]) -> Vec<usize> {
    let mut scored: Vec<(f32, usize)> = index
        .iter()
        .enumerate()
        .map(|(i, c)| (cosine(&query.hist, query.norm, &c.hist, c.norm), i))
        .collect();
    scored.sort_by(|a, b| {
        b.0
            .partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| index[a.1].id.cmp(&index[b.1].id))
    });
    scored.into_iter().map(|(_, i)| i).collect()
}

// ------------------------------------------------------------------------
// Recall@k
// ------------------------------------------------------------------------

/// Per-k recall row (fractions in `0..=1`).
#[derive(Clone, Debug, Default)]
pub struct RecallRow {
    pub k: usize,
    pub obj_text: f64,
    pub obj_full: f64,
    pub exact: f64,
    /// Analytic expected obj-text recall of a uniformly random ranking.
    pub random_text: f64,
}

/// Full eval outcome over a (query, index) pair.
#[derive(Clone, Debug, Default)]
pub struct EvalReport {
    pub n_query: usize,
    pub n_index: usize,
    /// Queries with ≥1 positive in the index, per ground truth.
    pub answerable_text: usize,
    pub answerable_full: usize,
    pub answerable_exact: usize,
    pub rows: Vec<RecallRow>,
}

/// Expected recall@k of a uniformly random ranking of `m` candidates with `p`
/// positives: `1 - C(m-p, k)/C(m, k)` = `1 - Π_{i=0}^{k-1} (m-p-i)/(m-i)`.
fn random_recall_at_k(m: usize, p: usize, k: usize) -> f64 {
    if p == 0 || m == 0 {
        return 0.0;
    }
    if p >= m || k >= m {
        return 1.0;
    }
    let mut miss = 1.0f64;
    for i in 0..k {
        miss *= (m - p - i) as f64 / (m - i) as f64;
    }
    1.0 - miss
}

/// Evaluate retrieval of `queries` against `index` at each cutoff in `ks`.
///
/// When `leave_one_out` is set, `queries` and `index` are the same slice and a
/// candidate sharing the query's `id` is excluded (a row never retrieves
/// itself). Positives are counted by ground-truth key equality among the
/// *remaining* candidates, so exact-source is honestly 0 whenever the corpus
/// dedups sources.
pub fn evaluate(queries: &[Item], index: &[Item], ks: &[usize], leave_one_out: bool) -> EvalReport {
    let mut report = EvalReport {
        n_query: queries.len(),
        n_index: index.len(),
        rows: ks.iter().map(|&k| RecallRow { k, ..Default::default() }).collect(),
        ..Default::default()
    };
    if queries.is_empty() {
        return report;
    }

    for q in queries {
        let ranked = rank(q, index);
        // Candidate index positions that are a positive under each ground truth,
        // and the effective candidate count (excluding self under LOO).
        let is_self = |c: &Item| leave_one_out && c.id == q.id;
        let mut pos_text = Vec::new();
        let mut pos_full = Vec::new();
        let mut pos_exact = Vec::new();
        let mut m = 0usize;
        for (i, c) in index.iter().enumerate() {
            if is_self(c) {
                continue;
            }
            m += 1;
            if c.key(GroundTruth::ObjText) == q.key(GroundTruth::ObjText) {
                pos_text.push(i);
            }
            if c.key(GroundTruth::ObjFull) == q.key(GroundTruth::ObjFull) {
                pos_full.push(i);
            }
            if c.key(GroundTruth::ExactSource) == q.key(GroundTruth::ExactSource) {
                pos_exact.push(i);
            }
        }
        if !pos_text.is_empty() {
            report.answerable_text += 1;
        }
        if !pos_full.is_empty() {
            report.answerable_full += 1;
        }
        if !pos_exact.is_empty() {
            report.answerable_exact += 1;
        }

        // Ranked order with self removed, for top-k membership.
        let ordered: Vec<usize> = ranked.into_iter().filter(|&i| !is_self(&index[i])).collect();
        for row in &mut report.rows {
            let k = row.k;
            let topk: std::collections::HashSet<usize> = ordered.iter().take(k).copied().collect();
            if pos_text.iter().any(|i| topk.contains(i)) {
                row.obj_text += 1.0;
            }
            if pos_full.iter().any(|i| topk.contains(i)) {
                row.obj_full += 1.0;
            }
            if pos_exact.iter().any(|i| topk.contains(i)) {
                row.exact += 1.0;
            }
            row.random_text += random_recall_at_k(m, pos_text.len(), k);
        }
    }

    let q = queries.len() as f64;
    for row in &mut report.rows {
        row.obj_text /= q;
        row.obj_full /= q;
        row.exact /= q;
        row.random_text /= q;
    }
    report
}

// ------------------------------------------------------------------------
// Deterministic held-out split
// ------------------------------------------------------------------------

/// Deterministic query/index partition: a row is a **query** iff
/// `sha256(id) mod query_div == 0` (≈ `1/query_div` of the corpus), else it is
/// in the **index**. Pure function of the ids — reproducible, no RNG.
pub fn split_held_out(items: Vec<Item>, query_div: u64) -> (Vec<Item>, Vec<Item>) {
    let div = query_div.max(1);
    let mut query = Vec::new();
    let mut index = Vec::new();
    for it in items {
        if id_bucket(&it.id) % div == 0 {
            query.push(it);
        } else {
            index.push(it);
        }
    }
    (query, index)
}

fn id_bucket(id: &str) -> u64 {
    let h = sha256_hex(id.as_bytes());
    u64::from_str_radix(&h[..16], 16).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(id: &str, src: &str, text: &[u8]) -> Item {
        let (hist, norm) = byte_histogram(text);
        Item {
            id: id.into(),
            src_key: sha256_hex(src.as_bytes()),
            text_key: sha256_hex(text),
            full_key: format!("full-{id}"), // unique per row (path-polluted mimic)
            hist,
            norm,
            text_len: text.len(),
            nsym: 0,
            obj_len: text.len(),
        }
    }

    #[test]
    fn cosine_identical_is_one_orthogonal_is_zero() {
        let (a, na) = byte_histogram(&[1, 1, 2, 3]);
        let (b, nb) = byte_histogram(&[1, 1, 2, 3]);
        assert!((cosine(&a, na, &b, nb) - 1.0).abs() < 1e-6);
        let (c, nc) = byte_histogram(&[10, 10]);
        let (d, nd) = byte_histogram(&[20, 20]);
        assert!(cosine(&c, nc, &d, nd).abs() < 1e-6); // disjoint byte values
    }

    #[test]
    fn rank_puts_identical_text_first() {
        let q = mk("q", "sq", &[5, 5, 6, 7, 7, 7]);
        let index = vec![
            mk("a", "sa", &[9, 9, 9, 9, 9, 9]),      // far
            mk("b", "sb", &[5, 5, 6, 7, 7, 7]),      // identical text
            mk("c", "sc", &[5, 5, 6, 7, 7, 8]),      // near
        ];
        let order = rank(&q, &index);
        assert_eq!(order[0], 1, "identical-.text candidate must rank first");
    }

    #[test]
    fn recall_credits_text_twin_not_random() {
        // 1 query with a .text twin in an index of distinct-code rows.
        let q = mk("q", "sq", &[1, 2, 3, 4]);
        let mut index = vec![mk("twin", "s_twin", &[1, 2, 3, 4])]; // twin (different source)
        for i in 0..20 {
            // distinct code, each a unique byte pattern
            index.push(mk(&format!("d{i}"), &format!("sd{i}"), &[i as u8, i as u8, 99, 100]));
        }
        let rep = evaluate(&[q], &index, &[1, 5], false);
        assert_eq!(rep.answerable_text, 1);
        assert_eq!(rep.answerable_exact, 0, "distinct sources → exact unanswerable");
        // Twin has identical histogram → rank 1 → recall@1 == 1.0.
        assert!((rep.rows[0].obj_text - 1.0).abs() < 1e-9);
        // Random baseline for 1 positive of 21 at k=1 is 1/21 ≈ 0.0476.
        assert!((rep.rows[0].random_text - 1.0 / 21.0).abs() < 1e-6);
        // Exact-source recall is 0 (twin is a different source).
        assert!(rep.rows[0].exact.abs() < 1e-9);
    }

    #[test]
    fn leave_one_out_excludes_self() {
        // Two identical-code rows (a behavioral twin pair) + one loner.
        let items = vec![
            mk("t0", "s0", &[7, 7, 8]),
            mk("t1", "s1", &[7, 7, 8]), // twin of t0
            mk("t2", "s2", &[1, 2, 3]), // unique
        ];
        let rep = evaluate(&items, &items, &[1], true);
        assert_eq!(rep.n_query, 3);
        // t0 and t1 each have a twin (the other); t2 has none → 2 answerable.
        assert_eq!(rep.answerable_text, 2);
        // Both twins retrieve each other at rank 1 → recall@1 = 2/3.
        assert!((rep.rows[0].obj_text - 2.0 / 3.0).abs() < 1e-9);
        // No row retrieves itself: exact-source recall is 0.
        assert!(rep.rows[0].exact.abs() < 1e-9);
    }

    #[test]
    fn random_recall_monotone_in_k() {
        assert_eq!(random_recall_at_k(100, 0, 5), 0.0);
        let r1 = random_recall_at_k(100, 1, 1);
        let r5 = random_recall_at_k(100, 1, 5);
        assert!(r5 > r1 && r1 > 0.0);
        assert!((r1 - 0.01).abs() < 1e-9); // 1/100
    }

    #[test]
    fn split_is_deterministic_and_partitions() {
        let items: Vec<Item> = (0..50)
            .map(|i| mk(&format!("t{i:03}"), &format!("s{i}"), &[i as u8]))
            .collect();
        let n = items.len();
        let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
        let (q, idx) = split_held_out(items, 5);
        assert_eq!(q.len() + idx.len(), n, "partition covers every row once");
        // Re-run on fresh clones → identical query id set.
        let items2: Vec<Item> = ids.iter().map(|id| mk(id, "s", &[0])).collect();
        let (q2, _) = split_held_out(items2, 5);
        let qset: Vec<&String> = q.iter().map(|i| &i.id).collect();
        let qset2: Vec<&String> = q2.iter().map(|i| &i.id).collect();
        assert_eq!(qset, qset2);
    }
}
