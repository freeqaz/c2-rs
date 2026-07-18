# CORPUS_MVP — the P1.2 `(source, IL, obj)` triple corpus

The corpus generator (`c2-harness::corpus`, driven by `c2rs corpus`) produces a
reproducible dataset of **`(source, IL-bundle, obj)` triples** by compiling
generated C++ functions through the real toolchain and capturing the c1xx→c2 IL
bundle plus the c2 obj. It feeds the downstream **retrieval baseline (P1.3)** —
obj-pattern → candidate-IL lookup — and supplies seeds for **IL-space search
(T-A)**.

## What one triple is

| Part | On disk | Notes |
|------|---------|-------|
| **source** | `triples/<id>/source.cpp` | Generated, self-contained, path-free. |
| **IL bundle** | `triples/<id>/il/<base>{ex,gl,sy,in,db}` | The surviving `_CL_*` bundle, captured via the P0.1 `strace`+`/Bd` recipe (real c2 runs → real obj, `unlink`-inject keeps the bundle). |
| **obj** | `triples/<id>/obj.bin` | The c2 object for that IL, **timestamp-normalized** (COFF `TimeDateStamp` at offset 4..8 zeroed). Stored as `obj.bin`, not `*.obj`, so a scrubbed sample is committable. |

Each captured bundle is parsed through the **K1 codec** (`IlModel::parse`), which
is fail-closed: `encode(parse(bundle)) == bundle` byte-for-byte or the triple is
flagged `codec_fail` (never silently stored). The codec's typed coverage
(currently the `.ex` operand-stream tokens + the framed `.gl` body-start offset)
is recorded per triple; the rest is opaque hex — fine for retrieval, a real
ceiling for a sequence-model consumer until K2 shrinks the opaque map.

## On-disk layout

```
<corpus-root>/               # gitignored generated artifact (see Committability)
  config.json                # generator config for this run
  manifest.jsonl             # one JSON object per line, one per triple
  triples/
    t00000/
      source.cpp
      il/_CL_<hash>ex  ...gl ...sy ...in ...db
      obj.bin
    t00001/ ...
```

`config.json`:

```json
{"generator":"straightline_int_v1","seed":0,"count":32,"timeout_secs":60}
```

## Manifest schema (`manifest.jsonl`)

One compact JSON object per line. Queryable by streaming lines (no whole-file
parse needed). Fields present on an `ok` triple:

| Field | Type | Meaning |
|-------|------|---------|
| `id` | string | Triple id, `t{:05}`. |
| `index` | int | Enumeration index that generated the source (`gen_source(seed,index)`). |
| `seed` | int | Generator seed. |
| `status` | string | `ok` \| `codec_fail` \| `capture_timeout` \| `capture_error`. |
| `error` | string? | Present on non-`ok` rows (and `codec_fail`). |
| `source_rel` | string | Path to `source.cpp`, relative to corpus root. |
| `source_sha256` | string | SHA-256 of the source bytes (dedup / integrity key). |
| `functions` | [string] | Function names in the TU. |
| `il_dir_rel` | string | Path to the `il/` dir. |
| `il_base` | string | Bundle base (`_CL_<hash>`; `sample<NN>` in the committed sample). |
| `il_files` | {string:int} | Present suffix → byte length, canonical order. |
| `codec_roundtrip` | bool | K1 codec round-trip held. |
| `ex_token_count` | int | Decoded `.ex` operand-stream tokens. |
| `ex_typed_bytes` / `ex_opaque_bytes` | int | Typed vs opaque byte split of `.ex` (coverage). |
| `gl_offsets` | [int] | Framed `.gl` body-start offsets (the `.ex` `4F 1F` offsets; one per function). K3's rewrite column. |
| `obj_rel` | string | Path to `obj.bin`. |
| `obj_len` | int | Normalized obj length. |
| `obj_sha256_norm` | string | SHA-256 of the normalized obj (retrieval key). |

Skip/error rows carry `id`/`index`/`seed`/`status`/`error` + the source fields
only.

## Source generation — deterministic

`gen_source(seed, index)` is a **pure function** of its inputs (a hand-rolled
splitmix64 stream — no `Date.now()`, no OS entropy), so a re-run reproduces the
source byte for byte. The generator (`straightline_int_v1`) emits:

- **1–3 functions per TU**, named `f{index}_{k}`.
- Each function: **1–4 parameters** (`int a, b, c, d`) and a **fully
  parenthesized left-associative chain of 1–4 binary ops** (`+ - *`) over its
  parameters and **narrow** (`1..=99`) / **wide** (`> 2^16`) integer literals.

This is the class the port + codec already handle (straight-line int arithmetic
leaves, literals wide/narrow, multi-function TUs). The generator dedups by source
text, so a run of `count` yields `count` **distinct** TUs. The first 128 indices
already yield ≥64 distinct TUs (MVP target: tens–hundreds), and the space is far
larger — raise `--count` to scale toward the roadmap's v0 ~10k-pair target.

Reproducibility of the **obj** has one caveat: MSVC bakes the `/Fo` output path
into the obj (`S_OBJNAME`), and c2 bakes the source path into `.gl`, so those
bytes are a deterministic function of `(source, output-path)`. Regenerating the
same seed into the **same corpus root** reproduces every obj byte-for-byte;
a different root changes the embedded path and thus the `.gl`/obj bytes (but not
the source, `.ex`, `.sy`, `.in`). This is the same `(source, path)` determinism
`oracle_selftest` documents.

## Committability (K1 finding — binds this generator)

A captured `.gl` embeds the host source path (`z:\home\…`; wibo maps `/`→`Z:\`)
and the obj embeds its output path. **Captured bundles/objs are therefore NOT
committable.** The corpus root is a gitignored artifact (`/corpus/` in
`.gitignore`); it is regenerated on demand with `c2rs corpus gen`.

Committed instead:

- the generator + schema (this doc);
- a tiny **synthetic** sample under `crates/c2-harness/tests/corpus_sample/` —
  hand-built, **path-free**, written through the *same* record/manifest code
  path as a real run (so its schema can never drift). Its bundle uses a
  `sample<NN>` base (not `_CL_*`, which `.gitignore` excludes) and `obj.bin`
  (not `*.obj`). Regenerate it with `c2rs corpus sample`.

## CLI

```
c2rs corpus gen [--seed N] [--count N] [--out DIR] [--timeout SECS]
c2rs corpus sample [DIR]     # write the portable synthetic sample
c2rs corpus stats <DIR>      # summarize a manifest
```

`gen` degrades to `SKIP: toolchain absent` (or strace-absent) with no error, like
every other toolchain-driven subcommand. Every capture is bounded by
`--timeout` (default 60 s): a capture that exceeds it is killed and recorded as a
`capture_timeout` skip — never a hang of the whole run (P0.6(a): a malformed IL
fn-set can *hang* c2, not just crash).

## Tests

- **portable** (`tests/corpus.rs::sample_corpus_committed_is_valid`) — loads the
  committed sample from disk and checks manifest↔files consistency + codec
  round-trip, no toolchain.
- **integration** (toolchain-gated) — generates a handful of real triples and
  asserts each codec round-trips, the obj is reproducible per-root, and the
  manifest indexes them.

## What P1.3 / T-A can draw from this

- **P1.3 retrieval:** stream `manifest.jsonl`, key on `obj_sha256_norm` /
  `ex_token_count` / `il_files`, and load `obj.bin` + the bundle for any
  candidate. The obj→candidate-IL NN baseline has everything it needs.
- **T-A IL-space search:** each triple is a known-good `(IL, obj)` pair with the
  codec's `gl_body_start_offsets()` already exposing the K3 rewrite column;
  in-class single-function triples sit in the zero-bookkeeping length-edit regime
  (P0.6(a)).
