# Measurements — the Santa Fe Depot, from HABS CA-1965

The station kit `stations/depot.yaml` cites these rows for every value no written source gives (S29-d
§5.2; `ARC-106`'s note of 2026-10-20). Measured 2026-10-20 by the S29-d implementer.

## 1. Method

1. Pick one photograph that shows the element and a known length **in the same plane**. HABS CA-1965-1
   (`habs-ca1965-22-1.jpg`, 1 920 × 1 530 px) is a frontal view of the front: the camera faces the gable
   end square on, so lengths in the façade's plane share one scale.
2. **The reference length** is the hall's end wall between the two towers: the hall is 16.8 m wide inside
   (55 ft; Wikipedia, Gaslamp Foundation), and its walls are taken as 0.6 m (the kit's `walls_mm.other`,
   a design value), so the end wall is 18.0 m outside. It spans x = 953 … 1 490 px at the plinth:
   **537 px = 18.0 m, 29.8 px/m**. The ground line is y = 1 073 px.
3. Measure each element's pixel length on the same photograph, divide by the scale, and round to 0.1 m.
4. **Error.** A single frontal measurement is ±10 %: the reference itself carries the wall-thickness
   assumption (±0.6 m of 18 m), and edges are read to ±3 px. The towers stand in the façade's plane (their
   ground arches open to the arcade behind them), so no perspective correction is made.
5. **Cross-check.** The five door bays in the arch measure 51 px wide (1.7 m: two leaves of 0.86 m, inside
   `VISUAL_SLICE.md` §3.2's 0.85 … 1.00 m) and 84 px to the crown of their round heads (2.8 m: a 2.15 m leaf
   under a transom). Both agree with the scale.

## 2. Values

All rows are from `habs-ca1965-22-1.jpg` unless stated; pixels are in that file's 1 920-px frame.

| Row | Element | Pixels | Result | Kit key |
| --- | --- | --- | --- | --- |
| M-1 | Tower shaft, width across the front | 743 … 993 = 250 | 8.4 m | `towers.plan_mm` |
| M-2 | Tower shaft, ground to the belfry's base | 1 073 − 710 = 363 | 12.2 m | `towers.shaft_mm` |
| M-3 | Belfry: width; base to cornice | 217; 710 − 597 = 113 | 7.3 m; 3.8 m | `towers.belfry_mm` |
| M-4 | Drum below the dome: width; height | 167; 597 − 533 = 64 | 5.6 m; 2.1 m | `towers.drum_mm` |
| M-5 | Dome: diameter; rise. Lantern: diameter; height. Finial | 143; 57. 61; 90. 17 | 4.8 m; 1.9 m. 2.1 m; 3.0 m. 0.6 m | `towers.dome_mm` (rise 2.0 m), `towers.lantern_mm` (2.9 m), `towers.finial_mm` |
| — | Tower, ground to the finial's top (the sum) | 1 073 − 370 = 703 | **23.6 m** | — (step-29 §6.2's 30 m placeholder retired) |
| M-6 | Hall eaves (the gable's foot) | 1 073 − 760 = 313 | 10.5 m | `shell.eaves_mm` |
| M-7 | Hall ridge (the gable's apex) | 1 073 − 632 = 441 | 14.8 m | `shell.ridge_mm`, `arch.gable_mm` |
| M-8 | Arch, clear width at the springing | 1 023 … 1 440 = 417 | 14.0 m | `arch.clear_mm[0]` |
| M-9 | Arch, ground to the crown's soffit | 1 073 − 733 = 340 | 11.4 m | `arch.clear_mm[1]` |
| M-10 | Door screen in the arch: height to the glazing; bays | 1 073 − 939 = 134; 5 counted | 4.5 m; 5 | `arch.screen_mm`, `arch.bays` |
| M-11 | Arcade (HABS CA-1965-3, `habs-ca1965-22-3.jpg`, an oblique view; **approximate, ±20 %**): arch crown and eaves read against the hall's eaves (M-6) on the same corner | — | ≈ 4.2 m; ≈ 5.2 m | `arcade.spring_mm` + `arcade.rise_mm`, `arcade.eaves_mm` |

## 3. What was not measured

- The arcade's pier pitch is a design value (the hall's bay, 51.8 m / 10 ≈ 5.2 m, rounded to S29-a's 5.4 m
  pier solids); HABS CA-1965-3 shows round arches on square piers at about that rhythm.
- The hall's clerestories (nine per long wall) follow NRHP 72000248 §7's "nine arched elements" and HABS
  CA-1965-3, which shows them in groups above the arcade's roof; their size is a design value.
- The wainscot height (8 ft, NRHP §7; 8½ ft, Wikipedia — the sources disagree) is S29-e's.
