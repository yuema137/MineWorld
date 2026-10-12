**Status:** informational reference audit (REUSE_POLICY §19), 2026-10-11. **Not authoritative architecture**: it changes no MineWorld contract; where it disagrees with a specification, the specification governs. Gathered by a research agent from primary web sources through a summarising fetch tool (no clone); items marked *unverified* must be re-read at source before any design cites them. Summary and rulings: [`PRIOR_ART_2026-10.md`](PRIOR_ART_2026-10.md).

# Prior art: AI NPC and agent runtimes for MineWorld Milestone D

Status: research findings, read-only audit. Access date for all sources: 2026-10-11 unless stated.
Scope: Inworld (Runtime / Character Engine), Convai, Google DeepMind Concordia, a16z AI Town, Stanford Generative Agents (Smallville).
Verification convention: **Verified** = read on the primary page during this audit. **Unverified** = search snippet, third-party aggregator, or not found on a primary source. Items marked Unverified must be checked before any decision depends on them.

---

## 1. Summary

- None of the five projects is a framework MineWorld can adopt as a core dependency. Each assumes it owns the game state, or is a hosted product with no offline mode. That conflicts with `docs/VISION.md` (a world with the models removed must stay valid) and with `ARCHITECTURE.md` (the authoritative server owns state; controllers only submit `ActionIntent`s).
- **Concordia** (Apache-2.0, Python, Google DeepMind) is the closest in spirit: a Game Master turns natural-language intents into outcomes, and components are modular. It is a research library, says it is "not an officially supported Google product," needs a text-sampling LLM API plus a text embedder, and has no stated determinism or recording facility. Its best use for us is as a design reference and as a possible optional controller adapter, not as a runtime.
- **AI Town** (MIT, TypeScript, Convex) is a deployable starter kit, not a library. Its agent loop is plain code with a `Math.random()` invite gate, and LLM calls are used for conversation text. It defaults to Ollama (llama3 + mxbai-embed-large) and can run locally. Its game state lives in Convex, so it is a full world, not a controller.
- **Stanford Generative Agents** (Apache-2.0 per one fetch, Python, Django frontend) is the origin of the memory-stream / reflection / planning design. The released code requires an OpenAI API key, has no local option, and does not record or replay model calls.
- **Inworld** and **Convai** are hosted, proprietary character APIs with usage-based or subscription pricing and cloud data processing. Inworld's Runtime and Character Engine could not be verified on any fetched page (see §2.1). Convai's SDKs are Unity/Unreal/Web plugins; its pricing page did not render. Neither can run offline, and neither can be recorded and replayed deterministically from our side. Both are disqualified as core dependencies.
- Our merged P4 memory (per-seat SQLite, bounded ≤6 KB context, Event-ID citations, `ARC-59`) and P5 backends (Ollama + 11 OpenAI-compatible presets + native Anthropic, budgets `ARC-57`, cassette replay `ARC-58`) already cover what these products offer at the layer we need. Reinvention risk is in the **agent-behaviour layer** (planning, reflection, NPC-to-NPC talk), not in memory or backends.
- Recommendation: do not adopt any of these for P6 or P7. Optionally, consider a Concordia-style or AI Town-style controller adapter later, behind `Controller`, outside the core, with a cassette-backed test. Details in §5 and §7.

---

## 2. Facts per project

### 2.1 Inworld AI (Runtime / Character Engine)

| Item | Finding | Status | Source (access 2026-10-11) |
|---|---|---|---|
| Licence | Not found for Runtime or Character Engine. The public GitHub org lists MIT and Apache-2.0 repos (e.g. `inworld-api-examples` MIT, `tts` Python MIT, `livekit_agents` Apache-2.0, `pipecat` BSD-2-Clause), but none named Runtime, Character, Engine, SDK or Studio on the visible page (10 of 29 repos shown). | Unverified for the product itself | https://github.com/inworld-ai |
| Pricing | Public pricing page lists On-Demand ("Start free", evaluation and prototyping), Creator $25/mo, Builder $100/mo, Developer $300/mo, Growth $1,500/mo, Enterprise custom. TTS-2 from $5 to $25 per 1M characters depending on tier. STT $0.15/hr (On-Demand). LLMs "At cost." The page does not mention Runtime or Character Engine. | Verified for the pricing page; Runtime/Character Engine pricing Unverified | https://inworld.ai/pricing |
| Products on the docs landing | Text-to-Speech, LLM Router, Realtime API, Speech-to-Text, plus CLI and MCP server. No Runtime, Character Engine or Character Studio entries. | Verified | https://docs.inworld.ai/ |
| Character Studio deprecation | **Not verified.** The docs landing has no deprecation text. A search for "Character Studio deprecated sunset" returned only unrelated products. The deprecation the operator mentioned has no primary source in this audit. | Unverified (operator-reported) | — |
| Memory, goals, actions, NPC-NPC talk | Not described on any fetched primary page. Voice is covered: the STT model "emits a voice profile — emotion, vocal style, accent, age, pitch"; TTS supports cloning and 200+ languages. | Memory/goals/actions Unverified; voice Verified | https://docs.inworld.ai/tts/tts.md ; https://docs.inworld.ai/realtime/overview.md |
| SDK languages | Not listed on the fetched pages. The only install command shown is the CLI: `npm install -g @inworld/cli`. The public repos show JavaScript, TypeScript and Python examples. | Partly verified | https://docs.inworld.ai/ ; https://github.com/inworld-ai |
| Self-hosting / on-prem / offline | Not mentioned on the pricing or docs pages. A third-party comparison page (fish.audio) mentions an on-premise option priced by contact. | Unverified | https://fish.audio/vs/pricing/inworld-ai/ (aggregator) |
| Data / privacy | No Inworld privacy statement was fetched. | Unverified | — |
| Determinism / record-replay | Not documented on fetched pages. Realtime API is "drop-in compatible with the OpenAI Realtime API" and is billed on model usage. | Unverified | https://docs.inworld.ai/realtime/overview.md |

### 2.2 Convai

| Item | Finding | Status | Source |
|---|---|---|---|
| Licence | Proprietary platform. No open-source licence for the platform was found. | Verified as proprietary by absence of an open-source repo; not checked on a licence page | https://convai.com/ |
| Pricing | Pricing page did not render (page title only). Aggregator ToolRadar lists Free, Basic $5/mo, Pro $15/mo, with possible overage fees. Forum post (2024) says standalone TTS API is for Enterprise/Partnership; a Free-plan user got a "FreePlan" error. Both are old or third-party. | Unverified | https://docs.convai.com/ ; https://toolradar.com/tools/convai/pricing ; https://forum.convai.com/t/which-is-the-price-plan-for-text-to-speech/1162 |
| Platforms | Unity and Unreal plugins, Web plugins, "Modding Frameworks", Pixel Streaming embed. A forum thread asks whether the Live API is Unity-only; the reply was unclear for Unreal. | Verified (platform list); Live API per-engine Unverified | https://docs.convai.com/ ; https://forum.convai.com/t/live-api-only-available-for-unity/5644 |
| Features | Knowledge upload, 65+ languages and 500+ voices, animation set, narrative flow design, long-term memory, emotional state, guardrails, vision perception, lip sync. The page does not name the LLM provider. | Verified (marketing text) | https://convai.com/ |
| Self-hosting | "Enterprise Ready" section lists "On Prem Deployment Options" as a heading only; no detail. | Verified (heading only); details Unverified | https://convai.com/ |
| Privacy | Policy last updated 21 Feb 2023. Data processed in the US. No statement on voice recordings, conversation retention, or whether user data trains models. Third-party LLM vendors are not named. Account data deleted within 30 days of closure. | Verified (2023 policy; may be stale) | https://convai.com/privacy-policy |
| Determinism / record-replay | Not documented. Real-time cloud character calls; no replay surface found. | Unverified (absence of evidence) | — |

### 2.3 Google DeepMind Concordia

| Item | Finding | Status | Source |
|---|---|---|---|
| Licence | Apache-2.0 (repository sidebar). PyPI package `gdm-concordia`. | Verified | https://github.com/google-deepmind/concordia |
| Maintenance | 1,379 commits on `main`. No release or last-commit date was visible on the fetched page. | Partly verified (recency Unverified) | https://github.com/google-deepmind/concordia |
| Status | "This is not an officially supported Google product." | Verified | https://github.com/google-deepmind/concordia |
| Pricing / hosting | Library; no hosted service. Cost is whatever LLM API you call. | Verified | https://github.com/google-deepmind/concordia |
| What it provides | Game Master (GM) entity that simulates the environment and resolves natural-language actions into outcomes. Entities built from modular Components (instructions, memory, action selection). Prefabs as templates. Engine handles turn-taking, simultaneous and asynchronous action, and roles per `run_loop`. Actions are "described in natural language"; no structured action format is described. | Verified | https://github.com/google-deepmind/concordia ; https://github.com/google-deepmind/concordia/blob/main/README.md |
| Memory | "You must also provide a text embedder for associative memory." Structure not described. | Verified (requirement) | https://github.com/google-deepmind/concordia/blob/main/README.md |
| Retrieval / goals / NPC-NPC talk | NPC-NPC talk is implicit (entities act in turns through the GM). Goal and retrieval mechanics are not described on the README; see the paper (arXiv 2312.03664) for detail. | Partly verified | https://arxiv.org/html/2312.03664v2 |
| LLM backends | "Any LLM API that supports sampling text should work." Install extra `[google]`. No local/Ollama extra named on the README. | Verified | https://github.com/google-deepmind/concordia/blob/main/README.md |
| SDK language | Python (PyPI, `pyproject.toml`). | Verified | https://pypi.org/project/gdm-concordia |
| Offline | Not stated. Needs a model API for every decision; a local model via a custom sampler would be possible but is not documented. | Unverified | — |
| Determinism / record-replay | Not stated on the README (no seeding or logging mentioned). | Verified (absence) | https://github.com/google-deepmind/concordia/blob/main/README.md |
| Authority over state | The GM owns the environment state in the simulation; the framework is built around the GM resolving actions. This is the opposite of our "server is authority; controllers submit intents" rule. | Verified (design reading of README) | https://github.com/google-deepmind/concordia/blob/main/README.md |

### 2.4 a16z AI Town (briefly)

| Item | Finding | Status | Source |
|---|---|---|---|
| Licence | MIT ("A MIT-licensed, deployable starter kit"). | Verified | https://github.com/a16z-infra/ai-town |
| Maintenance | About 10.6k stars, 1,147 commits, 67 open issues, 6 open PRs. No last-commit date visible. | Partly verified | https://github.com/a16z-infra/ai-town |
| Backend | Convex: game engine, database, vector search. Docker Compose option runs frontend, self-hosted Convex backend and dashboard locally. | Verified | https://github.com/a16z-infra/ai-town |
| LLM | Default chat `llama3`, embeddings `mxbai-embed-large`, through Ollama by default, "entirely locally". Also Together.ai and OpenAI-compatible APIs. | Verified | https://github.com/a16z-infra/ai-town |
| Music | Background music uses Replicate (cloud), so the default is not fully offline. | Verified | https://github.com/a16z-infra/ai-town |
| Agent loop | `Agent.tick()` in `convex/aiTown/agent.ts`: idle agents call `agentDoSomething`; invites accepted with `INVITE_ACCEPT_PROBABILITY` via `Math.random()`; walking uses `movePlayer()`; conversation messages from `agentGenerateMessage`; memories stored via `agentRememberConversation`. Whether each decision calls an LLM is not visible in that file. | Verified (file read) | https://github.com/a16z-infra/ai-town/blob/main/convex/aiTown/agent.ts |
| Memory | Embedding-based memories in Convex vector search, `NUM_MEMORIES_TO_SEARCH` controls retrieval count. Reflection not described on the fetched page. | Partly verified | https://github.com/a16z-infra/ai-town |
| Authority | The Convex backend owns all world state. The world is the app. | Verified (design reading) | https://github.com/a16z-infra/ai-town |
| Determinism / record-replay | Not offered. Randomness uses `Math.random()`. | Verified (in file read) | https://github.com/a16z-infra/ai-town/blob/main/convex/aiTown/agent.ts |

### 2.5 Stanford Generative Agents (Smallville)

| Item | Finding | Status | Source |
|---|---|---|---|
| Licence | Apache-2.0 per the GitHub sidebar on `joonspk-research/generative_agents`. The mirror `standardgalactic/generative_agents` shows no licence. The community fork `nicfok/smallville` is MIT and is not the original. | Verified for the original repo (sidebar read); cross-check the LICENSE file before relying on it | https://github.com/joonspk-research/generative_agents |
| Maintenance | 31 commits on `main`; no last-commit date visible. Appears to be a research artifact. | Partly verified | https://github.com/joonspk-research/generative_agents |
| Language / stack | Python 3.9 tested; Django frontend. | Verified | https://github.com/joonspk-research/generative_agents |
| LLM backend | OpenAI API key placed in `utils.py`. No local model option documented. Notes that "OpenAI's API can hang when it reaches the hourly rate limit." | Verified | https://github.com/joonspk-research/generative_agents |
| Memory / reflection / planning | Memory stream; the repo's history files "insert the memory records into the agents' memory stream." Full reflection and planning are in the paper, not the README. | Partly verified; architecture from paper (arXiv 2304.03442, not fetched in this audit) | https://github.com/joonspk-research/generative_agents |
| Licence of data/assets | Citation requested for code or data use; encouragement to support artists for assets. | Verified | https://github.com/joonspk-research/generative_agents |
| Determinism / record-replay | Not documented. | Verified (absence on README) | https://github.com/joonspk-research/generative_agents |

---

## 3. Concept map

Their model in the left column; MineWorld's counterpart in the right column. "Controller" means our `Controller` interface, with the `LMController` (P6) as the only model-driven implementation.

| Their concept | Where | MineWorld counterpart | Notes |
|---|---|---|---|
| Memory stream / embedding memories (Smallville, AI Town) | Stanford, AI Town | P4 per-seat SQLite memory store, `ARC-59`, `DEP-37` | Ours is a derivation of perceived events (`perceived` stream, `ARC-43`), with Event IDs kept for citation. Theirs keeps raw memories with embeddings. Ours is rebuildable with the store deleted (`I-5`). |
| Retrieval by relevance (AI Town `NUM_MEMORIES_TO_SEARCH`; Concordia associative memory via embedder) | AI Town, Concordia | P4 FTS5 match filter plus integer ranking; ≤6 KB bounded context | We do not need dense embeddings for MVP; FTS5 lexical match suffices for trigger words. An embedder would be a new dependency and must go through `DEP-` review. |
| Reflection / compression of memories | Stanford (paper) | P4 roll-ups with Event-ID citations (`ARC-59`, `AC-10`) | Our roll-ups are summaries whose sources must stay addressable. Theirs are free-text reflections. |
| Goals / plans | Stanford (paper), Inworld/Convai (marketing) | Routine policy in P6 (`I-15`: movement and routine deterministic; model decides only social choices) | Ours: model picks only social choices; routine and movement are deterministic. Theirs: LLM plans. |
| Actions as natural language (Concordia, Stanford) | Concordia README | `ActionIntent` (typed) via `SocialChoice` (P6) | Key difference. Natural language is resolved by a GM in Concordia. Our typed `ActionIntent` is validated by the authoritative server; natural language never mutates state. |
| Game Master resolves actions (Concordia) | Concordia | Authoritative server + System Packs (`ARCHITECTURE.md`) | Concordia's GM is a model-driven referee. In MineWorld, rules are deterministic Systems; a model may propose, the server decides. |
| NPC-NPC conversation (AI Town invites; Concordia turn engine) | AI Town `agent.ts` | Social systems plus `LMController` proposals; multi-seat coordination via seat exclusivity (`I-14`) | AI Town's invite gate uses `Math.random()`; ours must be seeded and reproducible (`CLAUDE.md` §4 rule 10). |
| Voice (Inworld, Convai) | Inworld TTS/STT, Convai | Presentation pack concern; not in cognition contracts | Voice belongs in a presentation adapter, never in simulation or cognition contracts (`CLAUDE.md` §4 rule 3). |
| LLM provider adapters (Concordia "any sampling API", AI Town Ollama/Together, Inworld Router) | All | P5 backends (`DEP-27`, `DEP-33`, `DEP-32`) | We already have local Ollama + 11 OpenAI-compatible presets + native Anthropic (API key only, `DEP-33`). Subscription backends are opt-in (`ARC-60`). |
| Recording / replay | None of the five | P5 cassettes (`ARC-58`, `AC-4`) | Unique to our design. No prior art here; this is where we are ahead of the reference set. |
| Budgets / cost ceilings | Inworld/Convai per-seat pricing; not configurable in the libraries | P5 wall-time budgets (`ARC-57`) | Our cap is enforced before the call. |

---

## 4. Are we reinventing?

Compared concretely with what is merged.

**Memory (P4, merged).** The reference set does not offer a memory design we lack, except dense embeddings and LLM-written reflections.
- Concordia offers an "associative memory" that requires an embedder. It does not describe storage, bounds, provenance or rebuild. We have all four.
- AI Town stores embeddings in Convex vector search. It has no per-seat bound documented and no citation of source events. Its memory is tied to the Convex deployment, so it cannot be a derivation of the world's event log.
- Stanford's memory stream is the origin of reflection and planning. Its reflection is a free-text LLM output with no event citation and no bound stated. Ours is the same idea with bounds and citations (`AC-10`).
- Conclusion: we are not reinventing a mature wheel for memory storage or bounding. The one candidate we might be missing is **dense retrieval**. Adding it would need a new dependency decision (`DEP-`) and a deterministic embedding fixture for CI. It is not needed for the Milestone D scenario, where FTS5 match on trigger words is enough.

**Backends (P5, merged).** The reference set's model layer is thin and mostly a single provider.
- AI Town supports Ollama, Together.ai and any OpenAI-compatible API. Our P5 covers the same ground with 11 presets plus Ollama and native Anthropic.
- Concordia takes "any LLM API that supports sampling text" but does not implement the adapters.
- Inworld's LLM Router is a hosted multi-provider proxy. Using it would make our world depend on a vendor (`ARC-60` reasoning applies).
- Conclusion: our backend layer is equal to or richer than the references. Adopting a hosted router would reduce control and add a privacy dependency. Reject.

**Agent behaviour layer (P6, not yet built).** This is where a reinvention question is live.
- Concordia's component/engine design (modular components, entities, turn engine) is a good design reference for how an `LMController` might be structured. We should not adopt its runtime, because its GM owns state.
- AI Town's `agent.ts` loop is a reasonable reference for trigger-driven conversation invites. We already have the equivalent in deterministic routine policy (`I-15`) and seat exclusivity (`I-14`).
- Conclusion: we should design P6 from our own contracts. The only items to borrow are ideas (see §5a).

**Recording and replay (P5, merged).** No reference project offers this. We are not reinventing it; we are building something the references lack.

**Hosted characters (Inworld, Convai).** These are not substitutes for our architecture. They duplicate the entire world-state and memory stack inside a vendor, which is the opposite of the project's premise.

---

## 5. What to adopt

### 5a. Design ideas (adopt as ideas, no dependency)

1. **Modular controller components** (Concordia): split an `LMController` into separate, testable parts (persona, memory view, choice prompt, parser) that each take typed inputs. This matches the `systems/persona/**` layout already in step-17 P6.
2. **Natural-language actions are proposals, not state changes** (Concordia's GM concept, inverted): the model writes a free-text intent, a parser maps it to a typed `SocialChoice`, and the server validates it. Ours is stricter than Concordia's, and that is intended.
3. **Retrieval-count knob** (AI Town `NUM_MEMORIES_TO_SEARCH`): expose a single bounded retrieval count in persona configuration, alongside the ≤6 KB context budget.
4. **Conversation invite as a status machine** (AI Town `invited` / `walkingOver` states): useful for the social system's state, but the state belongs to the owning System, not to a controller.
5. **Reflection as a periodic roll-up** (Stanford): already in P4 (`ARC-59`). Keep event citations, which Stanford does not provide.

### 5b. Candidates for an OPTIONAL controller adapter

The adapter lives outside the core, behind our `Controller` interface, and is never a core dependency. Each candidate lists conditions.

| Candidate | What it would do | Conditions before any work |
|---|---|---|
| **Concordia** (Apache-2.0, Python) | Run a Concordia `Entity` as one seat's controller, translating its natural-language output into `SocialChoice` through our parser | (1) Adapter lives in a separate optional package, not `cognition/lm-controller`; (2) the Concordia GM is not used as authority; (3) the adapter is covered by a cassette test that replays under our `ARC-58` key rules; (4) IC-10 scan passes (no provider names in core); (5) a `DECISIONS.md` record per `REUSE_POLICY.md`; (6) Concordia's dependencies do not reach CI core jobs; (7) no embedder required by default. |
| **AI Town** (MIT, TypeScript, Convex) | Not an adapter candidate. Its server owns state. | Reference only. Do not build an adapter; the architecture is incompatible with a server-authoritative world. |
| **Stanford Smallville code** (Apache-2.0 per sidebar, Python, OpenAI-only) | Reference only. Adapter would need OpenAI and no replay. | Reference for the memory and reflection prompts only, with citation. Do not adopt the code. |
| **Inworld / Convai** | Hosted character backend as a controller | Not recommended. Requires network, vendor account, cloud data flow, no replay. Violates `VISION.md` "models removed → world still valid" in spirit: the world would be meaningless without a hosted character. Only an optional presentation-side voice adapter could be considered (out of scope for P6). |

---

## 6. Why not core dependencies

1. **Authority.** Concordia's GM, AI Town's Convex backend, and Inworld/Convai character runtimes all own or mediate world state. MineWorld's rule is that the server is authoritative and controllers submit `ActionIntent`s (`CLAUDE.md` §4 rule 3; `ARCHITECTURE.md`). None of these can be made to satisfy it without forcing MineWorld inside their execution model, which `REUSE_POLICY.md` forbids.
2. **Models removed must keep the world valid.** Inworld and Convai characters have no meaning without their hosted model. Concordia's GM is a model. AI Town's agents are an LLM-driven town. Each would turn a world into an AI-NPC demo, which `VISION.md` names as a failure.
3. **Determinism and replay.** None of the five offers recorded, replayable, seeded model decisions. CI must never call a model (`CLAUDE.md` §4 rule 10). Our cassette system (`ARC-58`) has no equivalent to adopt.
4. **Local-first and privacy.** Inworld and Convai process data in vendor clouds; Convai's 2023 policy names no LLM vendors and does not address voice or training use. AI Town's default is local, but its Convex backend and Replicate music make it non-offline by default. Our default is local Ollama with optional keys (`DEP-32`, `DEP-33`).
5. **Maintenance and status.** Concordia says it is not an officially supported Google product. Stanford's repo is a research artifact (31 commits). Inworld's product lifecycle could not be verified, and the operator reports a Character Studio deprecation that this audit could not confirm. Vendor products with unclear lifecycle are a dependency risk.
6. **Licensing of the hosted products.** Inworld and Convai licences and SDK terms for Runtime/Character Engine are not verified; any dependency would need a licence review before adoption.

---

## 7. Recommendation for P6 / P7 scope

1. **Do not adopt** any of the five as a core dependency. Record this as a `DECISIONS.md` entry (rejection of hosted character runtimes and of Concordia/AI Town/Smallville as runtimes), per `REUSE_POLICY.md`, which requires a short record for rejections.
2. **P6 (`LMController` and `persona`)**: keep it on our own contracts (`SocialChoice`, decision log, revalidation, `I-14`, `I-15`). Borrow only the ideas in §5a.
3. **P6 retrieval**: keep FTS5 match plus integer ranking. Do not add dense embeddings in P6. If retrieval quality fails the Milestone D scenario, file a separate decision with a deterministic embedding fixture before adding any dependency.
4. **P7 (Milestone D test)**: use recorded cassettes (`IC-6`, `IC-8`) only. No reference project supplies a test pattern to copy; ours is the stronger one.
5. **Optional, post-Milestone D**: if the operator wants a Concordia-based NPC, open a separate design under `structured-coding` as an optional controller adapter with the §5b conditions. It must not block P6 or P7, and it must not appear in CI core.
6. **Verify before relying on anything in §2.1 for Inworld.** Re-check the Character Studio deprecation and Runtime/Character Engine licence on Inworld's own changelog or sales contact before writing any comparison in a design document.

---

## 8. Unverified items and gaps (must be closed before any decision depends on them)

- Inworld Runtime and Character Engine: licence, SDK languages, pricing, offline or on-prem support, memory/goals/actions design. Not found on fetched primary pages.
- Inworld Character Studio deprecation: reported by the operator; no primary source found in this audit.
- Convai pricing page: did not render; ToolRadar figures are third-party.
- Convai SDK licence terms and whether Live API is Unity-only.
- Concordia: release date and last commit; whether a local/Ollama sampler is supported in practice.
- AI Town: last commit date; whether per-decision LLM calls occur (only the file read is verified).
- Stanford: confirm LICENSE file text (sidebar says Apache-2.0); the architecture detail is from the paper (arXiv 2304.03442), which was not fetched.
- Access dates: all 2026-10-11. Star counts and commit counts are as shown at access and will drift.
