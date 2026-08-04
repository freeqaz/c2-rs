//! Small pure classifiers and key-normalizers shared by the scan and the
//! report. Split out of `gap.rs` unchanged; see [`super`] for the module docs.

use std::collections::BTreeMap;

/// The MSVC mangling class of `name`, for naming the unbound residue.
///
/// Coarse on purpose — it separates the populations that would be explained by
/// different stories, and nothing finer is measured.
pub(super) fn mangling_class(name: &str) -> &'static str {
    match name {
        n if n.starts_with("??1") => "dtor",
        n if n.starts_with("??0") => "ctor",
        n if n.starts_with("??_") => "special-generated",
        n if n.starts_with("??$") => "template-operator",
        n if n.starts_with("??") => "operator",
        n if n.starts_with("?$") => "template",
        n if n.starts_with('?') => "ordinary",
        _ => "undecorated",
    }
}

/// Whether `name` is a function **c2 synthesizes**, with no `.ex` body behind it.
pub(super) fn is_compiler_generated(name: &str) -> bool {
    ["??_G", "??_E", "??_D", "??__E", "??__F"]
        .iter()
        .any(|p| name.starts_with(p))
}

/// Sum a per-TU count map across the scan and rank it, most frequent first with
/// ties broken by key. The six axis histograms above differ only in which map they
/// read, and each used to spell this same fold out longhand.
pub(super) fn merge_counts<'a>(
    maps: impl Iterator<Item = &'a BTreeMap<String, usize>>,
) -> Vec<(String, usize)> {
    let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    for m in maps {
        for (k, n) in m {
            *map.entry(k.as_str()).or_insert(0) += n;
        }
    }
    let mut v: Vec<(String, usize)> = map.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v
}

/// Pull a normalized headline out of a cl.exe failure blob: the first line
/// containing `error C`, else the first non-empty line, truncated.
pub(super) fn normalize_cl_error(blob: &str) -> (String, String) {
    let detail = blob
        .lines()
        .map(str::trim)
        .find(|l| l.contains("error C"))
        .or_else(|| blob.lines().map(str::trim).find(|l| !l.is_empty()))
        .unwrap_or("(no output)")
        .to_string();
    // Aggregation key = the `Cnnnn` code when present, else a clipped line.
    let key = detail
        .split_whitespace()
        .find(|t| {
            let t = t.trim_end_matches(':');
            t.len() >= 4
                && t.starts_with('C')
                && t[1..].chars().all(|c| c.is_ascii_digit())
        })
        .map(|t| t.trim_end_matches(':').to_string())
        .unwrap_or_else(|| clip(&detail, 60));
    (key, clip(&detail, 200))
}

/// A stable bucket key for one of the port's per-function refusals.
///
/// The refusal messages are prose (they are what a `codegen-gap` TU reports), so
/// the key is the leading clause — everything before the first `:` — clipped.
/// That is deliberately the message's own words rather than a hand-maintained
/// enum: a key nobody has to remember to add is a key that cannot go stale, and
/// `docs/GAPS.md` §6's rule against guessed names applies here too. The keys are
/// meant to reach zero, not to be ranked forever.
pub(super) fn gate_key(msg: &str) -> String {
    let head = msg.split(':').next().unwrap_or(msg).trim();
    clip(head, 72)
}

/// Which destructor mangling a generated empty destructor's resolved callee is —
/// `"other"` when it is not one at all, which is the count that must stay 0.
///
/// MSVC spells the four: `??1` an ordinary destructor, `??_G` the scalar deleting
/// destructor, `??_E` the vector deleting one, `??_D` the vbase destructor. The
/// shape [`c2_il`] parses is a destructor whose whole body delegates to a
/// sub-object's destructor, so the callee is one of these by construction of the
/// *source*, independently of how `.gl` was read — which is what makes this a
/// grader for the binding rather than a restatement of it.
pub fn dtor_callee_class(name: &str) -> &'static str {
    for (p, k) in [
        ("??1", "1"),
        ("??_G", "G"),
        ("??_E", "E"),
        ("??_D", "D"),
    ] {
        if name.starts_with(p) {
            return k;
        }
    }
    "other"
}

pub(super) fn clip(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        let mut end = n;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}
