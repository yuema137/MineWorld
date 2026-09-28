## The character integration slot.
##
## `VISUAL_SLICE.md` sec.10: the scene owns one named attachment point, the
## environment reads nothing from what is in it, and replacing the occupant is a
## change to the slot's contents and never to the scene.
##
## Why this exists as a named thing rather than as a line in the player. Two
## tracks are running at once: this one builds the street and the room, and
## `vis/3d-human-pipeline` rebuilds the character that `VIS-3D-GODOT-1` failed
## on (`ARC-17`). If the environment reached into the character's mesh,
## skeleton, materials or animation names, the character landing would be a
## merge that re-opens the environment. It does not reach in, and this file is
## where that is enforced.
##
## The whole contract the environment relies on:
##
##     mount(parent, height, seed) -> CharacterSlot     put a body in the scene
##     drive(delta, ground_speed)                       tell it how fast it is moving
##     occupant                                         a Node3D, for visibility only
##     describe()                                       what is currently in the slot
##
## Nothing else. No bone name, no clip name, no material, no mesh, no height
## beyond the one passed in. **The occupant is a technical stand-in until the
## approved character lands, and it gets no subjective polish here** -- polishing
## a placeholder on this branch would be work thrown away and would also move
## the target the operator is asked to judge (`ARC-11`).
class_name CharacterSlot
extends Node3D

const SLOT_NAME := "CharacterSlot"

## What is in the slot right now, for the review package to state honestly.
var source := "empty"
## The body itself. Read for `visible` and for nothing else.
var occupant: NPC = null


static func mount(parent: Node3D, height_m: float, seed_v: int) -> CharacterSlot:
	var s := CharacterSlot.new()
	s.name = SLOT_NAME
	parent.add_child(s)
	s._occupy(height_m, seed_v)
	return s


func _occupy(height_m: float, seed_v: int) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_v
	occupant = NPC.make(rng, NPC.Pose.PUPPET, height_m, true)
	occupant.name = "Occupant"
	# The body is authored facing its local +Z, which is npc.gd's convention and
	# the opposite of Godot's. The compensation lives here, at the slot, so that
	# a replacement occupant with the other convention is a one-line change in
	# one file rather than a hunt through the scene.
	occupant.rotation.y = PI
	add_child(occupant)
	source = "npc.gd reference build over the Human rig (%s)" % Human.SRC.get_file()


## The only thing the environment ever tells the character: how fast the body it
## is attached to is moving over the ground. Not a gait, not a clip, not a
## blend -- what to do about the speed is the character's own business.
func drive(delta: float, ground_speed_mps: float) -> void:
	if occupant != null:
		occupant.step(delta, ground_speed_mps)


func describe() -> String:
	return source
