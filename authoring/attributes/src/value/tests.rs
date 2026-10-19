use super::*;
use crate::kind::{QuantityKind, Unit};

fn metres() -> Kind {
    Kind::Quantity(QuantityKind {
        unit: Unit::M,
        scale: 3,
        min: 500,
        max: 2500,
    })
}

fn number(text: &str) -> Loose {
    Loose::Number(text.to_owned())
}

/// M-c-3's test: exact decimal decoding by integer arithmetic. `1.005 × 1000` is
/// `1004.9999999999999` in binary floating point, so a float path gives 1004.
#[test]
fn quantities_decode_exactly_at_their_scale() {
    for (written, expected) in [
        ("1.66", 1660),
        ("1.660", 1660),
        ("0.5", 500),
        ("2.5", 2500),
        ("1.005", 1005),
        ("2", 2000),
    ] {
        assert_eq!(
            decode(&metres(), &number(written)),
            Ok(AttributeValue::Quantity(ScaledInt::new(expected, 3))),
            "{written}"
        );
    }
    assert_eq!(
        decode(&metres(), &Loose::Text("1.660".to_owned())),
        Ok(AttributeValue::Quantity(ScaledInt::new(1660, 3)))
    );
    assert!(matches!(
        decode(&metres(), &number("1.6605")),
        Err(ValueError::OutOfRange { .. })
    ));
    assert!(matches!(
        decode(&metres(), &number("2.6")),
        Err(ValueError::OutOfRange { .. })
    ));
    assert!(matches!(
        decode(&metres(), &Loose::Bool(true)),
        Err(ValueError::WrongKind { .. })
    ));
}

#[test]
fn values_serialize_tagged_by_kind_and_read_back() {
    let level = AttributeValue::Level(LevelValue {
        slug: ValueSlug::new("early-bird").expect("a slug"),
        rank: 0,
    });
    let quantity = AttributeValue::Quantity(ScaledInt::new(1660, 3));
    let instant = AttributeValue::Instant(WorldTime::from_seconds(-1_072_915_200));
    assert_eq!(
        serde_json::to_string(&level).expect("encodes"),
        r#"{"level":"early-bird","rank":0}"#
    );
    assert_eq!(
        serde_json::to_string(&quantity).expect("encodes"),
        r#"{"quantity":"1.660"}"#
    );
    assert_eq!(
        serde_json::to_string(&instant).expect("encodes"),
        r#"{"instant":-1072915200}"#
    );
    for value in [level, quantity, instant] {
        let text = serde_json::to_string(&value).expect("encodes");
        assert_eq!(
            serde_json::from_str::<AttributeValue>(&text).expect("decodes"),
            value
        );
    }
    assert!(serde_json::from_str::<AttributeValue>(r#"{"flag":true,"category":"a"}"#).is_err());
}
