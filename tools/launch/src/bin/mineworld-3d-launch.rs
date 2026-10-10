//! "MineWorld 3D" (`LAUNCHER.md` §2).
#![windows_subsystem = "windows"]

fn main() -> std::process::ExitCode {
    mineworld_launch::main(mineworld_launch::Mode::ThreeD)
}
