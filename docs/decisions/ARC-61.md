## ARC-61 — A System Pack may be configured per world, by a file its owner types

**Date** 2026-10-08 · **Approved by** the operator (`overall.md` "The World Interaction List": the
carrier stays `configure:`; QPL-2, QPL-12) and the primary session at PR IL-a's design freeze
(step-18-interaction-list §11; QIA-1 … QIA-6 accepted as recommended) · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1, §9 (configuration schema) · **Relates to** `ARC-15`,
`ARC-25`, `ARC-26`, `ARC-31`, `ARC-33`, `ARC-62`, `DEP-10`, `INV-13` · **Design**
`.structured-coding/plans/mvp0/step-18-interaction-list.md` §4.3, §4.10, §11 (S17, PR IL-a)

**Problem.** A System Pack's numbers and policies (a range, a capacity, a step) are compiled
constants. A world that wants another value has nowhere to say so: `world.yaml` names packs but carries
nothing for them, and a section (`ARC-31`) belongs to one person, place, item or organization, not to
the world. The S17 Interaction List (`ARC-63` onward) needs each pack to accept a world-level document
it types and enforces itself. Three constraints bound the answer: the loader must not learn what a
configuration means (`ARC-31`'s rule, one level up); a world that configures nothing must load exactly
as before, fact for fact; and a save must never resume against a configuration other than the one it
was created with, because resume does not re-read content (`ARC-25`).

**Choice.**

1. **The contract is `mineworld_authoring::PackConfiguration`**, beside `AuthoredSection`:

   ```text
   trait PackConfiguration: SystemIdentity
     type Configuration: DeserializeOwned + Debug + Send + Sync + 'static   deserializing is validating
     const FACTS: &'static [EventTypeId]       the event types its configuration may seed
     fn references(&Configuration) -> Vec<Reference>    entity keys it names (default none)
     fn requires(&Configuration) -> Vec<SystemId>        systems that must also be enabled (default none)
     fn seed(&Seeding, &Configuration) -> Result<Vec<Emission>, Rejection>
   ```

   A configuration has no subject, so `seed` takes none. The loader holds a decoded configuration type
   erased, as `AuthoredConfiguration` (owner, references, requires, seed), produced by the
   `DecodeConfiguration<T>` `DeserializeSeed` — exactly as `AuthoredContent` and `Decode` hold a
   section.
2. **A pack says it is configurable in its `impl SystemPack`**, with `mineworld_sdk::configures!();`,
   which defines `CONFIGURATION` (`Some(<its own id>)`), `CONFIGURATION_FACTS` (its `FACTS`) and
   `decode_configuration` together from the `PackConfiguration` impl. The defaults are the safe
   direction: `None`, no facts, and a decode that refuses "the '<id>' system takes no configuration".
   The installed set's `Capability` aggregates them: `configuration`, `decode_configuration`,
   `configuration_facts`.
3. **The carrier is `world.yaml` `configure:`**, an optional list of keys. Each key is an enabled
   System Pack's id or a reserved key (item 4), and names `configure/<key>.yaml`, decoded straight from the YAML stream into the owner's type, so a
   refusal keeps its line and column (`DEP-10`). The list's order is the author's and is the seeding
   order. Absent means empty.
4. **Reserved keys.** `classes` (the Interaction List's entity classes, `ARC-64`, IL-b) and `packages`
   (the licence-policy override hook of S16 E-b, FQ-b2 — owned by the framework crate
   `mineworld-packages`, not by a System Pack) are reserved: listing either is refused, "reserved for
   <what>; not configurable in this build". Both are wired in IL-b, which lets `packages` through to its
   framework owner rather than resolving it against the installed set. A test holds that no installed
   pack's id is a reserved key.
5. **Refused by name**, each naming the key and the file, with line and column where a YAML value is
   involved: a key that is no system of this build; a system the world does not enable; a system that
   takes no configuration; a reserved key; a key listed twice; a listed file missing; a `.yaml` file in
   `configure/` that is not listed; a `requires` system not enabled; a reference to an undeclared
   entity or one of another type; a configuration the owner's `seed` refuses; a seeded fact in another
   pack's vocabulary or of an event type outside the owner's `FACTS`; and configuration drift (item 7).
   Malformed YAML is the loader's existing `Malformed`.
6. **Seeding order.** Genesis states passages, then locations, then **configuration, in `configure:`
   order**, then sections. A section's reduction may therefore check its value against the configured
   state (the step a value must be a multiple of, the range it must lie within); it is reduced after it.
   A world without `configure:` takes no new branch and seeds exactly what it seeded before this
   decision: its ids, facts and digests do not move.
7. **Drift is refused at resume** (QPL-12). Every host that resumes or verifies a save from a World Pack
   — `mineworld run` resuming, `mineworld server --save`, `mineworld replay` — first calls
   `WorldPack::check_configuration(saved genesis facts)`. It assembles the world, seeds the configuration
   as genesis would, and compares, in order, the event type, the record and the visibility of every
   genesis fact whose type is in the union of the enabled packs' `configuration_facts`, on both sides.
   The first difference is `PackError::ConfigurationDrift { system, saved, here }`. Configuration added
   and configuration removed are both drift. Nothing in `persistence/` or the kernel changes: the
   comparator reads the save's genesis facts through the existing backend.
8. **The configuration facts' audience is guidance, not a loader rule** (QIA-4). An owner states its
   configuration facts `Visibility::SystemInternal` with no subjects: a world's configuration is
   nobody's perception and nobody's biography (`INV-13`). The loader does not enforce it, because a
   loader rule over visibility would be the loader judging a pack's vocabulary.

**Not in this decision.** What any pack's configuration says: the Interaction List's schema, classes
and sections (`ARC-63` … `ARC-65`, IL-b onward). Wiring `configure/packages.yaml` into the licence
policy (IL-b). Drift in content other than configuration — a renamed person, a moved table — which is
not checked (QPL-12's scope).

**Accepted limitations.**
- Configuration is not a rule. `MODULE_SPEC.md` §4 constraint 3 holds: a configuration parameterizes and
  restricts what its owning pack implements; it never adds behaviour no installed pack has.
- Resume still does not re-read content, so content drift outside configuration is still accepted
  silently, as before this decision.
- A configured pack's declaration does not change, so configuring a pack does not change its
  `SystemVersion`; a pack whose configuration *schema* changes raises it, as for any owned type.

**Note, 2026-10-08 (S17, PR IL-b; QIA-1, QTW-7, SD-IB-1, SD-IB-3, SD-IB-5) — framework keys, the
seeding context and `data:` attachments.**
- **Item 4's reserved keys become framework keys.** `classes` is decoded by
  `mineworld_authoring::EntityClasses` (`ARC-64`) and `packages` by `mineworld_packages::LicencePolicy`
  (`ARC-55` note). A framework key is never resolved against the installed set and never seeded on its
  own; listing it is optional, an unlisted file in `configure/` is still refused, and a test holds that
  no installed pack's id is a framework key.
- **The seeding context.** `PackConfiguration::seed(&Seeding, &Configuration, &ConfigurationContext)`:
  the context hands a configuration the world's entity classes and the bytes of its own attachments,
  read-only. `Seeding`, which sections share, is unchanged. A configuration may also be checked against
  the context before anything is seeded (`PackConfiguration::check`, default accept); a refusal there is
  typed (`ConfigurationRefusal`: an undefined class, two ambiguous entries) and named by the loader with
  the file, the list and the index. No shim keeps the old signature.
- **`data:` attachments.** A configuration may name files under the World Pack's `data/` directory with
  `authoring::Attachment`: a relative path written with `/`, first component `data`, every component a
  plain name (no `..`, no root, no drive, no `\`), checked as it decodes, at its line and column — the
  same on every platform. `PackConfiguration::attachments` lists them (default none). The loader reads
  each one and refuses one that is missing, one that resolves outside the pack after its links are
  followed, and one over 4 MiB, each by name. `seed` receives the bytes; the owner decodes them and
  states what it needs in its own fact, so a changed file is drift with no new mechanism. Files under
  `data/` that nothing names are allowed.

**Note, 2026-10-19 (PR N-b; SD-Nb-5 item 2) — an unconfigured pack's defaults are checked too.** `requires`
used to run only for a written `configure/<pack>.yaml`, so a world that enabled a pack and wrote no file was
never checked, and could keep a default that can never take effect (a tiredness that silently never falls).
The loader now asks, for every enabled pack with an Interaction List section that `configure:` does not list,
what its empty section requires (`SectionDecl::requires_default`, filled by `interactions!()`), and refuses a
missing system naming `world.yaml`, the pack and the system (`PackError::DefaultConfigurationRequiresSystem`).
A pack whose file is written is checked by that file alone. Every pack before `needs` requires nothing by
default, so no world changed.
