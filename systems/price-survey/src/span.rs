//! A span of world time as a section writes it: a whole number followed by `m`, `h` or `d`, such as
//! `"7d"` — the same grammar as `notices`' durations (`ARC-115`), so an author learns one form.

/// Seconds in a minute.
pub const MINUTE: u32 = 60;
/// Seconds in an hour.
pub const HOUR: u32 = 3_600;
/// Seconds in a day.
pub const DAY: u32 = 86_400;

/// A span of world time, in whole seconds, within the bounds its key allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span(u32);

impl Span {
    /// Reads `text` as a span between `min` and `max` seconds, inclusive.
    ///
    /// # Errors
    ///
    /// A message naming `key` and the text: one that does not parse, or one out of bounds.
    pub fn parse(text: &str, min: u32, max: u32, key: &str) -> Result<Self, String> {
        let malformed =
            || format!("{key}: a whole number followed by m, h or d, such as \"7d\", not `{text}`");
        if !text.is_ascii() {
            return Err(malformed());
        }
        let (digits, unit) = text.split_at(text.len().saturating_sub(1));
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(malformed());
        }
        let scale = match unit {
            "m" => MINUTE,
            "h" => HOUR,
            "d" => DAY,
            _ => return Err(malformed()),
        };
        let out_of_bounds = || format!("{key}: {} to {}, not `{text}`", render(min), render(max));
        let count: u64 = digits.parse().map_err(|_| out_of_bounds())?;
        let seconds = count
            .checked_mul(u64::from(scale))
            .ok_or_else(out_of_bounds)?;
        if seconds < u64::from(min) || seconds > u64::from(max) {
            return Err(out_of_bounds());
        }
        u32::try_from(seconds)
            .map(Self)
            .map_err(|_| out_of_bounds())
    }

    /// A span of `seconds`, unchecked: a default the pack itself chose within the bounds.
    pub(crate) const fn of(seconds: u32) -> Self {
        Self(seconds)
    }

    /// The span, in seconds.
    pub const fn seconds(self) -> u32 {
        self.0
    }
}

/// A bound as an author writes it: the largest unit that divides it.
fn render(seconds: u32) -> String {
    if seconds.is_multiple_of(DAY) {
        format!("{}d", seconds / DAY)
    } else if seconds.is_multiple_of(HOUR) {
        format!("{}h", seconds / HOUR)
    } else {
        format!("{}m", seconds / MINUTE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(text: &str) -> Result<u32, String> {
        Span::parse(text, MINUTE, 30 * DAY, "window").map(Span::seconds)
    }

    #[test]
    fn whole_numbers_of_minutes_hours_and_days_parse() {
        assert_eq!(window("1m"), Ok(60));
        assert_eq!(window("59m"), Ok(3_540));
        assert_eq!(window("1h"), Ok(3_600));
        assert_eq!(window("7d"), Ok(604_800));
        assert_eq!(window("30d"), Ok(2_592_000));
    }

    #[test]
    fn anything_else_is_refused_naming_the_key() {
        for text in ["1.5h", "h", "7", "7D", " 7d", "-1h", "", "7d "] {
            let refused = window(text).expect_err(text);
            assert!(
                refused.starts_with("window: a whole number"),
                "{text}: {refused}"
            );
        }
    }

    #[test]
    fn out_of_bounds_and_overflow_are_refused_with_the_bounds() {
        for text in ["0h", "0m", "31d", "99999999999d"] {
            let refused = window(text).expect_err(text);
            assert_eq!(refused, format!("window: 1m to 30d, not `{text}`"));
        }
    }
}
