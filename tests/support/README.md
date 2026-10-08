# mineworld-test-support

Scratch directories for MineWorld's own tests. A test that writes a save or a pack copy asks for one
and gets it removed when the test ends:

```rust
let save = mineworld_test_support::scratch!("run-300-a"); // a path that does not exist yet
let pack = mineworld_test_support::scratch!(empty "my-pack"); // an existing, empty directory
```

Set `MINEWORLD_KEEP_SCRATCH=failed` (or `all`) to keep them for inspection.

The rules are in [`docs/ENGINEERING_STANDARDS.md`](../../docs/ENGINEERING_STANDARDS.md) §22, "Test
scratch"; why this is our own code rather than `tempfile` is `DEP-29` in
[`docs/DECISIONS.md`](../../docs/DECISIONS.md).
