#!/usr/bin/env python3
"""Scratch enumerator: every LIVE (non-cfg(test), non-comment, non-string)
`to_be_bytes` site under crates/c2-core/src, with its enclosing item."""
import os, re, sys

ROOT = "crates/c2-core/src"

def strip(src):
    """Blank out comments and string/char literals, preserving line structure."""
    out = []
    i = 0
    n = len(src)
    while i < n:
        c = src[i]
        if c == '/' and i+1 < n and src[i+1] == '/':
            j = src.find('\n', i)
            j = n if j < 0 else j
            out.append(' ' * (j - i)); i = j
        elif c == '/' and i+1 < n and src[i+1] == '*':
            depth = 1; j = i + 2
            while j < n and depth:
                if src.startswith('/*', j): depth += 1; j += 2
                elif src.startswith('*/', j): depth -= 1; j += 2
                else: j += 1
            out.append(''.join(ch if ch == '\n' else ' ' for ch in src[i:j])); i = j
        elif c == 'r' and i+1 < n and src[i+1] in '#"':
            m = re.match(r'r(#*)"', src[i:])
            if m:
                h = m.group(1); end = '"' + h
                j = src.find(end, i + len(m.group(0)))
                j = n if j < 0 else j + len(end)
                out.append(''.join(ch if ch == '\n' else ' ' for ch in src[i:j])); i = j
            else:
                out.append(c); i += 1
        elif c == '"':
            j = i + 1
            while j < n:
                if src[j] == '\\': j += 2
                elif src[j] == '"': j += 1; break
                else: j += 1
            out.append(''.join(ch if ch == '\n' else ' ' for ch in src[i:j])); i = j
        elif c == "'":
            m = re.match(r"'(\\.|[^\\'])'", src[i:])
            if m:
                out.append(' ' * len(m.group(0))); i += len(m.group(0))
            else:
                out.append(c); i += 1
        else:
            out.append(c); i += 1
    return ''.join(out)

def cfg_test_spans(src):
    """Byte ranges of #[cfg(test)] items, over comment-stripped source."""
    spans = []
    for m in re.finditer(r'#\[cfg\(test\)\]', src):
        j = src.find('{', m.end())
        if j < 0: continue
        d = 0; k = j
        while k < len(src):
            if src[k] == '{': d += 1
            elif src[k] == '}':
                d -= 1
                if d == 0: break
            k += 1
        spans.append((m.start(), k+1))
    return spans

def enclosing(src, pos):
    best = None
    for m in re.finditer(r'\b(fn|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)', src[:pos]):
        best = (m.group(1), m.group(2))
    return best

def main():
    total = 0
    for dirpath, _, files in os.walk(ROOT):
        for fn in sorted(files):
            if not fn.endswith('.rs'): continue
            p = os.path.join(dirpath, fn)
            raw = open(p).read()
            s = strip(raw)
            spans = cfg_test_spans(s)
            for m in re.finditer(r'to_be_bytes', s):
                if any(a <= m.start() < b for a, b in spans): continue
                ln = s[:m.start()].count('\n') + 1
                enc = enclosing(s, m.start())
                total += 1
                print(f"{p}:{ln}\t{enc[0] if enc else '?'} {enc[1] if enc else '?'}\t{raw.splitlines()[ln-1].strip()[:90]}")
    print(f"\nTOTAL live to_be_bytes sites: {total}")

main()
