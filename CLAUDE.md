# CLAUDE.md

See **[AGENTS.md](AGENTS.md)** — it holds the instructions for this repository and is kept as the
single source so every agent reads the same thing.

Two constraints are worth repeating here because getting them wrong is expensive:

- **Never name the prior employer** anywhere — code, comments, commit messages, PR bodies, fixtures.
  It has been scrubbed from public history once already.
- **Binary size is a release blocker.** `libzen_ffi.a` is committed into `zen-go` against a hard
  100 MB GitHub limit and currently sits at ~94 MB. Measure before tagging; see
  [`docs/binary-size.md`](docs/binary-size.md).
