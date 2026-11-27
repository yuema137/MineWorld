//! Substitutes for the guide's operator-only blocks in the cold-start run (`docs/GUIDE_FORMAT.md` §4.7 rule
//! 11; pr-25g §6). Each runs after the guide phase, with the reader's binary, in the folder the page left, and
//! is reported under its own name: a substitute never turns a page's operator-only block into a pass. None
//! starts a language model (INV-S25-6).
//!
//! - 06.2, 14.2, 14.10: the server, started with the page's own command line and matched against the page's
//!   own lines (the same `...` and `[..]` rules as every console block), then stopped as an operator does.
//! - 06.3 (Linux and macOS): the 2D client's `--drive=walk`, headless, joined to that server as Ben.
//! - 14.3: the page's own shell block (`uv sync`, `uv run mineworld-agent --help`) and its console block.
//! - 14.4–14.7: the recorded agent session replayed through the real MCP adapter against a real server
//!   (`clients/agent/tests/test_agent_session.py`).

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use mineworld_test_support::process::{InterruptibleChild, interrupt, interruptible, kill};
use serde_json::json;

use super::cold_start::{Config, Layer};
use super::command::{self, Rules};
use super::page::{self, Body, Directive, Family};
use super::page_run;

/// How long a server may take to print the lines the page shows.
const SERVE_WITHIN: Duration = Duration::from_secs(120);

/// How long the 2D client's drive or the agent session may take.
const DRIVE_WITHIN: Duration = Duration::from_secs(900);

/// One substitute's outcome.
struct Outcome {
    name: String,
    seconds: f64,
    verdict: &'static str,
    detail: String,
}

/// Runs every substitute this layer and system have, writes `report/substitutes.json`, and fails when one
/// failed.
pub fn run(config: &Config) -> Result<(), String> {
    let mut outcomes = Vec::new();
    chapter_06(config, &mut outcomes);
    if config.layer == Layer::A {
        chapter_14(config, &mut outcomes);
    }
    let listed: Vec<_> = outcomes
        .iter()
        .map(|outcome| {
            json!({
                "name": outcome.name,
                "seconds": outcome.seconds,
                "verdict": outcome.verdict,
                "detail": outcome.detail,
            })
        })
        .collect();
    let path = config.report.join("substitutes.json");
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&listed).unwrap_or_default(),
    )
    .map_err(|error| format!("{} cannot be written: {error}", path.display()))?;
    let failed: Vec<String> = outcomes
        .iter()
        .filter(|outcome| outcome.verdict == "FAIL")
        .map(|outcome| format!("{}:\n{}", outcome.name, outcome.detail))
        .collect();
    for outcome in &outcomes {
        eprintln!(
            "{}  {}  {:.1}s",
            outcome.verdict, outcome.name, outcome.seconds
        );
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("substitutes failed:\n{}", failed.join("\n")))
    }
}

/// Times `work` and records it; `Ok(detail)` is a pass.
fn timed(outcomes: &mut Vec<Outcome>, name: &str, work: impl FnOnce() -> Result<String, String>) {
    let began = Instant::now();
    let (verdict, detail) = match work() {
        Ok(detail) => ("PASS", detail),
        Err(detail) => ("FAIL", detail),
    };
    outcomes.push(Outcome {
        name: name.to_owned(),
        seconds: began.elapsed().as_secs_f64(),
        verdict,
        detail,
    });
}

fn not_covered(outcomes: &mut Vec<Outcome>, name: &str, why: &str) {
    outcomes.push(Outcome {
        name: name.to_owned(),
        seconds: 0.0,
        verdict: "NOT COVERED",
        detail: why.to_owned(),
    });
}

fn chapter_06(config: &Config, outcomes: &mut Vec<Outcome>) {
    let folder = config.scratch_root.join("after-06-play-it");
    let mut server = None;
    timed(outcomes, "substitute 06.2 serve", || {
        let block = operator_block(&config.guide, "06-play-it", "Step 6.2")?;
        let started = Server::start(&config.binary, &folder, &block)?;
        let detail = started.output.clone();
        server = Some(started);
        Ok(detail)
    });
    match (&server, config.layer, &config.godot) {
        (Some(running), Layer::A, Some(godot)) if !cfg!(windows) => {
            let invite = running.invite.clone();
            let address = running.address.clone();
            timed(outcomes, "substitute 06.3 2D --drive=walk", || {
                drive(config, godot, &address, &invite)
            });
        }
        (_, Layer::A, _) if cfg!(windows) => not_covered(
            outcomes,
            "06.3 on Windows",
            "mineworld-2d is a bash script; chapter 06 sends Windows readers to clients/2d/README.md",
        ),
        (_, Layer::B, _) => not_covered(
            outcomes,
            "06.3 from the bundle",
            "chapter 06's download branch (R-d) does not yet say how a bundle reader joins",
        ),
        (None, _, _) => not_covered(outcomes, "06.3", "the 06.2 server did not start"),
        (Some(_), Layer::A, None) => not_covered(outcomes, "06.3", "no Godot was installed for it"),
        _ => {}
    }
    if let Some(running) = server {
        timed(outcomes, "06.2 stop (Ctrl+C)", || running.stop());
    }
}

fn chapter_14(config: &Config, outcomes: &mut Vec<Outcome>) {
    let folder = config.scratch_root.join("after-14-play-as-an-agent");
    let clone = config.clone.clone().unwrap_or_default();
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .map(PathBuf::from)
        .unwrap_or_default();
    let mut first_grant = None;
    timed(outcomes, "substitute 14.2 serve with a grant", || {
        let block = operator_block(&config.guide, "14-play-as-an-agent", "Step 14.2")?;
        let server = Server::start(&config.binary, &folder, &block)?;
        first_grant = Some(server.grant.clone());
        let detail = server.output.clone();
        server.stop()?;
        Ok(detail)
    });
    timed(
        outcomes,
        "substitute 14.3 install the agent adapter",
        || {
            let family = if cfg!(windows) {
                Family::PowerShell
            } else {
                Family::Posix
            };
            let shell = operator_shell(&config.guide, "14-play-as-an-agent", "Step 14.3", family)?;
            command::shell(&folder, &home, &clone, family, &shell, Rules::ColdStart)?;
            let version = operator_block(&config.guide, "14-play-as-an-agent", "Step 14.3")?;
            console_matches(&config.binary, &folder, &version)?;
            Ok("uv sync; uv run mineworld-agent --help; mineworld --version matched".to_owned())
        },
    );
    timed(
        outcomes,
        "substitute 14.4–14.7 recorded MCP session",
        || agent_session(config, &clone, &home),
    );
    timed(
        outcomes,
        "substitute 14.10 restart gives a new grant",
        || {
            let block = operator_block(&config.guide, "14-play-as-an-agent", "Step 14.10")?;
            let server = Server::start(&config.binary, &folder, &block)?;
            let second = server.grant.clone();
            server.stop()?;
            match first_grant {
                Some(first) if !first.is_empty() && first != second => Ok(format!(
                    "two starts, two grants ({} chars each)",
                    second.len()
                )),
                _ => Err(format!("the restart's grant {second:?} is not new")),
            }
        },
    );
    not_covered(
        outcomes,
        "14.8 a Python program",
        "needs a hand-made file and a grant (pr-25g §6.2)",
    );
    not_covered(
        outcomes,
        "14.9 mixed play",
        "needs the 2D client and a person",
    );
    not_covered(outcomes, "any live model", "never in CI (INV-S25-6)");
}

/// A console block of `page`'s step whose title starts with `step`, marked operator-only: its command and
/// expected lines.
fn operator_block(guide: &Path, name: &str, step: &str) -> Result<Vec<String>, String> {
    let parsed = read_page(guide, name)?;
    parsed
        .blocks
        .iter()
        .filter(|block| {
            block
                .step
                .as_deref()
                .is_some_and(|title| title.starts_with(step))
        })
        .filter(|block| block.directives.contains(&Directive::OperatorOnly))
        .find_map(|block| match &block.body {
            Body::Console { lines, .. } => Some(lines.clone()),
            _ => None,
        })
        .ok_or_else(|| format!("{name} has no operator-only console block in {step}"))
}

/// The operator-only shell block of `family` in `page`'s step `step`.
fn operator_shell(guide: &Path, name: &str, step: &str, family: Family) -> Result<String, String> {
    let parsed = read_page(guide, name)?;
    parsed
        .blocks
        .iter()
        .filter(|block| {
            block
                .step
                .as_deref()
                .is_some_and(|title| title.starts_with(step))
        })
        .find_map(|block| match &block.body {
            Body::Shell {
                family: shell,
                text,
            } if *shell == family => Some(text.clone()),
            _ => None,
        })
        .ok_or_else(|| format!("{name} has no {} block in {step}", family.word()))
}

fn read_page(guide: &Path, name: &str) -> Result<page::Page, String> {
    let path = guide.join(format!("{name}.md"));
    let source = std::fs::read_to_string(&path)
        .map_err(|error| format!("{} cannot be read: {error}", path.display()))?;
    page::parse(&source)
}

/// Runs a console block's commands and matches their output, as the runner does for any console block.
fn console_matches(binary: &Path, folder: &Path, lines: &[String]) -> Result<(), String> {
    for (line, expected) in page_run::commands(lines) {
        let words = command::words(&line)?;
        let (actual, code) = command::run(binary, folder, &words)?;
        if code != 0 {
            return Err(format!("`{line}` exited with {code}:\n{actual}"));
        }
        command::matches(&expected, &actual)
            .map_err(|difference| format!("`{line}` printed other output:\n{difference}"))?;
    }
    Ok(())
}

/// A server started from a page's block, its first lines matched, its codes read.
struct Server {
    child: InterruptibleChild,
    lines: Receiver<String>,
    output: String,
    address: String,
    invite: String,
    grant: String,
}

impl Server {
    /// Starts the block's one command in `folder` and reads its output until it matches the page's lines.
    fn start(binary: &Path, folder: &Path, block: &[String]) -> Result<Self, String> {
        let pairs = page_run::commands(block);
        let [(line, expected)] = pairs.as_slice() else {
            return Err(format!("the block holds {} commands, not one", pairs.len()));
        };
        let words = command::words(line)?;
        let (reader, writer) = std::io::pipe().map_err(|error| error.to_string())?;
        let mut command = Command::new(binary);
        command
            .args(&words[1..])
            .current_dir(folder)
            .stdin(Stdio::null())
            .stdout(writer.try_clone().map_err(|error| error.to_string())?)
            .stderr(writer);
        let mut child = interruptible(command)
            .spawn()
            .map_err(|error| format!("the server cannot be started: {error}"))?;
        let lines = read_lines(reader);
        let deadline = Instant::now() + SERVE_WITHIN;
        let mut output = String::new();
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match lines.recv_timeout(left) {
                Ok(next) => {
                    output.push_str(&next);
                    output.push('\n');
                    if command::matches(expected, &output).is_ok() {
                        break;
                    }
                }
                Err(_) => {
                    let _ = kill(&mut child);
                    let difference = command::matches(expected, &output)
                        .err()
                        .unwrap_or_default();
                    return Err(format!(
                        "`{line}` did not print the page's lines within {}s:\n{difference}",
                        SERVE_WITHIN.as_secs()
                    ));
                }
            }
        }
        let join = output
            .lines()
            .find(|text| text.contains("join with:"))
            .unwrap_or_default();
        let address = join
            .split("join with:")
            .nth(1)
            .and_then(|rest| rest.split_whitespace().next())
            .unwrap_or_default()
            .to_owned();
        let code_after = |line: &str| {
            line.rsplit("invite=")
                .next()
                .unwrap_or_default()
                .trim()
                .to_owned()
        };
        let invite = output
            .lines()
            .find(|text| text.starts_with("[mineworld] invite "))
            .map(code_after)
            .unwrap_or_default();
        let grant = output
            .lines()
            .find(|text| text.starts_with("[mineworld] grant "))
            .map(code_after)
            .unwrap_or_default();
        Ok(Self {
            child,
            lines,
            output,
            address,
            invite,
            grant,
        })
    }

    /// Stops the server as an operator does (Ctrl+C; Ctrl-Break on Windows) and reports how it ended and
    /// what it printed while stopping.
    fn stop(mut self) -> Result<String, String> {
        let status = match interrupt(&mut self.child) {
            Ok(status) => format!("{status}"),
            Err(error) => {
                let killed = kill(&mut self.child);
                format!("interrupt failed ({error}); killed: {:?}", killed.status())
            }
        };
        let rest: Vec<String> = self.lines.try_iter().collect();
        Ok(format!(
            "stopped with {status}; last lines:\n{}",
            rest.iter()
                .rev()
                .take(6)
                .rev()
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

/// Reads `reader` line by line on a thread of its own, so a child never blocks on a full pipe.
fn read_lines(reader: std::io::PipeReader) -> Receiver<String> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            let Ok(line) = line else { break };
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    receiver
}

/// Runs `command` to its end or until `within` has passed, returning what it printed and its exit code.
fn bounded(mut command: Command, within: Duration) -> Result<(String, i32), String> {
    let (reader, writer) = std::io::pipe().map_err(|error| error.to_string())?;
    command
        .stdin(Stdio::null())
        .stdout(writer.try_clone().map_err(|error| error.to_string())?)
        .stderr(writer);
    let mut child = command
        .spawn()
        .map_err(|error| format!("{command:?} cannot be started: {error}"))?;
    drop(command);
    let lines = read_lines(reader);
    let deadline = Instant::now() + within;
    let mut output = String::new();
    loop {
        output.extend(lines.try_iter().map(|line| line + "\n"));
        match child.try_wait() {
            Ok(Some(status)) => {
                std::thread::sleep(Duration::from_millis(200));
                output.extend(lines.try_iter().map(|line| line + "\n"));
                return Ok((output, status.code().unwrap_or(-1)));
            }
            Ok(None) if Instant::now() > deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "still running after {}s:\n{output}",
                    within.as_secs()
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(500)),
            Err(error) => return Err(error.to_string()),
        }
    }
}

/// 06.3: the page's `./mineworld-2d --server … --invite … --seat ben`, plus `--drive=walk`, in the clone.
fn drive(config: &Config, godot: &Path, address: &str, invite: &str) -> Result<String, String> {
    let clone = config.clone.as_deref().ok_or("layer a names its clone")?;
    if address.is_empty() || invite.is_empty() {
        return Err("the 06.2 server printed no join line with an invite".to_owned());
    }
    let mut command = Command::new("bash");
    command
        .arg("./mineworld-2d")
        .args([
            "--server",
            address,
            "--invite",
            invite,
            "--seat",
            "ben",
            "--drive=walk",
        ])
        .current_dir(clone)
        .env("GODOT", godot);
    let (output, code) = bounded(command, DRIVE_WITHIN)?;
    if code == 0
        && output
            .lines()
            .any(|line| line.trim() == "drive complete: PASS")
    {
        Ok(output
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n"))
    } else {
        Err(format!(
            "mineworld-2d --drive=walk exited with {code}:\n{output}"
        ))
    }
}

/// 14.4–14.7: the recorded session replayed through the real MCP adapter, with the reader's binary.
fn agent_session(config: &Config, clone: &Path, home: &Path) -> Result<String, String> {
    let binary = config.binary.to_string_lossy();
    let place = clone.to_string_lossy();
    let (family, text) = if cfg!(windows) {
        (
            Family::PowerShell,
            format!(
                "cd \"{place}\"\n$env:MINEWORLD_BIN = \"{binary}\"\n\
                 uv run --locked pytest clients/agent/tests/test_agent_session.py\n"
            ),
        )
    } else {
        (
            Family::Posix,
            format!(
                "cd \"{place}\"\nMINEWORLD_BIN=\"{binary}\" uv run --locked pytest \
                 clients/agent/tests/test_agent_session.py\n"
            ),
        )
    };
    let scripts = config.scratch_root.join("agent-session");
    std::fs::create_dir_all(&scripts).map_err(|error| error.to_string())?;
    command::shell(&scripts, home, clone, family, &text, Rules::ColdStart)?;
    Ok("clients/agent/tests/test_agent_session.py passed against the reader's binary".to_owned())
}
