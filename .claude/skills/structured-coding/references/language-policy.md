# Documentation language policy

## Authority

English is the only authoritative source for maintained documentation and specifications. Chinese files ending in `.zh-CN.md` are synchronized mirrors of explanatory documents. In a disagreement, follow English and correct the mirror. A mirror must not introduce, relax, or amend a requirement.

## Explanations with mirrors

Maintain an English source and a Chinese mirror for:

- The repository README.
- The skill's human workflow guide.
- The detailed agent workflow explanation.
- Prompt adaptation notes.
- Platform notes.

Keep corresponding sections, examples, caveats, tables, and links aligned. Chinese explanatory links should lead to Chinese mirrors where available. Links to specifications must lead to their English originals. Code, commands, paths, identifiers, status values, and reusable prompt examples remain unchanged.

Retain established English technical terms in Chinese prose, including LLM, agent, coding, bug, PR, commit, branch, HEAD, diff, review, audit, planning, execution, scope, invariant, checkpoint, validation, Gate, CI, hook, contract, ledger, handoff, compact, resume, and merge. Translate ordinary explanatory prose naturally; do not replace technical terms with inconsistent Chinese aliases or force every ordinary noun into English.

## English-only specifications

Keep `SKILL.md`, `prompts/*.md`, and `references/hook-contract.md` in English only. This maintenance policy and machine-readable metadata also remain English-only. Do not create translated specification copies or rewrite an existing specification merely to reorganize documentation languages.

## Synchronization procedure

1. Edit the authoritative English explanation first.
2. Update its complete Chinese mirror, including changed qualifications and examples. Preserve technical terms and specification references.
3. Review both documents for semantic agreement and omissions.
4. Update the pair's English and Chinese SHA-256 values in the authoring repository's `translations.json` only after that review.
5. Run `python3 scripts/build_packages.py` and `python3 scripts/build_packages.py --check` from the authoring repository root. Include both languages and regenerated packages in the same logical change.

The checker detects unacknowledged content changes, missing pairs, English-source language violations, and mismatched heading structure. It also verifies that the established specifications retain their recorded hashes. Fingerprints establish which bytes were reviewed; they cannot prove translation accuracy. Never mark a stale mirror synchronized merely by refreshing its checksum.

The specification baseline in `translations.json` records the files preserved during bilingual separation. A later intentional specification change requires its own explicit scope and review, followed by a deliberate baseline update.
