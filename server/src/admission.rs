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
//! GrantToken      a seat grant's secret (S27-a, `ARC-97`). Only ever generated here; printed once,
//!                 by the composition root, on the grant's startup line.
//! OfferedInvite   what a client sent. Possibly a near miss of a real credential, so treated as one.
//! ```
//!
//! None implements `Serialize` or `Display`, and all print as `<redacted>` under `Debug`, so none can
//! reach a frame, a fact, a save, an observation or a log line by accident (step-12 I-5). The world
//! thread never sees any: admission is decided in the connection's own task, before the world is
//! asked for a seat, and what crosses to it is a [`SeatRights`] — seat names and a boolean.
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

use std::collections::BTreeSet;
use std::fmt;
use std::time::Duration;

use mineworld_contracts::EntityKey;
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

    /// Whether this token is also a player's credential — the invite or a seat grant — which a server
    /// refuses, because every holder of it would then be the host (step-12 QS11D-4, `ARC-97`).
    pub fn is_a_credential_of(&self, admission: &Admission) -> bool {
        admission.holds(self.0.as_bytes())
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

/// A seat grant's secret (`PROTOCOL.md` §4.1 "Seat grants", `docs/DECISIONS.md` `ARC-97`).
///
/// Only ever generated — 16 bytes from the operating system's random source, as 32 lowercase
/// hexadecimal characters — never given by an operator, never parsed from an argument or the
/// environment. Like the invite it has a redacted `Debug` and neither `Serialize` nor `Display`;
/// `reveal` is called once, by the composition root, for the grant's startup line.
#[derive(Clone, PartialEq, Eq)]
pub struct GrantToken(String);

impl GrantToken {
    /// A fresh grant token.
    pub fn generate() -> Result<Self, AdmissionError> {
        random_hex().map(Self)
    }

    /// The token's text: for the grant's one startup line, and for a test that joins with it.
    pub fn reveal(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for GrantToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrantToken(<redacted>)")
    }
}

/// A narrower credential than the invite: it admits only the seats it names, and takes over a seat
/// another player holds only if it carries take-over (`PROTOCOL.md` §§4.1, 4.2).
///
/// Host state (`ARC-40`): it lives in the connection task's [`Admission`] and nowhere else. Nothing in
/// it says what kind of Controller will present it.
#[derive(Debug, Clone)]
pub struct SeatGrant {
    seats: BTreeSet<EntityKey>,
    take_over: bool,
    token: GrantToken,
}

impl SeatGrant {
    /// A grant for these seats, with a freshly generated token. A grant names at least one seat.
    pub fn generate(seats: BTreeSet<EntityKey>, take_over: bool) -> Result<Self, AdmissionError> {
        if seats.is_empty() {
            return Err(AdmissionError::GrantNamesNoSeat);
        }
        Ok(Self {
            seats,
            take_over,
            token: GrantToken::generate()?,
        })
    }

    /// The seats it admits, in key order.
    pub const fn seats(&self) -> &BTreeSet<EntityKey> {
        &self.seats
    }

    /// Whether it may take over a connected or held seat.
    pub const fn take_over(&self) -> bool {
        self.take_over
    }

    /// Its secret.
    pub const fn token(&self) -> &GrantToken {
        &self.token
    }

    fn rights(&self) -> SeatRights {
        SeatRights::Named {
            seats: self.seats.clone(),
            take_over: self.take_over,
        }
    }
}

/// What a credential lets its holder do with seats. Carries no secret, so it may cross to the world
/// thread inside a [`JoinRequest`](crate::seats::JoinRequest).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeatRights {
    /// The invite: every seat, and the takeover of any.
    Any,
    /// A seat grant: these seats only; the takeover of a connected or held one only if `take_over`.
    Named {
        /// The seats admitted.
        seats: BTreeSet<EntityKey>,
        /// Whether a connected or held seat may be taken over.
        take_over: bool,
    },
}

impl SeatRights {
    /// Whether these rights admit `seat` at all (`PROTOCOL.md` §4.1 check 4a).
    pub fn names(&self, seat: &EntityKey) -> bool {
        match self {
            Self::Any => true,
            Self::Named { seats, .. } => seats.contains(seat),
        }
    }

    /// Whether these rights may take a seat another player holds (`PROTOCOL.md` §4.2 rule 4).
    pub const fn may_take_over(&self) -> bool {
        match self {
            Self::Any => true,
            Self::Named { take_over, .. } => *take_over,
        }
    }
}

/// Which kind of credential admitted a connection, as the admin surface names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialKind {
    /// The server's invite.
    Invite,
    /// A seat grant.
    Grant,
}

/// The credential a `join` presented, once admitted: its kind and its rights, never its token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credential {
    /// Which kind it is.
    pub kind: CredentialKind,
    /// What it lets its holder do with seats.
    pub rights: SeatRights,
}

/// The check every `join` passes before the world is asked for anything.
#[derive(Debug, Clone)]
pub struct Admission {
    invite: InviteToken,
    grants: Vec<SeatGrant>,
}

impl Admission {
    /// Admits the holders of this invite, and nobody else.
    pub const fn new(invite: InviteToken) -> Self {
        Self {
            invite,
            grants: Vec::new(),
        }
    }

    /// Also admits the holders of these grants, each to the seats it names. Refuses a grant whose
    /// token equals the invite or another grant's: one secret must never mean two credentials.
    pub fn with_grants(
        mut self,
        grants: impl IntoIterator<Item = SeatGrant>,
    ) -> Result<Self, AdmissionError> {
        for grant in grants {
            if self.holds(grant.token.0.as_bytes()) {
                return Err(AdmissionError::CredentialsNotDistinct);
            }
            self.grants.push(grant);
        }
        Ok(self)
    }

    /// The grants, in the order they were given.
    pub fn grants(&self) -> &[SeatGrant] {
        &self.grants
    }

    /// Whether `text` is the invite or any grant's token — for refusing an admin token that would
    /// make a player the host. Compared in constant time.
    pub fn holds(&self, text: &[u8]) -> bool {
        let mut any = self.invite.0.as_bytes().ct_eq(text);
        for grant in &self.grants {
            any |= grant.token.0.as_bytes().ct_eq(text);
        }
        bool::from(any)
    }

    /// Which credential the offered text is — the invite or one grant — compared in constant time
    /// with every one of them, always: the comparisons are folded, never cut short at a match.
    ///
    /// `subtle` returns at once when the lengths differ. The length of a credential is not a secret —
    /// a generated one is always 32 characters — so that early answer tells a guesser nothing it could
    /// not read in `PROTOCOL.md`. A wrong offer is one answer, [`Unauthorized`], whichever kinds of
    /// credential this server has.
    pub fn admit(&self, offered: &OfferedInvite) -> Result<Credential, Unauthorized> {
        let offered = offered.0.as_bytes();
        let invite = bool::from(self.invite.0.as_bytes().ct_eq(offered));
        let mut granted: Option<&SeatGrant> = None;
        for grant in &self.grants {
            let hit = bool::from(grant.token.0.as_bytes().ct_eq(offered));
            granted = if hit { Some(grant) } else { granted };
        }
        match (invite, granted) {
            (true, _) => Ok(Credential {
                kind: CredentialKind::Invite,
                rights: SeatRights::Any,
            }),
            (false, Some(grant)) => Ok(Credential {
                kind: CredentialKind::Grant,
                rights: grant.rights(),
            }),
            (false, None) => Err(Unauthorized),
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
    /// A seat grant that names no seat.
    #[error("a seat grant names at least one seat")]
    GrantNamesNoSeat,
    /// A seat grant's token equal to the invite or to another grant's.
    #[error("the invite and every seat grant must be distinct credentials")]
    CredentialsNotDistinct,
    /// An admin token equal to the invite or to a seat grant.
    #[error(
        "the admin token must differ from the invite and from every seat grant; a player who holds \
         one would otherwise be the host"
    )]
    AdminTokenNotDistinct,
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
        assert_eq!(
            admission.admit(&OfferedInvite::new(INVITE)).map(|c| c.kind),
            Ok(CredentialKind::Invite)
        );
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
        for wrong in [
            "admin-token-7c1f",
            "admin-token-7c1e ",
            "admin-token-7c1",
            "",
        ] {
            assert!(!token.admits(wrong.as_bytes()), "{wrong:?} was admitted");
        }
        assert!(!format!("{token:?}").contains("admin-token-7c1e"));
        let same = InviteToken::given("admin-token-7c1e").expect("a legal invite");
        assert!(token.is_a_credential_of(&Admission::new(same)));
        assert!(!token.is_a_credential_of(&admission()));
    }

    fn seats(names: &[&str]) -> BTreeSet<EntityKey> {
        names
            .iter()
            .map(|name| EntityKey::new(*name).expect("a key"))
            .collect()
    }

    fn grant(names: &[&str], take_over: bool) -> SeatGrant {
        SeatGrant::generate(seats(names), take_over).expect("random bytes")
    }

    #[test]
    fn a_grant_token_is_fresh_hexadecimal_and_never_printed() {
        let first = grant(&["visitor"], false);
        let second = grant(&["visitor"], false);
        assert_ne!(first.token(), second.token(), "two grants are two secrets");
        for grant in [&first, &second] {
            let text = grant.token().reveal();
            assert_eq!(text.len(), 32);
            assert!(
                text.bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            );
            assert!(!format!("{grant:?}").contains(text), "{grant:?}");
        }
        let admission = admission().with_grants([first.clone()]).expect("distinct");
        assert!(!format!("{admission:?}").contains(first.token().reveal()));
        assert_eq!(
            SeatGrant::generate(BTreeSet::new(), false).expect_err("no seat"),
            AdmissionError::GrantNamesNoSeat
        );
    }

    #[test]
    fn the_invite_and_each_grant_admit_with_their_own_rights() {
        let visitor = grant(&["visitor"], false);
        let both = grant(&["alice", "wanderer"], true);
        let admission = admission()
            .with_grants([visitor.clone(), both.clone()])
            .expect("distinct");
        let invite = admission
            .admit(&OfferedInvite::new(INVITE))
            .expect("the invite");
        assert_eq!(invite.kind, CredentialKind::Invite);
        assert_eq!(invite.rights, SeatRights::Any);
        let narrow = admission
            .admit(&OfferedInvite::new(visitor.token().reveal()))
            .expect("the grant");
        assert_eq!(narrow.kind, CredentialKind::Grant);
        assert!(
            narrow
                .rights
                .names(&EntityKey::new("visitor").expect("key"))
        );
        assert!(!narrow.rights.names(&EntityKey::new("alice").expect("key")));
        assert!(!narrow.rights.may_take_over());
        let wide = admission
            .admit(&OfferedInvite::new(both.token().reveal()))
            .expect("the other grant");
        assert_eq!(
            wide.rights,
            SeatRights::Named {
                seats: seats(&["alice", "wanderer"]),
                take_over: true
            }
        );
        let mut near = visitor.token().reveal().to_owned();
        let last = near.pop().expect("a character");
        near.push(if last == '0' { '1' } else { '0' });
        for wrong in [near.as_str(), &visitor.token().reveal()[..31], ""] {
            assert_eq!(
                admission.admit(&OfferedInvite::new(wrong)),
                Err(Unauthorized),
                "{wrong:?} was admitted"
            );
        }
    }

    #[test]
    fn a_grant_that_collides_with_another_credential_is_refused() {
        let collides_with_invite = SeatGrant {
            seats: seats(&["visitor"]),
            take_over: false,
            token: GrantToken(INVITE.to_owned()),
        };
        assert_eq!(
            admission()
                .with_grants([collides_with_invite])
                .expect_err("equal to the invite"),
            AdmissionError::CredentialsNotDistinct
        );
        let one = grant(&["visitor"], false);
        let twin = SeatGrant {
            seats: seats(&["alice"]),
            take_over: true,
            token: one.token().clone(),
        };
        assert_eq!(
            admission()
                .with_grants([one.clone(), twin])
                .expect_err("two grants, one secret"),
            AdmissionError::CredentialsNotDistinct
        );
        let admission = admission().with_grants([one.clone()]).expect("distinct");
        let admin = AdminToken::given(one.token().reveal()).expect("a legal token");
        assert!(
            admin.is_a_credential_of(&admission),
            "an admin token equal to a grant"
        );
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
