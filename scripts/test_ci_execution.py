#!/usr/bin/env python3
"""Focused CI orchestration regressions; real archive fixture is opt-in via --integration."""

from __future__ import annotations

import contextlib
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import ci_archive
import ci_layer
import ci_results
import ci_shard


class ExecutionTests(unittest.TestCase):
    def test_repeat_is_a_test_but_build_is_not(self):
        commands = ci_layer.repeat_commands(
            ci_shard.commands(["x", ci_shard.DOC], "cargo")
        )
        self.assertEqual([ci_layer.is_test(c) for c in commands], [False, True, True])
        self.assertFalse(
            ci_layer.is_test(["python3", "other.py", "--", "cargo", "test"])
        )

    def test_failures_continue_and_first_status_survives(self):
        commands = ci_layer.repeat_commands(
            ci_shard.commands(["x", ci_shard.DOC], "cargo")
        )
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(ci_layer, "FAILED_TESTS", Path(directory) / "fail.md"),
            patch.object(ci_layer, "commands_of", return_value=commands),
            patch.object(ci_layer, "report"),
            patch.object(ci_layer, "disk"),
            patch.object(ci_layer, "result"),
            patch.object(
                ci_layer, "stream", side_effect=[(0, []), (17, []), (23, [])]
            ) as stream,
            contextlib.redirect_stdout(io.StringIO()),
        ):
            self.assertEqual(ci_layer.run("core-repeat"), 17)
            self.assertEqual(stream.call_count, 3)
            self.assertIn("exit 23", ci_layer.FAILED_TESTS.read_text())

    def test_build_failure_stops(self):
        commands = ci_shard.commands(["x", ci_shard.DOC])
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(ci_layer, "FAILED_TESTS", Path(directory) / "fail.md"),
            patch.object(ci_layer, "ROOT", Path(directory)),
            patch.object(ci_layer, "commands_of", return_value=commands),
            patch.object(ci_layer, "report"),
            patch.object(ci_layer, "disk"),
            patch.object(ci_layer, "result"),
            patch.object(ci_layer, "stream", return_value=(9, [])) as stream,
            contextlib.redirect_stdout(io.StringIO()),
        ):
            self.assertEqual(ci_layer.run("core"), 9)
            self.assertEqual(stream.call_count, 1)

    def test_archive_has_no_regular_cargo_build(self):
        commands = ci_shard.commands(
            [ci_shard.LIB_BINS, "x", "launcher", ci_shard.DOC], archive=Path("a")
        )
        self.assertEqual(len(commands), 3)
        self.assertEqual(commands[0][:3], ["cargo", "nextest", "run"])
        self.assertNotIn("--workspace", commands[0])
        self.assertIn("kind(test) & binary(=x)", commands[0][-1])
        self.assertNotIn("launcher", commands[0][-1])
        self.assertEqual(commands[1][-2:], ["--test", "launcher"])
        self.assertEqual(commands[2][-1], "--doc")

    def test_results_and_weights_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "junit.xml"
            for payload in (
                "broken",
                "<testsuites/>",
                "<testsuites><testsuite><testcase/></testsuite></testsuites>",
            ):
                path.write_text(payload)
                with self.assertRaises(
                    (ValueError, KeyError, ci_results.ElementTree.ParseError)
                ):
                    ci_results.failures(path)
                with self.assertRaises(
                    (ValueError, KeyError, ci_results.ElementTree.ParseError)
                ):
                    ci_shard.weigh([path])
            path.write_text(
                '<testsuites><testsuite name="pkg::x"><testcase name="a" time="4">'
                '<error type="timeout"/></testcase></testsuite></testsuites>'
            )
            self.assertEqual(ci_results.failures(path), ["`pkg::x`: `a` (timeout)"])
            with self.assertRaises(ValueError):
                ci_shard.weigh([path])
            path.write_text(
                '<testsuites><testsuite name="pkg::x"><testcase name="a" time="4"/>'
                "</testsuite></testsuites>"
            )
            self.assertEqual(ci_shard.weigh([path]), {"x": 4})
            with self.assertRaises(ValueError):
                ci_shard.weigh([path, path])
            for duration in ("NaN", "inf", "-1"):
                path.write_text(
                    path.read_text().replace('time="4"', f'time="{duration}"')
                )
                with self.assertRaises(ValueError):
                    ci_shard.weigh([path])
                path.write_text(
                    path.read_text().replace(f'time="{duration}"', 'time="4"')
                )

    def test_archive_provenance_and_corruption(self):
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(ci_archive, "identity", return_value={"os": "test"}),
        ):
            root = Path(directory)
            (root / "tests.tar.zst").write_bytes(b"archive")
            data = {
                "schema": 1,
                "identity": {"os": "test"},
                "units": [ci_shard.LIB_BINS, ci_shard.DOC],
                "sha256": ci_archive.digest(root / "tests.tar.zst"),
            }
            (root / "manifest.json").write_text(json.dumps(data))
            self.assertEqual(ci_archive.verify(root), data)
            for field, value in (
                ("identity", {"os": "wrong"}),
                ("sha256", "wrong"),
                ("units", []),
            ):
                (root / "manifest.json").write_text(json.dumps({**data, field: value}))
                with self.assertRaises(ValueError):
                    ci_archive.verify(root)


def fixture(root: Path) -> None:
    files = {
        "Cargo.toml": '[package]\nname="archive-probe"\nversion="0.1.0"\nedition="2024"\n'
        '[[test]]\nname="launcher"\nharness=false\n',
        "src/lib.rs": "//! ```\n//! assert_eq!(2 + 2, 4);\n//! ```\n"
        '//! ```\n//! panic!("intentional doctest failure");\n//! ```\n'
        "#[test] fn library_pass() {}\n",
        "src/main.rs": 'fn main() { println!("auxiliary binary reached"); }\n',
        "tests/alpha.rs": '#[test] fn first_failure() { panic!("intentional alpha"); }\n'
        '#[test] #[ignore] fn ignored_case() { panic!("ignored"); }\n',
        "tests/beta.rs": '#[test] fn second_failure() { panic!("intentional beta"); }\n'
        "#[test] fn auxiliary() {\n"
        ' assert!(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml").exists());\n'
        ' let out=std::process::Command::new(env!("CARGO_BIN_EXE_archive-probe")).output().unwrap();\n'
        " assert!(out.status.success());\n"
        ' assert!(String::from_utf8(out.stdout).unwrap().contains("auxiliary binary reached"));\n}\n',
        "tests/launcher.rs": 'fn main() { println!("CUSTOM HARNESS REACHED"); std::process::exit(11); }\n',
        ".config/nextest.toml": '[profile.ci]\nfail-fast=false\n[profile.ci.junit]\npath="junit.xml"\n',
    }
    for file, content in files.items():
        path = root / file
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
    subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=root, check=True)
    subprocess.run(["git", "init", "-q", str(root)], check=True)
    subprocess.run(["git", "-C", str(root), "add", "."], check=True)
    subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "-c",
            "user.name=CI fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
        check=True,
    )


def check_repeat(root: Path, source: Path, evidence: Path | None) -> None:
    for script in ("ci_repeat.py", "check_scratch.py"):
        (root / "scripts").mkdir(exist_ok=True)
        shutil.copyfile(source / "scripts" / script, root / "scripts" / script)
    repeated = ci_layer.repeat_commands(
        ci_shard.commands(["alpha", ci_shard.DOC], "cargo")
    )
    with (
        patch.object(ci_layer, "commands_of", return_value=repeated),
        patch.object(ci_layer, "report"),
        patch.object(ci_layer, "disk"),
    ):
        assert ci_layer.run("core-repeat", "1/1") != 0
    for name in ("repeat.txt", "repeat-doc.txt"):
        report = root / "artifacts/nightly" / name
        text = report.read_text()
        assert text.count("ran yes") == 2, text
        if evidence:
            shutil.copyfile(report, evidence / name)


def integration() -> None:
    """No shipping assertions change: intentional failures live only in a disposable workspace."""
    source = ci_layer.ROOT
    evidence = (
        Path(os.environ["MINEWORLD_CI_EVIDENCE"])
        if os.environ.get("MINEWORLD_CI_EVIDENCE")
        else None
    )
    with tempfile.TemporaryDirectory(prefix="ci-nextest-fixture-") as directory:
        root = Path(directory).resolve()
        fixture(root)
        (root / "temp").mkdir()
        archive = root / "artifacts/core-archive"
        with (
            patch.dict(
                os.environ,
                {
                    "TMPDIR": str(root / "temp"),
                    "TMP": str(root / "temp"),
                    "TEMP": str(root / "temp"),
                },
            ),
            patch.object(ci_archive, "ROOT", root),
            patch.object(ci_shard, "ROOT", root),
            patch.object(ci_layer, "ROOT", root),
            patch.object(ci_layer, "FAILED_TESTS", root / "failures.md"),
            patch.object(ci_shard, "CUSTOM_HARNESS", frozenset({"launcher"})),
            patch.object(ci_shard, "WEIGHTS", {"alpha": 10, "beta": 10}),
        ):
            assert ci_archive.build(archive) == 0
            baseline = subprocess.check_output(
                [
                    "cargo",
                    "nextest",
                    "list",
                    "--lib",
                    "--bins",
                    "--test",
                    "alpha",
                    "--test",
                    "beta",
                    "--message-format",
                    "json",
                ],
                cwd=root,
                text=True,
            )
            expected = json.loads(baseline)["rust-suites"]
            # Delete every build output. Consumers must extract the archived binaries, including the auxiliary.
            shutil.rmtree(root / "target")
            statuses = []
            identities = set()
            for shard in ("1/2", "2/2"):
                commands, _ = ci_shard.shard_commands(
                    shard, archive=archive, units=ci_archive.verify(archive)["units"]
                )
                with (
                    patch.object(ci_layer, "commands_of", return_value=commands),
                    patch.object(ci_layer, "report"),
                    patch.object(ci_layer, "disk"),
                ):
                    statuses.append(ci_layer.run("core", shard))
                if shard == "1/2":
                    summary = (root / "failures.md").read_text()
                    assert "--test launcher" in summary and "--doc" in summary, summary
                junit = root / "target/nextest/ci/junit.xml"
                if evidence:
                    evidence.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(junit, evidence / f"fixture-shard-{shard[0]}.xml")
                for suite in (
                    ci_results.ElementTree.parse(junit).getroot().findall("testsuite")
                ):
                    identities.update(
                        (suite.attrib["name"], case.attrib["name"])
                        for case in suite.findall("testcase")
                    )
            wanted = {
                (binary, name)
                for binary, suite in expected.items()
                for name, test in suite["testcases"].items()
                if not test["ignored"]
            }
            assert identities == wanted, (identities, wanted)
            assert statuses == [100, 100], statuses
            assert (root / "target/debug/archive-probe").exists()
            check_repeat(root, source, evidence)
            print(
                f"archive integration PASS: two failed shards, {len(identities)} identities match baseline; "
                "custom harness and doctests attempted after nextest failures"
            )


if __name__ == "__main__":
    if sys.argv[1:] == ["--integration"]:
        integration()
    else:
        unittest.main(argv=[sys.argv[0]])
