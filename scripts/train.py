#!/usr/bin/env python3
"""Merge trains: several related component PRs tested and merged together (operator CI policy of
2026-10-18, `docs/DECISIONS.md` ARC-154).

A component PR runs `fast` only. A train is a branch `train/<name>`, made from `main` by merging each
component PR's head in order with a merge commit, and opened as a PR of its own: CI runs the targeted test
layers for the union of the components' changes (scripts/ci_changes.py, mode `targeted`), or the full
matrix when the train PR is labelled `full-ci`. Merging the train PR with a merge commit lands every
component head on `main`, which is what makes GitHub mark each component PR as merged.

    python3 scripts/train.py build <name> <pr>... [--base main] [--replace]
        create train/<name> from origin/<base>, merge each PR's head in the order given (stopping, and
        naming the PR, on a conflict), push it, and open the train PR; --replace rebuilds an existing
        train (a bisect: rebuild it without the suspect) and force-pushes it
    python3 scripts/train.py status <name>
        the train PR's checks, and whether each component's head is still the one the train merged
    python3 scripts/train.py --self-test

The working tree is never touched: the train is assembled in a temporary worktree, which is removed
afterwards. `git` and `gh` (authenticated) do the work; nothing here talks to GitHub by itself.
Standard library only.
"""

from __future__ import annotations

import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PREFIX = "train/"
NAME = re.compile(r"^[a-z0-9][a-z0-9._-]{0,62}$")
BEGIN, END = "<!-- train:components -->", "<!-- /train:components -->"
LINE = re.compile(r"^- #(\d+) `([0-9a-f]{40})` (.*)$")


class TrainError(Exception):
    """A train cannot be built or read; the message says which PR or step."""


@dataclass(frozen=True)
class Component:
    number: int
    head: str  # the head commit the train merged
    title: str
    branch: str = ""


def run(*command: str, cwd: Path = ROOT, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(list(command), cwd=cwd, capture_output=True, text=True)
    if check and result.returncode != 0:
        raise TrainError(f"`{' '.join(command)}` failed: {(result.stderr or result.stdout).strip()}")
    return result


def check_name(name: str) -> str:
    if not NAME.match(name):
        raise TrainError(f"a train name is lower-case letters, digits, '.', '_' and '-', not {name!r}")
    return name


# ------------------------------------------------------------------------------------- the PR body


def body(name: str, base: str, components: list[Component]) -> str:
    lines = [
        f"Merge train `{PREFIX}{name}` into `{base}` (operator CI policy 2026-10-18, docs/DECISIONS.md ARC-154).",
        "",
        "Built by `scripts/train.py build`: each component PR's head below merged in order with a merge",
        "commit. Merge this PR with a **merge commit** (not squash, not rebase), so that every component head",
        "lands and GitHub marks each component PR as merged. `scripts/train.py status " + name + "` shows",
        "whether a component moved since this train was built; a moved component means rebuilding the train.",
        "",
        "Components:",
        "",
        BEGIN,
        *(f"- #{c.number} `{c.head}` {c.title}" for c in components),
        END,
        "",
        "🤖 Generated with [Claude Code](https://claude.com/claude-code)",
    ]
    return "\n".join(lines)


def components_of(text: str) -> list[Component]:
    """The components a train PR's body lists, in merge order."""
    if BEGIN not in text or END not in text:
        raise TrainError("the PR body has no train:components block: not built by scripts/train.py")
    block = text.split(BEGIN, 1)[1].split(END, 1)[0]
    found = [LINE.match(line.strip()) for line in block.splitlines() if line.strip()]
    if not found or not all(found):
        raise TrainError("the train:components block is malformed")
    return [Component(int(m.group(1)), m.group(2), m.group(3)) for m in found if m]


# ------------------------------------------------------------------------------------- the merges


def merge_all(tree: Path, branch: str, components: list[Component], identity: tuple[str, ...] = ()) -> None:
    """Merge each component's head into `branch` (checked out in `tree`), in order, with a merge commit.
    A conflict aborts that merge and names the PR; nothing after it is merged."""
    for component in components:
        message = f"train {branch}: merge #{component.number} ({component.title})"
        result = run("git", *identity, "merge", "--no-ff", "--no-edit", "-m", message, component.head,
                     cwd=tree, check=False)
        if result.returncode != 0:
            run("git", "merge", "--abort", cwd=tree, check=False)
            conflicted = (result.stdout + result.stderr).strip().splitlines()[-3:]
            raise TrainError(f"#{component.number} ({component.title}) does not merge onto the train: "
                             + " / ".join(conflicted))


def gh_json(*arguments: str) -> object:
    import json

    return json.loads(run("gh", *arguments).stdout)


def build(name: str, numbers: list[int], base: str, replace: bool) -> int:
    branch = PREFIX + check_name(name)
    if len(set(numbers)) != len(numbers):
        raise TrainError("a PR is listed twice")
    components: list[Component] = []
    for number in numbers:
        view = gh_json("pr", "view", str(number), "--json", "number,state,baseRefName,headRefName,headRefOid,title")
        assert isinstance(view, dict)
        if view["state"] != "OPEN":
            raise TrainError(f"#{number} is {view['state']}, not open")
        if view["baseRefName"] != base:
            raise TrainError(f"#{number} targets {view['baseRefName']}, not {base}")
        components.append(Component(number, view["headRefOid"], view["title"], view["headRefName"]))
    run("git", "fetch", "--quiet", "origin", base, *(f"pull/{c.number}/head" for c in components))
    exists = run("git", "ls-remote", "--exit-code", "--heads", "origin", branch, check=False).returncode == 0
    if exists and not replace:
        raise TrainError(f"{branch} exists on origin: pass --replace to rebuild it")
    scratch = Path(tempfile.mkdtemp(prefix="mineworld-train-"))
    tree = scratch / "tree"
    try:
        run("git", "worktree", "add", "--quiet", "--detach", str(tree), f"origin/{base}")
        run("git", "switch", "--quiet", "-C", branch, cwd=tree)
        merge_all(tree, branch, components)
        head = run("git", "rev-parse", "HEAD", cwd=tree).stdout.strip()
        push = ["git", "push", "--quiet", "origin", f"HEAD:refs/heads/{branch}"]
        run(*(push[:3] + ["--force"] + push[3:] if exists else push), cwd=tree)
    finally:
        run("git", "worktree", "remove", "--force", str(tree), check=False)
        run("git", "branch", "--quiet", "-D", branch, check=False)
        shutil.rmtree(scratch, ignore_errors=True)
    text = body(name, base, components)
    found = gh_json("pr", "list", "--head", branch, "--state", "open", "--json", "number,url")
    assert isinstance(found, list)
    if found:
        run("gh", "pr", "edit", str(found[0]["number"]), "--body", text)
        print(f"train {branch} rebuilt at {head}: {found[0]['url']}")
    else:
        titles = ", ".join(f"#{c.number}" for c in components)
        url = run("gh", "pr", "create", "--base", base, "--head", branch, "--title",
                  f"train: {name} ({titles})", "--body", text).stdout.strip()
        print(f"train {branch} built at {head}: {url}")
    for component in components:
        print(f"  #{component.number} {component.head[:12]} {component.title}")
    return 0


def status(name: str) -> int:
    branch = PREFIX + check_name(name)
    found = gh_json("pr", "list", "--head", branch, "--state", "all", "--json", "number,url,state,body")
    assert isinstance(found, list)
    if not found:
        raise TrainError(f"no PR has head {branch}")
    pr = found[0]
    print(f"train {branch}: #{pr['number']} {pr['state']} {pr['url']}")
    moved = 0
    for component in components_of(pr["body"]):
        view = gh_json("pr", "view", str(component.number), "--json", "headRefOid,state")
        assert isinstance(view, dict)
        same = view["headRefOid"] == component.head
        moved += not same
        verdict = "unchanged" if same else f"MOVED to {view['headRefOid'][:12]}: rebuild the train"
        print(f"  #{component.number} {view['state']:<6} {component.head[:12]} {verdict}  {component.title}")
    checks = run("gh", "pr", "checks", str(pr["number"]), check=False)
    print(checks.stdout.rstrip() or checks.stderr.rstrip())
    return 1 if moved else 0


# ------------------------------------------------------------------------------------- self-test


def self_test() -> int:
    failures: list[str] = []

    def check(condition: bool, message: str) -> None:
        if not condition:
            failures.append(message)

    parts = [Component(12, "a" * 40, "docs: one"), Component(7, "b" * 40, "fix(x): two `quoted`")]
    check(components_of(body("lane-a", "main", parts)) == parts, "the body round-trips its components in order")
    for bad in ("no block", f"{BEGIN}\n- #1 short title\n{END}", f"{BEGIN}\n{END}"):
        try:
            components_of(bad)
            failures.append(f"components_of accepted {bad!r}")
        except TrainError:
            pass
    for bad in ("", "Upper", "a/b", "-x", "x" * 64):
        try:
            check_name(bad)
            failures.append(f"check_name accepted {bad!r}")
        except TrainError:
            pass
    check(check_name("s32-wardrobe.2") == "s32-wardrobe.2", "a plain name is accepted")

    # The merges, on a real throwaway repository: two clean components land in order with merge commits
    # (each head an ancestor of the train), and a conflicting one stops the train and is named.
    identity = ("-c", "user.name=train self-test", "-c", "user.email=train@example.invalid")
    scratch = Path(tempfile.mkdtemp(prefix="mineworld-train-test-"))
    try:
        def git(*arguments: str) -> str:
            return run("git", *identity, *arguments, cwd=scratch).stdout.strip()

        git("init", "--quiet", "-b", "main")
        (scratch / "a.txt").write_text("base\n")
        git("add", "a.txt")
        git("commit", "--quiet", "-m", "base")

        def branch(name: str, file: str, text: str) -> str:
            git("switch", "--quiet", "-c", name, "main")
            (scratch / file).write_text(text)
            git("add", file)
            git("commit", "--quiet", "-m", name)
            head = git("rev-parse", "HEAD")
            git("switch", "--quiet", "main")
            return head

        one = Component(1, branch("one", "b.txt", "one\n"), "one")
        two = Component(2, branch("two", "c.txt", "two\n"), "two")
        clash = Component(3, branch("clash", "a.txt", "clash\n"), "clash")
        clash_too = Component(4, branch("clash-too", "a.txt", "other\n"), "clash too")
        git("switch", "--quiet", "-c", "train/t", "main")
        merge_all(scratch, "train/t", [one, two], identity)
        for component in (one, two):
            ancestor = run("git", "merge-base", "--is-ancestor", component.head, "HEAD", cwd=scratch, check=False)
            check(ancestor.returncode == 0, f"#{component.number}'s head is on the train")
        parents = git("log", "--merges", "--format=%s", "main..HEAD").splitlines()
        check(parents == ["train train/t: merge #2 (two)", "train train/t: merge #1 (one)"],
              f"one merge commit per component, in order: {parents}")
        try:
            merge_all(scratch, "train/t", [clash, clash_too], identity)
            failures.append("a conflicting component was merged")
        except TrainError as error:
            check("#4" in str(error) and "#3" not in str(error), f"the conflict names the PR: {error}")
        check(git("status", "--porcelain") == "", "a conflict leaves the train clean (merge aborted)")
    finally:
        shutil.rmtree(scratch, ignore_errors=True)

    for failure in failures:
        print(f"train self-test: FAIL: {failure}")
    print(f"train self-test: {'FAILED' if failures else 'passed'}")
    return 1 if failures else 0


def main(arguments: list[str]) -> int:
    try:
        if arguments == ["--self-test"]:
            return self_test()
        if len(arguments) == 2 and arguments[0] == "status":
            return status(arguments[1])
        if len(arguments) >= 3 and arguments[0] == "build":
            rest = list(arguments[2:])
            replace = "--replace" in rest
            rest = [argument for argument in rest if argument != "--replace"]
            base = "main"
            if "--base" in rest:
                at = rest.index("--base")
                if at + 1 >= len(rest):
                    raise TrainError("--base names a branch")
                base = rest[at + 1]
                del rest[at : at + 2]
            if not rest or not all(argument.lstrip("#").isdigit() for argument in rest):
                raise TrainError("build takes PR numbers")
            return build(arguments[1], [int(argument.lstrip("#")) for argument in rest], base, replace)
    except TrainError as error:
        print(f"train.py: {error}", file=sys.stderr)
        return 1
    print("usage: train.py build <name> <pr>... [--base B] [--replace] | status <name> | --self-test", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
