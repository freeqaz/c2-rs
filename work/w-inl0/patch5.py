#!/usr/bin/env python3
"""patch5.py — lane w-inl0 scratch: add the residue histogram's report and
metric plumbing. Kept because a rung's edits should be reproducible, not because
it is reusable.
"""
p = 'crates/c2-harness/src/gap/report.rs'
s = open(p).read()
s = s.replace('''    /// **Every differing function, by name and by word** — the witness list''', '''    /// **Board #980's residue, by production** — for every `fnbyte-differs`
    /// whose whole reference body is one `blr`, the callee's own blocking
    /// feature (`fnbyte-blr-stop|…`) and, when that callee is itself a
    /// recognized dead-temporary body, its callee's (`fnbyte-blr-stop2|…`).
    ///
    /// The prefix is the argument: `prefix` is `"fnbyte-blr-stop|"` or
    /// `"fnbyte-blr-stop2|"`, and one function serves both rather than two that
    /// can drift.
    pub fn fn_byte_blr_stops(&self, prefix: &str) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, n)| Some((k.strip_prefix(prefix)?.to_string(), n)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// **Every differing function, by name and by word** — the witness list''', 1)
open(p, 'w').write(s)

p = 'crates/c2-harness/src/gap/factors.rs'
s = open(p).read()
s = s.replace('''            for (key, n) in self.fn_byte_noeffect_stops().into_iter().take(8) {
                m.push((
                    Box::leak(format!("fnbyte-noeffect-stop-{key}").into_boxed_str()),
                    n.to_string(),
                ));
            }''', '''            for (key, n) in self.fn_byte_noeffect_stops().into_iter().take(8) {
                m.push((
                    Box::leak(format!("fnbyte-noeffect-stop-{key}").into_boxed_str()),
                    n.to_string(),
                ));
            }
            // The residue of board #980's own cluster, at both levels of the
            // chain. Top 6 each; a row here is a production and a count of
            // functions it holds, which is what a follow-on rung is sized off.
            for (prefix, tag) in
                [("fnbyte-blr-stop|", "blr-stop"), ("fnbyte-blr-stop2|", "blr-stop2")]
            {
                for (key, n) in self.fn_byte_blr_stops(prefix).into_iter().take(6) {
                    m.push((
                        Box::leak(format!("fnbyte-{tag}-{key}").into_boxed_str()),
                        n.to_string(),
                    ));
                }
            }''', 1)
open(p, 'w').write(s)
print("patched")
