# systems/group-activity/

**`mineworld-group-activity`**: inviting somebody, answering, joining, leaving. An activity done
together is a `Process` that this pack starts and ends.

```text
owns        Invitations, Participation, the group-activity Process
provides    invite, accept-invitation, decline-invitation, join-group-activity, leave-group-activity
emits       invited, invitation-accepted, invitation-declined, group-activity-started,
            joined-group-activity, left-group-activity, group-activity-ended
subscribes  its own facts, and presence's person-entered-place
depends on  presence
```

An accepted invitation either starts an activity at the place or joins the inviter's. The activity
ends when its hour is up (the kernel wakes this pack) or as soon as fewer than two people remain.
Walking into another place leaves it. Every ending names everyone who took part. That fact is what
`relationships` and a person's biography read.

An invitation stays open for 30 simulated minutes. That is two of `mineworld run`'s consult paces,
so every invitee is asked in time. The derivation is in `src/component.rs`.

The activity's `kind` (`coffee`, `walk`) is content this pack carries and never interprets.

```sh
cargo test -p mineworld-group-activity   # hand-built café, and an activity across a real restart
```

Design and evidence: [`step-09-social.md`](../../.structured-coding/plans/mvp0/step-09-social.md)
§4.2 (SD-10, C2). The command surface and the rules every pack follows are in
[`../../docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §3.

## Its section of the World's Interaction List

A world may say who invites, accepts and joins whom, how far an invitation reaches, how long things
last, and what its facts mean for history and perception, in `configure/group-activity.yaml` (listed
in `world.yaml`'s `configure:`), for everyone, for a class of person, or in one place:

```yaml
rules:
  - { action: invite, actor: noble, target: commoner, effect: forbid }
  - { action: accept-invitation, actor: noble, target: commoner, effect: forbid }
  - { action: join-group-activity, actor: noble, target: commoner, effect: forbid }
parameters:
  - { invitation_lifetime: 3600, invite_range: 3000, activity_length: 3600 }
  - { target: regular, invitation_lifetime: 600 }   # a class from configure/classes.yaml
regions:
  park: { parameters: [ { activity_length: 7200 } ] }
consequences:
  - { fact: group-activity-started, audience: participants }
  - { fact: joined-group-activity, actor: servant, biography: off }
```

| Action | `actor` | `target` |
| --- | --- | --- |
| `invite` | the inviter | the invitee |
| `accept-invitation` | the one accepting | the inviter |
| `join-group-activity` | the joiner | the member joined |

`decline-invitation` and `leave-group-activity` cannot be governed: a list can stop people coming
together, never trap anyone in an invitation or an activity. `accept-invitation` is governed because
accepting joins the inviter's activity. A forbidden request is refused `PermissionDenied`, and its
affordance shows it unavailable for that reason, before any other check of this pack.

| Parameter | Unit | Default | Bound | Looked up with |
| --- | --- | --- | --- | --- |
| `invitation_lifetime` | seconds | 1 800 | 1 … 86 400 | inviter, invitee, where it was made |
| `invite_range` | millimetres | 3 000 | 1 … 100 000 | inviter, invitee, the inviter's place |
| `activity_length` | seconds | 3 600 | 60 … 86 400 | inviter, invitee, where it starts |

| Fact | Roles | Audience (default → narrowest) | Biography (compiled) |
| --- | --- | --- | --- |
| `invited` | `actor` inviter, `target` invitee, `place` | participants | configurable (off) |
| `invitation-accepted`, `invitation-declined` | `actor` invitee, `target` inviter, `place` | participants | configurable (off) |
| `group-activity-started`, `group-activity-ended` | `place` | place → participants | configurable (on) |
| `joined-group-activity`, `left-group-activity` | `actor` the person, `place` | place → participants | configurable (on) |

Each invitation records the instant it lapses (`until`), and a controller reads that instant rather
than a lifetime of its own. The schema is [`../../docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md)
§4.2.
