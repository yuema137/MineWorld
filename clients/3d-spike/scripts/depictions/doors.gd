## The scene's one door registry (step-15 D-16c-4): every drawn door, whoever drew it -- a depiction
## file's place art (`DepictionModels`), a stub's box (`SliceBoxDepiction`), or an authoring source at
## export time (`tools/presentation/`). The layout binds each doorway the world discloses to the door it
## lands on; the probes and `--doors` read it too.
##
## A row: [sill centre (global), the façade's yaw, a label, the place volume of the room behind it (""
## for a closed door), the depth from the sill to that room's inner face, the room floor's height above
## the sill]. A closed door's depth is the façade skin's: nothing is drawn behind it.
class_name SliceDoors
extends RefCounted

static var rows: Array = []
