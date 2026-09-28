## The slice's player: the spike's controller, with the character in a slot.
##
## Deliberately a subclass and not a copy. `player.gd` owns one movement
## implementation, three cameras observe it, and `--drive` measures that a
## camera switch changes nothing the controller owns. Duplicating any of that
## to add a slot would mean two controllers drifting apart, and the reuse rule
## in `REUSE_POLICY.md` exists for exactly this case.
##
## The override is one method: where the base builds a body directly, this
## mounts a `CharacterSlot` and lets the slot build it. Everything else --
## speeds, capsule, gravity, look, the three cameras, the mode switch -- is the
## base class unchanged, so the slice inherits the movement work rather than
## re-deciding it.
class_name SlicePlayer
extends Player

## The named attachment point `VISUAL_SLICE.md` sec.10 requires.
var slot: CharacterSlot = null


func _make_body() -> void:
	slot = CharacterSlot.mount(self, BODY_HEIGHT, BODY_SEED)
	body = slot.occupant
