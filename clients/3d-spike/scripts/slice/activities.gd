## ARC-173 (S33-g §5.4): a figure is drawn doing an activity -- fishing, rowing, baking at the oven --
## only from a disclosed component or event that names it. An undisclosed Process is not drawn, for any
## pack.
##
## Today no pack discloses an activity of another person (the fishing pack tells only its angler,
## F-S33g-1; Q-S33g-1 ruled 2026-10-20: nobody else is drawn fishing until it discloses), so no figure
## may hold an activity clip at all. A body's animation tree may play only the clips that need no
## disclosure: locomotion (standing, walking, jogging -- drawn from the observed position) and sitting
## (a posture, drawn only where the world seats a person). Anything else a figure's tree can play is an
## activity drawn without a disclosure, and `undisclosed` names it, so the probe fails by name.
class_name SliceActivities
extends RefCounted

## The clips a figure plays without any disclosure naming an activity (`Human._build_tree`).
const UNCONDITIONED := ["Stand", "Walk", "Jog_Fwd", "Sit"]


## The clips `n`'s body can play that are activities (none may be, until a disclosure names one).
static func undisclosed(n: NPC) -> PackedStringArray:
	var out := PackedStringArray()
	if n == null or n.body == null:
		return out
	for tree in n.body.find_children("*", "AnimationTree", true, false):
		for clip in clips((tree as AnimationTree).tree_root):
			if not clip in UNCONDITIONED and not clip in out:
				out.append(clip)
	return out


## Every clip an animation node can play, through blend trees, blend spaces and state machines.
static func clips(node: AnimationNode) -> PackedStringArray:
	var out := PackedStringArray()
	if node is AnimationNodeAnimation:
		out.append(String((node as AnimationNodeAnimation).animation))
	elif node is AnimationNodeBlendTree:
		var bt := node as AnimationNodeBlendTree
		for name in bt.get_node_list():
			out.append_array(clips(bt.get_node(name)))
	elif node is AnimationNodeBlendSpace1D:
		var bs := node as AnimationNodeBlendSpace1D
		for i in bs.get_blend_point_count():
			out.append_array(clips(bs.get_blend_point_node(i)))
	elif node is AnimationNodeBlendSpace2D:
		var b2 := node as AnimationNodeBlendSpace2D
		for i in b2.get_blend_point_count():
			out.append_array(clips(b2.get_blend_point_node(i)))
	elif node is AnimationNodeStateMachine:
		var sm := node as AnimationNodeStateMachine
		for name in sm.get_node_list():
			out.append_array(clips(sm.get_node(name)))
	return out
