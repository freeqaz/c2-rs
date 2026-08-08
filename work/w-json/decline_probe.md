# The committal-dispatch probe — applied, run, reverted

Board **#1704**: `c2rs census` reports ONE key per file, the fall-through
blocker of whichever production came last. On an unpatched tree all ten cells of
`fixtures/cpp/wjson_utf8_copy_neg.cpp` read `expr-brfalse`, which is
`try_parse_assign_body_detail`'s key and says nothing about why any of them
declined. w-cfgclass §6.2's method, paying a **seventh** time.

The patch, applied to `crates/c2-il/src/func/body/mod.rs`, run once, and
reverted before the fixtures were committed:

```diff
-                if let Ok(shape) = try_parse_json_utf8_copy(seg, p, lo) {
-                    disp("disp-json-utf8-copy");
-                    return Ok(shape);
-                }
+                match try_parse_json_utf8_copy(seg, p, lo) {
+                    Ok(shape) => { disp("disp-json-utf8-copy"); return Ok(shape); }
+                    Err(e) => { disp("disp-json-utf8-copy"); return Err(e); }
+                }
```

Output: `work/w-json/neg_clauses.txt`. **Ten cells, ten DISTINCT keys.**

It also found the one thing only a per-cell read could: **`n7` never reaches the
clause it was written for.** `is_two_word_constant` is a post-match check, and
`n7`'s stream stops earlier — at the body's trailing `4F 02 20 00` directive,
which its segment does not carry — so it reports `json-return:eof`. Recorded
rather than reworded. That clause has **no fixture cell**; it is graded by a unit
test in the shape file instead, and this file is where that is written down.
