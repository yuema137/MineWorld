//! An identifier declared as a literal in code obeys the same rule as one read from an authored
//! file — and because it is a literal, breaking the rule is a compile error rather than a
//! failure when the world loads.

use mineworld_contracts::SystemId;

const ILLEGAL: SystemId = SystemId::from_static("First Stub");

fn main() {
    let _ = ILLEGAL;
}
