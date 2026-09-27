# Human review queue

Subjective decisions waiting for the operator. Engineering status lives separately in
[`MVP_STATUS.md`](MVP_STATUS.md) — a demo parked here does **not** mean MineWorld is blocked.

**Updated:** 2026-09-26

---

## Ready to review

### 2D presentation spike — READY

```bash
cd ~/mineworld-demos/2d && ./mineworld-2d
```

`WASD` or arrow keys to walk · `Esc` to quit. Screenshots in
`clients/2d-spike/screenshots/`.

**The question, and it is a fork in the road:** procedural SVG geometry has reached its honest
ceiling. It now gets composition, palette, scale, light logic and readability right — all things
a sprite pipeline would still need someone to decide — but it cannot produce painted texture,
varied organic silhouettes, or character detail. The remaining distance to the references is
exactly those three things.

- Is the current look acceptable as the default 2D style, or
- do we switch to generated sprite art dropped into this same scene graph?

The swap is cheaper than it sounds: `props.json` already separates art from placement, so
layout, projection, the scale rule and the shadow layer all survive it unchanged.

Also worth judging while walking: player legibility against the NPCs, camera framing, and
whether the walking speed feels right.

### 3D presentation spike — READY

```bash
cd ~/mineworld-demos/3d && ./mineworld-3d
```

`W/S` walk · `A/D` strafe · mouse look · `Shift` jog · **`F5` cycles the camera** (first person →
third person rear → third person front) · `Esc` releases the mouse. The current mode is named in
the HUD. Screenshots in `clients/3d-spike/screenshots/` — 11 through 15 cover the three modes,
a scenic view, and the camera pulling in against a wall.

**What to judge, since none of it survives a screenshot:** walking speed (1.45 m/s, deliberately
a stroll rather than a sprint), eye height (1.66 m), mouse sensitivity (~3300 px for a full turn,
deliberately calm), FOV 70, third-person camera distance (3.40 m rear, 2.55 m front), and whether
the late-afternoon light and the street's scale feel like an ordinary town.

Camera distance, lift, FOV and sensitivity are all first-guess defaults left deliberately
untuned, because they are exactly what a person has to feel rather than be told.

**Known and accepted:** characters read as mannequins closer than ~3 m, first person shows no
body (deliberate — a first-person body is a separate problem and was not worth delaying this),
and backed hard against a wall the rear camera pulls in to ~0.6 m and fills the frame with the
back of a head. Predictable rather than pretty, which is the right trade for one raycast.

---

## Known and already decided, recorded so they are not re-asked

- **Characters read as mannequins closer than ~3 m.** Nothing CC0 ships clothed, rigged,
  realistically proportioned people, and `ARC-4` prefers a coherent simple character to a
  detailed mismatched one. A real character pipeline (MPFB2 meshes + CMU motion via Blender) is
  MVP-1 work, not a spike task.
- **Facial fidelity is deliberately not the reference's.** `ARC-4` scopes
  `04_character_closeup.png` to proportions, clothing and framing, explicitly not to faces,
  because a photoreal face standard would put character production beyond community reach.

## Parked, needs the operator eventually

- Three of the four 3D reference images and all the missing 2D coverage: an interior at room
  scale, and a night or overcast condition. Non-blocking for MVP-0.
