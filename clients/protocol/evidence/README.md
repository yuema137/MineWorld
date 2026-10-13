# `clients/protocol/evidence`

What the real Godot 4.7.2 engine produced, running `demo/` against a real
`mineworld server worlds/social-cafe --agent alice`. Regenerate with `../run.sh evidence` and
`../run.sh`.

```text
request-2d.json            the frames the 2D flavour submitted, as they went out: four `move` strides
                           from the door to the counter beside Alice, then the `talk`
request-3d.json            the frames the 3D flavour submitted — the same seat, the same strides, the
                           same words, and a reported position on the `talk`, which is the only
                           difference AC-13 permits
transcript-2d.log          the run that produced the first, on a fresh world
transcript-3d.log          the run that produced the second, on another fresh world
transcript-3d-wanderer.log a third run, on the OTHER seat of the second world: Alice tells it what the
                           first client said, which is AC-15 seen from inside a real client
server.log                 the two servers those runs were made against, in that order
simultaneous-2d.log        two Godot clients at once against one server, with the agent: three
simultaneous-3d.log        participants, one world instance in both, and Alice telling one of them
server-simultaneous.log    what the other had said
demo-scene.png             the windowed scene
transcript-window.log      the run that produced it
server-window.log          the server it was made against
affordances-market-town.log  `../run.sh affordances`: the module's live check (`checks/affordances_check.gd`)
                           against `worlds/market-town`, seated as `visitor` — every complete
                           affordance offered, a `buy` submitted unchanged and accepted, an
                           unavailable `give` sent and answered `too_far_away`, an incomplete `talk`
                           not sent, the revision rising
server-affordances.log     the server it was made against (saved to a scratch directory, since removed)
reconnect-market-town.log  `../run.sh reconnect`: the module's opt-in reconnect
                           (`checks/reconnect_check.gd`) against `worlds/market-town --town --hold 10`
                           — seated as `visitor` from the town, socket dropped without `leave`,
                           welcomed again `held` as the same Person. No resume secret is printed
server-reconnect.log       the server it was made against
admin-social-cafe.log      `../run.sh admin`: the `clock` frame after the welcome, the host's pause
                           (announced, a submit refused `paused`), resume, and a kick (`closing`
                           "kicked") — `checks/admin_check.gd`. The admin token is generated per run,
                           passed through the environment, and in no file here
server-admin.log           the server it was made against
```

Each AC-13 flavour runs against a world of its own because a client walks from where the world seated
it, in strides of under 2 m that the server accepts one at a time (`server/PROTOCOL.md` §6.2): frames
recorded on a world another run had already walked in could not be replayed against a fresh one.

`tools/cli/tests/ac13_semantic_parity.rs` reads the two `request-*.json` files and compares them with
the server's own definition of a semantic core. They are frozen evidence: a change to them is a claim
that a client now sends something else, and it belongs with the run that produced it.
