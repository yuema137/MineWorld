//! Economy's own refusal codes (`ARC-112`): each a `Rejection::System` code a client maps to words
//! (`CORE_CONCEPTS.md` §12.1). Literals checked while this crate compiles (`RejectionCode::from_static`).

use mineworld_contracts::{Rejection, RejectionCode};

/// A world authored an amount, and declared no currency for it (`ARC-111`).
pub const CURRENCY_UNDECLARED: RejectionCode =
    RejectionCode::from_static("economy-currency-undeclared");
/// The seller has no ask for the kind.
pub const NOT_FOR_SALE: RejectionCode = RejectionCode::from_static("not-for-sale");
/// The price the buyer submitted is not the current asking price.
pub const PRICE_CHANGED: RejectionCode = RejectionCode::from_static("price-changed");
/// The buyer holds less than the price.
pub const INSUFFICIENT_FUNDS: RejectionCode = RejectionCode::from_static("insufficient-funds");
/// The seller holds none of the kind.
pub const OUT_OF_STOCK: RejectionCode = RejectionCode::from_static("out-of-stock");
/// The buyer cannot carry one more.
pub const CANNOT_CARRY: RejectionCode = RejectionCode::from_static("cannot-carry");
/// The kind already has an ask: re-price it instead.
pub const ALREADY_LISTED: RejectionCode = RejectionCode::from_static("already-listed");
/// A person may list only a kind it holds.
pub const NOT_HELD: RejectionCode = RejectionCode::from_static("not-held");
/// A new price equal to the ask changes nothing.
pub const PRICE_UNCHANGED: RejectionCode = RejectionCode::from_static("price-unchanged");
/// A price or reference value of 0 or above `PRICE_MAX` (10^12 minor units).
pub const PRICE_OUT_OF_RANGE: RejectionCode = RejectionCode::from_static("price-out-of-range");
/// A proposal would break a limit (`ARC-118`): its detail is `already-proposed` (the buyer has a live
/// proposal for the kind to the seller) or `seller-full` (the seller holds eight live proposals).
pub const PROPOSAL_LIMIT: RejectionCode = RejectionCode::from_static("proposal-limit");
/// No open proposal has this id: unknown, answered, withdrawn, declined or expired (`ARC-118`).
pub const NO_SUCH_PROPOSAL: RejectionCode = RejectionCode::from_static("no-such-proposal");
/// A person or organization authored with no `wallet`.
pub const WALLET_UNRESOLVED: RejectionCode =
    RejectionCode::from_static("economy-wallet-unresolved");
/// A `shop` on a person's file: only an organization runs one.
pub const SHOP_ON_PERSON: RejectionCode = RejectionCode::from_static("economy-shop-on-person");
/// `asks` on an organization's file: it lists through its shop's `prices`.
pub const ASKS_ON_ORGANIZATION: RejectionCode =
    RejectionCode::from_static("economy-asks-on-organization");
/// `reference` on a person's or organization's file: a reference value belongs to an item kind.
pub const REFERENCE_ON_HOLDER: RejectionCode =
    RejectionCode::from_static("economy-reference-on-holder");
/// An item file's `economy:` carrying anything but `reference`.
pub const ITEM_REFERENCE_ONLY: RejectionCode =
    RejectionCode::from_static("economy-item-reference-only");
/// A keeper named twice in one shop.
pub const KEEPER_DUPLICATE: RejectionCode = RejectionCode::from_static("economy-keeper-duplicate");
/// More than eight keepers in one shop.
pub const KEEPERS_TOO_MANY: RejectionCode = RejectionCode::from_static("economy-keepers-too-many");

/// The refusal carrying `code`, with no detail.
pub(crate) const fn refusal(code: RejectionCode) -> Rejection {
    Rejection::System { code, detail: None }
}
