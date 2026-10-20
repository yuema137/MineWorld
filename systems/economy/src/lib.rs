//! Money: wallets, shops, buying, and wages.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-38`), and the **only** mover of money (`CLAUDE.md` §4
//! rule 1). Amounts are integer minor units, so a negative balance cannot be represented.
//!
//! ```text
//! configures  configure/economy.yaml   the world's one currency (ARC-111)
//! section     economy              a person's or organization's `economy: { wallet }`; a person may
//!                                  add `asks`, an organization `shop: { at, prices, keepers }`; an
//!                                  item kind carries `{ reference }` only (ARC-112, ARC-113)
//! emits       currency-declared    at genesis, from the configuration; SystemInternal
//!             funded               at genesis: a holder's opening balance; visible to the holder
//!             reference-declared   at genesis: an item kind's reference value; SystemInternal
//!             shop-opened          at genesis: a shop in a place, run by an organization; public
//!             listed               an ask: at genesis, or from list-for-sale
//!             price-set, withdrawn from set-price and withdraw
//!             sold                 a sale, beside its money-transferred and items-transferred
//!             money-transferred    the one fact that moves money
//!             wage-unpaid          a wage-due the employer could not pay
//!             deduction-unpaid     a wage-deduction-due the employee could not pay (ARC-287)
//!             items-transferred    inventory's fact, for a purchase, through `transfer` (ARC-26)
//! owns        Wallet               on Persons and Organizations
//!             Asks                 on a seller: a Person, or a shop's operating Organization
//!             Shop                 on the Place it is in: operator, keepers, adjustments
//!             ReferenceValue       on an item kind; never charged
//!             one `economy-currency` Process: the declared currency
//! provides    buy { item, price }  one complete affordance per ask, in a shop or of a person seller
//!             list-for-sale, set-price, withdraw   incomplete; `for: shop` by a keeper (ARC-112)
//!             propose-price, accept-proposal, decline-proposal, withdraw-proposal   incomplete:
//!                                  haggling; acceptance settles as a buy at the proposed price,
//!                                  nothing reserved (ARC-118)
//! emits       price-proposed, proposal-declined, proposal-withdrawn, proposal-accepted
//!                                  heard by the buyer and whoever may answer, nobody else
//! owns        Proposals            on a seller: its live proposals, lazily expiring (ARC-118)
//! discloses   proposals            incoming and outgoing, about the observer itself (ARC-118)
//! hears       wage-due,            employment's, decoded through their types — no system dependency
//!             wage-deduction-due   (ARC-28, ARC-287)
//!             happening-began/-ended  happenings', likewise; answered only where configured
//!             (the same file)      optional price_modifiers (WE-a); they require happenings
//! emits       economy-configured   at genesis, the resolved modifiers; SystemInternal
//!             prices-adjusted      a configured happening began: a shop's multiplier, public
//!             prices-restored      that happening ended, public
//! owns        one `price-book`     the configured modifiers, when configured
//!               Process
//! discloses   Wallet, wealth       to the holder only (ARC-114)
//!             currency             to every living Person, about itself (ARC-111)
//!             asks                 a person seller's asks, available or not, to whoever perceives it
//!             the shop's listing   to whoever perceives its place: operator, prices, stock
//! depends on  inventory, presence
//! ```
//!
//! `happenings` states that a market day began and never touches a shop. Economy answers, where the
//! world configured it to: it states `prices-adjusted`, caused by the `happening-began`, and reduces it
//! into the shop's adjustments; every reader of a price applies [`Shop::adjust`] to the operator's ask
//! (WE-a design §6.2; `ARC-112`).
//!
//! `employment` states that a wage is due and never touches a wallet. Economy answers: it moves the
//! amount from employer to employee as a `money-transferred` caused by the `wage-due`, or records
//! `wage-unpaid` when the employer cannot pay (`CORE_CONCEPTS.md` §13.1, `ARC-28`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod listing;
mod offer;
mod sale;
mod section;

pub mod action;
pub mod book;
pub mod code;
pub mod component;
pub mod configuration;
pub mod currency;
pub mod event;
pub mod haggle;
pub mod money;
pub mod proposal;
pub mod system;

pub use action::{
    Buy, ListForSale, SetPrice, ShopSide, TRADE_RANGE, Withdraw, buy_requirement,
    person_buy_requirement, seller_requirement,
};
pub use book::{PriceBookProcess, PriceBookState};
pub use component::{
    Adjustment, Ask, AskLine, Asks, AsksView, Listed, Listing, MAX_KEEPERS, ReferenceValue, Shop,
    Wallet, Wealth,
};
pub use configuration::{
    Currency, EconomyConfiguration, Permille, PriceModifier, ResolvedModifier, SymbolAt,
};
pub use currency::{CurrencyProcess, CurrencyRecord, ReferenceLine};
// `event::Listed` (the fact `listed`) is reached through its module: the crate root's `Listed` is the
// shop listing's line, which other packs already read (S31-c §5.9).
pub use event::{
    CurrencyDeclared, DeductionUnpaid, EconomyConfigured, Funded, MoneyTransferred, PriceProposed,
    PriceSet, PricesAdjusted, PricesRestored, ProposalAccepted, ProposalDeclined, ProposalTerms,
    ProposalWithdrawn, ReferenceDeclared, ShopOpened, Sold, WageUnpaid, Withdrawn,
};
pub use haggle::{AcceptProposal, DeclineProposal, ProposePrice, WithdrawProposal};
pub use money::{PRICE_MAX, admit_payment, balance, price_in_range};
pub use proposal::{
    PROPOSAL_VALID_FOR, PROPOSALS_PER_SELLER, Proposal, ProposalId, ProposalLine, ProposalStatus,
    Proposals, ProposalsView,
};
pub use section::{AuthoredEconomy, AuthoredShop};
pub use system::EconomySystem;
