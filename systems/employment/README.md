# systems/employment/

**`mineworld-employment`**: jobs, and work as being there.

```text
section     job                  a person's `job: { employer, workplace, from, until, wage, produces,
                                 manages }`
configure   employment           lateness patience and the optional wage deduction (off by default)
emits       hired                at genesis; biographical
            shift-started        at the shift's start: was the employee at the workplace
            shift-ended          at its end: seconds worked
            wage-due             at its end, if anything was worked — economy decides and pays
            late-for-shift       arriving after the shift began (or never): to the employee and the
                                 manager; biographical
            wage-deduction-due   deduction on: at once without a manager, or from the manager's
                                 `penalize` — economy moves it
            items-produced       inventory's fact, for the employer, prorated by attendance
provides    penalize             to the employer's manager, one per open lateness, up to its bound
owns        Employment           on the employee; the `employed-by` edge; a `shift` Process per job
discloses   Employment           to the employee only; a manager's lists the late staff
depends on  inventory, presence
```

Nobody has to *do* work: a person who is at the workplace during the shift is working, read from
presence when the shift starts and from `person-entered-place` while it runs. The wage is
`wage × seconds worked ÷ 3 600`, in integer minor units. This pack never touches money — it says a
wage is due, and `economy` moves it.

```sh
cargo test -p mineworld-employment
```

Being late is a complaint by default; a world turns on deductions in `configure/employment.yaml`
([`ARC-287`](../../docs/decisions/ARC-287.md)).

The decision is [`ARC-38`](../../docs/DECISIONS.md). The section format is
[`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1. Design and evidence:
[`step-10-market.md`](../../.structured-coding/plans/mvp0/step-10-market.md) §4.5 (E-C3).
