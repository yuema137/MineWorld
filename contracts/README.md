# contracts/

The typed boundaries every other layer talks through: entities and components, actions and
events, observations, and network messages. Nothing here is implemented yet.

These land in the next three PRs, in this order:

```text
Entity / Component contracts
ActionIntent / Event contracts
System interface
```

What belongs here and what must never leak in is specified in
[`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md) and
[`../docs/CORE_CONCEPTS.md`](../docs/CORE_CONCEPTS.md).
