# Lakeside's site and far-field profile — data record (S33-a C2)

This file records how `derive.py` read USGS data, how it chose the village's site on Lake Cuyamaca's north shore,
and the profile table that C3 copies into `presentation/mineworld-default/{3D,2D}/far_field/lakeside.yaml`. It is
written for the reviewer and for whoever re-derives the far field. The design is
[`pr-s33a-setting-far-field.md`](../../../.structured-coding/plans/mvp0/pr-s33a-setting-far-field.md) §§4, 5.2,
5.3; the method is RL-b C8's (`pr-rl-b-3d-budget.md:204–205`).

## Status

- **Prepared ahead of C0** (2026-10-18/19), on base `origin/main @ a1e05874`. C0 waits for S29-a and S29-c.
  The halt's places in `inputs.json` are S29-a rev. 3's ruled values, marked `provisional`; C0 re-reads them
  when S29-a merges and re-runs `profile` (offline).
- **The C2 Opus review ran on 2026-10-19** (REQUEST CHANGES, no blockers). It confirmed the site, the math,
  determinism, reach and the explanation of Cuyamaca Peak's bearing, and **approved test (d)** as a bounded
  addition (step-33 §4.0 puts the village north of the lane). Applied after it: the lake sector tested at
  `reach_m`; Stonewall Peak cited as GNIS Cush-pii (gaz_id 273917); every GNIS summit's height taken from the
  3DEP grid top round its point; the Cuyamaca Peak sentence corrected; the colour-sampling deviation recorded.
  Recorded, not changed: rule 3 reads the shoreline's direction from one NHD segment, so neighbouring
  candidates can differ by tens of degrees (`r0-01100` scores 0.0°, its neighbours 6.8° and 38.5°); averaging
  over ±50 m would be a change to §5.2 and is left to the operator if the site is questioned.

## Commands

Run from the repository root:

```sh
python3 tools/far-field/lakeside/derive.py fetch     # the only network step; resumable
python3 tools/far-field/lakeside/derive.py site      # the candidate table and the rule's choice
python3 tools/far-field/lakeside/derive.py profile   # the profile table; run twice, the output is identical
```

`profile` was run twice and the two outputs compared with `cmp`: identical (AC-a-4's first half).

## Fetches

| Service | URL template, as used | Calls | Date |
| --- | --- | --- | --- |
| USGS NHD, Waterbody – Large Scale (layer 12) | `https://hydro.nationalmap.gov/arcgis/rest/services/nhd/MapServer/12/query?geometry=-116.62,32.96,-116.55,33.01&geometryType=esriGeometryEnvelope&inSR=4326&outSR=4326&where=gnis_name = 'Lake Cuyamaca'&returnGeometry=true&f=json` (the exact URL is in `cache/nhd.json`) | 1 | 2026-10-18 |
| USGS GNIS (the National Map geonames service, layer 5 Landforms, class Summit) | `https://carto.nationalmap.gov/arcgis/rest/services/geonames/MapServer/5/query?…&where=gaz_name = '<name>' AND gaz_featureclass = 'Summit'` (each URL in `cache/gnis.json`) | 6 | 2026-10-18 |
| USGS 3DEP, Elevation Point Query Service | `https://epqs.nationalmap.gov/v1/json?x=<lon>&y=<lat>&wkid=4326&units=Meters&includeDate=false` | 3 663 points in `cache/epqs.json` (a few hundred, from the superseded Stonewall pointer grid and the first site's profile, are cached but unused) | 2026-10-18 to 2026-10-19 |

The EPQS service answered HTTP 504 a few times during the runs; `fetch` backs off and resumes from the cache, so
no point was invented or skipped.

## The lake

NHD holds Lake Cuyamaca (GNIS 00270731) as two polygons: `27727059` (0.901 km²) and `27727063` (0.413 km²),
FCODE 39004 (lake or pond, perennial). Their sum, 1.31 km², is larger than the 45 ha that step-33 §3.2 quotes
from Wikipedia; NHD maps the reservoir's outline at full pool. The far field uses NHD's outline (the design's
choice, §4) and the surface level 1 406 m (step-33 §3.2).

## Summits

| Summit | Position from | Height from |
| --- | --- | --- |
| Cuyamaca Peak | NGS "Cuyamaca reset", as recorded in RL-b's summit table (`58bc0c4` `skyline.gd:77`) | the same: 1 985 m |
| Middle Peak, North Peak, Cush-pii, Monument Peak, Garnet Peak | found by USGS GNIS name and point (gaz_id in the table below); the summit is the highest USGS 3DEP sample of a 25 m grid within 150 m of the GNIS point, because a GNIS point can stand off the true top (Garnet Peak reads 1 782.4 m at its GNIS point and 1 796.5 m at the grid's top, published ≈ 1 801 m) | USGS 3DEP at that sample |
| Cush-pii (formerly Stonewall Peak) | GNIS lists Stonewall Peak under its current name, **Cush-pii, gaz_id 273917** (the first run looked it up as "Stonewall Peak" and found nothing; the C2 review found the renamed entry). A nearby feature "Cush-pii ’Estik" (gaz_id 272128) is not the summit | 3DEP grid top 1 741.7 m (Wikipedia: 5 730 ft = 1 747 m; the 25 m grid can miss the very top by a few metres, < 0.1° at 3.5 km) |
| Volcan Mountain | **GNIS lists no summit of this name** (only the range "Volcan Mountains", gaz_id 274354). Omitted, as the design allows for a far summit that cannot be cited | — |

## Choosing the site (design §5.2)

Rule, as `inputs.json` `site_rule` encodes it: candidates every 50 m along the lake's north-facing shoreline
(outward normal within ±45° of north), 20 m inland. A candidate is kept when:

- **(a)** NHD water covers every sample from 40 m to 300 m on every bearing from 150° to 210°;
- **(b)** no land sample on 120°–240° within 20–80 m rises more than 3 m above the candidate's ground
  ("no land rises", read with a 3 m tolerance — approximate, for the review);
- **(c)** Cuyamaca Peak's crest angle clears the highest 3DEP sample on its ray, at 250 m steps out to 4 km, by
  ≥ 0.3°;
- **(d) — bounded addition, for the C2 review.** No NHD water within 50 m on the northern half (270°→90°).
  The village stands north of the lane: the cottages reach 23 m north of the origin and the platform's north end
  44 m (step-33 §4.0; `pr-s33d` §5.5). Without (d), rule 3 chose `r0-01900` (32.994408, −116.564242), which has
  water 40 m to its north on 280°–30° — a spit, where the cottages would stand in the lake. With (d) the choice
  is `r0-01100`. If the review rejects (d), the design's §5.2 needs the operator (a different site changes the
  ruled setting's details, not the setting).

Then the shoreline closest to east–west (the outward normal closest to north) wins; ties within 1° go to the
longer water extent along 180°.

**Chosen: `r0-01100` — 32.991981° N, 116.569022° W, ground 1 408.88 m (3DEP), eye 1 410.54 m.** Its shoreline
normal points due north; the water runs 460 m along 180°.

The candidate table, as `site` printed it:

| id | lat | lon | shore normal deg | off E-W deg | ground m | (a) water 150-210 | (b) no rise | near max m | (c) peak margin deg | (d) land north | extent 180 m | kept |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| r0-00350 | 32.990029 | -116.576092 | 335.3 | 24.7 | 1410.04 | False | True | 1411.39 | 2.57 | True | 340 | no |
| r0-00400 | 32.990186 | -116.575565 | 341.5 | 18.5 | 1409.68 | False | True | 1409.78 | 2.77 | True | 420 | no |
| r0-00450 | 32.990346 | -116.575093 | 334.5 | 25.5 | 1410.97 | False | True | 1410.2 | 2.92 | False | 440 | no |
| r0-00500 | 32.990569 | -116.574697 | 319.6 | 40.4 | 1408.94 | False | True | 1411.33 | 2.91 | False | 450 | no |
| r0-00600 | 32.991098 | -116.573741 | 351.6 | 8.4 | 1409.2 | True | True | 1409.0 | 2.89 | True | 450 | **yes** |
| r0-00650 | 32.991164 | -116.573311 | 325.7 | 34.3 | 1409.58 | True | True | 1409.02 | 2.91 | False | 450 | no |
| r0-00700 | 32.991356 | -116.572740 | 348.7 | 11.3 | 1409.13 | True | True | 1409.58 | 3.04 | True | 470 | **yes** |
| r0-00750 | 32.991219 | -116.572217 | 0.0 | 0.0 | 1408.85 | True | True | 1408.67 | 3.09 | True | 430 | **yes** |
| r0-00800 | 32.991339 | -116.571803 | 334.5 | 25.5 | 1408.76 | True | True | 1408.85 | 3.22 | False | 410 | no |
| r0-00900 | 32.991987 | -116.571252 | 321.7 | 38.3 | 1408.64 | False | True | 1409.41 | 3.4 | True | 460 | no |
| r0-00950 | 32.991989 | -116.570508 | 27.9 | 27.9 | 1408.72 | True | True | 1408.7 | 3.43 | True | 440 | **yes** |
| r0-01000 | 32.991909 | -116.570089 | 0.0 | 0.0 | 1408.79 | True | True | 1408.69 | 3.45 | True | 430 | **yes** |
| r0-01050 | 32.991933 | -116.569580 | 353.2 | 6.8 | 1408.89 | True | True | 1408.71 | 3.52 | True | 440 | **yes** |
| r0-01100 | 32.991981 | -116.569022 | 0.0 | 0.0 | 1408.88 | True | True | 1408.71 | 3.56 | True | 460 | **yes** |
| r0-01150 | 32.992062 | -116.568652 | 321.5 | 38.5 | 1408.85 | True | True | 1408.84 | 3.56 | True | 490 | **yes** |
| r0-01200 | 32.992168 | -116.567940 | 25.4 | 25.4 | 1408.8 | True | True | 1408.69 | 3.52 | True | 590 | **yes** |
| r0-01250 | 32.992184 | -116.567625 | 333.6 | 26.4 | 1408.77 | True | True | 1408.71 | 3.55 | False | 630 | no |
| r0-01300 | 32.992450 | -116.567261 | 321.5 | 38.5 | 1408.69 | False | True | 1408.89 | 3.58 | False | 670 | no |
| r0-01700 | 32.995319 | -116.564691 | 342.0 | 18.0 | 1408.65 | False | True | 1408.92 | 3.57 | True | 1100 | no |
| r0-01750 | 32.995217 | -116.564053 | 21.7 | 21.7 | 1408.75 | False | True | 1408.81 | 3.75 | True | 60 | no |
| r0-01900 | 32.994408 | -116.564242 | 0.0 | 0.0 | 1408.69 | True | True | 1408.57 | 3.57 | False | 940 | no |
| r0-01950 | 32.994479 | -116.563722 | 0.0 | 0.0 | 1408.53 | True | True | 1408.7 | 3.73 | False | 890 | no |
| r0-02050 | 32.994938 | -116.562866 | 13.4 | 13.4 | 1408.58 | False | True | 1408.8 | 3.52 | True | 890 | no |
| r0-02150 | 32.994222 | -116.562538 | 343.4 | 16.6 | 1408.61 | False | True | 1408.56 | 3.5 | False | 790 | no |
| r0-02600 | 32.993166 | -116.561108 | 35.6 | 35.6 | 1408.82 | False | True | 1408.89 | 3.64 | False | 0 | no |
| r0-02900 | 32.991319 | -116.561179 | 353.2 | 6.8 | 1408.69 | False | True | 1408.6 | 3.83 | False | 390 | no |
| r0-02950 | 32.991362 | -116.560623 | 0.0 | 0.0 | 1408.6 | False | True | 1408.7 | 3.66 | True | 310 | no |
| r1-00000 | 32.990243 | -116.581435 | 29.9 | 29.9 | 1412.49 | True | True | 1411.64 | 0.9 | True | 590 | **yes** |
| r1-00050 | 32.990019 | -116.580970 | 29.8 | 29.8 | 1411.6 | False | True | 1411.51 | 1.2 | False | 710 | no |
| r1-00100 | 32.989948 | -116.580560 | 0.0 | 0.0 | 1411.54 | False | True | 1411.09 | 1.49 | True | 270 | no |
| r1-00150 | 32.989988 | -116.580068 | 349.6 | 10.4 | 1411.63 | False | True | 1411.25 | 1.79 | True | 280 | no |
| r1-00200 | 32.989923 | -116.579479 | 6.2 | 6.2 | 1411.89 | False | True | 1411.17 | 2.19 | True | 230 | no |
| r1-00250 | 32.990027 | -116.579091 | 332.5 | 27.5 | 1412.36 | False | True | 1412.25 | 2.53 | False | 230 | no |
| r1-00300 | 32.990209 | -116.578503 | 0.0 | 0.0 | 1412.15 | False | True | 1412.04 | 2.54 | True | 240 | no |
| r1-00350 | 32.990195 | -116.577957 | 3.0 | 3.0 | 1411.87 | False | True | 1411.37 | 2.53 | True | 240 | no |
| r1-00400 | 32.990179 | -116.577426 | 1.6 | 1.6 | 1412.2 | False | True | 1411.16 | 2.49 | True | 280 | no |
| r1-00450 | 32.990166 | -116.576890 | 1.6 | 1.6 | 1413.37 | False | True | 1411.67 | 2.5 | True | 200 | no |
| r1-01300 | 32.986759 | -116.579885 | 0.0 | 0.0 | 1413.8 | False | True | 1412.39 | 1.74 | False | 490 | no |
| r1-01500 | 32.985404 | -116.578798 | 33.0 | 33.0 | 1413.61 | False | True | 1413.55 | 1.76 | False | 410 | no |
| r1-01550 | 32.985332 | -116.578400 | 0.0 | 0.0 | 1413.47 | False | True | 1411.96 | 1.74 | True | 410 | no |
| r1-01600 | 32.985413 | -116.577965 | 338.3 | 21.7 | 1411.96 | False | True | 1412.47 | 1.73 | True | 390 | no |
| r1-01700 | 32.985935 | -116.577277 | 345.2 | 14.8 | 1412.76 | False | True | 1414.21 | 1.86 | True | 210 | no |
| r1-03700 | 32.990556 | -116.586358 | 345.2 | 14.8 | 1412.5 | False | True | 1415.17 | -3.01 | True | 190 | no |
| r1-03750 | 32.990387 | -116.585734 | 20.4 | 20.4 | 1412.78 | False | True | 1412.83 | -2.17 | True | 190 | no |
| r1-03800 | 32.990233 | -116.585325 | 0.0 | 0.0 | 1413.12 | False | True | 1412.18 | -1.85 | False | 190 | no |
| r1-03850 | 32.990230 | -116.584889 | 333.4 | 26.6 | 1412.64 | False | True | 1411.65 | -1.56 | False | 210 | no |
| r1-03900 | 32.990433 | -116.584397 | 336.6 | 23.4 | 1412.44 | False | True | 1412.4 | -1.12 | True | 240 | no |
| r1-03950 | 32.990536 | -116.583834 | 349.6 | 10.4 | 1413.83 | False | True | 1413.14 | -0.66 | True | 280 | no |
| r1-04000 | 32.990566 | -116.583262 | 0.0 | 0.0 | 1415.63 | False | True | 1414.33 | -0.09 | True | 340 | no |
| r1-04050 | 32.990487 | -116.582704 | 8.5 | 8.5 | 1415.39 | False | True | 1413.4 | 0.21 | True | 460 | no |
| r1-04100 | 32.990396 | -116.582154 | 15.4 | 15.4 | 1412.64 | True | True | 1412.26 | 0.38 | True | 470 | **yes** |
| r1-04150 | 32.990312 | -116.581648 | 10.2 | 10.2 | 1413.21 | True | True | 1412.32 | 0.78 | True | 530 | **yes** |

chosen: r0-01100

## The profile (`profile`, verbatim)

C3 copies these values into the two far-field files, value by value, with the row each comes from.

```text
site r0-01100  lat 32.991981  lon -116.569022  ground 1408.88 m (3DEP)  eye 1410.54 m

## Summits
| summit | layer | lat | lon | height m | bearing deg | distance m | crest angle deg | source |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Cuyamaca Peak | mid | 32.946743 | -116.606724 | 1985.0 | 215.0 | 6138 | 5.338 | NGS 'Cuyamaca reset' as recorded in RL-b's table (58bc0c4 skyline.gd:77); NGS 'Cuyamaca reset', the RL-b summit table (pr-rl-b-3d-budget.md:619-630; reverted skyline.gd SUMMITS :77) |
| Middle Peak | mid | 32.980253 | -116.600363 | 1797.4 | 246.0 | 3201 | 6.912 | the highest USGS 3DEP sample within 150 m of USGS GNIS gaz_id 272446 (Middle Peak); USGS 3DEP (EPQS) at that sample |
| North Peak | mid | 33.006029 | -116.586173 | 1825.5 | 314.3 | 2236 | 10.627 | the highest USGS 3DEP sample within 150 m of USGS GNIS gaz_id 272711 (North Peak); USGS 3DEP (EPQS) at that sample |
| Cush-pii (formerly Stonewall Peak) | mid | 32.961007 | -116.571923 | 1741.7 | 184.5 | 3455 | 5.478 | the highest USGS 3DEP sample within 150 m of USGS GNIS gaz_id 273917 (Cush-pii); USGS 3DEP (EPQS) at that sample |
| Monument Peak | far | 32.892316 | -116.420637 | 1913.0 | 128.6 | 17736 | 1.554 | the highest USGS 3DEP sample within 150 m of USGS GNIS gaz_id 272543 (Monument Peak); USGS 3DEP (EPQS) at that sample |
| Garnet Peak | far | 32.925592 | -116.458847 | 1796.5 | 125.7 | 12655 | 1.698 | the highest USGS 3DEP sample within 150 m of USGS GNIS gaz_id 1660665 (Garnet Peak); USGS 3DEP (EPQS) at that sample |
| Volcan Mountain | far | — | — | — | — | — | — | no GNIS summit named Volcan Mountain (cache/gnis.json: 0 hits) |

## Near rim, per 5-degree sector (land samples only; the highest angle)
| bearing deg | crest height m | distance m | angle deg | lat | lon |
| --- | --- | --- | --- | --- | --- |
| 0 | 1442.7 | 800 | 2.302 | 32.999176 | -116.569022 |
| 5 | 1429.2 | 800 | 1.336 | 32.999148 | -116.568274 |
| 10 | 1434.2 | 1600 | 0.839 | 33.006152 | -116.566042 |
| 15 | 1427.3 | 1600 | 0.596 | 33.005880 | -116.564581 |
| 20 | 1468.4 | 3200 | 1.023 | 33.019023 | -116.557283 |
| 25 | 1444.3 | 3200 | 0.592 | 33.018062 | -116.554517 |
| 30 | 1462.3 | 3200 | 0.915 | 33.016903 | -116.551862 |
| 35 | 1476.2 | 3200 | 1.164 | 33.015553 | -116.549337 |
| 40 | 1479.0 | 3200 | 1.213 | 33.014025 | -116.546962 |
| 45 | 1485.3 | 3200 | 1.326 | 33.012328 | -116.544755 |
| 50 | 1460.0 | 3200 | 0.873 | 33.010477 | -116.542733 |
| 55 | 1448.1 | 3200 | 0.661 | 33.008484 | -116.540911 |
| 60 | 1441.5 | 3200 | 0.542 | 33.006367 | -116.539303 |
| 65 | 1438.2 | 3200 | 0.483 | 33.004139 | -116.537921 |
| 70 | 1442.3 | 3200 | 0.556 | 33.001820 | -116.536777 |
| 75 | 1441.2 | 3200 | 0.536 | 32.999425 | -116.535877 |
| 80 | 1443.0 | 3200 | 0.570 | 32.996974 | -116.535230 |
| 85 | 1461.3 | 3200 | 0.897 | 32.994485 | -116.534841 |
| 90 | 1450.9 | 3200 | 0.710 | 32.991976 | -116.534711 |
| 95 | 1498.3 | 3200 | 1.559 | 32.989468 | -116.534842 |
| 100 | 1520.5 | 3200 | 1.956 | 32.986979 | -116.535234 |
| 105 | 1458.5 | 1600 | 1.713 | 32.988256 | -116.552452 |
| 110 | 1481.3 | 1600 | 2.528 | 32.987059 | -116.552902 |
| 115 | 1483.1 | 1600 | 2.593 | 32.985899 | -116.553475 |
| 120 | 1539.5 | 3200 | 2.297 | 32.977588 | -116.539313 |
| 125 | 1500.4 | 3200 | 1.597 | 32.975471 | -116.540921 |
| 130 | 1513.7 | 3200 | 1.835 | 32.973480 | -116.542744 |
| 135 | 1470.0 | 3200 | 1.051 | 32.971629 | -116.544766 |
| 140 | 1469.5 | 3200 | 1.044 | 32.969934 | -116.546973 |
| 145 | 1465.8 | 3200 | 0.976 | 32.968406 | -116.549347 |
| 150 | 1484.3 | 3200 | 1.308 | 32.967057 | -116.551871 |
| 155 | 1448.9 | 3200 | 0.675 | 32.965898 | -116.554526 |
| 160 | 1438.0 | 1600 | 0.975 | 32.978459 | -116.563155 |
| 165 | 1438.1 | 3200 | 0.481 | 32.964183 | -116.560144 |
| 170 | 1520.9 | 3200 | 1.963 | 32.963640 | -116.563066 |
| 175 | 1501.7 | 3200 | 1.619 | 32.963312 | -116.566033 |
| 180 | 1637.9 | 3200 | 4.058 | 32.963203 | -116.569022 |
| 185 | 1673.0 | 3200 | 4.687 | 32.963312 | -116.572011 |
| 190 | 1561.4 | 3200 | 2.688 | 32.963640 | -116.574978 |
| 195 | 1485.3 | 3200 | 1.326 | 32.964183 | -116.577900 |
| 200 | 1454.8 | 3200 | 0.781 | 32.964938 | -116.580753 |
| 205 | 1458.2 | 3200 | 0.842 | 32.965898 | -116.583518 |
| 210 | 1447.8 | 3200 | 0.654 | 32.967057 | -116.586173 |
| 215 | 1445.0 | 3200 | 0.604 | 32.968406 | -116.588697 |
| 220 | 1488.7 | 3200 | 1.387 | 32.969934 | -116.591071 |
| 225 | 1564.0 | 3200 | 2.735 | 32.971629 | -116.593278 |
| 230 | 1624.1 | 3200 | 3.812 | 32.973480 | -116.595300 |
| 235 | 1683.8 | 3200 | 4.880 | 32.975471 | -116.597123 |
| 240 | 1748.0 | 3200 | 6.029 | 32.977588 | -116.598731 |
| 245 | 1788.9 | 3200 | 6.762 | 32.979815 | -116.600114 |
| 250 | 1743.4 | 3200 | 5.948 | 32.982134 | -116.601260 |
| 255 | 1709.9 | 3200 | 5.348 | 32.984528 | -116.602161 |
| 260 | 1669.0 | 3200 | 4.616 | 32.986979 | -116.602810 |
| 265 | 1577.0 | 3200 | 2.968 | 32.989468 | -116.603202 |
| 270 | 1435.5 | 800 | 1.787 | 32.991981 | -116.577600 |
| 275 | 1450.0 | 800 | 2.827 | 32.992608 | -116.577567 |
| 280 | 1463.8 | 800 | 3.811 | 32.993230 | -116.577470 |
| 285 | 1475.0 | 800 | 4.615 | 32.993843 | -116.577308 |
| 290 | 1484.3 | 800 | 5.282 | 32.994441 | -116.577083 |
| 295 | 1575.9 | 1600 | 5.915 | 32.998061 | -116.584571 |
| 300 | 1606.7 | 1600 | 7.019 | 32.999175 | -116.583880 |
| 305 | 1644.1 | 1600 | 8.359 | 33.000233 | -116.583076 |
| 310 | 1664.8 | 1600 | 9.098 | 33.001229 | -116.582165 |
| 315 | 1684.8 | 1600 | 9.814 | 33.002155 | -116.581154 |
| 320 | 1697.2 | 1600 | 10.259 | 33.003003 | -116.580051 |
| 325 | 1684.9 | 1600 | 9.817 | 33.003768 | -116.578863 |
| 330 | 1669.4 | 1600 | 9.264 | 33.004442 | -116.577601 |
| 335 | 1639.7 | 1600 | 8.200 | 33.005022 | -116.576273 |
| 340 | 1613.7 | 1600 | 7.269 | 33.005502 | -116.574890 |
| 345 | 1570.7 | 1600 | 5.727 | 33.005880 | -116.573463 |
| 350 | 1529.9 | 1600 | 4.268 | 33.006152 | -116.572002 |
| 355 | 1519.1 | 1600 | 3.881 | 33.006315 | -116.570517 |

## Lake sector (rays that meet NHD water at reach_m = 80 m, where the far field's water begins)
bearings: 86 -> 249 deg (rounded outward: 85 -> 250)
NHD features: 27727059 0.901 km2, 27727063 0.413 km2

## Slots (scene frame -> bearing from the origin; true north is -z)
| slot | place | centre x, z m | bearing deg | distance m | footprint m | inside reach_m | status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| station | station | 19.0, 0.0 | 90.0 | 19.0 | 8.0 | True | S29-a not merged; C0 re-reads |
| landmark | pier | 3.0, 13.0 | 167.0 | 13.3 | 20.0 | True | frozen source |

## reach_m check (every halt floor corner and the pier head inside 80 m)
farthest corner 45.0 m -> inside
```

### Reading the profile

- **The lake lies south.** The lake sector is **85°→250°** (rounded outward), which contains 180°, as F-S33-2
  requires. It is tested at `reach_m` (80 m), where the far field's water begins (`shore_m` 80), so the far
  field's water meets S33-d's near lake. (The first run tested at 40 m, inside the near field, and gave a
  narrower 108°→235°; the C2 review caught it.)
- **Cush-pii (Stonewall Peak) stands straight across the water**, at 184.5°, 3.5 km, 5.5° above the horizon.
  **Cuyamaca Peak** is at 215.0°, 6.1 km, 5.3°. Step-33 §3.2 computed 209.0° at 5.04 km from the lake's named
  coordinate; the site lies 1.24 km at 60° (east-north-east) of that coordinate, which turns the bearing 6°
  and adds 1.1 km — the expected difference, explained.
- **North Peak** rises behind the village at 314.3°, 2.2 km, 10.6°: the highest form in the circle. Middle Peak
  is at 246.0°, 3.2 km, 6.9°. Monument and Garnet Peaks (the Laguna Mountains) stand at 128.6° and 125.7°,
  12–18 km, about 1.6°–1.7° above the horizon. The near rim on those bearings reaches 1.6°–1.8° (the 125° and 130°
  rows, 3.2 km out), so the Lagunas sit **at or just below** the near crest: by the design's rule (§5.4, far
  layer only if the profile shows it above the near rim) C3 omits them unless a finer sample shows them clear,
  and records the decision.
- Across the lake on 180°–185° the rim at 3.2 km climbs to 4.1°–4.7° (Cush-pii's shoulders); on 210°–215° it
  drops to 0.6°, which is where Cuyamaca Peak (5.3°) shows clear above it, as test (c) required.
- No profile point lies inside `reach_m` (80 m): the nearest rim sample used is 800 m away.
- The slots: the station 19.0 m at 90°, the landmark (the pier) 13.3 m at 167°. Both lie inside `reach_m`,
  which `filled` slots may (step-29 §6.7). The station's values are provisional until S29-a merges.

## Visibility

The design asks for the IMPROVE network's annual mean standard visual range at Agua Tibia (`AGTI1`). This
session could not read a stated annual mean visual range for AGTI1 from a primary source: the EPA's 2020
regional-haze technical addendum lists AGTI1 light extinction for the clearest and most-impaired days, but its
table was too large to read through the tools available, and no IMPROVE or CIRA page stating the annual mean was
found. Per the design's fallback (§4), **`visibility_m` 80 000, "approximate; replace with IMPROVE AGTI1"**,
RL-e0's value. The C2 review or C3 replaces it if the IMPROVE data viewer (CIRA, Colorado State University) is
read.

## Water colours (reference 06, in pixels)

Sampled from `presentation/mineworld-default/3D/references/06_lake_trail.png` (1 448 × 1 086, 8-bit RGB) with a
standard-library PNG reader in a scratch script (not committed) — a recorded deviation from the design's §4,
which names Godot's `Image` API; no local Godot slot was used for it, and the reader was checked on known
pixels (below). Each value is the mean of a pixel box, corners
inclusive of the first and exclusive of the second:

| Use | Box (x0, y0)–(x1, y1) | Mean | Where |
| --- | --- | --- | --- |
| **deep** | (900, 480)–(1 000, 500) | **#5f6363** | open water mid-lake, beyond the footbridge |
| **shallow** | (1 050, 690)–(1 110, 720) | **#6b747f** | near the shore, left of the ducks |
| rejected | (860, 490)–(940, 520) | #676967 | mixed with the far shore's reflection |
| rejected | (1 150, 640)–(1 250, 680) | #3f403e | under the pier: shadow |
| rejected | (1 240, 700)–(1 300, 740) | #aaaaaf | sun glare |

The decoder was checked on the sky (950, 40)–(1 050, 80) → #a2b0cb and the sun (1 340, 230)–(1 360, 250) →
#fdfcf7. The reference's water is a near-neutral grey-blue: it reflects a bright evening sky. The current
design-content colours (`#3d5f63`, `#6a9295`) are greener than the reference; C3 uses the values above, and the
operator's V-gate look decides.
