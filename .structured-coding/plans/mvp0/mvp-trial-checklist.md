# First playable MVP trial — operator checklist

**Status.** Draft checklist for the operator's hands-on trial of the first playable MVP (target 2026-11-30 to
2026-12-01). It is a test script, not a design authority: requirements live in
[`operator-playable-mvp-20261123.md`](operator-playable-mvp-20261123.md) §4, the nightlife design in
[`pr-nightlife.md`](pr-nightlife.md), the dialogue and memory design in
[`pr-playable-dialogue-memory.md`](pr-playable-dialogue-memory.md). The Chinese mirror
[`mvp-trial-checklist.zh-CN.md`](mvp-trial-checklist.zh-CN.md) is what the operator reads; this English file is
authoritative (`CLAUDE.md` §2.1 rule 5).

## 0. How the trial bundle is made

One command builds a fresh macOS bundle of one commit, with the 2D, 3D and 2D + 3D launchers, `START-HERE.txt` and
`manifest.json` (source commit, tool versions, every command with its time and exit code, every file's size and
SHA-256). It reuses `scripts/package.py assemble` and, with `--smoke`, `scripts/package.py smoke`:

```sh
git fetch origin
python3 scripts/trial_bundle.py --root ../_trial --smoke
```

Run it under the machine's shared heavy-job slot when other builds are running. The output is
`_trial/<YYYYMMDD>-<sha8>/`. The bundle's `runtime/launch.toml` (`tools/launch/LAUNCHER.md` §6) makes every mode
play Market Town: since #561 the 3D and 2D + 3D launchers otherwise start Social Café, because the packaged 3D
Market Town has not yet passed its Q-LS7 timing gate, so this trial is also its first measurement (check 2).
Every mode in `START-HERE.txt` uses one save folder, `trial-data/`, kept apart from the operator's normal saves,
so a world played in 2D continues in 3D. `START-HERE.txt` names every path relative to the trial folder. Integration
re-runs this command on the merged head, then the operator follows §2.

## 1. Dependency legend

| Mark | Meaning |
| --- | --- |
| main | Already on `main` (as of 2026-11-27, `c373b802`). |
| #565 | Dialogue + memory PR-B (launcher-supervised cognition, per-user stores, corrected memory). Open. |
| N2 | Nightlife N2: `sleep-until` and the single-player night advance. Designed and frozen; not started. |
| FE-b | Food-energy content and display: the kcal bar, the USD 10,000 endowment, per-serving kcal. Not yet a PR. |
| #551 | S34-d1: residents sit and stop bobbing, and route around one another. Open. |

An item whose mark is not `main` is tried only once that work is merged and the bundle is rebuilt. Until then it is
recorded `NOT RUN — waits for <mark>`, never as passed or failed.

## 2. The checks

Each check is "do this → you should see this". Record PASS, FAIL (with what was seen instead and a screenshot), or
NOT RUN with the reason.

| # | Do this | You should see | Needs |
| --- | --- | --- | --- |
| 1 | Run the `2D` line of `START-HERE.txt`. | A loading screen within seconds, then Market Town from above, with the clock at 07:00 on day 0. | main |
| 2 | Close every window. Run the `3D` line, and time it from pressing Enter to the first view of the street. | A loading screen, then Market Town on foot, at the time the 2D session reached (not 07:00 again). Write down the seconds to the first view: 3D Market Town has not passed its Q-LS7 timing gate, so this is its first measurement, not a pass or fail. | main (3D Market Town through the trial `launch.toml`) |
| 3 | Watch the street for a minute in either client. | Residents walk to work and about the town; nobody stands frozen, sinks into another, or bobs while seated. | main (walking); #551 (sitting, route-around) |
| 4 | Look at your own panel. | Money USD 10,000 and a kcal bar with today's target. | FE-b |
| 5 | Walk into the café or the corner store, select the shopkeeper and choose `Buy <item>` for one food and one drink. | Money drops by exactly the shown price, each item once in what you carry; the shop's stock drops by one. | main (buying); FE-b (USD 10,000 start) |
| 6 | Select yourself and choose `Eat <item>`, then `Drink <item>`. | The item leaves what you carry; the kcal bar rises by the item's kcal once. A zero-calorie drink adds no kcal. | FE-b |
| 7 | Let the clock pass 22:00 (the fastest day length in the settings menu, one world day per real hour, takes about 38 minutes from 07:00; or #12 once N2 lands), then try `Buy` at the café. | The purchase is refused with a "closed" reason; money and carried items do not change. The corner store still sells until 23:00. | main (N1 shop hours) |
| 8 | Select Alice (café, from 05:30) and choose `Talk to Alice`; tell her your name and one fact about you. | She answers about what you said, not a fixed line. | #565 |
| 9 | Close every window. Start again with the same line. Talk to Alice. | She remembers your name or your fact without being told again, and claims nothing you never told her. | #565 |
| 10 | Mid-session, note money, carried items, kcal and the clock. Quit from the Esc menu. Start the same mode again. | All four are exactly as noted; no second USD 10,000, no food eaten twice. | main (save); FE-b (kcal) |
| 11 | Close every window. Start the other client (2D after 3D, or 3D after 2D) straight away. | The same town, the same time, the same money and items; you stand where you left. | main |
| 12 | At home at night, choose `Sleep until 07:00`. | The night runs faster than real time: residents go home and sleep, and the shops close and open; you wake at 07:00 the next day. Quit during the night and restart: the world keeps the time it reached, and the rest of the night is ordinary time; an interrupted advance never resumes by itself. | N2 |
| 13 | Run the `2D + 3D` line. | Both windows open on the same world; a move in one shows in the other. Closing both saves and stops the server. | main |
| 14 | In the settings menu, change the day length and the language, then press Apply. | The clock speed changes; walking speed does not. Menus switch language. | main |

## 3. What to send back

For each check: its number, PASS / FAIL / NOT RUN, and for every FAIL what was seen and a screenshot. Include the
`manifest.json` `source_sha` of the bundle tried.
