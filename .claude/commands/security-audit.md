---
description: Run cargo audit, cargo deny, and npm audit; triage each finding by severity and relevance to this project; apply fixes where safe; suppress known-unfixable issues with documented justification and enforcement tests.
allowed-tools: Bash, Read, Edit, Write
---

# Security Audit — picasu

Run the full audit suite for this Rust/Vue gallery project and work through every finding.

## Step 1 — Run all audits

Run in parallel:

```bash
cd backend && cargo audit 2>&1
cd backend && cargo deny check 2>&1
cd frontend && npm audit 2>&1
cd frontend && npx npm-check-updates --format group 2>&1
```

Collect all output before proceeding.

## Step 2 — Triage each finding

For every advisory, vulnerability, or outdated package, assess:

**Severity** — use the published CVSS score where available; for cargo deny warnings (duplicates, stale entries) it is informational.

**Relevance** — the single most important question. A critical advisory for an RSA timing attack is irrelevant if only HMAC algorithms are used. Trace the code path:
- Which code in *this repo* calls into the affected crate?
- Is the vulnerable code path reachable? (Check call sites with `grep -rn`.)
- Does the attack require attacker-controlled input that this app actually exposes?
- Is it a dev/build-only dependency with no runtime exposure?

**Mitigation options**, in priority order:
1. Upgrade to a patched version (check `cargo update --dry-run` / `npm outdated`)
2. Remove the dependency if unused
3. Replace with an alternative that does not carry the vulnerability
4. Suppress with documented justification if unfixable and not reachable (see Step 4)

Do not suppress without first confirming the code path is unreachable.

## Step 3 — Apply safe fixes

**npm:** If `npm-check-updates` shows available updates:
- Bump all patch and minor versions: `npx npm-check-updates -u && npm install && npm audit fix`
- Verify tests pass: `just frontend-test`
- Verify type check: `just frontend-check`

**Cargo:** For patchable advisories:
- Run `cargo update` (patch-only by default) and verify `cargo audit` clears the advisory
- For major-version upgrades, assess breaking changes before updating `Cargo.toml`
- Run `just backend-test` after any change

**cargo deny stale entries:** Remove license allowances and exceptions that are no longer in the dependency tree (the warning `license-not-encountered` identifies them). Re-run `cargo deny check` to confirm no new errors are introduced.

## Step 4 — Suppress genuinely unfixable issues

Only suppress when:
- No patched version exists (`cargo audit` says "No fixed upgrade is available!")
- The vulnerable code path is confirmed unreachable in this project
- The dependency cannot be removed or replaced without significant refactoring

**For `cargo audit`:** Add to `backend/.cargo/audit.toml`:
```toml
[advisories]
ignore = ["RUSTSEC-YYYY-NNNN"]  # brief reason comment
```

**For `cargo deny`:** Add to `deny.toml` advisories ignore block with reason.

**Enforcement test (required for every suppression):** Write a test that would fail if the suppressed assumption is violated. Examples:
- Algorithm suppressed because HMAC-only: assert the JWT `Header::default().alg == Algorithm::HS256`
- Dependency suppressed because build-time only: add a compile-time cfg check or doc note
- Place the test in the module closest to the suppressed code path; include a comment citing the advisory ID

## Step 5 — cargo deny housekeeping

After fixing advisories, check for other deny issues:
- **Duplicate crate versions** (`warn`): note which are transitive (Rocket epoch, etc.) vs. ones we can resolve; log unresolvable ones
- **License issues**: add new licenses that appear in the dep tree; remove ones no longer present
- Run `cargo deny check` and confirm it exits with only expected warnings

## Step 6 — Verify and commit

Run the full suite:
```bash
just audit       # cargo audit + cargo deny + npm audit — all must exit 0
just test        # 46 backend + 34 frontend tests
```

If both pass, commit with a message that lists each advisory addressed, whether it was fixed or suppressed and why. Push to `origin main` (Codeberg) only when explicitly asked.

## Context for this project

**Stack:** Rust backend (Rocket, redb, bitcode) + Vue 3 frontend (Vite, Vitest)

**Known permanent suppressions** (as of 2026-06-15):
- `RUSTSEC-2023-0071` — `rsa` Marvin Attack. App uses `Algorithm::HS256` exclusively; RSA compiled in via `jsonwebtoken`'s `rust_crypto` feature but never invoked. Enforced by tests in `router/fairing/mod.rs`.
- `RUSTSEC-2024-0436` — `paste` unmaintained. Build-time proc-macro only, via `rav1e → image`. No runtime exposure, no fix available upstream.

**Audit tooling locations:**
- `backend/.cargo/audit.toml` — cargo audit ignore list
- `backend/deny.toml` — cargo deny config (advisories, licenses, bans, sources)
- `justfile` targets: `just audit`, `just backend-audit`, `just frontend-audit`, `just backend-deny`
- `frontend/package.json` — npm deps; use `npx npm-check-updates` to find updates

**Key invariants not to break:**
- `jsonwebtoken` must keep `features = ["rust_crypto"]` — removing it breaks HS256 encoding
- `rocket/tls` feature is intentionally not enabled; do not add it without validating the full deployment path
- TODO.md is gitignored and local — never stage or commit it
