//! A model's typed parameters, decoded where the decision list wrote them.
//!
//! A model whose parameters are scalar fields with literal bounds declares them with
//! `mineworld_sdk::parameters!` and decodes them with [`bounded`], so every bound is checked by the same
//! code as an Interaction List section's (`DECISIONS.md` `ARC-63` item 2; `pr-dl-a-decision-list.md`
//! SD-DLa-10). A model whose parameters do not fit that shape implements [`ModelParameters::decode`]
//! itself. Either way the decoder stands on the offending node when it refuses, so the refusal keeps its
//! line and column (`DEP-10`).

use core::fmt::{self, Debug};
use core::marker::PhantomData;

use mineworld_sdk::interactions::{Fields, Parameters};
use serde::de::{Error as _, MapAccess, Unexpected, Visitor};
use serde::{Deserializer, Serialize};

/// A model's typed, bounded parameter block, every field with a default.
pub trait ModelParameters:
    Default + Clone + PartialEq + Debug + Serialize + Send + Sync + 'static
{
    /// Decodes the block at the decoder's position, refusing an unknown key and an out-of-bound value
    /// there. A key the block omits keeps its default, so `{}` and an explicit default decode alike.
    ///
    /// # Errors
    ///
    /// The decoder's error, an unknown key, or a value outside its bound.
    fn decode<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}

/// No parameters: any key is refused.
impl ModelParameters for () {
    fn decode<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(NoParameters)
    }
}

struct NoParameters;

/// The keys a model without parameters accepts.
const NONE: &[&str] = &[];

impl<'de> Visitor<'de> for NoParameters {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an empty mapping: this model takes no parameters")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        // `unknown_field` rather than a custom message: the YAML decoder positions it at the key.
        if let Some(key) = map.next_key::<String>()? {
            return Err(A::Error::unknown_field(&key, NONE));
        }
        Ok(())
    }
}

/// Decodes a block made by `mineworld_sdk::parameters!`: its partial twin field by field through
/// [`Fields::decode_field`], which checks each bound while it stands on the value; an unknown key is
/// refused listing the known ones; then the defaults with what the block set applied over them.
///
/// # Errors
///
/// The decoder's error, an unknown key, a key written twice, or a value outside its bound.
pub fn bounded<'de, P: Parameters, D: Deserializer<'de>>(deserializer: D) -> Result<P, D::Error> {
    let partial = deserializer.deserialize_map(Bounded::<P>(PhantomData))?;
    let mut parameters = P::default();
    parameters.apply(&partial);
    Ok(parameters)
}

struct Bounded<P>(PhantomData<P>);

impl<'de, P: Parameters> Visitor<'de> for Bounded<P> {
    type Value = P::Partial;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "a mapping of parameters ({})",
            <P::Partial as Fields>::NAMES.join(", ")
        )
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<P::Partial, A::Error> {
        let mut partial = P::Partial::default();
        let mut seen: Vec<String> = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            if seen.contains(&key) {
                return Err(A::Error::custom(format!("'{key}' is written twice")));
            }
            // The SDK refuses a bound with a custom error, "'<field>' is <low> … <high>, not <value>",
            // which a YAML decoder cannot position. Restated as serde's `invalid_value`, it is given the
            // position of the key being decoded. Any other refusal (a value of the wrong type) is the
            // decoder's own and already positioned, so it passes unchanged.
            let bound = format!("'{key}' is ");
            let known = partial.decode_field(&key, &mut map).map_err(|refusal| {
                let said = refusal.to_string();
                if said.starts_with(&bound) {
                    A::Error::invalid_value(Unexpected::Other(&said), &"a value within its bound")
                } else {
                    refusal
                }
            })?;
            if !known {
                return Err(A::Error::unknown_field(&key, <P::Partial as Fields>::NAMES));
            }
            seen.push(key);
        }
        Ok(partial)
    }
}
