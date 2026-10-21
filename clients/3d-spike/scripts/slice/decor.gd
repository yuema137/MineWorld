## A depiction's decorative figures (S33-g §5.3, F-S33g-3): presentation people with no world entity,
## drawn as the depiction states them (`SliceDepiction.decor()`), each from the extras the depiction
## asks for (`SliceDepiction.extras_from()`, step-26 E-2) and never as a placeholder (S26 X-16).
##
## Generic: nothing here names a world, a place or a person. A figure is drawn only as a cast body; when
## the pack resolves no cast body for an extra (no extras, a Git LFS pointer checkout), the figure is
## left out with a warning rather than drawn as the town figure. No figure sits, fishes or enters a place
## (the depiction keeps them on its street), none has a collider, and none can be targeted: a decor
## figure is never mistaken for a person of the world.
##
## The characters are taken from the extras in order, offset by a hash of the world key, so the same
## world draws the same passers-by every run and no two decor figures share a character while the pool
## has enough of them.
class_name SliceDecor
extends RefCounted

## The most decor figures a depiction may state (step-33 §5: "few, ≤ 6"; S33-g §5.3 takes 5).
const MAX := 5
## A walking pace (S33-g §5.3, the S26 walk clip's band).
const WALK_MPS := 1.3

## Every decor figure drawn, in order: { index, character, pose, at } (read by the probes).
static var drawn: Array = []


## Draws `depiction`'s decor figures under `root` from `cast`. Returns the figures drawn.
static func build(root: Node3D, depiction: SliceDepiction, cast: SliceCast) -> Array[NPC]:
	drawn = []
	var out: Array[NPC] = []
	var wanted := depiction.decor()
	if wanted.is_empty():
		return out
	if wanted.size() > MAX:
		push_warning("decor: %d figures stated, at most %d are drawn" % [wanted.size(), MAX])
		wanted = wanted.slice(0, MAX)
	if cast == null or cast.extras.is_empty():
		push_warning("decor: no extras to draw decor figures from; none drawn")
		return out
	var offset := posmod(depiction.key.hash(), cast.extras.size())
	for i in wanted.size():
		var d: Dictionary = wanted[i]
		var character := String(cast.extras[(offset + i) % cast.extras.size()])
		var spec := cast.body_for(character)
		if String(spec.get("kind", "")) != "cast":
			push_warning("decor: figure %d (%s) has no cast body; left out, never a placeholder"
				% [i, character])
			continue
		var rng := RandomNumberGenerator.new()
		rng.seed = hash("%s/decor/%d" % [depiction.key, i])
		var walking := String(d.get("pose", "")) == "walking"
		var n := NPC.make_body(rng, NPC.Pose.WALK if walking else NPC.Pose.STAND, 0.0, spec)
		n.name = "Decor_%d" % i
		n.set_meta("decor", true)
		n.set_meta("character", character)
		n.set_meta("mw_category", "characters")
		root.add_child(n)
		var at: Vector3
		if walking:
			n.path_a = d["from"]
			n.path_b = d["to"]
			n.speed = WALK_MPS
			at = n.path_a
		else:
			at = d["at"]
			n.rotation.y = deg_to_rad(float(d.get("yaw", 0.0)))
		n.global_position = at
		drawn.append({"index": i, "character": character, "pose": "walking" if walking else "standing",
			"at": at})
		print("[decor] figure %d  [character %s, %s at %s]" % [i, character,
			"walking" if walking else "standing", at])
		out.append(n)
	return out
