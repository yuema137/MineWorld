//! Money: wallets, shops, buying, and wages.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-38`), and the **only** mover of money (`CLAUDE.md` §4
//! rule 1). Amounts are integer minor units, so a negative balance cannot be represented.
//!
//! ```text
//! configures  configure/economy.yaml   the world's one currency (ARC-111)
//! section     economy              a person's or organization's `economy: { wallet }`; an
//!                                  organization may add `shop: { at, prices }`
//! emits       currency-declared    at genesis, from the configuration; SystemInternal
//!             funded               at genesis: a holder's opening balance; visible to the holder
//!             shop-opened          at genesis: a shop in a place, run by an organization; public
//!             money-transferred    the one fact that moves money
//!             wage-unpaid          a wage-due the employer could not pay
//!             deduction-unpaid     a wage-deduction-due the employee could not pay (ARC-287)
//!             items-transferred    inventory's fact, for a purchase, through `transfer` (ARC-26)
//! owns        Wallet               on Persons and Organizations
//!             Shop                 on the Place it is in
//! provides    buy { item }         in a shop: one complete affordance per priced kind (ARC-34)
//! hears       wage-due,            employment's, decoded through their types — no system dependency
//!             wage-deduction-due   (ARC-28, ARC-287)
//!             happening-began/-ended  happenings', likewise; answered only where configured
//!             (the same file)      optional price_modifiers (WE-a); they require happenings
//! emits       economy-configured   at genesis, the resolved modifiers; SystemInternal
//!             prices-adjusted      a configured happening began: a shop's multiplier, public
//!             prices-restored      that happening ended, public
//! owns        one `price-book`     the configured modifiers, when configured
//!               Process
//! discloses   Wallet               to its holder
//!             the shop's listing   to whoever perceives its place: operator, prices, stock
//! depends on  inventory, presence
//! ```
//!
//! `happenings` states that a market day began and never touches a shop. Economy answers, where the
//! world configured it to: it states `prices-adjusted`, caused by the `happening-began`, and reduces it
//! into the shop's adjustments; every reader of a price reads [`Shop::effective_price`] (WE-a design
//! §6.2).
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
pub mod book;
pub mod code;
pub mod component;
pub mod configuration;
pub mod currency;
pub mod event;
pub mod money;
pub mod system;

pub use action::{Buy, buy_requirement, purchasable};
pub use book::{PriceBookProcess, PriceBookState};
pub use component::{Adjustment, Listed, Listing, Price, Shop, Wallet};
pub use configuration::{
    Currency, EconomyConfiguration, Permille, PriceModifier, ResolvedModifier, SymbolAt,
};
pub use currency::{CurrencyProcess, CurrencyRecord, ReferenceLine};
pub use event::{
    CurrencyDeclared, DeductionUnpaid, EconomyConfigured, Funded, MoneyTransferred, PricesAdjusted,
    PricesRestored, ShopOpened, WageUnpaid,
};
pub use money::{admit_payment, balance};
pub use section::{AuthoredEconomy, AuthoredShop};
pub use system::EconomySystem;
