## The 3D client's talk line (PR 16g1): E facing a person, or "talk" in their menu, opens the shared
## typed line (`MineWorldLineInput`, `res://mineworld_hud`) with "Say to {name}:"; Enter sends exactly
## what was typed to that person, Esc sends nothing.
##
## The person is pinned when the line opens (D-3DS-3): the camera may turn while the player types and
## the line still goes to whom they asked. Nothing typed is checked and nothing about the person is
## re-evaluated when it is sent — whether they are near enough, and whether the words are acceptable,
## is the server's answer (`too_far_away`, a refused utterance), shown like every other answer.
##
## Opening and closing are signalled so `SliceMain` holds gameplay through its one input gate while the
## player types (D-3DS-5).
class_name SliceTalkLine
extends Node

signal opened
signal closed

var link: SliceLink
var line: MineWorldLineInput
## Whom the open line will be sent to; "" when it is closed.
var target := ""
## The token of the last line sent, for the probes ("" when nothing was sent yet).
var last_token := ""


func _init(a_link: SliceLink, theme: Theme) -> void:
	link = a_link
	line = MineWorldLineInput.new(theme)
	line.name = "TalkLineInput"


func _ready() -> void:
	add_child(line)
	line.opened.connect(func() -> void: opened.emit())
	line.closed.connect(func() -> void:
		target = ""
		closed.emit())
	link.wants_words.connect(open_for)


## Opens the line for `who`, pinned.
func open_for(who: String) -> void:
	target = who
	line.ask(MineWorldText.text("ui.talk.prompt", {"target": link.shown_label(who)}), PackedStringArray(),
		_send.bind(who))


func is_open() -> bool:
	return line.is_open()


func cancel() -> void:
	line.cancel()


func _send(said: String, who: String) -> void:
	last_token = link.talk_to(who, said)
