//! The build's installed models: the [`Catalog`] trait and the [`models!`](crate::models) macro that
//! writes it (`docs/MODULE_SPEC.md` §5.1; `DECISIONS.md` `ARC-33` note of 2026-10-13).
//!
//! `cognition/installed` invokes the macro once, one line per model. The expansion is a monomorphic enum
//! — the pattern `installed!` uses for System Packs (`DEP-12`) — through which `decisions.yaml` model ids
//! resolve, parameters decode by the model's own type, and a host gets each seat's [`SeatModel`].

use core::fmt;
use std::sync::Arc;

use mineworld_sdk::Package;
use serde::Deserializer;

use crate::model::{Configured, DecisionModel, ModelId, SeatModel};

/// A model with its decoded parameters, ready to drive a seat: what a decision list resolves to.
#[derive(Clone)]
pub struct Decoded {
    model: Arc<dyn SeatModel>,
    /// The full, defaulted parameter block as JSON: equal for `{}` and an explicit default.
    canonical: String,
}

impl Decoded {
    /// `M` with `parameters`.
    ///
    /// # Panics
    ///
    /// When `M::Parameters` does not serialize as JSON, which a parameter block made of scalar fields
    /// always does.
    pub fn of<M: DecisionModel>(parameters: M::Parameters) -> Self {
        let canonical =
            serde_json::to_string(&parameters).expect("a model's parameters serialize as JSON");
        Self {
            model: Arc::new(Configured::<M>::new(parameters)),
            canonical,
        }
    }

    /// The model's id.
    pub fn id(&self) -> &ModelId {
        self.model.id()
    }

    /// The model, for a host to drive a seat with.
    pub fn model(&self) -> &Arc<dyn SeatModel> {
        &self.model
    }

    /// The parameters' canonical JSON, for the digest.
    pub fn canonical(&self) -> &str {
        &self.canonical
    }
}

impl fmt::Debug for Decoded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.id(), self.canonical)
    }
}

impl PartialEq for Decoded {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id() && self.canonical == other.canonical
    }
}

/// The models one build installs, in listed order. Written by [`models!`](crate::models); a host and
/// the decision-list reader name only this trait.
pub trait Catalog: Copy + Eq + fmt::Debug + Send + Sync + 'static {
    /// Every installed model, in listed order: in-process lines, then external ones.
    const AVAILABLE: &'static [Self];

    /// The id `decisions.yaml` names.
    fn id(self) -> &'static str;

    /// Whether the model runs out of process: it has no Rust type, joins a seat over the protocol, and
    /// must name a fallback.
    fn is_external(self) -> bool;

    /// An in-process model's package identity (`ARC-53`).
    fn package(self) -> Option<Package>;

    /// An in-process model's Rust type, for the installed set's guard test.
    fn type_name(self) -> Option<&'static str>;

    /// Decodes an in-process model's parameter block by the model's own type, at the decoder's position.
    ///
    /// # Errors
    ///
    /// The model's refusal; and always for an external model, whose parameters this build cannot type.
    fn decode<'de, D: Deserializer<'de>>(self, deserializer: D) -> Result<Decoded, D::Error>;

    /// An in-process model with its default parameters.
    fn defaults(self) -> Option<Decoded>;

    /// The installed model `id` names.
    fn find(id: &str) -> Option<Self> {
        Self::AVAILABLE
            .iter()
            .copied()
            .find(|model| model.id() == id)
    }

    /// Every installed id, for a refusal to list.
    fn listed() -> String {
        Self::AVAILABLE
            .iter()
            .map(|model| model.id())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Declares the build's installed models: one `Name => crate::Type,` line per in-process
/// [`DecisionModel`], one `external "<id>",` line per out-of-process model. Expands to `pub enum
/// Model`, `pub const AVAILABLE: &[Model]` and `impl Catalog for Model`.
///
/// ```text
/// mineworld_decision::models! {
///     Rule => mineworld_rule_controller::Rule,
///     external "lm",
/// }
/// ```
#[macro_export]
macro_rules! models {
    (@munch [$($in:tt)*] [$($ex:tt)*] external $ext:literal, $($rest:tt)*) => {
        $crate::models!(@munch [$($in)*] [$($ex)* $ext,] $($rest)*);
    };
    (@munch [$($in:tt)*] [$($ex:tt)*] $Name:ident => $ty:path, $($rest:tt)*) => {
        $crate::models!(@munch [$($in)* $Name => $ty;] [$($ex)*] $($rest)*);
    };
    (@munch [$($Name:ident => $ty:path;)*] [$($ext:literal,)*]) => {
        /// Every model this build installs, one variant per in-process line; `External` holds an
        /// out-of-process model's id.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Model {
            $(
                #[allow(missing_docs)]
                $Name,
            )*
            /// An out-of-process model, by id.
            External(&'static str),
        }

        /// Every installed model, in listed order: in-process lines, then external ones.
        pub const AVAILABLE: &[Model] = &[$(Model::$Name,)* $(Model::External($ext),)*];

        impl $crate::Catalog for Model {
            const AVAILABLE: &'static [Self] = AVAILABLE;

            fn id(self) -> &'static str {
                match self {
                    $(Self::$Name => <$ty as $crate::DecisionModel>::ID,)*
                    Self::External(id) => id,
                }
            }

            fn is_external(self) -> bool {
                matches!(self, Self::External(_))
            }

            fn package(self) -> ::core::option::Option<$crate::Package> {
                match self {
                    $(Self::$Name => ::core::option::Option::Some(
                        <$ty as $crate::DecisionModel>::PACKAGE,
                    ),)*
                    Self::External(_) => ::core::option::Option::None,
                }
            }

            fn type_name(self) -> ::core::option::Option<&'static str> {
                match self {
                    $(Self::$Name => ::core::option::Option::Some(::core::any::type_name::<$ty>()),)*
                    Self::External(_) => ::core::option::Option::None,
                }
            }

            fn decode<'de, D: $crate::__private::Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> ::core::result::Result<$crate::Decoded, D::Error> {
                match self {
                    $(Self::$Name => {
                        <<$ty as $crate::DecisionModel>::Parameters as $crate::ModelParameters>::decode(
                            deserializer,
                        )
                        .map($crate::Decoded::of::<$ty>)
                    })*
                    Self::External(id) => ::core::result::Result::Err(
                        <D::Error as $crate::__private::DeError>::custom(::std::format!(
                            "'{id}' runs out of process; this build cannot type its parameters"
                        )),
                    ),
                }
            }

            fn defaults(self) -> ::core::option::Option<$crate::Decoded> {
                match self {
                    $(Self::$Name => ::core::option::Option::Some($crate::Decoded::of::<$ty>(
                        ::core::default::Default::default(),
                    )),)*
                    Self::External(_) => ::core::option::Option::None,
                }
            }
        }
    };
    ($($line:tt)*) => {
        $crate::models!(@munch [] [] $($line)*);
    };
}
