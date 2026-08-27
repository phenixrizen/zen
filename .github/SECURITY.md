# Security Policy

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Report it privately through GitHub Security Advisories:
[Report a vulnerability](https://github.com/phenixrizen/zen/security/advisories/new).

Please include what an attacker can achieve, a minimal reproduction (a JDM graph and input if the
issue is in evaluation), and the affected version or commit.

## What to expect

This fork is maintained by one person, so response times are best-effort rather than contractual:

| stage | realistic timeframe |
| --- | --- |
| Acknowledgement | within a week |
| Initial assessment | within two weeks |
| Fix or mitigation plan | depends on severity and complexity |

If you have not heard back in two weeks, please ping the advisory thread.

## Scope

In scope: the Rust crates in this repository — evaluation, expression handling, the database node
and its SQLite handler, and the bindings.

Particularly interested in: anything that gets untrusted graph or expression input to escape its
evaluation boundary, reach the filesystem, or reach a database outside its configured root; and
any way to get a value into SQL statement text rather than a bound parameter.

Out of scope: vulnerabilities in upstream `gorules/zen` that are not specific to this fork should
also be reported to [upstream](https://github.com/gorules/zen). Denial of service from a graph you
authored yourself is not a vulnerability — evaluation is not sandboxed against its own author.

## Supported versions

Only the latest `master` is supported. There are no backported security releases.
