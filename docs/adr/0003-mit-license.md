# ADR 0003 — MIT license

Date: 2026-11-07
Status: accepted

## Context

The project is a tool other people will install, fork and possibly embed into
their own editor extensions. The Rust ecosystem commonly uses dual
`MIT OR Apache-2.0` licensing to get a patent grant alongside a maximally
permissive text.

## Decision

MIT, as already declared in `LICENSE` and in every `Cargo.toml`
(`license = "MIT"`).

## Consequences

+ Maximum adoption: compatible with GPLv3 projects, trivial for students to
  fork.
+ One license file, no ambiguity.
+ Matches the repository's existing license.
- No explicit patent grant (Apache-2.0 would provide one).

If a patent concern ever appears, the crates can be relicensed to
`MIT OR Apache-2.0` without breaking compatibility, since MIT is a subset of
what that dual license offers.
