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

### 3D presentation spike — REBUILDING

Currently runnable but being extended with three camera modes (F5 cycles first person → third
person rear → third person front). Will be marked ready when those land.

```bash
cd ~/mineworld-demos/3d && ./mineworld-3d
```

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
