extends Node

## THE ONLY script that names an action type or submits a request (`ARC-47` R1, R2).
##
## Each composer says how to *ask* for an action — what its payload is made of — and nothing else.
## Whether the action exists, is available, is near enough or would succeed is the server's answer,
## read from the result; nothing here reads an affordance's verdict (R3), and nothing here refuses to
## send what the player chose. A request whose result never arrives is not resent: the server
## allocates identity, so a retry could do a thing twice (`INV-6`).

## The action types this client composes. `scripts/check_client_rules.py` reads this list: no other
## script may hold one of these as a string literal.
const COMPOSED := ["move"]

## The link whose `MineWorldClient` the requests go through.
var link: Node


## Asks to stand at `local` (integer millimetres, `+y` north) in `place`, facing `yaw_mdeg`
## (`PROTOCOL.md` §6.2). Returns the request's token, or "" when there is no seat to ask from.
func move(place: String, local: Dictionary, yaw_mdeg: Variant) -> String:
	var to := MineWorldSpace.location(place, local, yaw_mdeg)
	return link.client.submit("move", null, {"to": to})
