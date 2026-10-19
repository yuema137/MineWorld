# Build your own world

This guide shows you how to make a MineWorld world of your own, from an empty folder.
You build one small world, step by step: **Corner Shop** — two rooms, three people and one shop.

Every step says **Do this**, then **You should see this**. Every command and every file in the
guide is run by the project's tests on Linux, macOS and Windows, so what you see should match.

| Chapter | What you do | About |
| --- | --- | --- |
| [01 Get MineWorld](01-get-mineworld.md) | get the `mineworld` command and run a first world | 10 min |
| [02 Your first world](02-your-first-world.md) | write a world with one room and two people | 15 min |
| [03 Two rooms](03-two-rooms.md) | add the shop and a door between the rooms | 10 min |
| [04 Three people](04-three-people.md) | give people names and a daily routine; read a person's day | 15 min |
| [05 One shop](05-one-shop.md) | goods, a company, a shop, money and a job | 20 min |
| [06 Play it](06-play-it.md) | host the world and walk around in it | 10 min |
| [07 Rules of the world](07-rules-of-the-world.md) | change who may do what, and where | 15 min |
| [08 Time and weather](08-time-and-weather.md) | give the world a date, a place on Earth and weather | 10 min |
| [09 Who decides](09-who-decides.md) | choose how each person makes up their mind | 15 min |
| [10 Forgetting](10-forgetting.md) | let liking fade between meetings | 15 min |
| 11 Happenings | a market every weekend | coming when happenings land |
| [12 Packs](12-packs.md) | see what this build has; make and use an Entity Pack | 10 min |
| [13 When something is refused](13-when-something-is-refused.md) | every refusal message in this guide, and its fix | look things up |

The finished world of chapters 02–05 is in [`examples/corner-shop/`](examples/corner-shop/world.yaml).

To see what one change does, [change one thing](compare/README.md): each page there runs Corner Shop
before and after one change, with the same seed, and shows both outputs.

Every field a file may use is listed in the generated [reference](../reference/README.md).
The words the guide uses, such as World Pack and Place, are defined in
[`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md).

For agents: the page format and the rules the test holds every page to are in
[`GUIDE_FORMAT.md`](../GUIDE_FORMAT.md).
