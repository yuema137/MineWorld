//! Owns the climate table's refusals (step-19 §17.4): each bound refused while decoding, naming its key,
//! with the deserializer's line and column; and line endings make no difference.

use super::Rules;
use crate::fixture::{SAN_DIEGO, rules};

fn decode(text: &str) -> Result<Rules, String> {
    serde_saphyr::from_str::<Rules>(text).map_err(|error| error.to_string())
}

/// San Diego with the first occurrence of `from` replaced by `to`.
fn with(from: &str, to: &str) -> String {
    assert!(SAN_DIEGO.contains(from), "{from}");
    SAN_DIEGO.replacen(from, to, 1)
}

#[test]
fn every_bound_is_refused_naming_its_key_with_a_position() {
    assert!(decode(SAN_DIEGO).is_ok());
    let cases = [
        (
            "p_wet_after_dry: 147",
            "p_wet_after_dry: 1001",
            "p_wet_after_dry",
        ),
        (
            "p_wet_after_dry: 147",
            "p_wet_after_dry: -1",
            "p_wet_after_dry",
        ),
        (
            "p_wet_after_wet: 447",
            "p_wet_after_wet: 1200",
            "p_wet_after_wet",
        ),
        ("[8, 28, 54, 93, 178]", "[8, 28, 54, 93]", "rain_tenth_mm"),
        (
            "[8, 28, 54, 93, 178]",
            "[2, 28, 54, 93, 178]",
            "rain_tenth_mm",
        ),
        (
            "[8, 28, 54, 93, 178]",
            "[8, 54, 28, 93, 178]",
            "rain_tenth_mm",
        ),
        ("tmax_dc: 191", "tmax_dc: 601", "tmax_dc"),
        ("tmin_dc: 102", "tmin_dc: -901", "tmin_dc"),
        ("tmin_dc: 102", "tmin_dc: 191", "tmin_dc"),
        ("t_noise_dc: 30", "t_noise_dc: 201", "t_noise_dc"),
        ("t_ar_permille: 600", "t_ar_permille: 1000", "t_ar_permille"),
        (
            "wet_tmax_shift_dc: -20",
            "wet_tmax_shift_dc: -201",
            "wet_tmax_shift_dc",
        ),
        ("fog_permille: 80", "fog_permille: 1001", "fog_permille"),
        (
            "thunder_permille: 40",
            "thunder_permille: 1001",
            "thunder_permille",
        ),
        (
            "overcast_morning_permille: 150",
            "overcast_morning_permille: 1001",
            "overcast_morning_permille",
        ),
        ("wind_dms: 25", "wind_dms: 1001", "wind_dms"),
        ("wind_from_deg: 300", "wind_from_deg: 360", "wind_from_deg"),
        (
            "wind_from_deg: 300",
            "wind_from_deg: 300, gusts: 9",
            "gusts",
        ),
    ];
    for (from, to, key) in cases {
        let refusal = decode(&with(from, to)).expect_err(to);
        assert!(refusal.contains(key), "{to} → {refusal}");
        assert!(
            refusal.contains("line 2"),
            "{to} → the first month's line: {refusal}"
        );
        assert!(refusal.contains("column"), "{to} → {refusal}");
    }
    // Eleven months; the error is at the list.
    let eleven: String = SAN_DIEGO
        .lines()
        .take(12)
        .map(|line| format!("{line}\n"))
        .collect();
    let refusal = decode(&eleven).expect_err("eleven months");
    assert!(
        refusal.contains("months") && refusal.contains("11"),
        "{refusal}"
    );
    // An unknown top-level key.
    let refusal = decode(&format!("{SAN_DIEGO}seasons: 4\n")).expect_err("an unknown key");
    assert!(refusal.contains("seasons"), "{refusal}");
    // Bound values are accepted.
    for (from, to) in [
        ("p_wet_after_dry: 147", "p_wet_after_dry: 0"),
        ("p_wet_after_wet: 447", "p_wet_after_wet: 1000"),
        ("[8, 28, 54, 93, 178]", "[3, 3, 3, 3, 3]"),
        ("t_ar_permille: 600", "t_ar_permille: 999"),
        ("wind_from_deg: 300", "wind_from_deg: 359"),
    ] {
        assert!(decode(&with(from, to)).is_ok(), "{to}");
    }
}

#[test]
fn crlf_and_lf_give_the_same_table() {
    assert_eq!(
        decode(&SAN_DIEGO.replace('\n', "\r\n")),
        Ok(rules(SAN_DIEGO))
    );
}

/// The copy carried in a fact decodes back to the same table, through the same validation.
#[test]
fn the_table_round_trips_through_its_encoding() {
    let table = rules(SAN_DIEGO);
    let bytes = serde_json::to_vec(&table).expect("encodes");
    assert_eq!(
        serde_json::from_slice::<Rules>(&bytes).expect("decodes"),
        table
    );
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    value["months"][0]["tmin_dc"] = serde_json::json!(500);
    assert!(
        serde_json::from_value::<Rules>(value).is_err(),
        "a fact cannot carry what a file could not"
    );
}
