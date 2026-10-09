//! Who may join: the server's invite, the invite a client offers, and the nickname a player gives.
//!
//! `docs/NETWORKING.md` §9 scopes MVP authentication to *a server invite token plus a player
//! nickname*, and `server/PROTOCOL.md` §4.1 says how a `join` is checked against them. This module
//! is that check and nothing else: it holds no world, consults no seat and knows no transport.
//!
//! # The secrets, and where they may appear
//!
//! ```text
//! InviteToken     the server's invite. Given by the operator or generated here. Printed once, by
//!                 the composition root, on the startup join line (the only call to `reveal`).
//! OfferedInvite   what a client sent. Possibly a near miss of the real invite, so treated as one.
//! ```
//!
//! Neither implements `Serialize` or `Display`, and both print as `<redacted>` under `Debug`, so
//! neither can reach a frame, a fact, a save, an observation or a log line by accident (step-12 I-5).
//! The world thread never sees either: admission is decided in the connection's own task, before
//! the world is asked for a seat.
//!
//! # The nickname
//!
//! A [`Nickname`] labels the *player*, never the Person — the Person's name is world data. It is
//! echoed to the player's own connection and shown to nobody else (`PROTOCOL.md` §4.1).
//!
//! # Dependencies (`docs/DECISIONS.md` `DEP-14`)
//!
//! `getrandom` makes an invite from the operating system's random source; `subtle` compares an
//! offered invite in constant time. This is the only file that names either.

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

/// How long a connection waits before it is told its invite was wrong, measured from its `join`.
///
/// One guess per connection, at most one answer per half second: enough against guessing a 128-bit
/// invite on a LAN or behind a tunnel. Rate limiting per address is the adopt route for public
/// hosting (`DEP-14`).
pub const UNAUTHORIZED_DELAY: Duration = Duration::from_millis(500);

/// The fewest bytes an operator-given invite may have.
pub const MIN_INVITE_LENGTH: usize = 8;

/// The most bytes an operator-given invite may have.
pub const MAX_INVITE_LENGTH: usize = 128;

/// The most Unicode scalar values a nickname may have, after trimming.
pub const MAX_NICKNAME_LENGTH: usize = 32;

/// How many random bytes a generated invite carries: 128 bits.
const GENERATED_INVITE_BYTES: usize = 16;

/// The server's invite: the one secret every joining client must present.
#[derive(Clone, PartialEq, Eq)]
pub struct InviteToken(String);

impl InviteToken {
    /// A fresh invite: 16 bytes from the operating system's random source, as 32 lowercase
    /// hexadecimal characters.
    pub fn generate() -> Result<Self, AdmissionError> {
        random_hex().map(Self)
    }

    /// An invite the operator chose: 8 to 128 bytes of printable ASCII, no whitespace — so that it
    /// survives a shell, an environment variable and the join line unchanged.
    ///
    /// The refusal never contains the text it refused: an operator's near-miss is still a secret.
    pub fn given(text: impl Into<String>) -> Result<Self, AdmissionError> {
        let text = text.into();
        if !operator_length(&text) {
            return Err(AdmissionError::InviteLength { length: text.len() });
        }
        if !operator_characters(&text) {
            return Err(AdmissionError::InviteCharacters);
        }
        Ok(Self(text))
    }

    /// The invite's text. Called once, by the composition root, to print the startup join line of a
    /// generated invite; nothing else needs to read it.
    pub fn reveal(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for InviteToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InviteToken(<redacted>)")
    }
}

/// The rules every operator-given token follows: 8 to 128 bytes …
fn operator_length(text: &str) -> bool {
    (MIN_INVITE_LENGTH..=MAX_INVITE_LENGTH).contains(&text.len())
}

/// … of printable ASCII with no whitespace, so that it survives a shell, cmd.exe, PowerShell and an
/// environment variable unchanged.
fn operator_characters(text: &str) -> bool {
    text.bytes().all(|byte| byte.is_ascii_graphic())
}

/// The token that opens the admin surface (`PROTOCOL.md` §11, `docs/DECISIONS.md` `ARC-44`): whoever
/// holds it is the host.
///
/// Given by the operator (`--admin-token`, `MINEWORLD_ADMIN_TOKEN`) under the invite's rules, never
/// generated, never printed. Like the invite it has a redacted `Debug` and neither `Serialize` nor
/// `Display`, so it cannot reach a frame, a save or a log line by accident (step-12 I-5).
#[derive(Clone)]
pub struct AdminToken(String);

impl AdminToken {
    /// The operator's admin token, or why it is unusable — said without repeating it.
    pub fn given(text: impl Into<String>) -> Result<Self, AdmissionError> {
        let text = text.into();
        if !operator_length(&text) {
            return Err(AdmissionError::AdminTokenLength { length: text.len() });
        }
        if !operator_characters(&text) {
            return Err(AdmissionError::AdminTokenCharacters);
        }
        Ok(Self(text))
    }

    /// Whether a request presented exactly this token, compared in constant time.
    pub fn admits(&self, offered: &[u8]) -> bool {
        bool::from(self.0.as_bytes().ct_eq(offered))
    }

    /// Whether this token is the server's invite too — which a server refuses, because every player
    /// would then be the host (step-12 QS11D-4).
    pub fn is_invite(&self, invite: &InviteToken) -> bool {
        self.admits(invite.0.as_bytes())
    }
}

impl fmt::Debug for AdminToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AdminToken(<redacted>)")
    }
}

/// The invite a client offered in its `join`: untrusted, and possibly a near miss of the real one.
#[derive(Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(from = "String")]
pub struct OfferedInvite(String);

impl OfferedInvite {
    /// The text a client sent.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

impl From<String> for OfferedInvite {
    fn from(text: String) -> Self {
        Self(text)
    }
}

impl fmt::Debug for OfferedInvite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OfferedInvite(<redacted>)")
    }
}

/// The secret that re-takes a seat after its connection's socket dropped (`PROTOCOL.md` §4.2).
///
/// 128 bits from the operating system's random source, as 32 lowercase hexadecimal characters. Made
/// on the world thread for every welcome — a fresh one each time, so an old one dies with its
/// binding — and sent only in its holder's own `welcome`. `Debug` is redacted and there is no
/// `Display`; it serializes (it is a welcome field) and nothing else of the server serializes it.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "String", from = "String")]
pub struct ResumeSecret(String);

impl ResumeSecret {
    /// A fresh secret.
    pub fn generate() -> Result<Self, AdmissionError> {
        random_hex().map(Self)
    }

    /// Whether a client offered exactly this secret, compared in constant time.
    pub fn matches(&self, offered: &OfferedResume) -> bool {
        bool::from(self.0.as_bytes().ct_eq(offered.0.as_bytes()))
    }

    /// The secret's text, for a client holding its own welcome (and a test).
    pub fn reveal(&self) -> &str {
        &self.0
    }
}

impl From<ResumeSecret> for String {
    fn from(secret: ResumeSecret) -> Self {
        secret.0
    }
}

impl From<String> for ResumeSecret {
    fn from(text: String) -> Self {
        Self(text)
    }
}

impl fmt::Debug for ResumeSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ResumeSecret(<redacted>)")
    }
}

/// The resume secret a client offered in its `join`: untrusted, possibly a near miss of a real one.
#[derive(Clone, PartialEq, Eq, Deserialize)]
#[serde(from = "String")]
pub struct OfferedResume(String);

impl OfferedResume {
    /// The text a client sent.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

impl From<String> for OfferedResume {
    fn from(text: String) -> Self {
        Self(text)
    }
}

impl fmt::Debug for OfferedResume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OfferedResume(<redacted>)")
    }
}

/// 16 bytes from the operating system's random source, as 32 lowercase hexadecimal characters.
fn random_hex() -> Result<String, AdmissionError> {
    let mut bytes = [0_u8; GENERATED_INVITE_BYTES];
    getrandom::fill(&mut bytes).map_err(|error| AdmissionError::Randomness(error.to_string()))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// A player's nickname: 1 to 32 Unicode scalar values after trimming, no control character.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Nickname(String);

impl Nickname {
    /// Trims the text and checks it.
    pub fn new(text: &str) -> Result<Self, AdmissionError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(AdmissionError::NicknameEmpty);
        }
        let length = trimmed.chars().count();
        if length > MAX_NICKNAME_LENGTH {
            return Err(AdmissionError::NicknameLength { length });
        }
        if trimmed.chars().any(char::is_control) {
            return Err(AdmissionError::NicknameControl);
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// The nickname as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Nickname {
    type Error = AdmissionError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::new(&text)
    }
}

impl From<Nickname> for String {
    fn from(nickname: Nickname) -> Self {
        nickname.0
    }
}

/// The check every `join` passes before the world is asked for anything.
#[derive(Debug, Clone)]
pub struct Admission {
    invite: InviteToken,
}

impl Admission {
    /// Admits the holders of this invite.
    pub const fn new(invite: InviteToken) -> Self {
        Self { invite }
    }

    /// Whether the offered invite is the server's, compared in constant time.
    ///
    /// `subtle` returns at once when the lengths differ. The length of an invite is not a secret — a
    /// generated one is always 32 characters — so that early answer tells a guesser nothing it could
    /// not read in `PROTOCOL.md`.
    pub fn admit(&self, offered: &OfferedInvite) -> Result<(), Unauthorized> {
        if bool::from(self.invite.0.as_bytes().ct_eq(offered.0.as_bytes())) {
            Ok(())
        } else {
            Err(Unauthorized)
        }
    }
}

/// The offered invite was not the server's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unauthorized;

/// What can be wrong with an invite or a nickname, said without repeating a secret.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdmissionError {
    /// An operator-given invite of an unusable length.
    #[error(
        "an invite is {MIN_INVITE_LENGTH} to {MAX_INVITE_LENGTH} characters, and this one is {length}"
    )]
    InviteLength {
        /// The offending length.
        length: usize,
    },
    /// An operator-given invite with whitespace, a control character or a non-ASCII character.
    #[error("an invite is printable ASCII without spaces")]
    InviteCharacters,
    /// An admin token of an unusable length.
    #[error(
        "an admin token is {MIN_INVITE_LENGTH} to {MAX_INVITE_LENGTH} characters, and this one is \
         {length}"
    )]
    AdminTokenLength {
        /// The offending length.
        length: usize,
    },
    /// An admin token with whitespace, a control character or a non-ASCII character.
    #[error("an admin token is printable ASCII without spaces")]
    AdminTokenCharacters,
    /// The operating system's random source failed.
    #[error("no random bytes for an invite: {0}")]
    Randomness(String),
    /// A nickname that is empty once trimmed.
    #[error("a nickname is not empty")]
    NicknameEmpty,
    /// A nickname longer than [`MAX_NICKNAME_LENGTH`] scalar values.
    #[error("a nickname is at most {MAX_NICKNAME_LENGTH} characters, and this one is {length}")]
    NicknameLength {
        /// The offending length, in Unicode scalar values.
        length: usize,
    },
    /// A nickname containing a control character.
    #[error("a nickname must not contain control characters")]
    NicknameControl,
}

#[cfg(test)]
mod tests {
    use super::*;

    const INVITE: &str = "3f9c0a1b2c3d4e5f60718293a4b5c6d7";

    fn admission() -> Admission {
        Admission::new(InviteToken::given(INVITE).expect("a legal invite"))
    }

    #[test]
    fn a_generated_invite_is_128_random_bits_in_lowercase_hexadecimal() {
        let first = InviteToken::generate().expect("random bytes");
        let second = InviteToken::generate().expect("random bytes");
        for invite in [&first, &second] {
            assert_eq!(invite.reveal().len(), 32);
            assert!(
                invite
                    .reveal()
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            );
        }
        assert_ne!(first, second, "two generations are two secrets");
    }

    #[test]
    fn an_operator_invite_is_refused_without_repeating_it() {
        for refused in [
            "short77",        // 7 bytes
            &"x".repeat(129), // 129 bytes
            "has a space1",   // whitespace
            "tab\there12",    // a control character
            "café-invite-1",  // non-ASCII
        ] {
            let error = InviteToken::given(refused).expect_err("an illegal invite");
            assert!(
                !error.to_string().contains(refused),
                "the refusal of {refused:?} repeats it: {error}"
            );
        }
        assert!(InviteToken::given("eight888").is_ok());
        assert!(InviteToken::given("x".repeat(128)).is_ok());
    }

    #[test]
    fn only_the_invite_itself_is_admitted() {
        let admission = admission();
        assert_eq!(admission.admit(&OfferedInvite::new(INVITE)), Ok(()));
        let mut last_changed = INVITE.to_owned();
        last_changed.pop();
        last_changed.push('8');
        for wrong in [
            last_changed.as_str(),
            &format!("{INVITE} "),
            &INVITE[..31],
            "",
        ] {
            assert_eq!(
                admission.admit(&OfferedInvite::new(wrong)),
                Err(Unauthorized),
                "{wrong:?} was admitted"
            );
        }
    }

    #[test]
    fn neither_invite_prints_its_text() {
        let invite = InviteToken::given(INVITE).expect("a legal invite");
        assert!(!format!("{invite:?}").contains(INVITE));
        assert!(!format!("{:?}", OfferedInvite::new(INVITE)).contains(INVITE));
        assert!(!format!("{:?}", admission()).contains(INVITE));
    }

    #[test]
    fn a_resume_secret_is_fresh_hexadecimal_matched_exactly_and_never_printed() {
        let first = ResumeSecret::generate().expect("random bytes");
        let second = ResumeSecret::generate().expect("random bytes");
        assert_ne!(first, second, "every welcome gets its own secret");
        for secret in [&first, &second] {
            assert_eq!(secret.reveal().len(), 32);
            assert!(
                secret
                    .reveal()
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            );
            assert!(!format!("{secret:?}").contains(secret.reveal()));
        }
        assert!(first.matches(&OfferedResume::new(first.reveal())));
        assert!(!first.matches(&OfferedResume::new(second.reveal())));
        assert!(!first.matches(&OfferedResume::new(&first.reveal()[..31])));
        let offered = OfferedResume::new(first.reveal());
        assert!(!format!("{offered:?}").contains(first.reveal()));
    }

    #[test]
    fn an_admin_token_follows_the_invite_rules_admits_only_itself_and_never_prints() {
        for refused in ["short", &"x".repeat(129), "has a space1", "café-token-1"] {
            let error = AdminToken::given(refused).expect_err("an illegal token");
            assert!(!error.to_string().contains(refused), "{error}");
        }
        let token = AdminToken::given("admin-token-7c1e").expect("a legal token");
        assert!(token.admits(b"admin-token-7c1e"));
        for wrong in ["admin-token-7c1f", "admin-token-7c1e ", "admin-token-7c1", ""] {
            assert!(!token.admits(wrong.as_bytes()), "{wrong:?} was admitted");
        }
        assert!(!format!("{token:?}").contains("admin-token-7c1e"));
        let same = InviteToken::given("admin-token-7c1e").expect("a legal invite");
        assert!(token.is_invite(&same));
        assert!(!token.is_invite(&InviteToken::given(INVITE).expect("a legal invite")));
    }

    #[test]
    fn a_nickname_is_trimmed_and_counted_in_scalar_values() {
        assert_eq!(
            Nickname::new("  Zephyrine-7 ").expect("legal").as_str(),
            "Zephyrine-7"
        );
        assert!(
            Nickname::new(&"é".repeat(32)).is_ok(),
            "32 two-byte scalars"
        );
        assert_eq!(
            Nickname::new(&"é".repeat(33)),
            Err(AdmissionError::NicknameLength { length: 33 })
        );
        assert_eq!(Nickname::new("   "), Err(AdmissionError::NicknameEmpty));
        assert_eq!(
            Nickname::new("bell\u{7}"),
            Err(AdmissionError::NicknameControl)
        );
    }
}
