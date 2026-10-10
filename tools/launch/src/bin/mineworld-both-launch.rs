//! "MineWorld 2D + 3D": both clients on one server, `AC-15`'s showcase (`LAUNCHER.md` §2).
#![windows_subsystem = "windows"]

fn main() -> std::process::ExitCode {
    mineworld_launch::main(mineworld_launch::Mode::Both)
}
