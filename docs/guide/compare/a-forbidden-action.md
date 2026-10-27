# A forbidden action

On this page nobody may talk inside the shop. You run Corner Shop before and after, with the same seed,
and compare how much people talk.

Who may do what, and where, is the world's **Interaction List**. A world changes it with files in its
`configure` folder, and no System Pack changes ([chapter 07](../07-rules-of-the-world.md) shows it step
by step).

### Step 1.1 — Start from the finished Corner Shop

**Do this.** In your worlds folder, make a folder `compare-rules`. Copy the finished Corner Shop of
chapter 05 into it, from the folder you cloned in chapter 01. On macOS and Linux:

<!-- guide: equals compare-rules/corner-shop docs/guide/examples/corner-shop -->

```sh
mkdir compare-rules
cp -R "$HOME/MineWorld/docs/guide/examples/corner-shop" compare-rules/
```

On Windows, in PowerShell:

<!-- guide: equals compare-rules/corner-shop docs/guide/examples/corner-shop -->

```powershell
mkdir compare-rules
Copy-Item -Recurse "$HOME\MineWorld\docs\guide\examples\corner-shop" compare-rules
```

Then run it for three days:

```console
$ mineworld run compare-rules/corner-shop --headless --seed 1 --days 3
...
requests   talk accepted 324
...
activity   days 1-3  ana move 150 talk 99  ben move 156 talk 116  cai move 152 talk 109
...
```

**You should see** these numbers: everyone talked, 324 times in all.

### Step 1.2 — Tell the world it has configuration

**Do this.** Replace `compare-rules/corner-shop/world.yaml` with this whole file. The new part is
`conversation`, under `configure` at the end:

```yaml file=compare-rules/corner-shop/world.yaml
# Corner Shop: the world this guide builds.
world:
  id: corner-shop
  name: Corner Shop
  version: 0.1.0
  license: MIT

mineworld: "^0.1"

systems:
  - presence
  - movement
  - conversation
  - relationships
  - naming
  - schedule
  - item
  - inventory
  - economy
  - employment
  - consumption

places:
  - home
  - shop

population:
  - ana
  - ben
  - cai

items:
  - bread
  - tea

organizations:
  - corner-shop-co

seats:
  - ana
  - ben
  - cai

configure:
  - economy
  - conversation

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

```console exit=1
$ mineworld validate compare-rules/corner-shop
[mineworld] configure: names 'conversation', but compare-rules/corner-shop/configure/conversation.yaml does not exist
```

**You should see** the check ask for `configure/conversation.yaml`.

### Step 1.3 — Change this: forbid talking in the shop

**Do this.** In `compare-rules/corner-shop/configure`, write `conversation.yaml`:

```yaml file=compare-rules/corner-shop/configure/conversation.yaml
regions:
  shop:
    rules:
      - action: talk
        effect: forbid
```

The rule names no role, so it holds for everyone in the shop. The actions and roles a rule may name are
in the reference: [`configure/conversation.yaml`](../../reference/interaction-list.md#configureconversationyaml).

```console
$ mineworld interactions compare-rules/corner-shop --place shop
interactions  Corner Shop (corner-shop)
section  conversation:
{
  "place": "shop",
  "region": {
    "consequences": [],
    "default": "permit",
    "parameters": {
      "forget_after_days": 0,
      "gap": 300,
      "range": 3000,
      "remembered": 32
    },
    "rules": [
      {
        "action": "talk",
        "effect": "forbid",
        "selectors": [
          "*",
          "*",
          "*",
          "*"
        ]
      }
    ],
    "scoped": []
  }
}
...
```

**You should see** one rule that forbids `talk`, with `*` for every role: anyone, to anyone.

### Step 1.4 — Run it again

**Do this.** Run the same three days with the same seed:

```console
$ mineworld run compare-rules/corner-shop --headless --seed 1 --days 3
...
requests   talk accepted 273
...
activity   days 1-3  ana move 161 talk 72  ben move 165 talk 104  cai move 170 talk 97
...
```

**You should see** these numbers.

### The world changes like this

| | Step 1.1 | Step 1.4 |
| --- | --- | --- |
| `talk accepted`, everyone | 324 | 273 |
| Ana's `talk` | 99 | 72 |
| Ben's `talk` | 116 | 104 |
| Cai's `talk` | 109 | 97 |

Ana works in the shop all day, so she loses the most talk. Ben and Cai only visit the shop, so they lose
a little. Nobody tried to talk in the shop and was refused: the world never offers a forbidden action,
so a person does not pick it. The people now spend their day a little differently, so their walking
numbers move too.

Back to [all comparisons](README.md).
