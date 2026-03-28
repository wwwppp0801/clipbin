# CLAUDE.md

## Project Overview

ClipBin is a macOS clipboard manager built with Tauri 2.0 + React + TypeScript + Tailwind CSS + SQLite + Zustand.

## Development Preferences

### Workflow
- Develop features incrementally, commit at each phase boundary
- Write devlog entries in `devlog/` for each phase
- Tag stable versions (e.g., `v0.1.0`) before starting major changes
- Push to remote regularly — don't accumulate unpushed commits
- Study similar open-source projects (e.g., Maccy) and write study notes

### Code Quality
- All code must have tests — run and pass before committing
- Rust: `cargo fmt` + `cargo clippy -D warnings` must pass
- Frontend: ESLint + Prettier must pass
- E2E tests with Playwright to simulate real UI interactions
- Run `pnpm vitest run` (frontend) and `cargo test` (Rust) before every commit

### Testing
- Tests in separate `tests/` directory (not inline), except Rust unit tests which use `#[cfg(test)] mod tests` by convention
- Frontend tests: Vitest + React Testing Library
- E2E tests: Playwright against Vite dev server with Tauri IPC mocked
- Rust tests: cargo test with in-memory SQLite

### Git
- Commit messages: conventional commits (feat/fix/chore)
- Include `Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>`
- Don't amend published commits

### UI/UX
- Follow Paste app's interaction pattern: bottom panel, card carousel, slide animations
- Window hidden by default, toggle with configurable hotkey (Shift+Cmd+V)
- Click outside to dismiss, entrance/exit animations
- Settings accessible via gear icon next to search bar

## Tech Stack

- **Desktop**: Tauri 2.0 (Rust backend)
- **Frontend**: React 19 + TypeScript + Tailwind CSS 4 + Zustand
- **Storage**: SQLite via sqlx (Rust-side only)
- **Clipboard**: arboard (Rust) + macOS CGEvent for paste simulation
- **Build**: Vite 8, pnpm
- **CI**: GitHub Actions (lint → test → build)

## Key Commands

```bash
# Frontend
pnpm dev              # Start Vite dev server
pnpm test             # Run Vitest
pnpm lint             # ESLint + TypeScript check
pnpm format           # Prettier format
pnpm test:e2e         # Playwright E2E tests

# Rust (from src-tauri/)
cargo test            # Run Rust tests
cargo fmt --all       # Format Rust code
cargo clippy --all-targets -- -D warnings  # Lint Rust code

# Full app
pnpm tauri dev        # Start full Tauri app (compiles Rust + starts Vite)
pnpm tauri build      # Build release .dmg
```

## Architecture Notes

- Clipboard monitoring: 500ms polling with arboard, hash-based deduplication
- Paste: write to clipboard → activate previous app → CGEvent Cmd+V simulation
- Self-triggered flag prevents monitor from re-recording paste actions
- Window positioning uses NSScreen.visibleFrame to avoid Dock overlap
- Settings persisted to JSON in app data directory
