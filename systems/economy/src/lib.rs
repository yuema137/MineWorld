//! Money: wallets, shops, buying, and wages.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-38`), and the **only** mover of money (`CLAUDE.md` §4
//! rule 1). Amounts are integer minor units, so a negative balance cannot be represented.
//!
//! ```text
//! section     economy              a person's or organization's `economy: { wallet }`; an
//!                                  organization may add `shop: { at, prices }`
//! emits       funded               at genesis: a holder's opening balance; visible to the holder
//!             shop-opened          at genesis: a shop in a place, run by an organization; public
//!             money-transferred    the one fact that moves money
//!             wage-unpaid          a wage-due the employer could not pay
//!             items-transferred    inventory's fact, for a purchase, through `transfer` (ARC-26)
//! owns        Wallet               on Persons and Organizations
//!             Shop                 on the Place it is in
//! provides    buy { item }         in a shop: one complete affordance per priced kind (ARC-34)
//! hears       wage-due             employment's, decoded through its type — no system dependency
//! discloses   Wallet               to its holder
//!             the shop's listing   to whoever perceives its place: operator, prices, stock
//! depends on  inventory, presence
//! ```
//!
//! `employment` states that a wage is due and never touches a wallet. Economy answers: it moves the
//! amount from employer to employee as a `money-transferred` caused by the `wage-due`, or records
//! `wage-unpaid` when the employer cannot pay (`CORE_CONCEPTS.md` §13.1, `ARC-28`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod offer;
mod section;

pub mod action;
pub mod component;
pub mod event;
pub mod money;
pub mod system;

pub use action::{Buy, buy_requirement, purchasable};
pub use component::{Listed, Listing, Price, Shop, Wallet};
pub use event::{Funded, MoneyTransferred, ShopOpened, WageUnpaid};
pub use money::{admit_payment, balance};
pub use section::{AuthoredEconomy, AuthoredShop};
pub use system::EconomySystem;
