---
status: open
type: chore
priority: high
area: testing
---

Replace the YAML-to-Rust codegen in `xtask/src/generator.rs` with a pure interpreter that lives in `urocissa/gallery-backend/src/tests/`. Eliminate the intermediate `scenarios_generated.rs` file and the `check-generated` drift guard.

**Current state**

`cargo xtask test-backend` reads `xtask/data/scenarios/backend/*.yaml`, emits Rust test code into `gallery-backend/src/tests/scenarios_generated.rs`, then runs `cargo nextest` on it. The generated code calls fixture helpers from `xtask::fixtures::*` and urocissa test bootstrap.

**Goal**

An interpreter in urocissa's test tree that reads YAML scenarios and executes them directly against a Rocket `Client` — no intermediate file, no codegen compile step, coverage counts toward urocissa. xtask shrinks to the `plan` tool only.

**Architecture**

```
gallery-backend/src/tests/
├── bootstrap.rs              stays
├── fixtures/                 moved from xtask/src/fixtures/
├── scenario_interpreter.rs   new — reads YAML, executes in-process
└── (scenarios_generated.rs   deleted)

xtask/src/
├── plan.rs                   stays
├── main.rs                   stays (drops test-backend, test-generator subcommands)
└── (generator.rs, fixtures/  removed)
```

`cargo xtask test-backend` becomes `cargo test -p urocissa -- scenario_interpreter`.

**Design constraint**

The interpreter must use only externally documented interfaces: Rocket startup via `build_rocket_with_config()`, the public HTTP API (all `GET`/`POST`/`PUT`/`DELETE` routes), and the `IMAGE_HOME` filesystem directory. No direct redb access, no private module imports. This keeps E2E tests valid across internal storage changes.

**Steps**

1. Move `xtask/src/fixtures/` to `gallery-backend/src/tests/fixtures/` — adjust imports, remove xtask dev-dep from urocissa if fixtures were the last consumer
2. Write `scenario_interpreter.rs` in urocissa tests — pure function that reads YAML, boots Rocket `Client`, steps through given/when/then, asserts
3. Validate against all 20 existing YAML scenarios
4. Delete `generator.rs`, `fixtures/`, `scenarios_generated.rs`, `check-generated` recipe, `gen` recipe
5. Update `justfile`: `test-backend` → `cargo test -p urocissa -- scenario_interpreter`
6. Update `docs/test-strategy.md` to reflect the new architecture
