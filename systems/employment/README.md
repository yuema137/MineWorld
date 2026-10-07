# systems/employment/

**`mineworld-employment`**: jobs, and work as being there.

```text
section     job                  a person's `job: { employer, workplace, from, until, wage, produces }`
emits       hired                at genesis; biographical
            shift-started        at the shift's start: was the employee at the workplace
            shift-ended          at its end: seconds worked
            wage-due             at its end, if anything was worked — economy decides and pays
            items-produced       inventory's fact, for the employer, prorated by attendance
owns        Employment           on the employee; the `employed-by` edge; a `shift` Process per job
discloses   Employment           to the employee only
depends on  inventory, presence
```

Nobody has to *do* work: a person who is at the workplace during the shift is working, read from
presence when the shift starts and from `person-entered-place` while it runs. The wage is
`wage × seconds worked ÷ 3 600`, in integer minor units. This pack never touches money — it says a
wage is due, and `economy` moves it.

```sh
cargo test -p mineworld-employment
```

The decision is [`ARC-38`](../../docs/DECISIONS.md). The section format is
[`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1. Design and evidence:
[`step-10-market.md`](../../.structured-coding/plans/mvp0/step-10-market.md) §4.5 (E-C3).
