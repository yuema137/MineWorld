//! What a mode plays: its defaults (`LAUNCHER.md` §2) and the optional `runtime/launch.toml` that
//! overrides them (§6). The file's grammar is a strict subset of TOML, read here rather than by a TOML
//! library: three sections of quoted strings do not justify a dependency (`ARC-78`). The per-user
//! cognition profile (`LAUNCHER.md` §10) is read with the same grammar and four more keys.

use std::fmt;
use std::path::{Component, Path, PathBuf};

use crate::Mode;
use crate::bundle::Client;

/// A world: one folder name under `runtime/worlds/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldName(String);

impl WorldName {
    pub fn new(name: &str) -> Result<Self, String> {
        let plain = !name.is_empty()
            && name != "."
            && name != ".."
            && !name.contains(['/', '\\', ':'])
            && !name.chars().any(char::is_control);
        if plain {
            Ok(Self(name.to_owned()))
        } else {
            Err(format!(
                "\"{name}\" is not a world folder name (one name under runtime/worlds/)"
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A seat, passed to a client as `--seat=`. The server, not the launcher, decides whether the world
/// offers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seat(String);

impl Seat {
    pub fn new(name: &str) -> Result<Self, String> {
        if name.is_empty() || name.chars().any(|c| c.is_whitespace() || c.is_control()) {
            Err(format!("\"{name}\" is not a seat name"))
        } else {
            Ok(Self(name.to_owned()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What one run plays: a world, and each client with its seat.
#[derive(Debug, PartialEq, Eq)]
pub struct Choice {
    pub world: WorldName,
    pub clients: Vec<(Client, Seat)>,
}

/// The four cognition keys of one profile section (`LAUNCHER.md` §10.1): all given, or none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CognitionKeys {
    /// `cognition-python`: the interpreter, absolute, run directly.
    pub python: PathBuf,
    /// `cognition-config`: the operator's cognition configuration, absolute; opaque to the launcher.
    pub config: PathBuf,
    /// `cognition-manifest`: the runtime manifest `--preflight` checks, absolute.
    pub manifest: PathBuf,
    /// `cognition-decisions`: the decision list, relative to the bundle root, plain components only.
    pub decisions: PathBuf,
}

/// Which file is read: `launch.toml` takes world and seats; the profile also the cognition keys.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Grammar {
    LaunchToml,
    Profile,
}

/// One section of `launch.toml` or of the profile, as given.
#[derive(Default)]
struct Section {
    world: Option<WorldName>,
    seat: Option<Seat>,
    seat_2d: Option<Seat>,
    seat_3d: Option<Seat>,
    python: Option<PathBuf>,
    config: Option<PathBuf>,
    manifest: Option<PathBuf>,
    decisions: Option<PathBuf>,
}

/// What `mode` plays: its defaults, overridden by `overrides` (the text of `launch.toml`, named `file`
/// in errors) when there is one.
pub fn choice(mode: Mode, overrides: Option<(&str, &str)>) -> Result<Choice, String> {
    let section = match overrides {
        Some((file, text)) => parse(file, text, mode, Grammar::LaunchToml)?,
        None => Section::default(),
    };
    Ok(applied(mode, section))
}

/// What `mode` plays under the per-user cognition profile (the text of `profile.toml`, named `file` in
/// errors), which replaces `launch.toml` for every mode: the section's world and seats over the
/// defaults, and its cognition keys — `None` when it gives none, and conversation is off for the mode.
pub fn profile(
    mode: Mode,
    file: &str,
    text: &str,
) -> Result<(Choice, Option<CognitionKeys>), String> {
    let mut section = parse(file, text, mode, Grammar::Profile)?;
    let keys = match (
        section.python.take(),
        section.config.take(),
        section.manifest.take(),
        section.decisions.take(),
    ) {
        (Some(python), Some(config), Some(manifest), Some(decisions)) => Some(CognitionKeys {
            python,
            config,
            manifest,
            decisions,
        }),
        (None, None, None, None) => None,
        _ => {
            return Err(format!(
                "{file}: the [{}] section gives some of cognition-python, cognition-config, \
                 cognition-manifest and cognition-decisions; give all four, or none",
                section_name(mode)
            ));
        }
    };
    Ok((applied(mode, section), keys))
}

fn section_name(mode: Mode) -> &'static str {
    match mode {
        Mode::TwoD => "2d",
        Mode::ThreeD => "3d",
        Mode::Both => "both",
    }
}

/// The defaults of `mode`, with what `section` gives in their place.
fn applied(mode: Mode, section: Section) -> Choice {
    let seat = |given: Option<Seat>, default: &str| {
        given.unwrap_or_else(|| Seat::new(default).expect("a default seat is a seat"))
    };
    let world = |given: Option<WorldName>, default: &str| {
        given.unwrap_or_else(|| WorldName::new(default).expect("a default world is a world"))
    };
    // Each mode plays its default world as a player who is no resident (`pr-launcher-solo.md` Q-LS1, Q-LS6):
    // `visitor`, and in Both `AC-15`'s second player `wanderer`. `launch.toml` may name another world or
    // seat, such as `seat = "carol"` to keep playing a save that was played as Carol.
    match mode {
        Mode::TwoD => Choice {
            world: world(section.world, PLAYED_WORLD),
            clients: vec![(Client::TwoD, seat(section.seat, "visitor"))],
        },
        Mode::ThreeD => Choice {
            world: world(section.world, WORLD_3D),
            clients: vec![(Client::ThreeD, seat(section.seat, "visitor"))],
        },
        Mode::Both => Choice {
            world: world(section.world, WORLD_3D),
            clients: vec![
                (Client::TwoD, seat(section.seat_2d, "visitor")),
                (Client::ThreeD, seat(section.seat_3d, "wanderer")),
            ],
        },
    }
}

/// The world 2D plays unless `launch.toml` names another (Q-LS1).
const PLAYED_WORLD: &str = "market-town";

/// The world 3D and Both play unless `launch.toml` names another. **The Q-LS7 gate flips this one line**
/// to `PLAYED_WORLD` once the packaged 3D Market Town timing passes (`pr-launcher-solo.md` Q-LS1, Q-LS7;
/// `LAUNCHER.md` §2). Until then 3D keeps the world its packaged startup was measured on.
const WORLD_3D: &str = "social-cafe";

/// What a `--smoke` run of `mode` plays: a fixed fixture, never `launch.toml` (Q-LS5). These are the
/// worlds and seats the probes' verdicts were recorded against (`scripts/package.py`), so a player's
/// choice of world can neither break nor mask a bundle check.
pub fn smoke(mode: Mode) -> Choice {
    let pinned = |world: &str, clients: &[(Client, &str)]| Choice {
        world: WorldName::new(world).expect("a fixture world is a world"),
        clients: clients
            .iter()
            .map(|(client, seat)| (*client, Seat::new(seat).expect("a fixture seat is a seat")))
            .collect(),
    };
    match mode {
        Mode::TwoD => pinned("market-town", &[(Client::TwoD, "carol")]),
        Mode::ThreeD => pinned("social-cafe", &[(Client::ThreeD, "visitor")]),
        Mode::Both => pinned(
            "social-cafe",
            &[(Client::TwoD, "visitor"), (Client::ThreeD, "wanderer")],
        ),
    }
}

/// Reads the whole file, refusing anything outside the grammar, and returns the section of `mode`.
fn parse(file: &str, text: &str, mode: Mode, grammar: Grammar) -> Result<Section, String> {
    let mut sections: [(Mode, Section); 3] = [
        (Mode::TwoD, Section::default()),
        (Mode::ThreeD, Section::default()),
        (Mode::Both, Section::default()),
    ];
    let mut current: Option<Mode> = None;
    for (index, raw) in text.lines().enumerate() {
        let at = |what: String| format!("{file}, line {}: {what}", index + 1);
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            current = Some(match name.trim() {
                "2d" => Mode::TwoD,
                "3d" => Mode::ThreeD,
                "both" => Mode::Both,
                other => {
                    return Err(at(format!(
                        "unknown section [{other}]; expected [2d], [3d] or [both]"
                    )));
                }
            });
            continue;
        }
        let Some(mode) = current else {
            return Err(at(
                "a setting before any section; begin with [2d], [3d] or [both]".to_owned(),
            ));
        };
        let (key, value) = pair(line).map_err(at)?;
        let section = &mut sections
            .iter_mut()
            .find(|(of, _)| *of == mode)
            .expect("every mode has a section")
            .1;
        set(section, mode, grammar, key, value).map_err(at)?;
    }
    let [two_d, three_d, both] = sections;
    Ok(match mode {
        Mode::TwoD => two_d.1,
        Mode::ThreeD => three_d.1,
        Mode::Both => both.1,
    })
}

/// `key = "value"`, with no escapes and no other form of value.
fn pair(line: &str) -> Result<(&str, &str), String> {
    let (key, value) = line
        .split_once('=')
        .ok_or_else(|| format!("expected key = \"value\", found {line:?}"))?;
    let value = value.trim();
    let quoted = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .filter(|inner| !inner.contains('"'))
        .ok_or_else(|| format!("the value must be one double-quoted string, found {value}"))?;
    Ok((key.trim(), quoted))
}

fn set(
    section: &mut Section,
    mode: Mode,
    grammar: Grammar,
    key: &str,
    value: &str,
) -> Result<(), String> {
    let twice = || format!("{key} is given twice in this section");
    if grammar == Grammar::Profile {
        let slot = match key {
            "cognition-python" => Some((&mut section.python, true)),
            "cognition-config" => Some((&mut section.config, true)),
            "cognition-manifest" => Some((&mut section.manifest, true)),
            "cognition-decisions" => Some((&mut section.decisions, false)),
            _ => None,
        };
        if let Some((slot, absolute)) = slot {
            return replace(slot, cognition_path(key, value, absolute)?).map_err(|()| twice());
        }
    }
    match (mode, key) {
        (_, "world") => replace(&mut section.world, WorldName::new(value)?).map_err(|()| twice()),
        (Mode::TwoD | Mode::ThreeD, "seat") => {
            replace(&mut section.seat, Seat::new(value)?).map_err(|()| twice())
        }
        (Mode::Both, "seat-2d") => {
            replace(&mut section.seat_2d, Seat::new(value)?).map_err(|()| twice())
        }
        (Mode::Both, "seat-3d") => {
            replace(&mut section.seat_3d, Seat::new(value)?).map_err(|()| twice())
        }
        (Mode::Both, other) => Err(format!(
            "unknown key {other}; [both] takes world, seat-2d and seat-3d{}",
            cognition_keys(grammar)
        )),
        (_, other) => Err(format!(
            "unknown key {other}; this section takes world and seat{}",
            cognition_keys(grammar)
        )),
    }
}

fn cognition_keys(grammar: Grammar) -> &'static str {
    match grammar {
        Grammar::LaunchToml => "",
        Grammar::Profile => {
            ", and cognition-python, cognition-config, cognition-manifest, cognition-decisions"
        }
    }
}

/// A cognition key's path: absolute for the interpreter, the configuration and the manifest; for the
/// decision list, relative to the bundle root with plain components only, so it names nothing outside it.
fn cognition_path(key: &str, value: &str, absolute: bool) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    if absolute {
        return if path.is_absolute() {
            Ok(path)
        } else {
            Err(format!("{key} must be an absolute path, found \"{value}\""))
        };
    }
    let plain = !value.is_empty()
        && Path::new(value)
            .components()
            .all(|part| matches!(part, Component::Normal(_)));
    if plain {
        Ok(path)
    } else {
        Err(format!(
            "{key} must be a path inside the bundle, relative to its root, found \"{value}\""
        ))
    }
}

fn replace<T>(slot: &mut Option<T>, value: T) -> Result<(), ()> {
    if slot.is_some() {
        return Err(());
    }
    *slot = Some(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seat(name: &str) -> Seat {
        Seat::new(name).expect("a seat")
    }

    #[test]
    fn the_file_overrides_only_its_own_modes_section() {
        let text = "# a player's edits\n\n[2d]\nworld = \"social-cafe\"\n  seat=\"visitor\"  \n\
                    [both]\nseat-3d = \"bob\"\n";
        let file = Some(("launch.toml", text));

        let two_d = choice(Mode::TwoD, file).expect("valid");
        assert_eq!(two_d.world.as_str(), "social-cafe");
        assert_eq!(two_d.clients, vec![(Client::TwoD, seat("visitor"))]);

        let three_d = choice(Mode::ThreeD, file).expect("valid");
        assert_eq!(three_d.world.as_str(), WORLD_3D);
        assert_eq!(three_d.clients, vec![(Client::ThreeD, seat("visitor"))]);

        let both = choice(Mode::Both, file).expect("valid");
        assert_eq!(
            both.clients,
            vec![
                (Client::TwoD, seat("visitor")),
                (Client::ThreeD, seat("bob"))
            ]
        );
        assert_eq!(
            choice(Mode::TwoD, None).expect("defaults").world.as_str(),
            "market-town"
        );
    }

    #[test]
    fn every_mode_plays_its_default_world_as_a_visitor_and_smoke_keeps_its_fixtures() {
        for (mode, default, seats) in [
            (
                Mode::TwoD,
                PLAYED_WORLD,
                vec![(Client::TwoD, seat("visitor"))],
            ),
            (
                Mode::ThreeD,
                WORLD_3D,
                vec![(Client::ThreeD, seat("visitor"))],
            ),
            (
                Mode::Both,
                WORLD_3D,
                vec![
                    (Client::TwoD, seat("visitor")),
                    (Client::ThreeD, seat("wanderer")),
                ],
            ),
        ] {
            let played = choice(mode, None).expect("defaults");
            assert_eq!(played.world.as_str(), default);
            assert_eq!(played.clients, seats);
        }
        // The fixtures the bundle's probes were recorded against, whatever the played defaults are.
        assert_eq!(
            smoke(Mode::TwoD),
            Choice {
                world: WorldName::new("market-town").expect("a world"),
                clients: vec![(Client::TwoD, seat("carol"))]
            }
        );
        assert_eq!(smoke(Mode::ThreeD).world.as_str(), "social-cafe");
        assert_eq!(
            smoke(Mode::Both).clients,
            vec![
                (Client::TwoD, seat("visitor")),
                (Client::ThreeD, seat("wanderer"))
            ]
        );
    }

    #[test]
    fn anything_outside_the_grammar_is_refused_with_its_line() {
        for (text, line, says) in [
            ("world = \"x\"\n", 1, "before any section"),
            ("[4d]\n", 1, "unknown section [4d]"),
            ("[2d]\nseat-2d = \"x\"\n", 2, "unknown key seat-2d"),
            ("[both]\nseat = \"x\"\n", 2, "unknown key seat"),
            ("[2d]\nworld = 'x'\n", 2, "double-quoted"),
            ("[2d]\nworld = \"a\" # note\n", 2, "double-quoted"),
            ("[2d]\nworld\n", 2, "expected key"),
            ("[2d]\nworld = \"../etc\"\n", 2, "not a world folder name"),
            ("[3d]\nseat = \"two words\"\n", 2, "not a seat name"),
            (
                "[3d]\nseat = \"a\"\n\n[3d]\nseat = \"b\"\n",
                5,
                "given twice",
            ),
        ] {
            // The error is raised whichever mode is asked for: the whole file is checked.
            let refusal = choice(Mode::TwoD, Some(("launch.toml", text)))
                .expect_err(&format!("refused: {text:?}"));
            assert!(
                refusal.starts_with(&format!("launch.toml, line {line}: "))
                    && refusal.contains(says),
                "{text:?} → {refusal}"
            );
        }
    }

    #[test]
    fn the_profile_takes_all_four_cognition_keys_or_none_and_launch_toml_takes_none() {
        let (python, config, manifest) = if cfg!(windows) {
            (
                "C:\\py\\python.exe",
                "C:\\c\\cognition.toml",
                "C:\\c\\m.json",
            )
        } else {
            ("/py/bin/python", "/c/cognition.toml", "/c/m.json")
        };
        let text = format!(
            "[2d]\nworld = \"market-town\"\nseat = \"visitor\"\ncognition-python = \"{python}\"\n\
             cognition-config = \"{config}\"\ncognition-manifest = \"{manifest}\"\n\
             cognition-decisions = \"runtime/decisions/market-town-alice-lm.yaml\"\n\
             [3d]\nworld = \"market-town\"\n"
        );
        let (two_d, keys) = profile(Mode::TwoD, "profile.toml", &text).expect("valid");
        assert_eq!(two_d.clients, vec![(Client::TwoD, seat("visitor"))]);
        assert_eq!(
            keys,
            Some(CognitionKeys {
                python: PathBuf::from(python),
                config: PathBuf::from(config),
                manifest: PathBuf::from(manifest),
                decisions: PathBuf::from("runtime/decisions/market-town-alice-lm.yaml"),
            })
        );
        // A section without the keys plays without conversation; a mode without a section, its defaults.
        let (three_d, keys) = profile(Mode::ThreeD, "profile.toml", &text).expect("valid");
        assert_eq!((three_d.world.as_str(), keys), ("market-town", None));
        let (both, keys) = profile(Mode::Both, "profile.toml", &text).expect("valid");
        assert_eq!((both.world.as_str(), keys), ("social-cafe", None));

        for (text, says) in [
            (
                format!("[2d]\ncognition-python = \"{python}\"\n"),
                "give all four, or none",
            ),
            (
                "[2d]\ncognition-python = \"python3\"\n".to_owned(),
                "must be an absolute path",
            ),
            (
                format!(
                    "[2d]\ncognition-python = \"{python}\"\ncognition-config = \"{config}\"\n\
                     cognition-manifest = \"{manifest}\"\ncognition-decisions = \"../x.yaml\"\n"
                ),
                "inside the bundle",
            ),
            (
                format!("[2d]\ncognition-config = \"{config}\"\ncognition-config = \"{config}\"\n"),
                "given twice",
            ),
            ("[2d]\ncognition-model = \"x\"\n".to_owned(), "unknown key"),
        ] {
            let refusal = profile(Mode::TwoD, "profile.toml", &text)
                .expect_err(&format!("refused: {text:?}"));
            assert!(
                refusal.starts_with("profile.toml") && refusal.contains(says),
                "{text:?} → {refusal}"
            );
        }
        let refusal = choice(
            Mode::TwoD,
            Some((
                "launch.toml",
                &format!("[2d]\ncognition-python = \"{python}\"\n"),
            )),
        )
        .expect_err("launch.toml refuses the profile's keys");
        assert!(
            refusal.contains("unknown key cognition-python"),
            "{refusal}"
        );
    }
}
