//! The process helper's contract (step-14 §15.3–15.4, MW-1 and MW-2), on every platform.
//!
//! - A kill's verdict fails closed: a child that ended by itself — with code 0, or with code 1, which
//!   on Windows is also `TerminateProcess`'s code — is never reported killed; a running child that is
//!   killed is.
//! - An interrupt reaches a child spawned through `interruptible` and ends it by the platform's
//!   default when the child handles nothing (Unix: signal 2; Windows: `STATUS_CONTROL_C_EXIT`).
//!
//! The children are this test binary itself, re-executed into `child_entry_point` with a role, so that
//! no platform-specific program (`sleep`, `true`) is needed.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use mineworld_test_support::process::{InterruptibleChild, interrupt, interruptible, kill};

/// Which child to be, when this binary is re-executed as one.
const ROLE: &str = "MINEWORLD_PROCESS_TEST_ROLE";

/// What a sleeping child prints once it is running, so that the parent acts on a live child.
const READY: &str = "process-test-child ready";

/// The child's body. Run as a test of its own (no role), it does nothing.
#[test]
fn child_entry_point() {
    match std::env::var(ROLE).as_deref() {
        Err(_) => {}
        Ok("exit-0") => std::process::exit(0),
        Ok("exit-1") => std::process::exit(1),
        Ok("sleep") => {
            println!("{READY}");
            std::thread::sleep(Duration::from_secs(120));
            // Never reached by these tests: they end the child first. A distinct code, so that a
            // child that outlived its test cannot pass for one that was stopped.
            std::process::exit(3);
        }
        Ok(other) => panic!("{ROLE}={other:?} is not a role this binary knows"),
    }
}

fn child_command(role: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("this test binary"));
    command
        .args([
            "child_entry_point",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(ROLE, role)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    command
}

/// Reads the child's stdout until it says it is running. The harness prints `test <name> ... ` with no
/// newline before the test's own output, so the marker ends a line rather than being one.
fn wait_until_ready(child: &mut Child) {
    let stdout = child.stdout.take().expect("piped stdout");
    let mut lines = BufReader::new(stdout).lines();
    loop {
        match lines.next() {
            Some(Ok(line)) if line.ends_with(READY) => return,
            Some(Ok(_)) => {}
            Some(Err(error)) => panic!("the child's stdout could not be read: {error}"),
            None => panic!("the child closed its stdout before it was running"),
        }
    }
}

#[test]
fn a_child_that_ended_by_itself_is_never_reported_killed() {
    for (role, code) in [("exit-0", 0), ("exit-1", 1)] {
        let mut child = child_command(role).spawn().expect("the child starts");
        let finished = child.wait().expect("the child is reaped");
        assert_eq!(finished.code(), Some(code), "{role}: the child's own exit");
        let killed = kill(&mut child);
        assert!(
            !killed.killed(),
            "{role}: a child that ended by itself, even with TerminateProcess's code 1, is not \
             killed: {killed:?}"
        );
        assert_eq!(killed.status().code(), Some(code), "{role}: {killed:?}");
    }
}

#[test]
fn a_running_child_that_is_killed_is_reported_killed() {
    let mut child = child_command("sleep").spawn().expect("the child starts");
    wait_until_ready(&mut child);
    let killed = kill(&mut child);
    assert!(killed.killed(), "the kill ended the child: {killed:?}");
    assert_ne!(
        killed.status().code(),
        Some(3),
        "the child did not finish its sleep"
    );
}

#[test]
fn an_interrupt_reaches_an_interruptible_child_and_ends_one_that_handles_nothing() {
    let mut child: InterruptibleChild = interruptible(child_command("sleep"))
        .spawn()
        .expect("the child starts");
    wait_until_ready(&mut child);
    let ended = interrupt(&mut child).expect("the interrupt is delivered");
    assert!(
        ended_by_default_interrupt(ended),
        "the platform's default for an unhandled interrupt ended the child: {ended:?}"
    );
}

#[cfg(unix)]
fn ended_by_default_interrupt(status: std::process::ExitStatus) -> bool {
    use std::os::unix::process::ExitStatusExt;
    status.signal() == Some(2)
}

#[cfg(windows)]
fn ended_by_default_interrupt(status: std::process::ExitStatus) -> bool {
    // STATUS_CONTROL_C_EXIT: the default console handler's exit for Ctrl-C and Ctrl-Break.
    status.code().map(|code| code as u32) == Some(0xC000_013A)
}
