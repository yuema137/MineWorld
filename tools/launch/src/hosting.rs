//! Child-process policy for a played town versus the explicitly scripted smoke fixture.
//! The capability stays in memory and child environments; it is never logged or persisted.

use std::process::Command;

use crate::Failure;

pub enum Hosting {
    /// A player's run: town controllers, a private per-run admin capability, and — when one player
    /// plays alone — the single-player declaration `--solo` (`ARC-69` item 5; `pr-launcher-solo.md`
    /// Q-LS2). `--solo` restricts no admission; it only permits what a single-player world may do.
    Played {
        admin: String,
        solo: bool,
    },
    Smoke,
}

impl Hosting {
    pub fn new(smoke: bool, single_player: bool) -> Result<Self, Failure> {
        if smoke {
            return Ok(Self::Smoke);
        }
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes).map_err(|error| {
            Failure::new(format!(
                "cannot obtain randomness for local host controls: {error}"
            ))
        })?;
        Ok(Self::Played {
            admin: bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
            solo: single_player,
        })
    }

    pub fn is_smoke(&self) -> bool {
        matches!(self, Self::Smoke)
    }

    pub fn configure_server(&self, command: &mut Command) {
        match self {
            Self::Played { solo, .. } => {
                command.arg("--town");
                if *solo {
                    command.arg("--solo");
                }
            }
            Self::Smoke => {
                command.args(["--agent", "alice"]);
            }
        }
        self.configure_client(command);
    }

    pub fn configure_client(&self, command: &mut Command) {
        // Never inherit a capability for some unrelated host, including in scripted smoke clients.
        match self {
            Self::Played { admin, .. } => {
                command.env("MINEWORLD_ADMIN_TOKEN", admin);
            }
            Self::Smoke => {
                command.env_remove("MINEWORLD_ADMIN_TOKEN");
            }
        }
    }
}
