# OpenAPI Generator Pipeline

## Motivation

The backend exposes ~63 route handlers across GET, POST, PUT, and DELETE
modules. Keeping the API documentation and test coverage in sync with the
actual implementation is a constant drift problem in any live codebase.

This project avoids that drift by making the code itself the sole authority:

- **`routes![]` macros** are the single source of truth for _which routes exist_.
- **`#[utoipa::path]` annotations** are the single source of truth for _which
  routes have OpenAPI specs_.

From these two inputs, the pipeline generates the OpenAPI schema, a human-readable
API reference, and coverage metrics — eliminating any separate list that could
fall out of sync.

The goal is an exact, auditable mapping between:

1. Available implemented API
2. Documentation reference
3. Complete API testing based on generated coverage metrics

This minimises the risk of undocumented or untested functions due to code drift.

## Pipeline

```
┌─────────────────────┐     ┌──────────────────────────┐
│  routes![] macros   │     │  #[utoipa::path(...)]    │
│  (which routes)     │     │  (OpenAPI metadata)      │
└──────┬──────────────┘     └──────────┬───────────────┘
       │                               │
       ▼                               ▼
┌──────────────────────────────────────────────────┐
│            cargo xtask openapi-gen               │
│                                                  │
│  1. Scan all routes![] for handler names         │
│  2. Scan handler source for #[utoipa::path]      │
│  3. Generate backend/src/openapi.rs               │
│  4. Compile server with openapi feature          │
│  5. Run picasu-openapi → openapi.json            │
└──────┬─────────────────────────────┬────────────┘
       │                             │
       ▼                             ▼
┌──────────────┐      ┌──────────────────────────┐
│ openapi.json │      │ docs/openapi-reference.md │
│ (spec)       │      │ (widdershins markdown)    │
└──────────────┘      └──────────────────────────┘
```

### Steps

1. **`cargo xtask openapi-gen`** — the core command. It:
   - Parses every `routes![]` invocation in `router/{get,post,put}/mod.rs` and
     `router/delete.rs` to discover every registered handler.
   - For each handler, reads its source file to check for a
     `#[utoipa::path]` or `#[cfg_attr(feature = "openapi", utoipa::path(...))]`
     annotation.
   - Writes `backend/src/openapi.rs` containing only the annotated routes, with
     the correct `__path_*` imports and `paths(...)` registration.
   - Compiles `backend` with `--features openapi` and runs the `picasu-openapi`
     binary, which calls `ApiDoc::openapi().to_json()`.
   - Writes the result to `backend/openapi.json`.

2. **`just openapi-docs`** — chains `openapi-gen` with:
   - `widdershins` to convert `openapi.json` → `docs/openapi-reference.md`
   - `prettier` for consistent markdown formatting

3. **`just openapi-docs-check`** — runs the full generation plus coverage
   check, then fails if any of the three committed files (`openapi.rs`,
   `openapi.json`, `docs/openapi-reference.md`) differ from the working tree.
   Wired into `just precommit` on the `main` branch.

### Coverage Tool

`cargo xtask openapi-coverage` is a fast, no-compilation check that reports:

- Total registered routes (all must be annotated)
- Annotated vs missing routes
- Coverage percentage

It exits non-zero if any route lacks a `#[utoipa::path]` annotation. No
module-level exemptions exist — every route is subject to the check.

## Tag conventions

| Tag | Routes | Description |
|---|---|---|
| *(none)* | Standard data API endpoints | `GET /get/...`, `POST /post/...`, `PUT /put/...`, `DELETE /delete/...` |
| `pages` | SPA HTML page routes | `GET /home`, `GET /albums`, `GET /login`, etc. — serve `index.html` |
| `development` | Debug-only tooling | `GET /put/generate_random_data` — generates fake data for testing |

## Workflow

### Adding a new data API route

1. Add the handler function to a `routes![]` block.
2. Add `#[cfg_attr(feature = "openapi", utoipa::path(...))]` with the route's
   HTTP method, path, parameters, and response types. Pick the appropriate
   tag (or omit for standard data APIs).
3. Run `just openapi-docs` — this regenerates all three committed files.
4. Run `cargo xtask openapi-coverage` to confirm 100%.
5. Commit the handler, its annotation, and the three generated files together.

Pre-commit hook or CI enforces step 3 — if the generated files are out of
date, the change is rejected.

### Removing a route

Delete the handler and its entry from `routes![]`. Run `just openapi-docs`.
The route disappears from all generated files automatically.

### Changing a route's signature

Update the `#[utoipa::path(...)]` annotation. Run `just openapi-docs`.
The spec and reference docs reflect the change immediately.

## Key Design Decisions

### Why generate `openapi.rs` instead of maintaining it manually?

The original approach required manually importing every `__path_*` symbol and
listing every handler in `paths(...)`. This was error-prone and duplicated what
`routes![]` already declares. The generator eliminates this maintenance burden
while guaranteeing completeness.

### Why not move `openapi.rs` to xtask?

`#[derive(OpenApi)]` references `__path_*` items generated by
`#[utoipa::path(...)]` proc-macros in the same crate. Cross-crate access would
require re-exporting every `__path_*` symbol from `backend`'s public API — more
boilerplate, not less. The generated file lives in `backend` but is written by
xtask before compilation.

### Why the stack size override?

`ApiDoc::openapi()` builds the complete schema tree at runtime. With many
registered schemas and deeply nested types (AbstractData's three flattened
variants), the recursive traversal exceeds Linux's default 2 MB thread stack.
`RUST_MIN_STACK=16777216` is set in the `justfile` for the `openapi-gen`
recipe. Only the generation step needs it — normal backend operation is
unaffected.

### Why no explicit `components(schemas(...))`?

utoipa automatically registers any type referenced as a request or response body
in a `#[utoipa::path]` annotation, including all transitively reachable types.
The explicit schema list was redundant and has been removed.

## Files

| File                             | Generator                 | Role                             |
| -------------------------------- | ------------------------- | -------------------------------- |
| `backend/src/openapi.rs`          | `cargo xtask openapi-gen` | ApiDoc struct with all routes    |
| `backend/openapi.json`            | `ApiDoc::openapi()`       | OpenAPI 3.1 spec                 |
| `docs/openapi-reference.md`      | widdershins               | Human-readable API reference     |
| `xtask/src/openapi.rs`           | —                         | Generator + coverage tool source |
