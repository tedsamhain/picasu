---
status: in-progress
type: feature
priority: high
area: testing
---

## Goal

Implement a Playwright-based scenario interpreter for spec-driven frontend testing, mirroring the backend YAML scenario pattern. The interpreter reads UI YAML specs from `xtask/data/scenarios/ui/`, executes them against a real browser driving the full stack (backend + frontend), and produces structured JSON output for AI course-correction.

## Vocabulary

The UI DSL vocabulary is already designed in `docs/scenario-dsl.md` and encoded in `xtask/data/schema.json` (`uiWhenItem`, `uiThenItem` definitions). This task implements the runner.

## Architecture (no cargo xtask involvement)

```
playwright.config.ts
  └─ webServer: starts backend (port 5673) + vite dev server (port 5173)
  └─ test: tests/playwright/*.spec.ts

tests/playwright/
  ├── interpreter.spec.ts     # Loads YAML, runs each scenario as a test
  ├── executeGiven.ts         # POST seed data to backend API
  ├── executeWhen.ts          # Maps YAML verbs → Playwright locator actions
  ├── executeThen.ts          # Maps YAML assertions → Playwright assertions
  ├── report.ts               # Structured JSON output per scenario
  ├── types.ts                # Zod schemas mirroring schema.json UI definitions
  └── scenarios/              # Symlink or reference to xtask/data/scenarios/ui/
```

## Phases

### Phase 1 — Install + config + scaffolding

- `npm install -D @playwright/test`
- `npx playwright install --with-deps chromium`
- Create `playwright.config.ts` with `webServer` for backend + frontend
- Add `npm run test:e2e` script to package.json
- Add `just frontend-e2e` recipe
- Create `tests/playwright/` directory with type definitions and YAML loader

### Phase 2 — Interpreter

- `executeGiven.ts`: POST seed data to running backend (uses same API endpoints)
- `executeWhen.ts`: navigate, click, fill, select, submit → Playwright locators
- `executeThen.ts`: visible, hidden, text, toast, modal, route → Playwright assertions
- `interpreter.spec.ts`: orchestrates load → given → when → then per scenario

### Phase 3 — AI feedback format

- Custom Playwright reporter or JSON output with per-step results
- Screenshot + DOM snapshot on failure
- Console log capture

### Phase 4 — First scenarios

- `ui/home-page.yaml`: navigate to `/home`, assert grid visible
- `ui/albums-page.yaml`: navigate to `/albums`, assert headings
- `ui/login-page.yaml`: fill credentials, submit, assert redirect

## Progress

2026-06-18: Created task. Starting Phase 1.
