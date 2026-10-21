"""What one seat has learned about money and item names, kept for its handle (pr-s31g §5.2).

Economy discloses the world's `currency` to every living person about itself: its name, symbol, exponent,
symbol placement and every item kind's reference value. The item pack discloses the names of item kinds on a
place, in its `item-catalogue`. A `Lexicon` keeps the newest currency and every name and reference value the
seat's own observations disclosed, by item id, so a fact read while the seat is between places still names its
item. Names and reference values are configuration: they do not change in a run. This is the seat's own past
perception, like its `RefTable` (INV-S27-2), never anything another seat saw.

Nothing is invented: with no disclosed currency every `display` is `None`; with no disclosed name, `name` is
`None`. `display` is the amount in major units as a plain decimal, with no symbol, no locale and no separator
(SD-S31g-2): the symbol and its placement are world text, shown once in `View.currency`.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from mineworld_sdk.wire.ids import EntityId

from mineworld_agent.models import Amount, CurrencyLine, WorldText


def display_of(minor: int, exponent: int | None) -> str | None:
    """`minor` minor units written in major units with `exponent` digits after the point: `"4.50"` for 450 at
    exponent 2, `"450"` at 0, `"0.450"` at 3. `None` when no currency is disclosed (`exponent` is `None`)."""
    if exponent is None:
        return None
    if exponent == 0:
        return str(minor)
    scale = 10**exponent
    return f"{minor // scale}.{minor % scale:0{exponent}d}"


@dataclass(frozen=True, slots=True)
class Currency:
    """The `currency` record as read: the line a view shows, and every kind's reference value by item."""

    line: CurrencyLine
    references: dict[EntityId, int]


@dataclass(slots=True)
class Lexicon:
    """One handle's currency, item names and reference values, learned from its own observations."""

    currency: CurrencyLine | None = None
    """The newest `currency` record on me."""
    names: dict[EntityId, str] = field(default_factory=dict[EntityId, str])
    """Item kind → its name, from every `item-catalogue` my places disclosed; a name, once disclosed, is kept."""
    references: dict[EntityId, int] = field(default_factory=dict[EntityId, int])
    """Item kind → its reference value in minor units, from `currency.references`; kept the same way."""

    @property
    def exponent(self) -> int | None:
        return None if self.currency is None else self.currency.exponent

    def learn(self, currency: Currency | None, names: dict[EntityId, str]) -> None:
        """Take what one observation disclosed: called by `render` before any line is built."""
        if currency is not None:
            self.currency = currency.line
            self.references.update(currency.references)
        self.names.update(names)

    def amount(self, minor: int) -> Amount:
        """`minor` as an `Amount`, its `display` from the disclosed exponent."""
        return Amount(minor=minor, display=display_of(minor, self.exponent))

    def name(self, item: EntityId) -> WorldText | None:
        """The item kind's name as world text, or `None` when no catalogue I saw named it."""
        found = self.names.get(item)
        return None if found is None else WorldText(text=found, kind="name")

    def reference(self, item: EntityId) -> Amount | None:
        """The item kind's reference value, or `None` when the currency names none for it."""
        value = self.references.get(item)
        return None if value is None else self.amount(value)
