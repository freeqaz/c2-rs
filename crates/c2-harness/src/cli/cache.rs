//! `c2rs cache` — inspect and reclaim the capture cache (board #3265).
//!
//! The cache had no reclamation path at all: the key includes the workload
//! tree's git identity, so every commit to the workload mints a fresh
//! generation of every TU and **nothing evicts**. The 2026-08-04 cleanup was
//! done by hand and the count grew straight back. This is that cleanup as code,
//! with the predicate the cleanup actually used rather than the age one its
//! design doc proposed and its landing refuted.
//!
//! **Every subcommand here enumerates one level with `read_dir` and never
//! globs.** A shell glob over a cache root has twice taken this machine down
//! with the OOM killer. `gc` is a dry run unless `--apply` is given.

use std::io::Write as _;
use std::process::ExitCode;
use std::time::Duration;

use c2_harness::capture_cache::{
    self, classify_entry, default_cache_root, gc, generations, parse_key_material,
    GcOptions, ENTRY_BLOB, LOCK_DIR,
};

use crate::{Args, Arity, Spec};

// PROV[N] not load-bearing — a CLI argument specification for this crate's own `c2rs` binary. Nothing in it is derived from `c2.dll`; a wrong value changes a usage message or a parse, never a graded byte.
static CACHE_SPEC: Spec = Spec::new(
    "cache",
    &[
        ("--cache", Arity::Value),
        ("--apply", Arity::Flag),
        ("--limit", Arity::Value),
        ("--min-age", Arity::Value),
        ("--drop-generation", Arity::Repeated),
        ("--sample", Arity::Value),
    ],
)
.positionals(2);

// PROV[N] not load-bearing — the usage string listing this subcommand's verbs.
const VERBS: &str = "stat | index | generations | show <key> | gc";

pub(crate) fn cmd_cache(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&CACHE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    // Unlike the capture path, this command has nothing to degrade *to*: every
    // verb is about a root. So an unresolvable one is a hard refusal.
    let root = match args.path("--cache") {
        Some(p) => p,
        None => match default_cache_root() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("cache: {e}");
                return ExitCode::from(2);
            }
        },
    };
    let verb = args.positionals().first().map(String::as_str).unwrap_or("");
    if !root.is_dir() {
        eprintln!("cache: no cache root at {}", root.display());
        return ExitCode::from(2);
    }
    match verb {
        "stat" => cmd_stat(&root),
        "generations" => match args.num::<usize>("--sample") {
            Ok(v) => cmd_generations(&root, v.unwrap_or(1)),
            Err(c) => c,
        },
        "index" => cmd_index(&root),
        "show" => cmd_show(&root, args.positionals().get(1).map(String::as_str)),
        "gc" => cmd_gc(&args, &root),
        "" => {
            eprintln!("cache: give a verb: {VERBS}");
            ExitCode::from(2)
        }
        other => {
            eprintln!("cache: unknown verb {other}; want {VERBS}");
            ExitCode::from(2)
        }
    }
}

/// Bounded census. Counts only — never a list of 21.5 M names.
fn cmd_stat(root: &std::path::Path) -> ExitCode {
    let (mut entries, mut strays, mut locks) = (0usize, 0usize, 0usize);
    let rd = match std::fs::read_dir(root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("cache: {}: {e}", root.display());
            return ExitCode::FAILURE;
        }
    };
    for ent in rd.flatten() {
        let name = ent.file_name().to_string_lossy().into_owned();
        if name == LOCK_DIR {
            locks = std::fs::read_dir(ent.path()).map(|r| r.flatten().count()).unwrap_or(0);
            continue;
        }
        if name.len() == 32 && name.bytes().all(|b| b.is_ascii_hexdigit()) {
            entries += 1;
        } else {
            strays += 1;
        }
    }
    println!("cache root: {}", root.display());
    println!("  entries:    {entries}");
    println!("  strays:     {strays}");
    println!("  held locks: {locks}  ({LOCK_DIR}/)");
    ExitCode::SUCCESS
}

fn cmd_generations(root: &std::path::Path, sample: usize) -> ExitCode {
    let (gens, read) = match generations(root, sample) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("cache: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("cache root: {}", root.display());
    if sample > 1 {
        println!(
            "SAMPLED 1 entry in {sample} — {read} read. Counts below are of the SAMPLE; \
             they are not entry totals and must not be scaled up silently."
        );
    }
    println!("{} distinct generation(s), most populous first:", gens.len());
    for (digest, n, ctx) in &gens {
        println!("\n  {digest}  {n} sampled entries");
        for line in ctx.lines() {
            println!("      {line}");
        }
    }
    println!(
        "\nDeleting one is `cache gc --drop-generation <digest> --apply`. Which are dead is \
         YOUR call: a shared root legitimately holds several live generations at once (a \
         fixture capture keys as `tree unknown`, the workload scan carries the workload tree), \
         so this tool will not guess."
    );
    ExitCode::SUCCESS
}

/// `<src-arg>\t<entry dir>` for every entry, on stdout.
///
/// The bulk half of the supported reader — `show` for one entry, this for all of
/// them. Prints the counts to **stderr** so stdout stays a clean TSV and a
/// truncated run is still visible; a consumer that redirects stdout to a file
/// sees the tally on its terminal.
fn cmd_index(root: &std::path::Path) -> ExitCode {
    let out = std::io::stdout();
    let mut w = std::io::BufWriter::new(out.lock());
    let mut emit = |src: &str, dir: &std::path::Path| {
        let _ = writeln!(w, "{src}\t{}", dir.display());
    };
    let r = capture_cache::index(root, &mut emit);
    let _ = w.flush();
    match r {
        Ok((n, unreadable)) => {
            eprintln!("indexed {n} entries, {unreadable} unreadable");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("cache: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Decode one entry. **The supported reader** — nothing else should carry its
/// own implementation of the key format or the blob container, which is why the
/// untracked `cacheindex.py`/`cachekey.py` scripts in various worktrees are
/// superseded by this rather than ported.
fn cmd_show(root: &std::path::Path, key: Option<&str>) -> ExitCode {
    let Some(key) = key else {
        eprintln!("cache show: give a 32-hex entry key");
        return ExitCode::from(2);
    };
    let dir = root.join(key);
    let raw = match std::fs::read(dir.join(ENTRY_BLOB)) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cache show: {}/{ENTRY_BLOB}: {e}", dir.display());
            return ExitCode::FAILURE;
        }
    };
    let blob = match c2_il::decode_entry(&raw) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cache show: {ENTRY_BLOB} did not decode: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(k) = parse_key_material(blob.key) else {
        eprintln!("cache show: key material did not parse");
        return ExitCode::FAILURE;
    };
    println!("entry:      {}", dir.display());
    println!("blob:       {} bytes", raw.len());
    println!("generation: {}", capture_cache::digest128(k.context));
    print!("{}", String::from_utf8_lossy(k.context));
    println!("src-arg:    {}", k.src_arg);
    println!("cwd:        {}", if k.cwd.is_empty() { "(none)" } else { k.cwd });
    println!("verdict:    {:?}", classify_entry(blob.key));
    let obj = dir.join("out.obj");
    match std::fs::metadata(&obj) {
        Ok(m) => println!("out.obj:    {} bytes", m.len()),
        Err(e) => println!("out.obj:    MISSING ({e}) — this entry cannot be served"),
    }
    for suffix in c2_il::IL_SUFFIXES {
        match blob.il(suffix) {
            Some(b) => println!("  .{suffix:<3}     {} bytes", b.len()),
            None => println!("  .{suffix:<3}     absent"),
        }
    }
    println!("--- meta ---");
    print!("{}", blob.meta);
    ExitCode::SUCCESS
}

fn cmd_gc(args: &Args, root: &std::path::Path) -> ExitCode {
    let limit = match args.num::<usize>("--limit") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let min_age = match args.num::<u64>("--min-age") {
        Ok(v) => Duration::from_secs(v.unwrap_or(3600)),
        Err(c) => return c,
    };
    let opts = GcOptions {
        apply: args.has("--apply"),
        limit,
        min_age,
        drop_generations: args.all("--drop-generation").into_iter().map(str::to_string).collect(),
    };
    if !opts.apply {
        println!("DRY RUN — nothing will be deleted. Add --apply to act.");
    }
    println!("cache root: {}", root.display());
    let rep = match gc(root, &opts, &|n| println!("  … {n} entries scanned")) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("cache: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("  scanned:            {}", rep.scanned);
    println!("  live:               {}", rep.live);
    println!("  UNREACHABLE:        {}  (source gone; the key can never be formed again)", rep.unreachable);
    println!("  kept, unknown:      {}  (could not establish — never deleted)", rep.unknown);
    for d in &rep.unknown_detail {
        println!("      {d}");
    }
    println!("  kept, too young:    {}  (< {}s)", rep.too_young, opts.min_age.as_secs());
    println!("  kept, locked:       {}", rep.locked);
    if !opts.drop_generations.is_empty() {
        println!("  dropped generation: {}", rep.generation_dropped);
    }
    println!("  strays (kept):      {}", rep.strays);
    for s in &rep.stray_names {
        println!("      {s}");
    }
    if opts.apply {
        println!("  DELETED:            {}", rep.deleted);
        println!("  delete failed:      {}", rep.delete_failed);
    } else {
        println!(
            "  would delete:       {}",
            rep.unreachable + rep.generation_dropped
        );
    }

    // Lockfiles need their own pass: `KeyLock`'s staleness break only runs on
    // CONTENTION, so a lock abandoned on a key nobody wants again is never
    // looked at and sits forever.
    let (held, reapable, kept) = reap_locks(root, opts.apply);
    println!(
        "  lockfiles:          {held} present, {} {}, {kept} kept (live or unestablishable)",
        reapable,
        if opts.apply { "reaped" } else { "reapable" }
    );
    ExitCode::SUCCESS
}

/// Kept out of `gc`'s hot loop deliberately: this is the only place that reads
/// `/proc`, and it runs once per lockfile rather than once per entry.
fn reap_locks(root: &std::path::Path, apply: bool) -> (usize, usize, usize) {
    let dir = root.join(LOCK_DIR);
    let (mut held, mut reaped, mut kept) = (0usize, 0usize, 0usize);
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return (0, 0, 0);
    };
    for ent in rd.flatten() {
        held += 1;
        let p = ent.path();
        let Ok(body) = std::fs::read_to_string(&p) else {
            kept += 1;
            continue;
        };
        let Ok(pid) = body.trim().parse::<u32>() else {
            kept += 1;
            continue;
        };
        // Dead pid AND the pid has not been reused by something that could be
        // the owner. Anything unestablishable is a KEEP: deleting a live lock
        // silently un-guards a key, and a leaked lockfile costs one inode.
        let cmdline = std::fs::read(format!("/proc/{pid}/cmdline"));
        let dead = cmdline.is_err();
        let reused = cmdline
            .map(|c| !String::from_utf8_lossy(&c).contains("c2rs"))
            .unwrap_or(false);
        if dead || reused {
            if apply && std::fs::remove_file(&p).is_ok() {
                reaped += 1;
            } else if !apply {
                reaped += 1;
            }
        } else {
            kept += 1;
        }
    }
    (held, reaped, kept)
}
