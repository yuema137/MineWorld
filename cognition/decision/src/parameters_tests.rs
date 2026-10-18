//! Parameter blocks decode with their defaults and refuse at the offending node, with its line and
//! column (`pr-dl-a-decision-list.md` C2 failure case).

use crate::parameters::{ModelParameters, bounded};

mineworld_sdk::parameters! {
    /// A probe model's tuning, for these tests.
    pub struct Patience, partial PatiencePartial {
        /// How many consults the probe waits.
        patience: u32 = 3, 1 ..= 10;
        /// How far it reaches, millimetres.
        reach: i64 = 1_400, 0 ..= 5_000;
    }
}

fn decode(text: &str) -> Result<Patience, serde_saphyr::Error> {
    serde_saphyr::with_deserializer_from_str(text, |file| bounded::<Patience, _>(file))
}

fn refused(text: &str) -> (u64, u64, String) {
    let error = decode(text).expect_err("refused");
    let at = error
        .location()
        .unwrap_or_else(|| panic!("a positioned refusal: {error:?} // {:?}", error.locations()));
    (at.line(), at.column(), error.to_string())
}

#[test]
fn an_omitted_key_keeps_its_default_and_an_empty_block_is_the_defaults() {
    assert_eq!(decode("{}").expect("decodes"), Patience::default());
    let set = decode("patience: 5\n").expect("decodes");
    assert_eq!((set.patience, set.reach), (5, 1_400));
    assert_eq!(
        decode("patience: 3\nreach: 1400\n").expect("decodes"),
        Patience::default(),
        "explicit defaults equal the defaults"
    );
}

#[test]
fn an_out_of_bound_value_is_refused_at_its_key() {
    let (line, column, message) = refused("reach: 10\npatience: 99\n");
    assert_eq!((line, column), (2, 1), "{message}");
    assert!(
        message.contains("'patience' is 1 … 10, not 99"),
        "{message}"
    );
}

#[test]
fn a_value_of_the_wrong_type_keeps_the_decoder_s_own_position() {
    let (line, column, message) = refused("reach: 10\npatience: many\n");
    assert_eq!((line, column), (2, 11), "{message}");
    assert!(!message.contains("within its bound"), "{message}");
}

#[test]
fn an_unknown_key_is_refused_at_its_key_listing_the_known_ones() {
    let (line, column, message) = refused("reach: 10\n\npatienc: 4\n");
    assert_eq!(line, 3, "{message}");
    assert_eq!(column, 1, "{message}");
    assert!(
        message.contains("patienc") && message.contains("patience"),
        "{message}"
    );
}

#[test]
fn a_key_written_twice_is_refused() {
    let error = decode("patience: 4\npatience: 5\n").expect_err("refused");
    assert!(error.location().is_some(), "{error}");
}

#[test]
fn a_model_without_parameters_refuses_any_key_at_its_position() {
    fn none(text: &str) -> Result<(), serde_saphyr::Error> {
        serde_saphyr::with_deserializer_from_str(text, |file| <()>::decode(file))
    }
    none("{}").expect("an empty block is no parameters");
    let error = none("\n  speed: 3\n").expect_err("refused");
    let at = error.location().expect("positioned");
    assert_eq!((at.line(), at.column()), (2, 3), "{error}");
    assert!(error.to_string().contains("speed"), "{error}");
}
