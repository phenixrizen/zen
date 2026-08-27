# AGENTS.md

Instructions for AI agents working in this repository.

## What this is

`phenixrizen/zen` is a **maintained fork** of [`gorules/zen`](https://github.com/gorules/zen), a
business rules engine that evaluates JDM (JSON Decision Model) documents. It is maintained by
Phenix Rizen (Nathan Rockhold) because upstream states it cannot accept code contributions.

Related repositories, all forks maintained alongside this one:

| repo | what it is |
| --- | --- |
| [`phenixrizen/zen-go`](https://github.com/phenixrizen/zen-go) | Go binding. Vendors prebuilt `libzen_ffi.a` archives built here. |
| [`phenixrizen/zen-ios`](https://github.com/phenixrizen/zen-ios) | Swift package. Vendors an XCFramework built here. |

Both are **distribution packages**: their binary contents are build artifacts pushed by workflows in
this repository. A change to evaluation belongs here, not there.

## Hard constraints

**Never name the prior employer.** Not in code, comments, commit messages, PR bodies, issue
templates, test fixtures, or tooling. This has been scrubbed from history once already; do not
reintroduce it. Corpus-specific migration tooling lives in a private repository, not here.

**Binary size is a release blocker.** `libzen_ffi.a` is committed into `zen-go` and subject to
GitHub's hard 100 MB per-file limit. The Linux targets currently sit at ~94 MB — under 7 MB of
headroom. If you change anything that links into `bindings/c`, measure the size before tagging.
See [`docs/binary-size.md`](docs/binary-size.md).

**Never commit a `.db` fixture.** Build read-only fixtures at test time from a committed `.sql`.

## Verifying changes

Run all of these. CI runs the same ones.

```bash
cargo fmt --all -- --check

TZ=UTC cargo test --workspace --all-features \
  --exclude zen-ffi --exclude zen-nodejs --exclude zen-python --locked

TZ=UTC cargo test --workspace \
  --exclude zen-ffi --exclude zen-nodejs --exclude zen-python --locked

cargo check -p zen-engine --target wasm32-wasip1-threads --all-features
```

### Things that will mislead you

- **`arbitrary_precision` changes numeric output.** With it off, numbers serialize through `f64`.
  Some upstream fixtures encode 28-digit decimal expectations and only pass with it on, which is why
  the two test commands above differ. If you touch numeric serialization, run both.
- **The database differential suites skip unless pointed at a corpus.** They read `ZEN_CATALOG_DIR`
  and `ZEN_CORPUS_MANIFEST`. Unset means "no bundle" and they skip — a green run does not mean they
  executed.
- **A local build is not a proxy for CI.** Linux and Windows are cross-compiled with
  `cargo zigbuild`, which adds ~30 MB over a native build. Size measured locally will understate the
  artifact by a wide margin.

## Conventions

- [Conventional Commits](https://www.conventionalcommits.org/). Release automation parses them, so
  the prefix determines the version bump.
- **Squash merges only.** Merge commits caused every changelog entry to appear two or three times in
  the 2.1.0 release. The repository is configured squash-only; do not work around it.
- A bug fix needs a regression test, and you should confirm it **fails without the fix**. Say so.
- Explain *why* in commit bodies. A reviewer can read the diff; they cannot read your reasoning.

## Publishing

Packages publish under the fork's own names so they never collide with upstream:

| registry | package |
| --- | --- |
| crates.io | `phenixrizen-zen-engine`, `-expression`, `-types`, `-tmpl`, `-macros`, `-database-sqlite` |
| npm | `@phenixrizen/zen-engine` plus per-platform packages |
| PyPI | `phenixrizen-zen-engine` |

The Rust crates keep their original **library** names, so `use zen_engine::…` is unchanged — only
the dependency line moves.

Releases are cut by release-please: merging its release PR creates tags, and the tags trigger the
publish workflows. Authentication is a **GitHub App token**, not a PAT — `GITHUB_TOKEN` cannot be
used because GitHub refuses to let events it creates trigger other workflows, so the tag would be
cut and nothing would publish.

Cross-repository pushes to `zen-go` and `zen-ios` use **per-repository deploy keys**, each scoped to
write to exactly one repository.

Maven Central and NuGet publishing is **disabled** behind the `PUBLISH_JVM_DOTNET` repository
variable, because those credentials are not configured.

## Further reading

| document | subject |
| --- | --- |
| [`docs/binary-size.md`](docs/binary-size.md) | The 100 MB ceiling, current margins, and the options for widening them |
| [`docs/driver-choice.md`](docs/driver-choice.md) | Why the SQLite handler uses `rusqlite` rather than a pure-Rust engine |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Development setup and the test gate |
