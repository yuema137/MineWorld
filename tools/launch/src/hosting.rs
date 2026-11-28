//! Child-process policy for a played town versus the explicitly scripted smoke fixture.
//! The capability stays in memory and child environments; it is never logged or persisted.

use std::process::Command;

use crate::Failure;

pub enum Hosting {
    Played { admin: String },
    Smoke,
}

impl Hosting {
    pub fn new(smoke: bool) -> Result<Self, Failure> {
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
        })
    }

    pub fn is_smoke(&self) -> bool {
        matches!(self, Self::Smoke)
    }

    pub fn configure_server(&self, command: &mut Command) {
        match self {
            Self::Played { .. } => {
                command.arg("--town");
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
            Self::Played { admin } => {
                command.env("MINEWORLD_ADMIN_TOKEN", admin);
            }
            Self::Smoke => {
                command.env_remove("MINEWORLD_ADMIN_TOKEN");
            }
        }
    }
}
