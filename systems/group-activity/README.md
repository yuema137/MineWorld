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
