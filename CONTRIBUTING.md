# Contributing

Contributions are welcome. This is a maintained fork of
[`gorules/zen`](https://github.com/gorules/zen); upstream states it cannot accept code
contributions, so this fork exists to take them.

Maintained by Phenix Rizen (Nathan Rockhold).

## Scope: what belongs here

| belongs in this fork | send upstream instead |
| --- | --- |
| Fixes upstream has not responded to | Anything you would rather see in the canonical project first |
| The fork's own additions (`databaseNode`, `zen-database-sqlite`, `$params`) | JDM standard/spec changes — GoRules owns the standard |
| Correctness fixes with a regression test | — |

A bug that reproduces on upstream is worth reporting
[there](https://github.com/gorules/zen/issues) too. Both projects benefit, and this fork tracks
upstream `master`.

## Development setup

Requires a stable Rust toolchain. Nothing else — the SQLite handler is pure Rust
([Turso](https://github.com/tursodatabase/turso)), so there is no C compiler or vendored
amalgamation in the build.

```bash
git clone https://github.com/phenixrizen/zen.git
cd zen
cargo build --workspace
```

## The test gate

Run all four commands before opening a PR. CI runs the same ones.

```bash
# 1. Formatting.
cargo fmt --all -- --check

# 2. Full workspace, all features.
cargo test --workspace --all-features \
  --exclude zen-ffi --exclude zen-nodejs --exclude zen-python --locked

# 3. Again with default features. CI runs both; feature unification means
#    they do not exercise the same code.
cargo test --workspace \
  --exclude zen-ffi --exclude zen-nodejs --exclude zen-python --locked

# 4. WASM still builds.
cargo check -p zen-engine --target wasm32-wasip1-threads --all-features
```

### Things that will trip you up

- **`arbitrary_precision` changes numeric output.** With it off, numbers serialize through `f64`.
  Some upstream fixtures (`customer-lifetime-value.json`) encode 28-digit decimal expectations and
  can only pass with the feature on, which is why step 2 and step 3 differ. If you touch numeric
  serialization, run both.
- **Snapshot tests.** `cargo insta` review if you change engine output. A PR should not carry
  unexplained `.snap` changes.
- **Tests that need reference data skip by default.** `zen-database-sqlite`'s differential suites
  read `ZEN_REFDATA_DIR` and `ZEN_CORPUS_MANIFEST`. Unset means "no bundle available" and they
  skip. Never commit a `.db` fixture — build read-only fixtures at test time from a committed
  `.sql`.

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/). Release automation parses them, so
the prefix determines the version bump.

```
fix(expression): honour TZ when resolving the local timezone
feat(engine): add database node and handler extension point
test(database-sqlite): require an explicit refdata bundle path
```

Explain *why* in the body, not just what. A reviewer can read the diff; they cannot read your
reasoning.

## Pull requests

- Branch off `master`.
- One logical change per PR.
- A bug fix needs a regression test, and you should confirm the test **fails without the fix**.
  State that you did.
- Do not commit absolute paths, machine-specific defaults, or generated artifacts.

## Tracking upstream

```bash
git remote add upstream https://github.com/gorules/zen.git
git fetch upstream
git merge upstream/master
```

Keep fork-specific changes additive where practical, so upstream merges stay cheap.
