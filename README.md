# ClipBin

> A lightweight clipboard manager for macOS

🌐 [clipbin.app](https://clipbin.app)

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| UI | React + TypeScript + Tailwind CSS |
| Desktop | Tauri 2.0 |
| System | Rust (clipboard monitoring, native APIs) |
| Storage | SQLite (via sqlx) |
| State | Zustand |

---

## Development Plan

### Phase 1 — MVP (Client Only)

- [ ] Project scaffold: Tauri 2.0 + React + TypeScript
- [ ] Clipboard monitoring (text, images, file paths)
- [ ] Local storage with SQLite (history + full-text search)
- [ ] Menu bar icon
- [ ] Global shortcut to summon/dismiss the panel
- [ ] History list UI with search
- [ ] Click to paste

### Phase 2 — Polish

- [ ] Pinned items / favorites
- [ ] Collections (user-defined groups)
- [ ] Rich preview (images, code, links)
- [ ] Keyboard navigation in panel
- [ ] App settings panel
- [ ] Auto-launch on login
- [ ] Exclude apps (e.g. password managers)

### Phase 3 — Cloud Sync

- [ ] Backend service (Node.js / Go + PostgreSQL)
- [ ] User auth (Apple Sign In)
- [ ] Incremental sync across devices
- [ ] iPhone / iPad client

---

## Project Structure (planned)

```
clipbin/
├── src/                  # React frontend
│   ├── components/
│   ├── stores/           # Zustand state
│   └── main.tsx
├── src-tauri/            # Tauri + Rust backend
│   ├── src/
│   │   ├── clipboard.rs  # Clipboard monitoring
│   │   ├── db.rs         # SQLite operations
│   │   └── main.rs
│   └── tauri.conf.json
└── README.md
```
