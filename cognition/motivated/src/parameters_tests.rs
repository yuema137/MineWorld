//! The parameter block's acceptance and refusals (`pr-dl-b-motivated.md` SD-DLb-7, C4 validation).
//!
//! Decoded from JSON here, because `serde_json` positions its errors (line and column) exactly as the decision
//! list's YAML reader does, and the block itself is format-neutral.

use mineworld_decision::ModelParameters;

use crate::parameters::MotivatedParameters;

/// Decodes `text`, returning the refusal's message and its one-based line and column.
fn refusal(text: &str) -> (String, usize, usize) {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    match MotivatedParameters::decode(&mut deserializer) {
        Ok(_) => panic!("{text} is accepted but should be refused"),
        Err(error) => (error.to_string(), error.line(), error.column()),
    }
}

fn accepted(text: &str) -> MotivatedParameters {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    MotivatedParameters::decode(&mut deserializer).expect("the block is accepted")
}

#[test]
fn an_empty_block_is_the_defaults_and_so_is_an_explicit_default() {
    let empty = accepted("{}");
    assert_eq!(empty, MotivatedParameters::default());
    let explicit = accepted(
        r#"{"top_band": 100, "commit_minutes": 180, "social_need_hours": 3, "reply_floor": 250}"#,
    );
    assert_eq!(explicit, empty, "{{}} and an explicit default decode alike");
}

#[test]
fn a_scalar_out_of_its_bound_is_refused_with_its_position() {
    let (message, line, column) = refusal("{\n  \"top_band\": 600\n}");
    assert!(
        message.contains("'top_band' is 0 … 500, not 600"),
        "{message}"
    );
    // The decoder reports where it stands after reading the value, which may be the next line.
    assert!(
        line == 2 || line == 3,
        "the refusal stands at the value: line {line}"
    );
    assert!(column > 0, "{column}");
}

#[test]
fn an_unknown_key_is_refused_by_name() {
    let (message, line, _) = refusal(r#"{"nonsense": 1}"#);
    assert!(message.contains("unknown field `nonsense`"), "{message}");
    assert_eq!(line, 1);
}

#[test]
fn an_unknown_curve_set_is_refused_and_the_shipped_one_named() {
    let (message, _, _) = refusal(r#"{"curves": "atus-a3-1999"}"#);
    assert!(
        message.contains("'atus-a3-1999' is not a shipped curve set"),
        "{message}"
    );
    assert!(message.contains("atus-a3-2025"), "{message}");
}

#[test]
fn an_unknown_option_in_base_is_refused() {
    let (message, _, _) = refusal(r#"{"base": {"fly": 300}}"#);
    assert!(message.contains("'fly' is not an option"), "{message}");
}

#[test]
fn an_unknown_role_is_refused() {
    let (message, _, _) = refusal(r#"{"roles": {"space": ["residential"]}}"#);
    assert!(message.contains("'space' is not a role"), "{message}");
}

#[test]
fn a_malformed_tag_is_refused_by_name() {
    let (message, _, _) = refusal(r#"{"roles": {"home": ["Not Valid!"]}}"#);
    assert!(message.contains("'Not Valid!' is not a tag"), "{message}");
}

#[test]
fn a_role_with_more_than_eight_tags_is_refused() {
    let tags = (0..9)
        .map(|index| format!("\"t{index}\""))
        .collect::<Vec<_>>()
        .join(",");
    let (message, _, _) = refusal(&format!(r#"{{"roles": {{"home": [{tags}]}}}}"#));
    assert!(message.contains("lists 9 tags, at most 8"), "{message}");
}

#[test]
fn an_unknown_curve_name_is_refused() {
    let (message, _, _) = refusal(r#"{"curve": {"happiness": [[0, 500]]}}"#);
    assert!(message.contains("'happiness' is not a curve"), "{message}");
}

#[test]
fn a_curve_of_twenty_five_points_is_refused() {
    let points = (0..25)
        .map(|x| format!("[{x}, 500]"))
        .collect::<Vec<_>>()
        .join(",");
    // An unbounded-input curve, so the count is what is refused (an hour curve would refuse x = 24 first).
    let (message, _, _) = refusal(&format!(r#"{{"curve": {{"loneliness": [{points}]}}}}"#));
    assert!(message.contains("at most 24 points, not 25"), "{message}");
}

#[test]
fn a_curve_whose_x_does_not_increase_is_refused() {
    let (message, _, _) = refusal(r#"{"curve": {"sociability": [[3, 0], [2, 5]]}}"#);
    assert!(
        message.contains("point 1 does not have a greater x"),
        "{message}"
    );
}

#[test]
fn a_curve_value_above_one_thousand_is_refused() {
    let (message, _, _) = refusal(r#"{"curve": {"regard": [[0, 1001]]}}"#);
    assert!(message.contains("point 0 has y 1001"), "{message}");
}

#[test]
fn an_hour_outside_the_day_is_refused_for_an_hour_curve() {
    let (message, _, _) = refusal(r#"{"curve": {"sociability": [[24, 500]]}}"#);
    assert!(
        message.contains("point 0 has x 24, outside the input 0 … 23"),
        "{message}"
    );
}

#[test]
fn a_key_written_twice_is_refused() {
    let (message, _, _) = refusal(r#"{"top_band": 100, "top_band": 200}"#);
    assert!(message.contains("'top_band' is written twice"), "{message}");
}

#[test]
fn an_override_replaces_its_curve_in_the_derived_settings() {
    let parameters = accepted(r#"{"curve": {"regard": [[-1000, 0], [1000, 1000]]}}"#);
    let regard = parameters.settings().curves.regard.clone();
    assert_eq!(regard.points(), &[(-1_000, 0), (1_000, 1_000)]);
}
