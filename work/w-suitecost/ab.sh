#!/bin/bash
# w-suitecost A/B driver. Interleaves serial and parallel runs so box load,
# which other lanes move minute to minute, is spread across both arms rather
# than confounded with one. Every row records the 1-min load average taken
# immediately before the run starts.
set -uo pipefail
cd "$(dirname "$0")/../.."
OUT=work/w-suitecost/logs
mkdir -p "$OUT"
TSV="$OUT/ab.tsv"
[ -s "$TSV" ] || printf 'run\tarm\tjobs\tload_before\twall_s\tsum_s\tcpu_s\tpassed\tfailed\tignored\ttargets\tnames\n' >"$TSV"

# CPU seconds (user+sys, including every descendant) for the last foreground
# pipeline. Wall clock on a box other lanes are using is not a load-robust
# quantity; CPU-seconds is the work actually done, and the ratio
# cpu_serial/cpu_parallel says whether overlapping changed the WORK or only its
# schedule.
# `/usr/bin/time` is not installed on this box. bash's `times` builtin reports
# CHILDREN user/sys as its second line, so running the command in a subshell and
# calling `times` there gives exactly the descendants' CPU and nothing else.
cpu_read() { awk '{n=split($0,f," "); t=0; for(i=1;i<=n;i++){split(f[i],g,"m"); sub(/s$/,"",g[2]); t+=g[1]*60+g[2]} printf "%.1f", t}' "$1"; }

serial() {
    local tag="$1" lb
    lb=$(cut -d' ' -f1 /proc/loadavg)
    local t0 t1
    t0=$(date +%s.%N)
    (
        C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast \
            >"$OUT/$tag.log" 2>&1
        times >"$OUT/$tag.cpuraw"
        tail -1 "$OUT/$tag.cpuraw" >"$OUT/$tag.cpu"
    )
    t1=$(date +%s.%N)
    local wall sum p f i n names
    wall=$(awk -v a="$t0" -v b="$t1" 'BEGIN{printf "%.1f", b-a}')
    read -r sum p f i n < <(awk '/^test result:/{for(j=1;j<=NF;j++){if($j=="in"){t=$(j+1);gsub(/s$/,"",t);s+=t}
        if($j=="passed;")p+=$(j-1); if($j=="failed;")f+=$(j-1); if($j=="ignored;")g+=$(j-1)} n++}
        END{printf "%.1f %d %d %d %d\n", s, p, f, g, n}' "$OUT/$tag.log")
    awk '
      / *Running .*\(/ { s=$0; sub(/.*\(/,"",s); sub(/\).*/,"",s); k=split(s,q,"/"); b=q[k]; sub(/-[^-]*$/,"",b); cur=b; next }
      / *Doc-tests /   { sub(/^ *Doc-tests +/,""); cur="doc-"$0; next }
      /^test result:/  { next }
      /^test .* \.\.\. / { line=$0; sub(/^test /,"",line); j=index(line," ... ")
        nm=substr(line,1,j-1); v=substr(line,j+5); sub(/ <[0-9.]+s>$/,"",v); sub(/,.*$/,"",v)
        gsub(/^[ \t]+|[ \t]+$/,"",v); printf "%s :: %s :: %s\n", cur, nm, v }
    ' "$OUT/$tag.log" | sort >"$OUT/$tag.names"
    names=$(wc -l <"$OUT/$tag.names")
    local cpu; cpu=$(cpu_read "$OUT/$tag.cpu")
    printf '%s\tserial\t1\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$tag" "$lb" "$wall" "$sum" "$cpu" "$p" "$f" "$i" "$n" "$names" >>"$TSV"
    echo "$tag serial wall=${wall}s sum=${sum}s cpu=${cpu}s load0=$lb names=$names"
}

parallel() {
    local tag="$1" jobs="$2" tt="${3:-}" lb
    lb=$(cut -d' ' -f1 /proc/loadavg)
    local extra=()
    [ -n "$tt" ] && extra=(--test-threads "$tt")
    local t0 t1
    t0=$(date +%s.%N)
    (
        C2RS_REQUIRE_TOOLCHAIN=1 scripts/partest.sh --jobs "$jobs" "${extra[@]}" \
            --out "$OUT/$tag" >"$OUT/$tag.log" 2>&1
        times >"$OUT/$tag.cpuraw"
        tail -1 "$OUT/$tag.cpuraw" >"$OUT/$tag.cpu"
    )
    t1=$(date +%s.%N)
    local wall sum p f i n names
    wall=$(awk -v a="$t0" -v b="$t1" 'BEGIN{printf "%.1f", b-a}')
    sum=$(awk '/sum-of-target-walls/{for(j=1;j<=NF;j++) if($j=="sum-of-target-walls"){t=$(j+1);gsub(/s$/,"",t);print t}}' "$OUT/$tag.log")
    read -r p f i n < <(awk '/^partest: [0-9]+ passed/{print $2, $4, $6, $8}' "$OUT/$tag.log")
    cp "$OUT/$tag/names.txt" "$OUT/$tag.names"
    names=$(wc -l <"$OUT/$tag.names")
    local cpu; cpu=$(cpu_read "$OUT/$tag.cpu")
    printf '%s\tparallel\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$tag" "$jobs${tt:+/tt$tt}" "$lb" "$wall" "$sum" "$cpu" "$p" "$f" "$i" "$n" "$names" >>"$TSV"
    echo "$tag parallel j=$jobs${tt:+ tt=$tt} wall=${wall}s sum=${sum}s cpu=${cpu}s load0=$lb names=$names"
}

"$@"
