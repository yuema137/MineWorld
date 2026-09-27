//! `mineworld-server` — hosts one world and serves the clients connected to it.
//!
//! ```text
//! mineworld-server [--listen ADDRESS]
//! ```
//!
//! One binary for every deployment (`docs/NETWORKING.md` §1): `127.0.0.1:7878` is the default and
//! `--listen 0.0.0.0:7878` is how friends on a LAN reach it. There is no separate single-player
//! mode to choose, because there is no separate single-player path to choose it with.
//!
//! # The world it hosts, today
//!
//! An **empty** world: no systems, no entities, and no seats, which `INV-12` says is a valid
//! MineWorld world and `ENGINEERING_STANDARDS.md` §22 says must keep running with no client
//! attached. It is what this binary can honestly host until a World Pack can be loaded: that is
//! PR 05c, which supplies the world, its systems, its perception and its seat roster to exactly the
//! `WorldHost::spawn` call below, with nothing in this crate changing.
//!
//! So the server is complete and the world is not: `GET /status` answers, `/ws` upgrades, a join is
//! refused because an empty roster offers no seat, and every observation a seat would have received
//! would be empty because a world with no perception system exposes nothing.

use std::net::SocketAddr;
use std::process::ExitCode;

use mineworld_kernel::World;
use mineworld_server::{HostConfig, HostError, HostedWorld, SeatRoster, WorldHost, app};

/// Where the server listens when nothing says otherwise: the local player's own machine.
const DEFAULT_LISTEN: &str = "127.0.0.1:7878";

#[tokio::main]
async fn main() -> ExitCode {
    let listen = match listen_address() {
        Ok(address) => address,
        Err(usage) => {
            eprintln!("{usage}");
            return ExitCode::FAILURE;
        }
    };

    match run(listen).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[server] {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run(listen: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let host = WorldHost::spawn(HostConfig::default(), || {
        // The whole of world assembly, and the seam PR 05c fills: install the World Pack's systems,
        // create its entities, name the seats a client may occupy, and hand over the perception its
        // presence system provides.
        Ok::<_, HostError>(HostedWorld::new(World::new()).seating(SeatRoster::empty()))
    })
    .await?;

    let (listener, address) = app::bind(listen).await?;
    let status = host.status().await?;
    println!(
        "[server] listening on http://{address} (ws://{address}/ws), protocol {}",
        status.protocol
    );
    println!(
        "[server] world: {} entities, {} system(s), {} seat(s) — an empty world until a World Pack \
         can be loaded (PR 05c)",
        status.entities,
        status.systems.len(),
        status.seats.len()
    );

    app::serve_with_shutdown(listener, host.clone(), async {
        let _ = tokio::signal::ctrl_c().await;
        println!("[server] stopping");
    })
    .await?;

    host.shutdown().await;
    Ok(())
}

/// Reads `--listen`, or explains itself.
///
/// Hand-rolled rather than adding an argument parser: this binary has one option, and the CLI proper
/// — `mineworld server <world>`, with `create`, `validate` and `inspect` — is PR 05c's, where the
/// dependency decision belongs with the commands that need it.
fn listen_address() -> Result<SocketAddr, String> {
    let mut arguments = std::env::args().skip(1);
    let mut listen = DEFAULT_LISTEN.to_owned();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--listen" => {
                listen = arguments
                    .next()
                    .ok_or_else(|| "--listen needs an address, such as 0.0.0.0:7878".to_owned())?;
            }
            "--help" | "-h" => {
                return Err(format!(
                    "mineworld-server [--listen ADDRESS]    (default {DEFAULT_LISTEN})"
                ));
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    listen
        .parse()
        .map_err(|error| format!("--listen {listen} is not an address: {error}"))
}
