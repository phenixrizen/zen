# Binary size and the 100 MB ceiling

`bindings/c` builds `libzen_ffi.a`, a static archive that gets **committed into
[`phenixrizen/zen-go`](https://github.com/phenixrizen/zen-go)** so cgo consumers can link it. That
commit is subject to GitHub's hard 100 MB per-file limit, which makes binary size a release
blocker rather than a nicety.

## Current margin

Measured from the `zen-engine-v2.1.0` build (rusqlite driver, `cargo zigbuild --all-features`):

| platform | size | headroom |
| --- | ---: | ---: |
| `darwin_arm64` | 40.5 MB | 59.5 MB |
| `darwin_amd64` | 41.8 MB | 58.2 MB |
| `windows_amd64` | 62.6 MB | 37.4 MB |
| **`linux_arm64`** | **93.4 MB** | **6.6 MB** |
| **`linux_amd64`** | **94.1 MB** | **5.9 MB** |

**The Linux targets have under 7 MB of headroom.** A dependency bump can put them back over,
and when that happens the failure is not a warning — the push is rejected outright:

```
remote: error: File deps/linux_amd64/libzen_ffi.a is 105.10 MB;
               this exceeds GitHub's file size limit of 100.00 MB
remote: error: GH001: Large files detected.
error: failed to push some refs to 'github.com:phenixrizen/zen-go.git'
```

The whole push fails, not just that file, so a release stops dead.

## Why Git LFS is not the answer

GitHub's own error suggests LFS. **It does not work for a Go module.** Go's module proxy serves
repository contents verbatim and does not resolve LFS pointers, so `go get` would hand consumers
pointer files instead of libraries and every downstream build would fail to link. The archives have
to be real files in the repository.

## Why Linux is so much larger

Linux and Windows are cross-compiled with `cargo zigbuild` against a pinned glibc
(`x86_64-unknown-linux-gnu.2.17`); macOS is built natively. The same source built natively on Linux
produces **64 MB**, versus **94 MB** through zigbuild — roughly **30 MB of zigbuild overhead**,
independent of which database driver is linked.

That overhead is the largest single lever nobody has pulled yet. It is also why local size
measurements are not a safe proxy for what CI produces: the turso driver measured 84 MB locally and
105 MB in CI, and only the second number mattered.

## What the SQLite driver costs

| build | size (native Linux) |
| --- | ---: |
| no database handler | 60 MB |
| `rusqlite` (bundled amalgamation) | 64 MB |
| turso (pure-Rust reimplementation) | 84 MB |

Real SQLite costs about 4 MB. The pure-Rust reimplementation cost about 24 MB — six times more —
which is what pushed the CI artifact past the limit in the first place. See
[`driver-choice.md`](driver-choice.md).

## The other half of the problem

Size is two problems, and staying under 100 MB only solves one.

Every release commits a **fresh full set** of five archives into `zen-go`. They change on every
build, so git cannot deduplicate them. At roughly **332 MB per release**, and with `.git` already
at **640 MB**, ten releases add ~3 GB to history — permanently, because git history is append-only,
and every `git clone` pays it.

Upstream has the same problem; this fork inherited it. Staying under the file limit does nothing
about it.

## Options, if the margin needs to grow

1. **Chase the zigbuild overhead.** ~30 MB, the biggest single win, and it costs nothing in
   capability. Nobody has investigated why the glibc-pinned build is so much larger than native.
2. **Split the archive.** Ship `libzen_ffi.a` without the database handler (~60 MB) alongside a
   separate SQLite archive (~4 MB), linked only when wanted. Fixes the ceiling properly and lets
   consumers who do not use `databaseNode` skip it. Restructures `bindings/c` into two crates.
3. **Stop committing binaries.** Publish them as release assets and fetch on install. This is the
   only option that fixes the history growth, but it breaks plain `go get`: cgo needs the archive at
   build time, so consumers would need a `go generate` step or a build tag. Real UX cost — do not do
   this without deciding the trade is worth it.

Options 1 and 2 address the ceiling. Only option 3 addresses the bleed.

## If you change anything that links into `bindings/c`

Check the size before tagging a release. A local build is not enough — measure what CI produces:

```bash
cargo zigbuild --release --all-features --target x86_64-unknown-linux-gnu.2.17 \
  --manifest-path bindings/c/Cargo.toml
ls -l target/x86_64-unknown-linux-gnu/release/libzen_ffi.a
```

Anything approaching 95 MB should be treated as a release blocker, not a warning.
