# `clients/protocol/evidence`

What the real Godot 4.7.2 engine produced, running `demo/` against a real
`mineworld server worlds/social-cafe --agent alice`. Regenerate with `../run.sh evidence` and
`../run.sh`.

```text
request-2d.json            the frames the 2D flavour submitted, as they went out
request-3d.json            the frames the 3D flavour submitted — the same seat, the same words,
                           and a reported position, which is the only difference AC-13 permits
transcript-2d.log          the run that produced the first
transcript-3d.log          the run that produced the second
transcript-3d-wanderer.log a third run, on the OTHER seat: Alice tells it what the first client
                           said, which is AC-15 seen from inside a real client
demo-scene.png             the windowed scene
transcript-window.log      the run that produced it
server.log                 the server those four runs were made against, in that order
simultaneous-2d.log        two Godot clients at once against one server, with the agent: three
simultaneous-3d.log        participants, one world instance in both, and Alice telling one of them
server-simultaneous.log    what the other had said
```

`tools/cli/tests/ac13_semantic_parity.rs` reads the two `request-*.json` files and compares them with
the server's own definition of a semantic core. They are frozen evidence: a change to them is a claim
that a client now sends something else, and it belongs with the run that produced it.
