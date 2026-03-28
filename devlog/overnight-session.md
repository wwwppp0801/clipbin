# Overnight Development Session

**Date**: 2026-03-29 (01:00 - 04:00)

## Summary

Massive feature push while user was sleeping. 50+ commits, CI staying green throughout.

## Features Implemented

### Content Types & Detection
- **Rich text (HTML)**: NSPasteboard `public.html` detection, green badge
- **File URLs**: NSPasteboard NSURL reading for Finder file copies
- **URL detection**: Auto-detect http/https URLs, show link icon + domain
- **Source app tracking**: NSWorkspace.frontmostApplication captures app name
- **changeCount optimization**: Skip processing when clipboard unchanged

### Paste System (Maccy-inspired)
- **CGEvent Cmd+V simulation**: Core Graphics keyboard event posting
- **Previous app activation**: NSRunningApplication.activateWithOptions
- **Self-triggered flag**: Prevents re-recording own paste actions
- **File paste**: NSPasteboard NSURL writeObjects for Finder support
- **Copy without paste**: Cmd+C copies to clipboard, stays open

### UI Components
- **Footer bar**: Clip count, pinned count, Clear All with double-confirm
- **Preview dialog**: Double-click to view full content, Copy/Paste buttons
- **Toast notifications**: "Copied to clipboard" on Cmd+C
- **Empty state**: Clipboard icon, contextual messages, hotkey hint
- **Right-edge gradient**: Hints more content to scroll
- **Card scale animation**: Selected card 2% larger, hovered 1%
- **Use count badges**: "Nx" shown for frequently copied clips

### Keyboard Navigation
- **1-9**: Quick paste Nth clip
- **Cmd+C**: Copy without pasting
- **Cmd+P**: Toggle pin
- **Delete/Backspace**: Remove selected clip
- **Home/End**: Jump to first/last
- **Tab**: Focus search input
- **Escape**: Dismiss panel
- **Arrow keys**: Navigate cards
- **Auto-focus**: Typing characters auto-focuses search

### Settings & Configuration
- **Ignored apps**: 1Password, Keychain Access, KeePassXC by default
- **Max clips enforcement**: Auto-delete oldest when limit exceeded
- **Keyboard shortcuts reference**: 12 shortcuts listed in settings
- **Version display**: v0.2.0 shown in settings

### Pin System
- **Pin/unpin via context menu or Cmd+P**
- **Pinned clips sorted first**
- **Pinned clips survive Clear All and limit enforcement**
- **📌 indicator on pinned cards**

### Performance
- **NSPasteboard.changeCount**: Skip processing unchanged clipboard
- **Hash dedup**: SHA-256, only process changed content

## Documentation
- Product guide with 4 screenshots (docs/PRODUCT_GUIDE.md)
- Maccy study notes (devlog/maccy-study-notes.md)
- macshot study notes (devlog/macshot-study-notes.md)
- Updated CLAUDE.md with lessons learned
- Updated README with all features

## Test Coverage
| Layer | Framework | Count |
|-------|-----------|-------|
| Rust | cargo test | 29 |
| Frontend | Vitest | 39 |
| E2E | Playwright | 14 |
| **Total** | | **82** |

## Tags
- v0.1.0: Initial working MVP
- v0.2.0: Rich features, paste implementation
- v0.2.1: Source app tracking, ignored apps, preview
- v0.3.0: URL detection, scroll improvements, full keyboard support

## CI Status
All runs green throughout the session. Lint (clippy + eslint + prettier + tsc) → Test → Build pipeline.
