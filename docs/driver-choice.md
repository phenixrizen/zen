# Why the SQLite handler uses `rusqlite` and not a pure-Rust engine

The `databaseNode` reads reference data through `core/database-sqlite`. That crate was first built
on [turso](https://github.com/tursodatabase/turso), a pure-Rust reimplementation of SQLite, and was
switched to `rusqlite` — SQLite itself, compiled from the bundled amalgamation — in
[#5](https://github.com/phenixrizen/zen/pull/5).

This note exists so the reasoning is not relitigated from scratch.

## The original goal, and why it did not survive

turso was chosen to keep C out of the build. That goal was **never actually met**: `aws-lc-rs`
already compiles C through `reqwest` → `rustls` in `core/engine`, and does so on upstream too. The
fork was paying for a purity it did not have.

## What turso cost, concretely

Four distinct production failures inside a single day of real use:

| failure | effect |
| --- | --- |
| `mimalloc` declared as a process-wide `#[global_allocator]` | Any host freeing a returned string with libc `free()` **aborted the process**. It broke plain expression evaluation — code that never touches the database node. |
| `fts` feature → tantivy → `zstd-sys` | Compiled C. The exact thing turso was chosen to avoid; the archive carried zstd object files. |
| `turso_sdk_kit` shelling out to `windres` | Broke the Windows cross-build entirely. `turso_sdk_kit` is a non-optional dependency, so it could not be feature-gated away. |
| ~24 MB of extra binary | Pushed `libzen_ffi.a` past GitHub's 100 MB limit, blocking the zen-go dependency sync. See [`binary-size.md`](binary-size.md). |

The first is the serious one. A library declaring a global allocator changes allocation for the
**entire process** it is linked into, not just its own crate. Any FFI consumer that frees a returned
pointer with the system allocator crashes — which is exactly what the cgo bindings do.

## Size

| build | size (native Linux) |
| --- | ---: |
| no database handler | 60 MB |
| `rusqlite` (bundled) | 64 MB |
| turso | 84 MB |

Real SQLite: ~4 MB. The Rust reimplementation: ~24 MB.

## Correctness

Both drivers were validated against the same differential harness — 295 real production queries run
through both the declarative `databaseNode` path and the raw SQL path, compared against a SQLite
baseline:

```
RAW      matched 295/295, errors 0
         matched 295/295
of the 232 that return rows: 232 matched
```

Identical. The swap changed no behaviour.

Beyond that corpus, SQLite has one of the most exhaustive test suites in software. turso 0.7 is a
young reimplementation that produced a process-aborting allocator bug within a day.

## What the trade actually is

`rusqlite` brings `libsqlite3-sys` and `cc` into the build — C, compiled from a pinned amalgamation
so every platform runs an identical version. That is a real cost and it was accepted deliberately.

If the pure-Rust constraint is ever reinstated as a hard requirement, the honest path is to make the
database handler an optional, separately-linked archive rather than to re-adopt an engine with these
properties.

## Design consequence

`rusqlite` is synchronous; `DatabaseHandler` is async. Queries therefore run on
`tokio::task::spawn_blocking`. A local SQLite read is short, but blocking an executor thread for it
would stall every other evaluation sharing that thread. Connections move into the blocking task and
return to the pool afterwards — they are `Send` but not `Sync`.

`render.rs` (SQL generation) and `config.rs` are driver-agnostic and were untouched by the swap,
which is why it was a small change: 6 references across 4 files.
