# Documentation

Notes on decisions and constraints specific to this fork. For engine and JDM documentation, the
upstream project's docs remain the reference.

| document | subject |
| --- | --- |
| [`binary-size.md`](binary-size.md) | The 100 MB per-file ceiling that gates releases, current margins per platform, why Git LFS cannot help, and the options for widening the margin |
| [`driver-choice.md`](driver-choice.md) | Why the SQLite handler uses `rusqlite` rather than a pure-Rust reimplementation, and what the pure-Rust attempt actually cost |

Agent instructions live in [`../AGENTS.md`](../AGENTS.md).
