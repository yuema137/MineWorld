# Nutrition engine source-map evidence — read-only searches
Base b457a94d9a1ace6a39005de683cc393886e19288; root working-copy source, not executable validation.

## Search
`rg -n PersistentWorld::create --glob *.rs --glob !target/**`
```text
server/src/runtime/tests.rs:196:            let (persisted, _) = PersistentWorld::create(
tests/acceptance/tests/configuration_seam.rs:169:    PersistentWorld::create(
persistence/tests/retention.rs:44:    let (persisted, _) = PersistentWorld::create(
tests/acceptance/tests/arrival_resolvers_resume.rs:124:    let (persisted, _) = PersistentWorld::create(
persistence/tests/save.rs:41:    let (persisted, _) = PersistentWorld::create(
persistence/tests/save.rs:410:    let (persisted, _) = PersistentWorld::create(
persistence/tests/save.rs:665:    PersistentWorld::create(
persistence/tests/save.rs:991:    let (mut persisted, _) = PersistentWorld::create(
persistence/tests/host_journal.rs:28:    let (persisted, _) = PersistentWorld::create(
persistence/tests/kill_and_resume.rs:459:        let (persisted, _) = PersistentWorld::create(
systems/happenings/tests/persisted.rs:39:    PersistentWorld::create(
systems/sleep/tests/integrity.rs:220:    let (persisted, _) = PersistentWorld::create(
systems/needs/tests/lazy.rs:204:    let (persisted, _) = PersistentWorld::create(
systems/group-activity/tests/persisted.rs:57:    let (mut persisted, _) = PersistentWorld::create(
systems/posture/tests/shipped.rs:191:    let (mut persisted, _) = PersistentWorld::create(
systems/posture/tests/persisted.rs:33:    let (mut persisted, _) = PersistentWorld::create(
systems/employment/tests/persisted.rs:56:    let (persisted, _) = PersistentWorld::create(
systems/calendar/tests/calendar.rs:270:    PersistentWorld::create(
systems/movement/tests/persisted.rs:57:    let (mut persisted, began) = PersistentWorld::create(
systems/weather/tests/record.rs:526:        PersistentWorld::create(
systems/inventory/tests/persisted.rs:45:    let (persisted, _) = PersistentWorld::create(
systems/weather/tests/weather.rs:379:    PersistentWorld::create(
systems/schedule/tests/persisted.rs:36:    let (mut persisted, _) = PersistentWorld::create(
tools/cli/src/run.rs:337:            let (world, began) = PersistentWorld::create(
tools/cli/src/serve.rs:518:    let (world, began) = PersistentWorld::create(
tools/cli/tests/walking.rs:864:        PersistentWorld::create(
systems/economy/tests/haggle_persisted.rs:48:    let (persisted, _) = PersistentWorld::create(
```

## Search
`rg -n SectionConfigured|Interactions<|SCHEMA_VERSION sdk/rust/src/interactions systems/needs`
```text
sdk/rust/src/interactions/mod.rs:36:    Consequence, Interactions, Ruling, SectionConfigured, consequence, declare, install,
sdk/rust/src/interactions/mod.rs:178:        Ok(vec![Emission::new::<SectionConfigured<S>>(
systems/needs/src/event.rs:30:    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
systems/needs/src/event.rs:60:    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
sdk/rust/src/interactions/lookup.rs:30:pub struct SectionConfigured<S> {
sdk/rust/src/interactions/lookup.rs:35:impl<S: InteractionSection> Event for SectionConfigured<S> {
sdk/rust/src/interactions/lookup.rs:38:    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
sdk/rust/src/interactions/lookup.rs:43:/// reduction of [`SectionConfigured`].
sdk/rust/src/interactions/lookup.rs:46:pub struct Interactions<S: InteractionSection> {
sdk/rust/src/interactions/lookup.rs:52:impl<S: InteractionSection> Component for Interactions<S> {
sdk/rust/src/interactions/lookup.rs:55:    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
sdk/rust/src/interactions/lookup.rs:58:impl<S: InteractionSection> OwnedBy<S> for Interactions<S> {}
sdk/rust/src/interactions/lookup.rs:60:impl<S: InteractionSection> Interactions<S> {
sdk/rust/src/interactions/lookup.rs:70:        .owning::<Interactions<S>>()
sdk/rust/src/interactions/lookup.rs:71:        .emitting::<SectionConfigured<S>>()
sdk/rust/src/interactions/lookup.rs:72:        .subscribing_to::<SectionConfigured<S>>()
sdk/rust/src/interactions/lookup.rs:81:    tables.component::<Interactions<S>>()
sdk/rust/src/interactions/lookup.rs:147:) -> Option<(&'w Interactions<S>, bool)> {
sdk/rust/src/interactions/lookup.rs:150:            .component::<Interactions<S>>(place.entity_id())
sdk/rust/src/interactions/lookup.rs:153:            .components::<Interactions<S>>()
systems/needs/tests/support/mod.rs:46:use mineworld_sdk::interactions::{self, InteractionSection, Section, SectionConfigured};
systems/needs/tests/support/mod.rs:58:    Emission::new::<SectionConfigured<NeedsSystem>>(
```

## Search
`rg -n needs|nourishment|hunger clients/agent/src cognition/lm-controller/src clients/2d/scripts/hud clients/3d-spike/scripts/slice clients/protocol -g *.py -g *.gd`
```text
clients/3d-spike/scripts/slice/offer_menu.gd:238:			shown += "  ·  " + MineWorldText.text("ui.needs-range",
clients/2d/scripts/hud/trade_panel.gd:116:## Collects what the button still needs — an item kind from the picker, a price from the typed line — then
clients/2d/scripts/hud/readers.gd:14:## The panels that show what I am told about my needs, my sleep and my day (S35 N-e): their rows come from
clients/2d/scripts/hud/readers.gd:16:const NEEDS_PANELS := ["needs", "asleep", "agenda"]
clients/2d/scripts/hud/readers.gd:184:## reference by its name, never JSON). `bars` (needs: one level per row, -1 for none) and `button`
clients/2d/scripts/hud/readers.gd:196:	if known and kind == "needs":
clients/2d/scripts/hud/readers.gd:197:		panel["bars"] = MineWorldNeeds.own_needs(observation).map(func(e: Dictionary) -> int: return e["level"])
clients/2d/scripts/hud/readers.gd:205:	if kind == "needs" and typeof(payload) == TYPE_DICTIONARY:
clients/2d/scripts/hud/readers.gd:206:		return _list(MineWorldNeeds.own_needs(observation), func(e: Dictionary) -> String:
clients/3d-spike/scripts/slice/slice_link.gd:59:## Every observation the link receives, before anything is drawn (S35 N-e: the needs rows show in any place).
clients/agent/src/mineworld_agent/__main__.py:162:            _stderr(f"{flag} needs --http")
clients/agent/src/mineworld_agent/__main__.py:215:        _stderr("--allow-host needs --allow-remote")
clients/protocol/mineworld/observation.gd:278:## carries no display text on purpose (`DD-13`), and needs a fallback for a code it does not know.
clients/protocol/mineworld/observation.gd:283:## What the owning system declared the action needs of space, unevaluated.
clients/protocol/mineworld/observation.gd:285:## For *showing* a requirement — "needs 3 m" — without checking it. The server has already checked
clients/protocol/mineworld/needs.gd:4:## What the packs disclose about my needs, my sleep, my next part of the day and my employer's
clients/protocol/mineworld/needs.gd:9:## named by the `words` map the `needs` record discloses, never by a name written here, so a third need
clients/protocol/mineworld/needs.gd:10:## needs no change in this file. Every function returns `{}` or `[]` when nothing is disclosed, and a
clients/protocol/mineworld/needs.gd:13:## The component types the packs disclose (`needs` and `asleep` are N-b's and N-a's; `agenda` is LE-a's).
clients/protocol/mineworld/needs.gd:14:const NEEDS := "needs"
clients/protocol/mineworld/needs.gd:23:## My needs in the `words` map's order: `[{need, word, level}]`. `level` is the record's number under the
clients/protocol/mineworld/needs.gd:25:static func own_needs(obs: MineWorldObservation) -> Array:
clients/3d-spike/scripts/slice/dressing.gd:6:## disagrees rather than to scale it in the node tree, and a prop that needs a
clients/3d-spike/scripts/slice/dressing.gd:142:## frame -- what a collider or a neighbouring prop needs to know.
cognition/lm-controller/src/mineworld_cognition/__main__.py:38:    """`scripted` is the backend a `recording.mode = "scripted"` configuration needs, supplied in code by a test's
clients/protocol/mineworld/world_client.gd:219:## controller drives needs no flag (`PROTOCOL.md` §4.2). Every four-argument call is unchanged.
clients/protocol/mineworld/world_client.gd:293:		push_warning("[mineworld] not seated; a request needs a seat first")
clients/3d-spike/scripts/slice/needs_hud.gd:70:			"needs":
clients/3d-spike/scripts/slice/needs_hud.gd:71:				for need in MineWorldNeeds.own_needs(obs):
clients/agent/src/mineworld_agent/view.py:148:    """The offer was shown incomplete: it carries no payload, so it needs its own operation (`say`,
clients/agent/src/mineworld_agent/view.py:353:        needs=disclosed.needs(mine),
clients/agent/src/mineworld_agent/view.py:654:        raise Incomplete(f"{wanted.action_type!r} needs its own operation")
clients/3d-spike/scripts/slice/slice_probe_world.gd:28:## What the probe types into the talk line when a check needs a line said (PR 16g1: the 3D client has no
clients/3d-spike/scripts/slice/slice_probe.gd:610:	# a new camera needs frames before its image is trusted: captured at once,
clients/3d-spike/scripts/slice/slice_probe.gd:650:## needs a number, and a number taken without warm-up or repeats is noise.
clients/protocol/checks/needs_check.gd:3:# The reader check for `MineWorldNeeds` (`clients/protocol/mineworld/needs.gd`): the shapes the packs
clients/protocol/checks/needs_check.gd:5:# observations of tests/fixtures/observations/needs/ are checked by the claims added at C2.
clients/protocol/checks/needs_check.gd:7:#   godot --headless --path clients/protocol --script res://checks/needs_check.gd
clients/protocol/checks/needs_check.gd:10:# (`step-15-demo-3d.md` §18.4 A-1). Prints one `[needs] PASS|FAIL <claim>: <observed>` line per claim
clients/protocol/checks/needs_check.gd:14:const Needs := preload("res://mineworld/needs.gd")
clients/protocol/checks/needs_check.gd:25:	_own_needs()
clients/protocol/checks/needs_check.gd:31:	print("[needs] %s — %d failure(s)" % ["PASS" if _failures == 0 else "FAIL", _failures])
clients/protocol/checks/needs_check.gd:39:	print("[needs] %s %s: %s%s" % ["PASS" if same else "FAIL", claim, JSON.stringify(observed),
clients/protocol/checks/needs_check.gd:55:func _own_needs() -> void:
clients/protocol/checks/needs_check.gd:56:	var ana := _observation([{"component_type": "needs", "payload": {"hunger": 640, "tiredness": 120,
clients/protocol/checks/needs_check.gd:57:		"words": {"hunger": "hungry", "tiredness": "ok"}, "asleep": false}}])
clients/protocol/checks/needs_check.gd:58:	_claim("own needs in the words map's order", Needs.own_needs(ana), [
clients/protocol/checks/needs_check.gd:59:		{"need": "hunger", "word": "hungry", "level": 640},
clients/protocol/checks/needs_check.gd:61:	var third := _observation([{"component_type": "needs", "payload": {"hunger": "x", "tiredness": 300,
clients/protocol/checks/needs_check.gd:62:		"words": {"hunger": "peckish", "tiredness": "sleepy", "thirst": "peckish", "odd": 5}}}])
clients/protocol/checks/needs_check.gd:64:		Needs.own_needs(third), [
clients/protocol/checks/needs_check.gd:65:		{"need": "hunger", "word": "peckish", "level": -1},
clients/protocol/checks/needs_check.gd:68:	_claim("no needs record: none", Needs.own_needs(_observation([])), [])
clients/protocol/checks/needs_check.gd:115:## The recorded observations (tests/fixtures/observations/needs/, `SOURCE.md`): each expected value below
clients/protocol/checks/needs_check.gd:118:	var directory := ProjectSettings.globalize_path("res://").path_join("../../tests/fixtures/observations/needs")
clients/protocol/checks/needs_check.gd:120:	_claim("market town at night: my needs, in the pack's words", Needs.own_needs(night), [
clients/protocol/checks/needs_check.gd:121:		{"need": "hunger", "word": "peckish", "level": 593},
clients/protocol/checks/needs_check.gd:129:	_claim("commute late: my needs, famished and exhausted", Needs.own_needs(late), [
clients/protocol/checks/needs_check.gd:130:		{"need": "hunger", "word": "famished", "level": 1000},
clients/protocol/checks/needs_check.gd:138:	_claim("commute sleeper: Ben's needs, fine and tired", Needs.own_needs(sleeper), [
clients/protocol/checks/needs_check.gd:139:		{"need": "hunger", "word": "ok", "level": 329},
clients/protocol/checks/needs_check.gd:150:		print("[needs] FAIL %s: the fixture does not parse" % name)
clients/protocol/checks/needs_check.gd:157:	_claim("a frame with no pack: needs", Needs.own_needs(bare), [])
clients/agent/src/mineworld_agent/disclosed.py:28:| `needs`          | needs          | the holder           | `MeLine.needs`               |
clients/agent/src/mineworld_agent/disclosed.py:585:def needs(me: PerceivedEntity | None) -> NeedsLine | None:
clients/agent/src/mineworld_agent/disclosed.py:586:    """My needs, as the `needs` record names them: each key of its `words` map, with the pack's word and the
clients/agent/src/mineworld_agent/disclosed.py:588:    found = _obj(payload(me, "needs"))
clients/agent/src/mineworld_agent/disclosed.py:601:    return NeedsLine(needs=lines)
clients/agent/src/mineworld_agent/models.py:71:"""The deficit scale of a need's level (`MODULE_SPEC.md` §4.1): a level is `0 … 1000`, as the needs pack writes it."""
clients/agent/src/mineworld_agent/models.py:479:    """One of my needs as `needs` discloses it: its name, the pack's word for it now, and the level."""
clients/agent/src/mineworld_agent/models.py:482:    """The need's name, as the `needs` record's `words` map names it (`hunger`, `tiredness`, …)."""
clients/agent/src/mineworld_agent/models.py:490:    """needs' `Needs`, disclosed to me only: how I feel, in the pack's own words."""
clients/agent/src/mineworld_agent/models.py:492:    needs: list[NeedLine]
clients/agent/src/mineworld_agent/models.py:573:    needs: NeedsLine | None = None
clients/agent/src/mineworld_agent/models.py:574:    """How I feel: my needs and the pack's word for each, when needs discloses them (holder only)."""
clients/agent/src/mineworld_agent/models.py:658:    """Whether `attempt` can send it as it is (`ARC-34`); an incomplete one needs its own operation."""
cognition/lm-controller/src/mineworld_cognition/record.py:50:    """Where an entry came from, as far as a reviewer needs to know: never how to reach it."""
clients/3d-spike/scripts/slice/slice_main.gd:567:##          and it is the doorway that needs it.
clients/3d-spike/scripts/slice/slice_main.gd:603:## VoxelGI each. Bounded on purpose: the volume that needs real bounce is a room, and a VoxelGI large
clients/3d-spike/scripts/slice/slice_main.gd:736:		_needs = SliceNeedsHud.new()
clients/3d-spike/scripts/slice/slice_main.gd:737:		_needs.name = "NeedsHud"
clients/3d-spike/scripts/slice/slice_main.gd:738:		add_child(_needs)
clients/3d-spike/scripts/slice/slice_main.gd:739:		_needs.attach(hud, _env)
clients/3d-spike/scripts/slice/slice_main.gd:744:var _needs: SliceNeedsHud = null
clients/3d-spike/scripts/slice/slice_main.gd:778:	if _needs != null:
clients/3d-spike/scripts/slice/slice_main.gd:779:		_needs.bind_link(link)
cognition/lm-controller/src/mineworld_cognition/config.py:143:            "when the server needs no key. A preset fills it when this table leaves it out. The key "
cognition/lm-controller/src/mineworld_cognition/config.py:147:    """The NAME of the environment variable holding the key, or empty when the server needs none."""
cognition/lm-controller/src/mineworld_cognition/config.py:607:            raise ValueError(f"recording.cassette: mode {mode!r} needs a cassette")
cognition/lm-controller/src/mineworld_cognition/config.py:704:            raise ConfigError("recording.mode: scripted mode needs a backend supplied in code")
clients/3d-spike/scripts/slice/needs_rows_probe.gd:9:## (`clients/2d/scripts/harness/needs_rows.gd`) prints the same lines; the parity test compares them.
clients/3d-spike/scripts/slice/needs_rows_probe.gd:11:##   godot --headless --path clients/3d-spike --script res://scripts/slice/needs_rows_probe.gd -- --fixtures=<dir>
clients/3d-spike/scripts/slice/needs_rows_probe.gd:13:const NeedsHud := preload("res://scripts/slice/needs_hud.gd")
cognition/lm-controller/src/mineworld_cognition/controller/prompt.py:223:            *_needs(situation),
cognition/lm-controller/src/mineworld_cognition/controller/prompt.py:269:def _needs(situation: Situation) -> list[str]:
clients/3d-spike/scripts/slice/slice_probe_market.gd:61:## How many straight legs a `walk_near` takes at most: a turn round a table needs two or three.
clients/3d-spike/scripts/slice/slice_probe_market.gd:532:		return "the gate step needs an open menu"
clients/protocol/demo/demo.gd:546:			line += "  (needs %.1f m)" % (float(reach) / 1000.0)
cognition/lm-controller/src/mineworld_cognition/controller/validate.py:158:            reason="bad_amount", detail="sleep needs a number of minutes, 1 or more"
cognition/lm-controller/src/mineworld_cognition/controller/fill/base.py:5:seat has what the content is made from; it shows the model what it needs in labels only; it asks for typed fields
cognition/lm-controller/src/mineworld_cognition/controller/triggers.py:134:    """What `needs` and `asleep` disclose to me now, as the situation holds it."""
cognition/lm-controller/src/mineworld_cognition/controller/seat.py:439:        """The `feeling` trigger (S35 N-e, §4.7): when my needs' words change, or on waking. Its event is the newest
cognition/lm-controller/src/mineworld_cognition/memory/scopes/interface.py:9:scope author needs them and the person scope's retrieval (`memory.retrieve`) re-exports them.
cognition/lm-controller/src/mineworld_cognition/memory/scopes/interface.py:141:    """Per-seat tables only: an event-keyed table is not hosted until a scope needs one (F-S28d-4)."""
cognition/lm-controller/src/mineworld_cognition/controller/situation.py:36:NEEDS_COMPONENT = "needs"
cognition/lm-controller/src/mineworld_cognition/controller/situation.py:56:    """One need of mine and the pack's word for it now (`needs`' `words` map, in the record's order)."""
cognition/lm-controller/src/mineworld_cognition/controller/situation.py:102:    """My needs and the pack's words for them, as `needs` discloses them to me (S35 N-e); empty when none."""
cognition/lm-controller/src/mineworld_cognition/controller/situation.py:245:    """My needs, in the order the `needs` record's `words` map gives them. Nothing is named here: the pack's
cognition/lm-controller/src/mineworld_cognition/controller/situation.py:247:    needs = _component(observation, NEEDS_COMPONENT)
cognition/lm-controller/src/mineworld_cognition/controller/situation.py:248:    words = needs.get("words") if isinstance(needs, dict) else None
cognition/lm-controller/src/mineworld_cognition/backend/strict_schema.py:3:A strict mode refuses the numeric and length bounds and needs every object closed. `lower_schema`
cognition/lm-controller/src/mineworld_cognition/backend/cli_bridge.py:20:Starting a subprocess needs an event loop that supports it: on Windows the default proactor loop, not
```

## Search
`rg -n pub fn create|fn persisted|fn new|SqliteBackend::create persistence/src/world.rs tools/cli/src/serve.rs tools/cli/src/run.rs`
```text
tools/cli/src/serve.rs:491:fn persisted(
tools/cli/src/serve.rs:516:    let backend = SqliteBackend::create(save, Durability::PowerLoss).map_err(HostError::build)?;
persistence/src/world.rs:128:    pub fn create(
tools/cli/src/run.rs:317:    fn new(pack: &WorldPack, save: Option<&Path>) -> Result<Self, String> {
tools/cli/src/run.rs:335:            let backend = SqliteBackend::create(save, Durability::ProcessCrash).map_err(stopped)?;
tools/cli/src/run.rs:504:    const fn new() -> Self {
tools/cli/src/run.rs:549:    fn new(began: &[EventEnvelope]) -> Self {
```

## Search
`rg -n ConfigurationRefusal::|pub fn resolve|fn check|fn seed sdk/rust/src/interactions/mod.rs sdk/rust/src/interactions/resolve.rs worldpack/src/configure.rs`
```text
sdk/rust/src/interactions/mod.rs:164:    fn check(&self, context: &ConfigurationContext<'_>) -> Result<(), ConfigurationRefusal> {
sdk/rust/src/interactions/mod.rs:168:    fn seed(
sdk/rust/src/interactions/resolve.rs:237:/// [`ConfigurationRefusal::ClassUndefined`] for a selector naming a class that is neither defined nor
sdk/rust/src/interactions/resolve.rs:238:/// implicit; [`ConfigurationRefusal::Ambiguous`] for two entries of equal specificity that overlap and
sdk/rust/src/interactions/resolve.rs:240:pub fn resolve<S: InteractionSection>(
sdk/rust/src/interactions/resolve.rs:253:                return Err(ConfigurationRefusal::ClassUndefined {
sdk/rust/src/interactions/resolve.rs:449:    ConfigurationRefusal::Ambiguous {
worldpack/src/configure.rs:270:fn check(classes: &EntityClasses, configured: &FoundConfiguration) -> Result<(), PackError> {
worldpack/src/configure.rs:277:            ConfigurationRefusal::ClassUndefined { class, entry } => PackError::ClassUndefined {
worldpack/src/configure.rs:283:            ConfigurationRefusal::Ambiguous {
worldpack/src/configure.rs:325:fn check_requires(found: &[FoundConfiguration], systems: &[Capability]) -> Result<(), PackError> {
worldpack/src/configure.rs:342:fn check_default_requires(
worldpack/src/configure.rs:395:fn check_nothing_undeclared(root: &Path, keys: &[ConfigurationKey]) -> Result<(), PackError> {
worldpack/src/configure.rs:429:pub(crate) fn check_references(pack: &WorldPack) -> Result<(), PackError> {
worldpack/src/configure.rs:449:pub(crate) fn seed(
worldpack/src/configure.rs:565:    pub fn check_configuration(&self, saved_genesis: &[EventEnvelope]) -> Result<(), PackError> {
```

## Search
`rg -n schema_version|component_type|SCHEMA_VERSION sdk/python/src/mineworld_sdk/wire/contract.py systems/needs/src clients/protocol/mineworld/observation.gd`
```text
sdk/python/src/mineworld_sdk/wire/contract.py:190:    component_type: ComponentTypeIdField
sdk/python/src/mineworld_sdk/wire/contract.py:191:    schema_version: U32
sdk/python/src/mineworld_sdk/wire/contract.py:299:    schema_version: U32
clients/protocol/mineworld/observation.gd:127:func component(id: String, component_type: String) -> Dictionary:
clients/protocol/mineworld/observation.gd:128:	var payload: Variant = component_value(id, component_type)
clients/protocol/mineworld/observation.gd:137:func component_value(id: String, component_type: String) -> Variant:
clients/protocol/mineworld/observation.gd:143:				and String(record.get("component_type", "")) == component_type:
clients/protocol/mineworld/observation.gd:152:func own_component(component_type: String) -> Dictionary:
clients/protocol/mineworld/observation.gd:153:	return component(observer(), component_type)
systems/needs/src/component.rs:24:    component_type = "needs",
systems/needs/src/component.rs:25:    schema_version = 1,
systems/needs/src/component.rs:92:    component_type = "nourishment",
systems/needs/src/component.rs:93:    schema_version = 1,
systems/needs/src/event.rs:30:    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
systems/needs/src/event.rs:60:    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
```
