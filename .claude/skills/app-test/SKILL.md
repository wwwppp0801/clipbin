---
name: app-test
description: Test ClipBin app by launching it, populating clipboard, simulating hotkeys, taking screenshots, and verifying UI state. Use when you need to do visual testing, capture product screenshots, or verify app functionality end-to-end on macOS.
---

# App UI Testing

Automate testing the ClipBin app by controlling it programmatically on macOS.

## Prerequisites

Install if missing:
- `cliclick`: `brew install cliclick` (mouse simulation)
- `Pillow`: `pip3 install Pillow` (image cropping)

Built-in (no install needed):
- `screencapture`, `osascript`, `pbcopy`

## Step 1: Start the App

```bash
pkill -f "target/debug/clipbin" 2>/dev/null
kill $(lsof -ti:1420) 2>/dev/null
sleep 1
source "$HOME/.cargo/env" && pnpm tauri dev &
sleep 25
pgrep -f "target/debug/clipbin" && echo "App running"
```

## Step 2: Populate Clipboard with Sample Data

Each `pbcopy` with `sleep 1` gives the 500ms monitor time to capture:

```bash
echo "npm install -g @tauri-apps/cli" | pbcopy && sleep 1
echo "https://github.com/wwwppp0801/clipbin" | pbcopy && sleep 1
echo '{"name": "ClipBin", "version": "0.4.0"}' | pbcopy && sleep 1
echo "Hello World! Test clip." | pbcopy && sleep 1
echo "SELECT * FROM clips ORDER BY last_used_at DESC;" | pbcopy && sleep 1
```

## Step 3: Open Clipboard Panel

Simulate `Shift+Cmd+V` (key code 9 = V):

```bash
osascript -e 'tell application "System Events" to key code 9 using {shift down, command down}'
sleep 1
```

## Step 4: Take & Crop Panel Screenshot

```bash
screencapture -x /tmp/clipbin-panel.png
```

Crop to just the panel (bottom ~35% of screen):

```python
from PIL import Image
img = Image.open('/tmp/clipbin-panel.png')
w, h = img.size
panel = img.crop((20, int(h * 0.63), w - 20, int(h * 0.94)))
panel.save('docs/images/01-main-view.png')
```

## Step 5: Close Panel

```bash
osascript -e 'tell application "System Events" to key code 53'  # Esc
sleep 1
```

## Step 6: Trigger Screenshot Tool

Simulate `Shift+Cmd+A` (key code 0 = A):

```bash
osascript -e 'tell application "System Events" to key code 0 using {shift down, command down}'
sleep 2
```

## Step 7: Simulate Mouse Drag to Select Area

```bash
cliclick dd:300,200 dm:800,500 du:800,500
sleep 3  # Wait for editor window to open
```

## Step 8: Screenshot & Resize Editor Window

```bash
# Optionally resize the editor window
osascript -e '
tell application "System Events"
    set frontProcess to first process whose frontmost is true
    tell frontProcess
        set position of window 1 to {100, 50}
        set size of window 1 to {900, 700}
    end tell
end tell'
sleep 1
screencapture -x /tmp/clipbin-editor.png
```

## Key Codes Reference (macOS)

| Key | Code | Example |
|-----|------|---------|
| A | 0 | `key code 0 using {shift down, command down}` → ⇧⌘A |
| V | 9 | `key code 9 using {shift down, command down}` → ⇧⌘V |
| C | 8 | `key code 8 using {command down}` → ⌘C |
| Esc | 53 | `key code 53` → Dismiss |
| Enter | 36 | `key code 36` → Paste selected |
| Delete | 51 | `key code 51` → Delete clip |

## cliclick Reference

| Action | Syntax | Example |
|--------|--------|---------|
| Click | `c:x,y` | `cliclick c:500,300` |
| Double-click | `dc:x,y` | `cliclick dc:500,300` |
| Right-click | `rc:x,y` | `cliclick rc:500,300` |
| Drag start | `dd:x,y` | `cliclick dd:300,200` |
| Drag move | `dm:x,y` | `cliclick dm:800,500` |
| Drag end | `du:x,y` | `cliclick du:800,500` |

## Multi-Monitor Testing

Get screen layout (essential before multi-monitor tests):

```bash
swift -e '
import AppKit
for (i, screen) in NSScreen.screens.enumerated() {
    let f = screen.frame
    let vf = screen.visibleFrame
    print("Screen \(i): frame=(\(f.origin.x),\(f.origin.y),\(f.width),\(f.height)) visible=(\(vf.origin.x),\(vf.origin.y),\(vf.width),\(vf.height))")
}
'
```

Move mouse to external screen (use X coordinate beyond primary width):

```bash
# If primary is 3008px wide, external starts at x=3008
cliclick m:3800,800
```

Capture all screens (one file per display):

```bash
screencapture -x /tmp/screen1.png /tmp/screen2.png
```

## Context Menu Testing

Right-click on a clip card:

```bash
cliclick rc:200,1540  # Adjust Y based on panel position
```

Note: `cliclick rc` triggers native right-click but may not reliably fire webview `contextmenu` events in all cases.

## Tips

- Always `sleep` after hotkey simulation (animations take ~250ms, use 1s buffer)
- Screenshot editor needs ~3s to open after area selection
- `screencapture -x` suppresses the shutter sound
- `screencapture -R x,y,w,h` captures a specific region non-interactively
- Crop coordinates depend on screen resolution (check with `system_profiler SPDisplaysDataType | grep Resolution`)
- Use `pgrep -f "target/debug/clipbin"` to verify app is running
- After testing, view screenshots with `Read` tool to verify visual correctness
- For multi-monitor: use `screencapture -x file1.png file2.png` to capture all displays
- Panel position on primary: bottom of screen, Y ≈ screen_height - 260 - 12
- When debugging blur/focus issues, add `eprintln!` to `setup_blur_hide` and `show_window` in `tray.rs`, then check stderr with `pnpm tauri dev 2>/tmp/debug.log`
