# ClipBin

> A lightweight clipboard manager for macOS, inspired by [Paste](https://pasteapp.io/).

## Features

- **Clipboard Monitoring** — Automatically captures text, images, and file copies
- **Card Carousel UI** — Bottom-of-screen panel with horizontally scrollable cards
- **Instant Search** — Full-text search with SQLite FTS5, debounced input
- **Click to Paste** — Select a clip to write it to clipboard and paste into the active app via simulated Cmd+V
- **File Support** — Copies files via native NSPasteboard file URL API (works with Finder)
- **Image Support** — Captures clipboard images, stores as PNG, displays thumbnails
- **Keyboard Navigation** — Arrow keys to select, Enter to paste, Escape to dismiss
- **Global Hotkey** — `Shift+Cmd+V` to toggle the panel (configurable in settings)
- **Auto-hide** — Panel dismisses when clicking outside or pressing Escape
- **Slide Animations** — Smooth entrance/exit animations from the bottom of the screen
- **Rich Text Detection** — Detects HTML/rich text from web copies (green badge)
- **Pin Clips** — Right-click → Pin to keep important clips permanently
- **Source App Tracking** — Shows which app content was copied from
- **Number Key Shortcuts** — Press 1-9 for instant paste
- **Cmd+C to Copy** — Copy without pasting, with toast notification
- **Delete/Backspace** — Remove selected clip
- **Tab** — Switch focus to search input
- **Ignored Apps** — Skip clipboard from password managers (1Password, KeePassXC, etc.)
- **Clear All** — Footer button to wipe history (preserves pinned)
- **Max Clips Limit** — Auto-delete oldest clips (configurable, default 500)
- **Settings** — Hotkey, max clips, ignored apps, keyboard shortcuts reference
- **Menu Bar Icon** — Paperclip template tray icon, auto-adapts to light/dark
- **Smart Detection** — Distinguishes text, rich text, file paths, and images
- **Deduplication** — Hash-based dedup, re-copies update timestamp instead of creating duplicates

## Screenshots

The panel appears at the bottom of the screen above the Dock:

```
┌─────────────────────────────────────────────────────────┐
│  ─────  (drag handle)                                   │
│  [Search clips...                              ] [⚙]   │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐      │
│  │ Text    │ │ Text    │ │ File    │ │ Image   │  ···  │
│  │ Hello.. │ │ const.. │ │ /path.. │ │ [thumb] │      │
│  │         │ │         │ │         │ │         │      │
│  │ just now│ │ 5m ago  │ │ 1h ago  │ │ 2h ago  │      │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘      │
└─────────────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| UI | React 19 + TypeScript + Tailwind CSS 4 |
| Desktop | Tauri 2.0 |
| Backend | Rust (clipboard monitoring, native macOS APIs) |
| Storage | SQLite via sqlx (FTS5 full-text search) |
| State | Zustand |
| Paste | Core Graphics CGEvent (Cmd+V simulation) |
| macOS APIs | NSPasteboard, NSScreen, NSWorkspace, NSRunningApplication |

## Project Structure

```
clipbin/
├── src/                        # React frontend
│   ├── App.tsx                 # Root component, animation state, event listeners
│   ├── components/
│   │   ├── SearchBar.tsx       # Search input + settings button
│   │   ├── ClipList.tsx        # Horizontal card carousel with keyboard nav
│   │   ├── ClipCard.tsx        # Individual clip card with context menu
│   │   └── SettingsDialog.tsx  # Hotkey + max clips configuration
│   ├── stores/
│   │   └── clipStore.ts        # Zustand store (clips CRUD, search, Tauri IPC)
│   └── lib/
│       └── utils.ts            # formatRelativeTime, truncateText helpers
│
├── src-tauri/                  # Tauri + Rust backend
│   ├── src/
│   │   ├── main.rs             # Entry point
│   │   ├── lib.rs              # App setup: DB, tray, monitor, hotkey
│   │   ├── clipboard.rs        # Clipboard polling, hash dedup, file URL reading
│   │   ├── db.rs               # SQLite: migrations, CRUD, FTS5 search
│   │   ├── paste.rs            # Write clipboard + CGEvent Cmd+V + app activation
│   │   ├── commands.rs         # Tauri IPC commands
│   │   ├── models.rs           # Clip, ClipDto, ContentType data types
│   │   ├── tray.rs             # System tray, window positioning, blur-hide
│   │   └── settings.rs         # Settings persistence (JSON)
│   └── tauri.conf.json         # Window config, permissions, bundle
│
├── tests/
│   ├── frontend/               # Vitest + React Testing Library
│   │   ├── setup.ts            # Tauri API mocks
│   │   ├── clipStore.test.ts
│   │   ├── ClipCard.test.tsx
│   │   ├── ClipList.test.tsx
│   │   ├── SearchBar.test.tsx
│   │   └── utils.test.ts
│   └── e2e/
│       └── app.spec.ts         # Playwright E2E tests (9 scenarios)
│
├── devlog/                     # Development logs per phase
├── .github/workflows/          # CI (lint + test + build) and Release
└── CLAUDE.md                   # Development conventions
```

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (1.94+)
- [Node.js](https://nodejs.org/) (22+)
- [pnpm](https://pnpm.io/) (10+)
- Xcode Command Line Tools

### Setup

```bash
git clone https://github.com/wwwppp0801/clipbin.git
cd clipbin
pnpm install
```

### Run

```bash
# Full Tauri app (Rust backend + React frontend)
pnpm tauri dev

# Frontend only (for UI development)
pnpm dev
```

### Test

```bash
# Rust tests (22 tests)
cd src-tauri && cargo test

# Frontend tests (31 tests)
pnpm test

# E2E tests (9 tests)
pnpm test:e2e

# All lint checks
pnpm lint
cd src-tauri && cargo clippy --all-targets -- -D warnings -A unexpected_cfgs
```

### Build

```bash
pnpm tauri build    # Produces .dmg in src-tauri/target/release/bundle/dmg/
```

## Architecture

### Clipboard Monitoring
- Polls system clipboard every 500ms via `arboard` + NSPasteboard FFI
- Detects text, images (PNG encoded), and file URLs (native NSPasteboard API)
- SHA-256 hash deduplication — re-copies update timestamp, not create duplicates
- Self-triggered flag prevents re-recording our own paste actions

### Paste Flow
1. Remember frontmost app PID via `NSWorkspace.frontmostApplication`
2. Hide ClipBin window
3. Activate previous app via `NSRunningApplication.activateWithOptions`
4. Wait 150ms for focus to settle
5. Write content to system clipboard (text via arboard, files via NSPasteboard fileURL)
6. Simulate `Cmd+V` via Core Graphics `CGEvent`

### Window Management
- Positioned above Dock using `NSScreen.visibleFrame`
- Auto-hides on blur (focus loss) with 400ms grace period
- Slide-up/slide-down CSS animations coordinated via Tauri events

## License

MIT
