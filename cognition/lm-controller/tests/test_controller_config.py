"""The LM controller's configuration: refusals by name, and precedence (pr-s10-p6 §5.13, C3).

A refusal names the key or the file and never repeats a value from the file (I-16). Precedence is most
specific first: a seat's override, then its backend's, then `[retry]`, then the bundled default.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from mineworld_cognition.config import (
    BUNDLED_RETRY,
    CognitionConfig,
    ConfigError,
    controller_for,
    layer_policy,
    load,
)

BASE = '[recording]\nmode = "scripted"\n'
PLANTED = "sk-test-PLANTED-value"


def write(directory: Path, text: str) -> Path:
    path = directory / "cognition.toml"
    path.write_text(text, encoding="utf-8")
    return path


@pytest.mark.parametrize(
    ("extra", "names"),
    [
        ("bogus = 1\n", "bogus"),
        ("[controller]\nconsult_deadline_s = 121\n", "controller.consult_deadline_s"),
        ("[controller]\nmax_calls = 0\n", "controller.max_calls"),
        ("[controller]\nrecall_episodes = 9\n", "controller.recall_episodes"),
        ('invite_env = "abc-123"\n', "invite_env"),
        ('nickname = ""\n', "nickname"),
        ('[retry.transport]\npolicy = "Bad Id"\n', "retry.transport.policy"),
        (
            "[seat_overrides.alice.controller]\nmax_calls = 99\n",
            "seat_overrides.alice.controller.max_calls",
        ),
    ],
)
def test_out_of_bounds_and_unknown_keys_are_refused_by_name(
    tmp_path: Path, extra: str, names: str
) -> None:
    path = write(tmp_path, extra + BASE)
    with pytest.raises(ConfigError) as refused:
        load(path)
    message = str(refused.value)
    assert names in message
    assert "abc-123" not in message
    assert "Bad Id" not in message


def test_a_pasted_invite_value_is_never_echoed(tmp_path: Path) -> None:
    path = write(tmp_path, f'invite_env = "{PLANTED}"\n' + BASE)
    with pytest.raises(ConfigError) as refused:
        load(path)
    assert PLANTED not in str(refused.value)
    assert "invite_env" in str(refused.value)


def test_a_store_inside_a_world_directory_is_refused(tmp_path: Path) -> None:
    world = tmp_path / "world"
    world.mkdir()
    (world / "world.yaml").write_text("id: x\n", encoding="utf-8")
    path = write(tmp_path, 'store = "world/state"\n' + BASE)
    with pytest.raises(ConfigError, match="store directory"):
        load(path)


def test_relative_store_and_decisions_are_resolved_against_the_file(tmp_path: Path) -> None:
    path = write(tmp_path, 'store = "state"\ndecisions_json = "seats.json"\n' + BASE)
    config = load(path)
    assert config.store == str(tmp_path.resolve() / "state")
    assert config.decisions_json == str(tmp_path.resolve() / "seats.json")


def test_a_configuration_without_the_new_keys_still_loads(tmp_path: Path) -> None:
    config = load(write(tmp_path, BASE))
    assert config.invite_env == "MINEWORLD_INVITE"
    assert config.controller.consult_deadline_s == 20
    assert config.seats is None


def _config(data: dict[str, object]) -> CognitionConfig:
    return CognitionConfig.model_validate({"recording": {"mode": "scripted"}, **data})


LAYER_POLICY_CASES: list[tuple[str, dict[str, object], str, str | None, str, str | None]] = [
    # (description, config data, seat, backend, expected policy id, expected first parameter)
    ("bundled default", {}, "alice", None, "fixed", "attempts"),
    (
        "global beats bundled",
        {"retry": {"content": {"policy": "none"}}},
        "alice",
        None,
        "none",
        None,
    ),
    (
        "backend beats global",
        {
            "retry": {"content": {"policy": "none"}},
            "backends": {
                "local": {
                    "base_url": "http://127.0.0.1:1/v1",
                    "model": "m",
                    "retry": {"content": {"policy": "fixed", "parameters": {"attempts": 9}}},
                }
            },
        },
        "alice",
        "local",
        "fixed",
        "attempts",
    ),
    (
        "seat beats backend and global",
        {
            "retry": {"content": {"policy": "none"}},
            "backends": {
                "local": {
                    "base_url": "http://127.0.0.1:1/v1",
                    "model": "m",
                    "retry": {"content": {"policy": "fixed", "parameters": {"attempts": 9}}},
                }
            },
            "seat_overrides": {
                "alice": {"retry": {"content": {"policy": "fixed", "parameters": {"attempts": 2}}}}
            },
        },
        "alice",
        "local",
        "fixed",
        "attempts",
    ),
]


@pytest.mark.parametrize(
    ("description", "data", "seat", "backend", "policy", "parameter"),
    LAYER_POLICY_CASES,
    ids=[case[0] for case in LAYER_POLICY_CASES],
)
def test_layer_precedence_is_seat_then_backend_then_global_then_bundled(
    description: str,
    data: dict[str, object],
    seat: str,
    backend: str | None,
    policy: str,
    parameter: str | None,
) -> None:
    chosen = layer_policy(_config(data), seat, backend, "content")
    assert chosen.policy == policy
    if parameter is not None:
        assert parameter in chosen.parameters
    if description == "bundled default":
        assert chosen == BUNDLED_RETRY["content"]


def test_a_seat_override_changes_only_its_own_bounds() -> None:
    config = _config(
        {
            "controller": {"recall_episodes": 8},
            "seat_overrides": {"alice": {"controller": {"recall_episodes": 6}}},
        }
    )
    assert controller_for(config, "alice").recall_episodes == 6
    assert controller_for(config, "bob").recall_episodes == 8
    assert controller_for(config, "alice").max_calls == 5
