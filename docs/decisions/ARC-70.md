## ARC-70 — Client UI text is addressed by message keys; a Presentation Pack carries the wording as gettext files

**Date** 2026-10-08 · **Approved by** the primary session at S12 PR 13b's design freeze (QS13b-1) ·
**Relates to** `ARC-46`, `ARC-47`, `DD-13`, `DEP-16`, [`REUSE_POLICY.md`](REUSE_POLICY.md) · **Design**
step-13 §15.4 D-b-9; [`clients/2d/PRESENTATION.md`](../clients/2d/PRESENTATION.md) §7

**Problem.** The contract carries no display text (`DD-13`): an action type, a rejection code and a
component are identifiers, and a client words them. The 2D client's menus, results and panels add
dozens of user-visible strings. The operator requires every client UI string to be translatable,
with translations as standard files in the Presentation layer, so that a user adds a language
without code ("Framework, not demo" item 4, `en` default, `zh-Hans` next).

**Decided.**

1. Every user-visible string of a reference client is a **message key** looked up through the
   engine's translation server, with arguments substituted after the lookup. Keys are built from data
   where the thing worded is data: `action.<type>`, `action.<type>.done`, `reason.<code>` — so no
   client file holds an action type or a reason code to word it. Fixed UI text uses `ui.*`, panel
   titles `panel.*`, suggestion lists `suggest.*`, formats `format.*`.
2. A Presentation Pack carries the wording as **gettext `.po` files**, `i18n/<locale>.po`, loaded at
   runtime from the pack directory (never imported into the client project). `en` is the default and
   the one 13b ships.
3. A key the wording does not hold falls back to a readable form of the key's last part
   (`reason.too_far_away` → "too far away"); a client with no pack still says something true. A code
   the wording does not know is therefore shown as the code itself.
4. Wording holds no rule: no number, no condition, no filter. A suggestion list is a suggestion; what
   the player types is sent unchanged and the server decides whether it is valid.
5. The language switch, `zh-Hans` and the setting's persistence belong to the client-settings lane,
   which may change the file format behind the same keys.

**Options compared** (`REUSE_POLICY.md`, adopt → adapt → extend → build):

```text
Godot TranslationServer + gettext .po, loaded at runtime   adopt: the engine's own; .po is the standard
                                                           translators' tools edit; Godot 4.7 loads a .po
                                                           from an absolute path unimported (probed)
Godot CSV translations                                     reject for packs: need the editor's import
                                                           step, so a pack outside the project cannot be
                                                           loaded at runtime
our own JSON wording table                                 reject: non-standard; the operator asked for
                                                           standard files
third-party i18n addons                                    none needed over the engine's own
```

**Isolating interface.** One script per client reads the wording (`clients/2d/scripts/hud/words.gd`);
every other script asks it for a key's text.

**Note 2026-10-09 — extended to both clients by S20 SET-a** (design step-20 §3.6, ruling QSET-11,
QSET-16; this note stands in for the planned `ARC-SET-b`). The format and every key of point 1 are
unchanged. Catalogs are **layered per key**: the shared settings module's `locale/` (keys both clients
use, among them `action.*` and `reason.*`, moved verbatim from the 2D pack), then the Presentation Pack's
`i18n/` (the pack's own wording, and any override of a shared key it marks `#. override`), then the
user's `user://locale/`, the later layer winning. Languages are discovered from the `.po` files present
(a language is offered when some layer gives it a `language.self_name`), so a `fr.po` dropped into any
layer adds French with no code. The language is switched live. The fallback is English, then point 3's
readable form; a raw key is never shown. World content, stdout logs and evidence lines are never
translated. `words.gd` keeps its API and delegates loading and the fallback to
`clients/shared/settings/text.gd`. The translation comparison above is confirmed for both clients by
step-20 §4.1.
