//! Showing a failure to a player who has no terminal (`LAUNCHER.md` §9). Best effort: a dialog that
//! cannot be shown leaves the message in the log and on standard error, where it already is.

/// Shows `message` in the platform's alert, and returns when the player has dismissed it.
pub fn show(message: &str) {
    platform::show(message);
}

#[cfg(target_os = "macos")]
mod platform {
    use std::process::{Command, Stdio};

    pub fn show(message: &str) {
        // The message is an argument of the script, never part of its text: nothing in it can be read as
        // AppleScript.
        let _ = Command::new("osascript")
            .args([
                "-e",
                "on run argv",
                "-e",
                "display alert \"MineWorld could not start\" message (item 1 of argv) as critical",
                "-e",
                "end run",
                message,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use std::process::{Command, Stdio};

    pub fn show(message: &str) {
        let tries: [(&str, Vec<&str>); 2] = [
            (
                "zenity",
                vec![
                    "--error",
                    "--no-markup",
                    "--title=MineWorld",
                    "--text",
                    message,
                ],
            ),
            ("kdialog", vec!["--title", "MineWorld", "--error", message]),
        ];
        for (program, arguments) in tries {
            let shown = Command::new(program)
                .args(arguments)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            // Not installed: try the next. Installed but refused (no display): no dialog at all.
            if shown.is_ok() {
                return;
            }
        }
    }
}

#[cfg(windows)]
mod platform {
    // user32, linked by the GUI subsystem; declared here rather than taken from `windows-sys`, as
    // `DEP-29`'s note did for `GenerateConsoleCtrlEvent` (ARC-78).
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(window: isize, text: *const u16, caption: *const u16, kind: u32) -> i32;
    }

    const MB_OK: u32 = 0x0000_0000;
    const MB_ICONERROR: u32 = 0x0000_0010;

    fn wide(text: &str) -> Vec<u16> {
        // An interior NUL would end the string early; the message never needs one.
        text.encode_utf16()
            .map(|unit| if unit == 0 { u16::from(b' ') } else { unit })
            .chain(std::iter::once(0))
            .collect()
    }

    #[allow(unsafe_code)]
    pub fn show(message: &str) {
        let text = wide(message);
        let caption = wide("MineWorld could not start");
        // SAFETY: both pointers are to NUL-terminated UTF-16 buffers that live until the call returns;
        // a null owner window is allowed. The call reads them and writes no memory of this process.
        unsafe {
            MessageBoxW(0, text.as_ptr(), caption.as_ptr(), MB_OK | MB_ICONERROR);
        }
    }
}
