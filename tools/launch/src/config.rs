//! What a mode plays: its defaults (`LAUNCHER.md` §2) and the optional `runtime/launch.toml` that
//! overrides them (§6). The file's grammar is a strict subset of TOML, read here rather than by a TOML
//! library: three sections of quoted strings do not justify a dependency (`ARC-78`).

use std::fmt;

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

/// One section of `launch.toml`, as given.
#[derive(Default)]
struct Section {
    world: Option<WorldName>,
    seat: Option<Seat>,
    seat_2d: Option<Seat>,
    seat_3d: Option<Seat>,
}

/// What `mode` plays: its defaults, overridden by `overrides` (the text of `launch.toml`, named `file`
/// in errors) when there is one.
pub fn choice(mode: Mode, overrides: Option<(&str, &str)>) -> Result<Choice, String> {
    let section = match overrides {
        Some((file, text)) => parse(file, text, mode)?,
        None => Section::default(),
    };
    let seat = |given: Option<Seat>, default: &str| {
        given.unwrap_or_else(|| Seat::new(default).expect("a default seat is a seat"))
    };
    let world = |given: Option<WorldName>, default: &str| {
        given.unwrap_or_else(|| WorldName::new(default).expect("a default world is a world"))
    };
    Ok(match mode {
        Mode::TwoD => Choice {
            world: world(section.world, "market-town"),
            clients: vec![(Client::TwoD, seat(section.seat, "carol"))],
        },
        Mode::ThreeD => Choice {
            world: world(section.world, "social-cafe"),
            clients: vec![(Client::ThreeD, seat(section.seat, "visitor"))],
        },
        Mode::Both => Choice {
            world: world(section.world, "social-cafe"),
            clients: vec![
                (Client::TwoD, seat(section.seat_2d, "visitor")),
                (Client::ThreeD, seat(section.seat_3d, "wanderer")),
            ],
        },
    })
}

/// Reads the whole file, refusing anything outside the grammar, and returns the section of `mode`.
fn parse(file: &str, text: &str, mode: Mode) -> Result<Section, String> {
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
        set(section, mode, key, value).map_err(at)?;
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

fn set(section: &mut Section, mode: Mode, key: &str, value: &str) -> Result<(), String> {
    let twice = || format!("{key} is given twice in this section");
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
            "unknown key {other}; [both] takes world, seat-2d and seat-3d"
        )),
        (_, other) => Err(format!(
            "unknown key {other}; this section takes world and seat"
        )),
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
        assert_eq!(three_d.world.as_str(), "social-cafe");
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
}
