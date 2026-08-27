## What this changes

<!-- The behaviour difference, in a sentence or two. -->

## Why

<!-- What problem this solves. If it fixes a bug, describe how the bug manifests. -->

## Testing

<!-- How you verified this. For a bug fix, confirm the regression test fails without the fix. -->

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo test --workspace --all-features --exclude zen-ffi --exclude zen-nodejs --exclude zen-python --locked`
- [ ] `cargo test --workspace --exclude zen-ffi --exclude zen-nodejs --exclude zen-python --locked`
- [ ] For a bug fix: the new test fails without the fix

## Relationship to upstream

<!-- Delete whichever does not apply. -->

- [ ] Fork-specific — builds on this fork's own additions
- [ ] Also affects upstream `gorules/zen`
- [ ] Merges cleanly with current upstream `master`

## Notes for the reviewer

<!-- Anything surprising: a deliberate trade-off, a snapshot change and why, a follow-up you left out. -->
