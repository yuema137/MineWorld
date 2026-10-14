//! The happenings' randomness: counter-based, so every draw is a pure function of `(seed, day, key,
//! index)` and no draw depends on how many came before it (INV-WE-2; design §5.3).
//!
//! An entry's draw key is FNV-1a 64 (Fowler, Noll, Vo; public domain) over its own names, so reordering
//! or adding entries moves no other entry's draws.

#[cfg(test)]
mod tests;

/// FNV-1a 64's offset basis and prime.
const FNV_OFFSET: u64 = 0xCBF2_9CE4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;

/// FNV-1a 64 over `parts` joined by zero bytes: `"random" 0x00 kind 0x00 place`.
pub(crate) fn key(parts: &[&str]) -> u64 {
    let mut hash = FNV_OFFSET;
    for (index, part) in parts.iter().enumerate() {
        let separator = (index > 0).then_some(0_u8);
        for byte in separator.into_iter().chain(part.bytes()) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash
}
