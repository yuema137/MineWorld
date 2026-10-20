# systems/posture/

**`mineworld-posture`**: where people may sit, and who is seated. A place file lists its sit points;
a person within reach sits on one, and stands up again.

```text
section     sit-points:          in a place file: key, at {x, y}, facing, group?
configure   posture              rule on sit, reach (750 mm), the two facts' audiences
owns        SitPoints (place), posture-state (person, while seated)
provides    sit { point }, stand {}
emits       place-sit-points, arrived (presence's), sat-down, stood-up
subscribes  its own facts, presence's arrived, movement's walk-started
depends on  presence, movement
```

```yaml
# places/cafe.yaml
sit-points:
  - { key: window-bench.1, group: window-bench, at: { x: 2700, y: 1160 }, facing: 180000 }
  - { key: window-bench.2, group: window-bench, at: { x: 3300, y: 1160 }, facing: 180000 }
```

A sit point is free floor in front of a seat: the server never knows a height. One person sits on a
point at a time. A seated person who moves, or sets off on a walk, stands up. Both offers are
complete (`ARC-34`), so any client or agent can sit from the observation alone.

With `bodies` enabled, a seated person is held where they sat (S34-b): walkers stop short of them,
`shove` is refused, and a sit point less than 260 mm from a solid, or outside the floor, is refused
when the world loads.

Specification: [`docs/decisions/ARC-147.md`](../../docs/decisions/ARC-147.md),
[`docs/CORE_CONCEPTS.md`](../../docs/CORE_CONCEPTS.md) §6.4.

```sh
cargo test -p mineworld-posture   # scratch worlds through the real loader, and a real restart
```
