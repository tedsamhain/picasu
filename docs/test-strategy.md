# Test Strategy

## Philosophy

Test logic the compiler cannot verify. Don't test what the type system, the framework,
or the serialization library already guarantee.

In practice this means:
- **Test pure functions** with non-trivial branches or invariants (filters, transforms, priority logic).
- **Test schema contracts** at boundaries where silent corruption is possible (bitcode positional encoding).
- **Test integration paths** where multiple components interact in ways the types don't constrain (index → dedup → flush → album update).
- **Don't test** Rocket routing, redb reads/writes, bitcode round-trips on unchanged structs, or Vue rendering internals.

---

## Current state

| Layer | Tool | Status |
|---|---|---|
| Backend format | `cargo fmt --check` | ✅ in precommit |
| Backend lint | `cargo clippy -- -D warnings` | ✅ in precommit |
| Backend tests | `cargo test` (4 tests in `router/builder.rs`) | ✅ in precommit — minimal coverage |
| Frontend format | `prettier --check` | ✅ in precommit |
| Frontend types | `vue-tsc --noEmit` | ✅ in precommit |
| Frontend lint | `eslint` (strictTypeChecked + vue strongly-recommended) | ✅ in precommit |
| Frontend tests | — | ❌ none |
| Security audit | — | ❌ not wired up |
| E2E | — | ❌ not started |

---

## Backend

### Unit tests

Target: pure functions with no I/O, no DB, no server. These live in `#[cfg(test)]` blocks
in the same file as the code under test and run with `cargo test` / `cargo nextest`.

Priority targets:

| Function | File | Why |
|---|---|---|
| `prettify_dir_name` | `dir_album.rs` | Pure string transform; edge cases around separators, casing, unicode |
| Schema version dispatch | `ser_de.rs` | Silent corruption if the `[0xFF, version]` prefix logic regresses; encode/decode round-trips per version |
| `Expression` filter predicates | `generate_filter.rs` | Filters are composed at runtime; incorrect predicate logic silently returns wrong results |
| `compute_timestamp` | `abstract_data.rs` | Priority logic across EXIF, file, and fallback timestamps |
| `belongs_to_album` path-prefix branch | `combined.rs` | The dir-vs-manual discriminator; path-prefix semantics are subtle |

### Integration tests

Target: multi-component flows where the types don't constrain the interaction.
redb uses a single embedded file, so a test can spin up a real DB in a `tempdir` — no
mocking needed, and no separate process required.

Priority flows:

- Index file → deduplication → `FlushTreeTask` → album `self_update` round-trip
- Dir album creation and path-prefix membership (file inside subtree vs. outside)
- Schema migration: write a v1-encoded record directly (raw bytes), read it back through
  `from_bytes`, verify the promoted `dir_path: None` field is present

### Tooling gaps

- **cargo nextest**: drop-in replacement for `cargo test`; faster parallel execution,
  better output. Replace `cargo test` in the `justfile` once installed.
- **cargo audit**: CVE scan of the dependency tree. Run locally and in CI.
- **cargo deny**: policy enforcement for licenses, duplicate deps, and CVEs via `deny.toml`.
- **clippy `unwrap_used`**: currently allowed everywhere. Promote to `warn` outside test
  modules to surface latent panics.

---

## Frontend

### Unit tests — Vitest

The lexer (`src/script/lexer/`) is the highest-value first target: it has a hand-written
grammar (Chevrotain), is pure logic with no DOM dependency, and is complex enough that
regressions are non-obvious. Pinia store reducers are the next target — pure state
transforms, no component rendering needed.

Vitest integrates with the existing Vite config with minimal setup (`vite.config.ts`
already present).

### Dead code — knip

Run `npx knip` periodically to find unused exports and imports. Not suitable for
precommit (generates noise on in-progress refactors); better as an occasional manual
sweep or CI step.

### Security — npm audit

`npm audit` should run in CI on every PR. Not in precommit: too slow, and the existing
`overrides` in `package.json` mean some findings are known and intentional.

### E2E — Playwright (deferred)

Requires a running backend, so not practical in precommit. Intended for CI only.
Golden-path smoke tests: login → gallery view → open image → share link flow.

---

## What is not tested and why

| Area | Reason |
|---|---|
| Rocket route definitions | Framework handles dispatch; type-checked at compile time |
| redb read/write primitives | Covered by redb's own test suite |
| bitcode round-trips on unchanged structs | Covered by bitcode's own test suite; only the *version dispatch logic* needs testing |
| Vue component rendering | Deferred to E2E; unit-testing rendering adds maintenance cost without proportional value |
| Individual Pinia getters backed by a single field | No logic to test; the type system covers it |
