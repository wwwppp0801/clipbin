# macshot 源码学习笔记

**日期**: 2026-03-29
**项目**: https://github.com/sw33tLie/macshot
**语言**: Swift, AppKit

---

## 1. 架构概述

macshot 是一个 macOS 截图工具，单文件架构（主要逻辑在 `ScreenCaptureView.swift`）。

## 2. 屏幕截图实现

### 截图 API
- 使用 `CGWindowListCreateImage` 获取全屏截图
- 参数：`kCGWindowListOptionOnScreenOnly` + `kCGNullWindowID`
- 截取整个屏幕，然后用户选择区域裁剪

### 全屏覆盖
- 创建一个 `NSWindow`，level = `.screenSaver`（最高层级）
- 窗口尺寸 = 整个屏幕
- 背景显示全屏截图 + 50% 暗化遮罩
- 用户在上面拖拽选择区域

### 区域选择
```swift
override func mouseDown(with event: NSEvent) {
    startPoint = convert(event.locationInWindow, from: nil)
}

override func mouseDragged(with event: NSEvent) {
    let current = convert(event.locationInWindow, from: nil)
    selectionRect = NSRect(/* from startPoint to current */)
    needsDisplay = true  // 触发重绘
}

override func mouseUp(with event: NSEvent) {
    captureSelectedRegion()
}
```

### 区域裁剪
```swift
func captureSelectedRegion() -> NSImage? {
    let scale = window?.backingScaleFactor ?? 2.0
    let pixelW = Int(selectionRect.width * scale)
    let pixelH = Int(selectionRect.height * scale)
    // 创建 CGContext，从原始截图中裁剪选中区域
    let cgCtx = CGContext(data: nil, width: pixelW, height: pixelH, ...)
    cgCtx.draw(screenshot, in: destRect)
    return NSImage(cgImage: cgCtx.makeImage()!, size: selectionRect.size)
}
```

## 3. 选区绘制

- 选中区域保持原亮度（不暗化）
- 非选中区域 50% 暗化
- 选区边框：白色虚线 + 尺寸标签
- 实时显示选区尺寸（像素值）

## 4. 编辑工具

macshot 本身较简单，主要编辑功能：
- 选区拖拽调整（四角 + 四边）
- 选区确认后复制到剪贴板

更复杂的编辑（标注、箭头、文字）不在 macshot 范围内。

## 5. 复制到剪贴板

```swift
let pasteboard = NSPasteboard.general
pasteboard.clearContents()
pasteboard.writeObjects([capturedImage])  // NSImage 实现了 NSPasteboardWriting
```

## 6. 对 ClipBin 的启示

如果要在 ClipBin 中实现截图功能：

### 方案 A：调用系统截图
```bash
screencapture -i -c  # 交互式截图，复制到剪贴板
```
最简单，但不能自定义编辑 UI。

### 方案 B：自己实现（类似 macshot）
1. 用 `CGWindowListCreateImage` 截全屏
2. 创建全屏覆盖窗口（Tauri 创建新窗口或用原生 NSWindow）
3. 前端实现选区拖拽（HTML Canvas）
4. 选区确认后裁剪 + 复制到剪贴板

### 方案 C：混合方案
- 用 Rust FFI 调用 `CGWindowListCreateImage` 截全屏
- base64 发送给前端
- 前端用 Canvas 实现选区 + 简单编辑
- 编辑完成后发回 Rust 写入剪贴板

**推荐**：方案 A 最可靠（利用 macOS 原生能力），方案 C 最灵活但工作量大。
