# Fixture: create, check and run a world

This page is the runner's own fixture (`docs/GUIDE_FORMAT.md`). The runner executes it in a scratch folder
against the real `mineworld` binary, so a change to the command's output that is not reflected here is red.

### Step 1 — Create a world

```console
$ mineworld create seaside
created seaside: World Pack 'seaside' — 1 place(s), 2 person(s), seats first, second; 2 genesis fact(s)
next: mineworld run seaside --headless --seed 1 --days 7
```

**You should see** a new World Pack named `seaside`.

### Step 2 — Check the world

```console
$ mineworld validate seaside
seaside (seaside)
  systems    presence, movement, conversation
  places     home
  people     first, second
  seats      first, second
  settlement none

  1  home
  2  first
  3  second

2 genesis fact(s): the world's initial state, each one caused by the world coming into existence
seaside is a valid World Pack.
```

**You should see** `is a valid World Pack.` at the end.

### Step 3 — Run one day

```console
$ mineworld run seaside --headless --seed 1 --days 1
[mineworld] run seaside (seaside): seats first, second; seed 1; to day 1; pace 900s; in memory
...
activity   days 1-1  first move 44 talk 51  second move 30 talk 58
faults     0
...
wall       [..] s
```

**You should see** one activity line for day 1 and `faults     0`.
