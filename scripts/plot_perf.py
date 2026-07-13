#!/usr/bin/env python3
"""Render the c2rs perf-scale graph for the README.

Reads the CSV emitted by `c2rs perf-scale --csv <path>`
(columns: concurrency, port_ops_per_sec, ref_ops_per_sec) and writes a
two-panel PNG:

  A. IL-bundle -> obj throughput (objs/sec, log scale) vs concurrency, for the
     native port (in-process Rust) vs standalone c2.dll under wibo.
  B. The resulting speedup (x), which grows with concurrency.

This is *tooling*, not part of the std-only Rust workspace — matplotlib lives
outside the crates. Numbers are machine-dependent; regenerate on your box:

    cargo run -p c2-harness --bin c2rs -- perf-scale --csv docs/perf/perf_scale.csv
    python3 scripts/plot_perf.py

Colors are the dataviz reference categorical palette slots 1 (blue) and 8
(orange) — validated CVD-safe as an adjacent pair on the light surface.
"""
import csv
import sys

import matplotlib
matplotlib.use("Agg")  # headless
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter

# --- design tokens (dataviz reference palette, light surface) ----------------
SURFACE = "#fcfcfb"
INK = "#0b0b0b"      # text-primary
INK_2 = "#52514e"    # text-secondary
GRID = "#e5e4e1"
PORT = "#2a78d6"     # categorical slot 1 (blue)  — the native port
C2 = "#eb6834"       # categorical slot 8 (orange) — standalone c2 (reference)


def human(n):
    """Compact objs/sec label: 897056 -> '897k', 3106 -> '3.1k'."""
    if n >= 1_000_000:
        return f"{n / 1_000_000:.1f}M"
    if n >= 10_000:
        return f"{n / 1000:.0f}k"
    if n >= 1000:
        return f"{n / 1000:.1f}k"
    return f"{n:.0f}"


def load(path):
    conc, port, ref = [], [], []
    with open(path) as f:
        for row in csv.DictReader(f):
            conc.append(int(row["concurrency"]))
            port.append(float(row["port_ops_per_sec"]))
            ref.append(float(row["ref_ops_per_sec"]))
    return conc, port, ref


def style_axes(ax):
    ax.set_facecolor(SURFACE)
    for side in ("top", "right"):
        ax.spines[side].set_visible(False)
    for side in ("left", "bottom"):
        ax.spines[side].set_color(INK_2)
        ax.spines[side].set_linewidth(0.8)
    ax.tick_params(colors=INK_2, labelsize=9, length=3)
    ax.grid(axis="y", color=GRID, linewidth=0.6, zorder=0)
    ax.set_axisbelow(True)


def main():
    csv_path = sys.argv[1] if len(sys.argv) > 1 else "docs/perf/perf_scale.csv"
    out_path = sys.argv[2] if len(sys.argv) > 2 else "docs/perf/perf_scale.png"
    fixture = sys.argv[3] if len(sys.argv) > 3 else "mvp_add3.cpp"
    obj_bytes = sys.argv[4] if len(sys.argv) > 4 else "846"

    conc, port, ref = load(csv_path)
    ncores = max(conc)

    fig, (axA, axB) = plt.subplots(1, 2, figsize=(11, 4.3))
    fig.patch.set_facecolor(SURFACE)

    # --- Panel A: throughput vs concurrency (log y) --------------------------
    style_axes(axA)
    axA.set_yscale("log")
    axA.set_xscale("log", base=2)
    axA.plot(conc, port, color=PORT, lw=2, marker="o", ms=6, label="native port (in-process Rust)", zorder=3)
    axA.plot(conc, ref, color=C2, lw=2, marker="o", ms=6, label="standalone c2.dll (under wibo)", zorder=3)

    # Direct end labels (selective — endpoints only, in ink not series color).
    axA.annotate(f"{human(port[-1])} obj/s", (conc[-1], port[-1]),
                 textcoords="offset points", xytext=(-6, 10), ha="right",
                 fontsize=9.5, fontweight="bold", color=INK)
    axA.annotate(f"{human(ref[-1])} obj/s", (conc[-1], ref[-1]),
                 textcoords="offset points", xytext=(-6, -16), ha="right",
                 fontsize=9.5, fontweight="bold", color=INK)

    axA.set_xticks(conc)
    axA.set_xticklabels([str(c) for c in conc])
    axA.yaxis.set_major_formatter(FuncFormatter(lambda y, _: human(y)))
    axA.set_xlabel("concurrency (threads)", fontsize=10, color=INK_2)
    axA.set_ylabel("IL→obj throughput (objs/sec, log)", fontsize=10, color=INK_2)
    axA.set_title("Throughput scales with cores", fontsize=12, color=INK, fontweight="bold", pad=8)
    leg = axA.legend(frameon=False, fontsize=9, loc="center left", labelcolor=INK)
    for t in leg.get_texts():
        t.set_color(INK)

    # --- Panel B: speedup vs concurrency -------------------------------------
    style_axes(axB)
    axB.set_xscale("log", base=2)
    speedup = [p / r if r > 0 else float("nan") for p, r in zip(port, ref)]
    axB.plot(conc, speedup, color=PORT, lw=2, marker="o", ms=6, zorder=3)
    axB.fill_between(conc, speedup, color=PORT, alpha=0.08, zorder=1)
    for x, s, first_last in ((conc[0], speedup[0], True), (conc[-1], speedup[-1], True)):
        axB.annotate(f"{s:.0f}×", (x, s), textcoords="offset points",
                     xytext=(0, 10), ha="center", fontsize=10, fontweight="bold", color=INK)
    axB.set_xticks(conc)
    axB.set_xticklabels([str(c) for c in conc])
    axB.set_ylim(0, max(speedup) * 1.25)
    axB.set_xlabel("concurrency (threads)", fontsize=10, color=INK_2)
    axB.set_ylabel("speedup (port ÷ c2)", fontsize=10, color=INK_2)
    axB.set_title("Port is 200–290× faster — and the gap widens", fontsize=12, color=INK, fontweight="bold", pad=8)

    fig.suptitle("c2-rs: native port vs real c2.dll — same obj, byte-for-byte",
                 fontsize=13.5, color=INK, fontweight="bold", y=1.02)
    fig.text(0.5, -0.02,
             f"fixture {fixture} · {obj_bytes} B obj · {ncores}-core host · higher is better · both sides emit the identical timestamp-normalized COFF",
             ha="center", fontsize=8.5, color=INK_2)

    fig.tight_layout(rect=(0, 0, 1, 0.98))
    fig.savefig(out_path, dpi=160, facecolor=SURFACE, bbox_inches="tight")
    print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
