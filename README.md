> [!NOTE]
> ## This is a maintained fork
>
> **`phenixrizen/zen` is a community-maintained fork of [`gorules/zen`](https://github.com/gorules/zen), maintained by Phenix Rizen (Nathan Rockhold).**
>
> It exists because upstream states it cannot accept code contributions, and submitted fixes go
> unreviewed in practice — including a straightforward timezone defect fix with a regression test,
> which has sat without a response.
>
> This fork tracks upstream `master` and **does accept contributions**. See
> [CONTRIBUTING.md](CONTRIBUTING.md).
>
> ### What this fork adds
>
> | addition | what it does |
> | --- | --- |
> | `databaseNode` + `DatabaseHandler` | A first-class node for looking reference data up from a decision graph, with a host extension point. Values are always bound, never interpolated. |
> | `zen-database-sqlite` | A **pure-Rust** SQLite handler built on [Turso](https://github.com/tursodatabase/turso) — no C, no vendored amalgamation, no `cc` in your build. |
> | Decision-level `$params` | Static parameters supplied per decision and reachable from switch, expression, and function nodes. |
> | `TZ` is honoured | `local` timezone resolution respects the `TZ` environment variable instead of only `/etc/localtime`. |
> | Exact fractional numbers | Fixes silent truncation of every non-integer value when `arbitrary_precision` is off. |
>
> ### Packages
>
> This fork publishes under its own names, so it never collides with upstream:
>
> | | package |
> | --- | --- |
> | Rust | `phenixrizen-zen-engine` (also `-expression`, `-types`, `-tmpl`, `-macros`) |
> | Node.js | `@phenixrizen/zen-engine` |
> | Python | `phenixrizen-zen-engine` |
> | .NET | `PhenixRizen.ZenEngine` |
>
> The Rust crates keep their original library names, so `use zen_engine::…` is unchanged — only
> the dependency line moves.
>
> Not affiliated with or endorsed by GoRules. MIT licensed, same as upstream, with the original
> copyright retained in [LICENSE](LICENSE).

# ZEN Engine

**Business logic humans can read and machines can run.** One copy of your rules: the owner reads it, every system runs it.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![crates.io](https://img.shields.io/crates/v/phenixrizen-zen-engine.svg)](https://crates.io/crates/phenixrizen-zen-engine)
[![npm](https://img.shields.io/npm/v/@phenixrizen/zen-engine.svg)](https://www.npmjs.com/package/@phenixrizen/zen-engine)
[![PyPI](https://img.shields.io/pypi/v/phenixrizen-zen-engine.svg)](https://pypi.org/project/phenixrizen-zen-engine/)

ZEN Engine is a cross-platform, open-source Business Rules Engine (BRE) written in **Rust**, with native bindings for **Node.js**, **Python**, **Go**, **Java**, **Kotlin** and **.NET**, plus iOS and Android packages. Decisions evaluate in microseconds, run identically on every platform, and are stored as portable JSON. Loading the JSON is up to you: file system, database or service call.

## Rules that read like sentences

Conditions are written the way the business says them, in the ZEN Expression Language. The developer view is one toggle away, and the two can never drift apart: there is only one source of truth, and this engine runs it.

<img width="1280" alt="Readable rules" src=".github/images/tables.png">

## Rules as graphs, or as documents

Model a decision on a visual canvas of decision tables, switches, expressions, functions and reusable sub-decisions. Or write it as a policy document with prose, typed data models and tables. Both compile to the same engine and return the same answers.

<img width="1280" alt="Graphs and documents" src=".github/images/graphs-docs.png">

JDM is the JSON Decision Model: the document format this engine loads and evaluates. A JDM document is either a **graph** (decision tables, switches, expressions, functions and reusable sub-decisions on a canvas) or a **policy** (prose, typed data models and tables). Both compile to the same engine and return the same answers.

## What's new in 2.0

Version 2.0 is the first stable release of the new engine line:

- **Policy documents**: model decisions as readable documents with typed data models, expressions, decision tables, match blocks and assertions. Policies compile to the same engine as graphs and return the same answers.
- **Workspace analysis**: static type checking across policies and graphs. Type flow, exhaustiveness checking, write-conflict detection and precise diagnostics, all available before anything runs.
- **Per-column collect**: decision table output columns can collect across all matching rows (`tags[]`) while the rest of the table stays first-match.
- **Pre-compiled engine**: decisions are parsed and compiled once at load; evaluation is allocation-light and repeat-safe.
- **Hardened runtime**: out-of-range numbers, arithmetic overflow and malformed inputs return errors or nulls instead of crashing the process.
- **Unified bindings**: configurable loaders, batch evaluation and consistent error envelopes across Node.js, Python, Go and FFI consumers.

> [!IMPORTANT]
> **Migrating from 0.x (Rust crates):** `arbitrary_precision` is no longer enabled by default in zen-engine, zen-expression, zen-types and zen-tmpl. If you rely on arbitrary-precision number handling, add `features = ["arbitrary_precision"]` to your dependency. Bindings (Node.js, Python, C, UniFFI) are unaffected, they opt in automatically.

## Quickstart

### Rust

```toml
[dependencies]
phenixrizen-zen-engine = "2"
```

```rust
use serde_json::json;
use std::sync::Arc;
use zen_engine::model::DecisionContent;
use zen_engine::DecisionEngine;

async fn evaluate() {
    let decision_content: DecisionContent =
        serde_json::from_str(include_str!("jdm_graph.json")).unwrap();
    let engine = DecisionEngine::default();
    let decision = engine.create_decision(Arc::new(decision_content)).unwrap();

    let result = decision.evaluate(json!({ "input": 12 }).into()).await;
}
```

### Node.js

```bash
npm i @phenixrizen/zen-engine
```

```typescript
import { ZenEngine } from '@phenixrizen/zen-engine';
import fs from 'fs/promises';

const content = await fs.readFile('./jdm_graph.json');
const engine = new ZenEngine();

const decision = engine.createDecision(content);
const result = await decision.evaluate({ input: 15 });
```

### Python

```bash
pip install phenixrizen-zen-engine
```

```python
import zen

with open("./jdm_graph.json", "r") as f:
    content = f.read()

engine = zen.ZenEngine()

decision = engine.create_decision(content)
result = decision.evaluate({"input": 15})
```

Full guides, including loaders for multi-decision graphs and batch evaluation:

* **Node.js** — [source](bindings/nodejs/README.md) | [npm](https://www.npmjs.com/package/@phenixrizen/zen-engine)
* **Python** — [source](bindings/python/README.md) | [PyPI](https://pypi.org/project/phenixrizen-zen-engine/)
* **Go** — [phenixrizen/zen-go](https://github.com/phenixrizen/zen-go)
* **Java / Kotlin** — [source](bindings/uniffi)
* **.NET** — [source](bindings/uniffi) | [NuGet](https://www.nuget.org/packages/PhenixRizen.ZenEngine)
* **Rust (core)** — [source](core/engine) | [crates.io](https://crates.io/crates/phenixrizen-zen-engine)


## Support matrix

| Arch             | Rust               | Node.js            | Python             | Go                 | Java / Kotlin      | .NET               |
|:-----------------|:-------------------|:-------------------|:-------------------|:-------------------|:-------------------|:-------------------|
| linux-x64-gnu    | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| linux-arm64-gnu  | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| darwin-x64       | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| darwin-arm64     | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| win32-x64-msvc   | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| linux-x64-musl   | :heavy_check_mark: | :heavy_check_mark: | :x:                | :x:                | :x:                | :x:                |
| linux-arm64-musl | :heavy_check_mark: | :heavy_check_mark: | :x:                | :x:                | :x:                | :x:                |
| linux-s390x      | :heavy_check_mark: | :x:                | :x:                | :x:                | :heavy_check_mark: | :x:                |
| wasm32 (WASI)    | :heavy_check_mark: | :heavy_check_mark: | :x:                | :x:                | :x:                | :x:                |

Mobile: **Swift (iOS XCFramework)** and **Android (AAR)** packages are published from the same core via UniFFI.

## Contribution

**Contributions are welcome here.** This fork exists partly because upstream cannot take them.

Read [CONTRIBUTING.md](CONTRIBUTING.md) for the development setup and the full test gate. In short:

```bash
cargo fmt --all -- --check
cargo test --workspace --all-features \
  --exclude zen-ffi --exclude zen-nodejs --exclude zen-python --locked
cargo test --workspace \
  --exclude zen-ffi --exclude zen-nodejs --exclude zen-python --locked
```

Security issues should not go in a public issue — see [SECURITY.md](.github/SECURITY.md).

A bug that also reproduces on upstream `gorules/zen` is worth reporting
[there](https://github.com/gorules/zen/issues) as well, so both projects benefit.

The JDM standard itself is GoRules'. This fork aims to stay compatible with it rather than diverge.

## License

[MIT License](https://opensource.org/licenses/MIT)
