//! Ending a child process in a test the same way on every platform (step-14 §15.3–15.4; `DEP-29`
//! note of 2026-10-09).
//!
//! - [`kill`] ends a child with no warning: no shutdown, no checkpoint, no flush. It returns a
//!   [`Killed`], whose [`Killed::killed`] says whether the kill is what ended the child. A test asserts
//!   that verdict, never a raw exit status.
//! - [`interruptible`] and [`interrupt`] stop a child the way an operator's Ctrl-C (Unix) or Ctrl-Break
//!   (Windows) does, so that it takes its graceful path. Only a child spawned through
//!   [`interruptible`] can be interrupted: the type makes the dangerous call impossible.
//!
//! What "killed" means differs by platform, and so does the evidence:
//!
//! | platform | [`kill`]                                   | [`Killed::killed`] reads              |
//! | -------- | ------------------------------------------ | ------------------------------------- |
//! | Unix     | `SIGKILL`                                  | signal 9                              |
//! | Windows  | `TerminateProcess(handle, 1)` (std's code) | exit code 1, and the child was running |
//!
//! On Windows an exit code of 1 alone could be a child that failed by itself, so [`kill`] checks that
//! the child was still running immediately before it kills; every killing test also proves separately
//! that the child had not finished. The residual race — a child exiting with code 1 on its own in the
//! instant between that check and the kill — is an accepted limitation (`DEP-29` note).

use std::fmt;
use std::io;
use std::ops::{Deref, DerefMut};
use std::process::{Child, Command, ExitStatus};

/// How a child ended after [`kill`], and whether the kill is what ended it. Only [`kill`] makes one.
#[derive(Clone, Copy)]
pub struct Killed {
    status: ExitStatus,
    was_running: bool,
}

/// Kills `child` and reaps it.
///
/// First asks whether the child is still running; a child that has already exited is not killed
/// again, and its [`Killed`] says it was not killed. Panics, naming the step, if the platform refuses
/// to check, kill or reap: a test must never take a failed kill for a death.
pub fn kill(child: &mut Child) -> Killed {
    let was_running = child
        .try_wait()
        .expect("the child's state can be read before the kill")
        .is_none();
    if was_running {
        child.kill().expect("the kill is delivered");
    }
    let status = child.wait().expect("the killed child is reaped");
    Killed {
        status,
        was_running,
    }
}

impl Killed {
    /// Whether the kill ended the child: it was running when killed, and its status is this
    /// platform's mark of a kill (signal 9 on Unix; exit code 1, `TerminateProcess`'s, on Windows).
    pub fn killed(&self) -> bool {
        self.was_running && ended_by_kill(self.status)
    }

    /// The child's exit status, as reaped.
    pub fn status(&self) -> ExitStatus {
        self.status
    }
}

impl fmt::Debug for Killed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Killed")
            .field("status", &self.status)
            .field("was_running", &self.was_running)
            .field("killed", &self.killed())
            .finish()
    }
}

#[cfg(unix)]
fn ended_by_kill(status: ExitStatus) -> bool {
    use std::os::unix::process::ExitStatusExt;
    status.signal() == Some(9)
}

#[cfg(windows)]
fn ended_by_kill(status: ExitStatus) -> bool {
    // std's `Child::kill` is `TerminateProcess(handle, 1)`.
    status.code() == Some(1)
}

/// A command that will be spawned so that [`interrupt`] can reach it. Made by [`interruptible`].
#[derive(Debug)]
pub struct Interruptible(Command);

/// Prepares `command` to be interrupted. On Windows the child gets a console process group of its own
/// (`CREATE_NEW_PROCESS_GROUP`), the only way a Ctrl-Break can reach it and nothing else; on Unix
/// nothing changes.
pub fn interruptible(command: Command) -> Interruptible {
    #[cfg(windows)]
    let command = {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        let mut command = command;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP);
        command
    };
    Interruptible(command)
}

impl Interruptible {
    /// Spawns the child.
    pub fn spawn(mut self) -> io::Result<InterruptibleChild> {
        self.0.spawn().map(|child| InterruptibleChild { child })
    }
}

/// A child spawned through [`interruptible`]: a [`Child`] (by `Deref`) that [`interrupt`] accepts.
#[derive(Debug)]
pub struct InterruptibleChild {
    child: Child,
}

impl Deref for InterruptibleChild {
    type Target = Child;

    fn deref(&self) -> &Child {
        &self.child
    }
}

impl DerefMut for InterruptibleChild {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.child
    }
}

/// Stops `child` the way an operator does — `SIGINT` on Unix, Ctrl-Break on Windows — and waits for
/// it, returning how it ended. A child that handles the stop exits by its own graceful path; one that
/// does not is ended by the platform's default (Unix: signal 2; Windows: `STATUS_CONTROL_C_EXIT`,
/// `0xC000013A`).
///
/// Only a child spawned through [`interruptible`] is accepted; a plain [`Child`] does not compile:
///
/// ```compile_fail
/// let mut child = std::process::Command::new("sleep").arg("1").spawn().expect("spawns");
/// let _ = mineworld_test_support::process::interrupt(&mut child);
/// ```
///
/// ```no_run
/// use mineworld_test_support::process::{interrupt, interruptible};
/// let mut command = std::process::Command::new("sleep");
/// command.arg("1");
/// let mut child = interruptible(command).spawn().expect("spawns");
/// let _ = interrupt(&mut child);
/// ```
///
/// An error is returned, never ignored, when the stop cannot be sent — on Windows, naming a missing
/// console, the one case in which Ctrl-Break cannot be delivered at all.
pub fn interrupt(child: &mut InterruptibleChild) -> io::Result<ExitStatus> {
    send_interrupt(child.child.id())?;
    child.child.wait()
}

/// `SIGINT`, through the shell's builtin `kill`: a minimal CI image has a shell and may have no procps,
/// and this crate takes no libc dependency to signal a child (`DEP-29` note).
#[cfg(unix)]
fn send_interrupt(pid: u32) -> io::Result<()> {
    let sent = Command::new("sh")
        .args(["-c", &format!("kill -INT {pid}")])
        .status()?;
    if sent.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "`kill -INT {pid}` failed: {sent}"
        )))
    }
}

/// Ctrl-Break to the child's own process group, whose id is the child's pid (`CREATE_NEW_PROCESS_GROUP`).
#[cfg(windows)]
#[allow(unsafe_code)]
fn send_interrupt(pid: u32) -> io::Result<()> {
    // kernel32, which std links on every Windows target; declared here rather than taken from
    // `windows-sys` (`DEP-29` note, step-14 QW-2).
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GenerateConsoleCtrlEvent(event: u32, group: u32) -> i32;
    }
    const CTRL_BREAK_EVENT: u32 = 1;
    const ERROR_INVALID_HANDLE: i32 = 6;
    // SAFETY: the call takes two integers by value and reads or writes no memory of this process. Its
    // only effect is to raise Ctrl-Break in the process group `pid`, which `interruptible` created for
    // this child alone, so no other process — this one included — receives it.
    let sent = unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid) };
    if sent != 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(ERROR_INVALID_HANDLE) {
        Err(io::Error::other(format!(
            "no console attached; Ctrl-Break cannot be delivered to process group {pid}: {error}"
        )))
    } else {
        Err(io::Error::other(format!(
            "GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, {pid}) failed: {error}"
        )))
    }
}
