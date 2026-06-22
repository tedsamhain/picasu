---
status: open
type: chore
priority: high
area: devops
---

## Fork & Rebrand: urocissa → chestnut

Fork this repo into a new standalone project called **chestnut** with separate git remote, CI, and branding.

**Logo**: side-profile stylized squirrel holding a chestnut.

### Task list

1. **Rust crate rename** — `urocissa` → `chestnut` in `Cargo.toml` (workspace + gallery-backend), binary name, `urocissa-openapi` bin; update `main.rs`, `openapi.rs`, `xtask/src/main.rs`
2. **Environment variables** — `UROCISSA_*` → `CHESTNUT_*` in `justfile`, `compose.yaml`, `Dockerfile`, `docs/CONFIG.md`, `gallery-backend/src/public/constant/storage.rs`
3. **CI/CD pipelines** — Docker image refs (`hsa00000/urocissa` → new org/image), release artifact names, GitHub workflow files (`.github/workflows/*.yml`)
4. **Installers/scripts** — NSIS installer (`installer.nsi`), PowerShell build scripts (`build_installer.ps1`, `build_linux_binary.ps1`)
5. **Frontend config** — Playwright backend binary ref, gallery-site base URL (`nuxt.config.ts`)
6. **Gallery site** — GitHub repo links, release links in `gallery-site/app.vue`
7. **Tests** — Placeholder filename convention in `backend_api.rs`
8. **EXIF metadata** — Software tag in `xtask/src/test_image.rs`
9. **Documentation** — `README.md`, `docs/LINUX.md`, `docs/WINDOWS.md`, `docs/BUILD_INSTALLER.md`, `docs/openapi-*.md`, `docs/CONFIG.md`
10. **Meta files** — `.gitignore` data dir, `.claude/commands/security-audit.md`, `openapi.json`, `.plan/tasks/` internal references
11. **Branding assets** — Replace `logo.ico`, `logo.png`, favicons with squirrel + chestnut design
12. **License** — Update copyright holder in `LICENSE`

**NOTE** we don't need a big fancy website up front, github home is fine.
Could just delete parts we don't need / use, like the gallery-site and powershell scripts.

Good opportunity for any project-level restructuring, e.g. rename gallery-backend and gallery-frontend.
