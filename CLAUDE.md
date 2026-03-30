# CLAUDE.md

## Project Overview

ClipBin is a macOS clipboard manager built with Tauri 2.0 + React + TypeScript + Tailwind CSS + SQLite + Zustand.

## Development Preferences

### Workflow
- Develop features incrementally, commit at each phase boundary
- Write devlog entries in `devlog/` for each phase
- Tag stable versions (e.g., `v0.1.0`) before starting major changes
- **Push to remote frequently** — commit and push after every meaningful change
- Study similar open-source projects (e.g., Maccy, macshot) and write study notes
- Install needed tools/dependencies without asking — user gave blanket permission

### Code Quality
- All code must have tests — run and pass before committing
- Rust: `cargo fmt` + `cargo clippy -D warnings -A unexpected_cfgs` must pass
- Frontend: ESLint + Prettier + `tsc --noEmit` must pass
- E2E tests with Playwright to simulate real UI interactions
- Run `pnpm vitest run` (frontend) and `cargo test` (Rust) before every commit

### Testing
- Rust unit tests use `#[cfg(test)] mod tests` (inline, cargo convention)
- Frontend tests: Vitest + React Testing Library in `tests/frontend/`
- E2E tests: Playwright against Vite dev server with Tauri IPC mocked in `tests/e2e/`
- Rust tests: cargo test with in-memory SQLite
- Current test counts: 29 Rust + 48 frontend + 15 E2E = **92 total**

### Git
- Commit messages: conventional commits (feat/fix/chore/test/docs)
- Include `Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>`
- Don't amend published commits
- Push after every commit

### UI/UX
- Follow Paste app's interaction pattern: bottom panel, card carousel, slide animations
- Window hidden by default, toggle with configurable hotkey (Shift+Cmd+V)
- Click outside to dismiss (blur), Escape to dismiss, entrance/exit animations
- Settings accessible via gear icon next to search bar
- Number keys 1-9 for quick paste, arrow keys for navigation, Enter to paste

## Tech Stack

- **Desktop**: Tauri 2.0 (Rust backend)
- **Frontend**: React 19 + TypeScript + Tailwind CSS 4 + Zustand
- **Storage**: SQLite via sqlx (Rust-side only)
- **Clipboard**: arboard (text/image) + NSPasteboard FFI (file URLs, HTML)
- **Paste**: CGEvent Cmd+V simulation + NSRunningApplication for app activation
- **macOS native**: cocoa, objc, core-graphics crates for system integration
- **Build**: Vite 8, pnpm
- **Screenshot**: macOS screencapture + HTML Canvas editor
- **Website**: Cloudflare Workers (clipbin.app, i18n: EN/中文/日本語/한국어)
- **CI**: GitHub Actions (lint → test → build → release)

## Key Commands

```bash
# Frontend
pnpm dev              # Start Vite dev server
pnpm test             # Run Vitest (31 tests)
pnpm lint             # ESLint + TypeScript check
pnpm format           # Prettier format
pnpm test:e2e         # Playwright E2E tests (11 tests)

# Rust (from src-tauri/)
cargo test            # Run Rust tests (25 tests)
cargo fmt --all       # Format Rust code
cargo clippy --all-targets -- -D warnings -A unexpected_cfgs  # Lint

# Full app
pnpm tauri dev        # Start full Tauri app
pnpm tauri build      # Build release .dmg
```

## Architecture Notes

- Clipboard monitoring: 500ms polling with arboard + NSPasteboard FFI
- Content types: Text, Html (rich text), Image (PNG), FilePath (NSPasteboard fileURL)
- Hash-based deduplication (SHA-256), re-copies update timestamp
- Paste flow: hide window → activate previous app (NSRunningApplication) → write clipboard → CGEvent Cmd+V
- Self-triggered flag prevents monitor from re-recording paste actions
- Window positioning: cursor-based screen detection → NSScreen.visibleFrame for correct monitor
- Animation: slide-up/slide-down CSS, coordinated via Tauri events (window-will-show/hide)
- Grace period (600ms) after show/hide to prevent blur/hotkey interference (extended for multi-monitor)
- Settings: standalone window (public/settings.html), persisted to JSON in app data directory
- First launch detection: check if settings.json exists before loading defaults
- Max clips enforcement (default 500) — deletes oldest non-pinned clips

## Key Lessons Learned

### macOS Tray Icons
- Use `icon_as_template(true)` for menu bar icons — macOS auto-adapts for light/dark
- Dedicated tray icon file (44x44 @2x) separate from app bundle icons
- Use `include_bytes!` to embed icon at compile time
- Add padding (3px) to prevent clipping

### Window Management
- Tauri WebviewWindow steals focus (unlike NSPanel) — must track previous app PID
- `NSWorkspace.frontmostApplication` to remember, `NSRunningApplication.activateWithOptions` to restore
- Blur-hide needs grace period to avoid instant dismiss after show
- **Multi-monitor**: `NSScreen.mainScreen` returns the screen with keyboard focus, NOT the primary display. Use `NSScreen.screens()[0]` for the primary display height (needed for coordinate conversion)
- **Coordinate conversion**: macOS uses bottom-left origin, Tauri uses top-left of primary screen. For external monitors, `tauri_y = primary_height - macOS_bottom_left_y`
- **Multi-monitor focus**: window repositioning across screens can trigger transient blur events. Re-focus after 150ms delay + extended grace period (600ms) to absorb them

### Clipboard Types
- `arboard` only reads text and images — need NSPasteboard FFI for file URLs and HTML
- File copies use `public.file-url` NSPasteboard type, not plain text
- Web copies have both `public.html` and `public.utf8-plain-text` simultaneously
- Priority: file URLs > HTML+text > text only > image

### Animation
- Don't use opacity transitions with transparent windows (causes white flash)
- Pure translateY is cleaner for slide effects
- Coordinate Rust show/hide with frontend animation via Tauri events
- Don't use complex state locks (AtomicBool) — simple timestamp grace period is more reliable

### Context Menu & Blur Race Conditions
- macOS ctrl+click fires both `click` and `contextmenu` events; `click` fires first
- Must check `e.ctrlKey` in click handler to avoid triggering paste on right-click
- Native `popup_menu()` blocks the main thread; queued blur events fire after it returns
- Call `set_blur_paused(true)` from frontend BEFORE `invoke("show_clip_context_menu")` to prevent race condition where blur fires before Rust command runs
- After popup_menu returns, `mark_action()` before `set_blur_paused(false)` absorbs queued blur events via grace period

### Standalone Windows (Settings, Screenshot Editor)
- Use `public/*.html` with vanilla JS + `window.__TAURI_INTERNALS__` for separate windows
- Do NOT use Vite React entry points in `public/` — Vite HMR `@vitejs/plugin-react` preamble detection fails for non-main entry points
- `\u` escape sequences in HTML don't work — use actual Unicode characters (⇧⌘⌫←→)
- Pattern: `WebviewWindowBuilder::new(app, "label", WebviewUrl::App("/file.html"))` to create

### CI
- `cargo clippy -A unexpected_cfgs` needed for old objc crate macros
- `#[allow(deprecated)]` on cocoa crate usage (deprecated but functional)
- Always run full check locally before push: fmt, clippy, vitest, tsc, eslint
