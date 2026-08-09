# wb-label — raw grid output, verbatim

Real `cl.exe` 16.00.11886.00 under wibo, image sha256 `c80981…6258`.
`work/wb-label/labgrid.py` (in-the-middle form) and `work/wb-label/seedgrid.py`
(counterfactual beside in-the-middle). `lead = stride − base`.

## 1. `labgrid.py` — the in-the-middle grid

```text
probe              mode            base  stride  extra  minted  raw a0/P/a1/a2
ctl-plain          /O1 /GS- /c        5       5      0       5  2568/2573/2578/2583
ctl-plain          /Ox /GS- /c        4       4      0       3  2556/2560/2564/2568
ctl-leaf           /O1 /GS- /c        5       1      -       1  2566/None/2572/2577
ctl-leaf           /Ox /GS- /c        4       1      -       0  2554/None/2559/2563
ctl-for            /O1 /GS- /c        5       9      4       7  2573/2582/2587/2592
ctl-for            /Ox /GS- /c        4       9      5       5  2561/2570/2574/2578
X1-switch-table    /O1 /GS- /c        5       6      1       5  2586/2592/2597/2602
X1-switch-table    /Ox /GS- /c        4       7      3       3  2574/2581/2585/2589
X2-for-if          /O1 /GS- /c        5       7      2       5  2574/2581/2586/2591
X2-for-if          /Ox /GS- /c        4       7      3       3  2562/2569/2573/2577
X3-while-ret       /O1 /GS- /c        5       8      3       5  2573/2581/2586/2591
X3-while-ret       /Ox /GS- /c        4       7      3       3  2561/2568/2572/2576
X4-try-2catch      /O1 /GS- /EHsc /c  5      28     10      28  2574/2589/2607/2612
X4-try-2catch      /Ox /GS- /EHsc /c  4      25     11      24  2562/2577/2591/2595
X5-switch-in-for   /O1 /GS- /c        5       8      3       5  2590/2598/2603/2608
X5-switch-in-for   /Ox /GS- /c        4      10      6       3  2578/2588/2592/2596
X6-unroll          /O1 /GS- /c        5       7      2       5  2573/2580/2585/2590
X6-unroll          /Ox /GS- /c        4      11      7       3  2561/2572/2576/2580
p-none             /O1 /GS- /c        5       5      0       5  2568/2573/2578/2583
p-none             /Ox /GS- /c        4       4      0       3  2556/2560/2564/2568
p-if               /O1 /GS- /c        5       5      0       5  2570/2575/2580/2585
p-if               /Ox /GS- /c        4       5      1       3  2558/2563/2567/2571
p-ifelse           /O1 /GS- /c        5       5      0       5  2571/2576/2581/2586
p-ifelse           /Ox /GS- /c        4       4      0       3  2559/2563/2567/2571
p-for              /O1 /GS- /c        5       7      2       5  2573/2580/2585/2590
p-for              /Ox /GS- /c        4      14     10       5  2561/2575/2579/2583
p-while            /O1 /GS- /c        5       7      2       5  2572/2579/2584/2589
p-while            /Ox /GS- /c        4      14     10       5  2560/2574/2578/2582
p-dowhile          /O1 /GS- /c        5       6      1       5  2572/2578/2583/2588
p-dowhile          /Ox /GS- /c        4       5      1       3  2560/2565/2569/2573
p-switch           /O1 /GS- /c        5       6      1       5  2586/2592/2597/2602
p-switch           /Ox /GS- /c        4       7      3       3  2574/2581/2585/2589
H1-if-in-while     /O1 /GS- /c        5       7      2       5  2573/2580/2585/2590
H1-if-in-while     /Ox /GS- /c        4       7      3       3  2561/2568/2572/2576
H2-ifelse-in-for   /O1 /GS- /c        5       7      2       5  2575/2582/2587/2592
H2-ifelse-in-for   /Ox /GS- /c        4       7      3       3  2563/2570/2574/2578
H3-two-ifs         /O1 /GS- /c        5       5      0       5  2571/2576/2581/2586
H3-two-ifs         /Ox /GS- /c        4       5      1       3  2559/2564/2568/2572
H4-switch-in-while /O1 /GS- /c        5       8      3       5  2589/2597/2602/2607
H4-switch-in-while /Ox /GS- /c        4      10      6       3  2577/2587/2591/2595
H5-for-in-for      /O1 /GS- /c        5       9      4       5  2577/2586/2591/2596
H5-for-in-for      /Ox /GS- /c        4      14     10       3  2565/2579/2583/2587
H6-dowhile-in-if   /O1 /GS- /c        5       6      1       5  2573/2579/2584/2589
H6-dowhile-in-if   /Ox /GS- /c        4       6      2       3  2561/2567/2571/2575
```

**The control held on 22 of 22 rows** (`base` is 5 at `/O1` and 4 at `/Ox`
everywhere, measured in each obj as `first(a2) − first(a1)`).

**The `/Gy` pre-pass, re-verified at breadth.** Every TU here has four
functions, and `first(a0)` at `/O1` minus `first(a0)` at `/Ox` is **12 = 3 × 4**
on **22 of 22** rows — including the EH row, the switch rows and the loop rows.
`coff::plan_labels`' *"three slots per function, whatever kind"* is confirmed on
a population it was never re-checked against.

## 2. `seedgrid.py` — the counterfactual form beside the in-the-middle form

```text
cell        mode            cf z.$M cf lead mid base mid stride
s_ctl       /O1 /GS- /c        2555     +0        5       1
s_decl8     /O1 /GS- /c        2571    +16        5       1
s_loc2      /O1 /GS- /c        2557     +2        5       1
s_loc8      /O1 /GS- /c        2563     +8        5       1
s_loop      /O1 /GS- /c        2562     +7        5       3
s_dowhile   /O1 /GS- /c        2560     +5        5       2

s_ctl       /Ox /GS- /c        2549     +0        4       1
s_decl8     /Ox /GS- /c        2565    +16        4       1
s_loc2      /Ox /GS- /c        2551     +2        4       1
s_loc8      /Ox /GS- /c        2557     +8        4       1
s_loop      /Ox /GS- /c        2562    +13        4       9
s_dowhile   /Ox /GS- /c        2554     +5        4       2
```

## 3. `triple.cod` — the `/FAsc` listing of §4.2.2's byte-identical triple

`c2rs listing work/wb-label/probe/triple.cpp --flag /O1 --flag /GS- --flag /c`,
9 PROC / 9 `.text` COMDAT / 11 PUBLIC, obj 2777 B.

```text
  a0        $M2613 $M2614 $T2615
  p_dowhile   (no $M)   label printed: $LL3@p_dowhile
  a1        $M2620 $M2621 $T2622        -> stride(p_dowhile) = 7 - 5 = 2
  p_forever   (no $M)   label printed: $LL3@p_forever
  a2        $M2629 $M2630 $T2631        -> stride(p_forever) = 9 - 5 = 4
  p_goto      (no $M)   label printed: $top$2561
  a3        $M2636 $M2637 $T2638        -> stride(p_goto)    = 7 - 5 = 2
  p_mulli     (no $M)   NO label printed
  a4        $M2644 $M2645 $T2646        -> stride(p_mulli)   = 8 - 5 = 3
```

All four strides reproduce `LABEL_COUNTER.md` §4.2.1 exactly (`leaf-dowhile` 2,
`leaf-forever` 4, `leaf-goto-back` 2, `leaf-for-k` 3).
