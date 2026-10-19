## ARC-60 — Subscriptions: the user's own CLI as a backend, opt-in, and never Claude's

**Date** 2026-10-09 · **Status** accepted; the bridge core implemented by S10 PR P5b; the Codex preset
**conditional** on the re-read of OpenAI's Terms of Use (P5b C1 → C4) · **Approved by** the operator's
rulings QP5b-1 (drop the Claude subscription route) and QP5b-2 (Codex as an opt-in for the user's own
local use), and the primary session (QP5b-3), 2026-10-09 · **Relates to** `DEP-27`, `DEP-32`, `DEP-33`,
`ARC-57`, `ARC-58` · **Design** `pr-s10-p5b-hosted-subscriptions.md` §4, §5.2 … §5.4, I-B1 … I-B5, AB-3
… AB-9

**Problem.** The operator asked for subscriptions "authorized by Codex's or Claude Code's own login"
beside API keys. A subscription is the user's account with a vendor, and the vendor's terms decide what
a third-party program may do with it.

**The evidence** (verbatim; dates and URLs in the design's §4):

- Anthropic, Claude Code "Legal and compliance" (`code.claude.com/docs/en/legal-and-compliance`, read
  2026-10-09): "Anthropic does not permit third-party developers to offer Claude.ai login into their own
  applications, or to route requests through Free, Pro, or Max plan credentials on behalf of their
  users. Moreover, developers may not collect, store, or intermediate Claude.ai credentials or session
  tokens". Enforcement "may do so without prior notice". Developers "should use API key authentication
  through Claude Console or a supported cloud provider".
- OpenAI, Codex authentication (`learn.chatgpt.com/docs/auth`, read 2026-10-09): "API keys are still the
  recommended default for automation." "Treat `~/.codex/auth.json` like a password." OpenAI's Terms of
  Use answered HTTP 403 to every fetch (planning, freeze, and P5b C1 on 2026-10-09); the clause quoted
  from search excerpts ("Automatically or programmatically extract data or Output") is not yet re-read
  from the live page.

**Choice.**

1. **No Claude subscription route** (QP5b-1). No `claude-code-subscription` kind exists; `config.load`
   reports it as unknown. Claude is supported by the user's own API key (`DEP-33`).
2. **A generic bridge**, `backend/cli_bridge.py` (`CliBridgeBackend`), runs the user's own installed and
   logged-in model CLI for one tool-less answer, under these invariants:
   - **I-B1** MineWorld never reads, writes, copies or forwards a CLI's credentials; the CLI
     authenticates itself;
   - **I-B2** the child's environment is an allowlist (system paths, home and temporary directories,
     locale, XDG directories, the preset's home variable): no `*_API_KEY`, no `*_TOKEN`, nothing from an
     `env_file`;
   - **I-B3** no model-facing text in argv: the prompt goes through stdin, the schema through a
     temporary file; argv holds only literals and validated configuration values (the Windows `.cmd`
     argument-injection class, CVE-2024-24576);
   - **I-B4** the call is bounded and its timeout kills the whole process tree (POSIX: a new session and
     `killpg`; Windows: a new process group and `taskkill /T /F`);
   - **I-B5** a subscription backend exists only when its preset was ruled in **and** the user set
     `acknowledge_terms = "<kind>"`; `replay` and `scripted` modes never construct it.
   The child runs in an empty temporary directory, so no project configuration is picked up. A bridged
   call passes the same gateway: the budget pre-check before the spawn, the same cassettes, the CLI's
   reported usage or the estimate.
3. **Codex through the user's ChatGPT plan** (QP5b-2): an opt-in for the user's own local use only, off
   by default, never in a shipped example, gated by `acknowledge_terms`, and documented after every
   API-key route. **Conditional:** its preset is built only after OpenAI's Terms of Use are re-read from
   the live page and recorded in the design's §4.2. If the re-read forbids the use, the route is dropped
   as Claude's was.

**Limitation accepted.** The user's own global CLI configuration (hooks, MCP servers) runs during a
bridged call; MineWorld does not and cannot disable it. A CLI's flags may change between versions; a
preset is one file.

**Revisit** if a vendor changes its terms or enforces against such use (remove the preset: one file and
one registry line), or if a vendor publishes an explicit permission or prohibition for third-party
programs running its CLI with a subscription.
