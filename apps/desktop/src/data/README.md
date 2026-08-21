# data

One accessor per data set. **Screens import from here, never from `src/fixtures/`.**

Every accessor is the seam where a fixture becomes a Tauri call. Today it returns the
fixture; at M1 the body becomes `invoke(...)` or a `Channel` subscription and the
screen does not change. That is the whole reason the indirection exists — the swap is
one edit inside the accessor, and `scripts/check-no-direct-fixture-import.sh` keeps it
that way.

Each accessor names, in a comment, the command that will replace it. The names match
the `// replaced by:` header on the fixture module it reads.
