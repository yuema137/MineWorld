extends RefCounted

## Who this client draws seated, from each observation (S33-f; `ARC-145`): the shared decision,
## `MineWorldSitRule` (`clients/shared/sitting/`), and nothing of its own. Drawing a seated person in
## 2D is S34-e's; this keeps the decision so both clients agree on it (AC-f-4).
##
## The world's sit-point depiction file is the 3D Presentation Pack's (`sit_points/<world>.yaml`): only
## its optional fallback positions matter here, for the shared rule's fallback branch in a world that
## does not disclose its sit points. No shipped depiction file carries one since S34-c2 (`ARC-172`): the
## shipped worlds with sit points disclose them, so this client follows the posture branch there.

## The pack whose `sit_points/` the rule reads, relative to the repository.
const SIT_PACK := "presentation/mineworld-default/3D"

var rule: MineWorldSitRule
## id -> the point key the person is drawn seated at, from the last observation.
var seated := {}


func _init(repo_root: String, world_key: String) -> void:
	rule = MineWorldSitRule.load_for(repo_root.path_join(SIT_PACK), world_key)


## The decision for one observation.
func reconcile(observation) -> Dictionary:
	seated = rule.decide_observed(observation)
	return seated
