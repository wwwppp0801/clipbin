# Maccy 源码学习笔记

**日期**: 2026-03-29
**项目**: https://github.com/p0deje/Maccy
**语言**: Swift, SwiftUI, SwiftData

---

## 1. 剪贴板监控

- **轮询机制**: NSTimer 每 0.5s 检查一次（可配置 0.1-10s）
- **变化检测**: 对比 `NSPasteboard.changeCount` 整数值，变了才处理内容
- **启示**: 我们的 500ms 轮询和 Maccy 一样，策略正确

## 2. 内容类型处理

Maccy 支持 **一个历史条目对应多种内容类型**（关键设计差异）：

```
HistoryItem (1) → HistoryItemContent (N)
                   ├── .string (纯文本)
                   ├── .rtf (富文本)
                   ├── .html (HTML)
                   └── .png (图片)
```

复制一段网页文字时，剪贴板同时包含纯文本、RTF、HTML 三种格式。Maccy 全部保存，粘贴时全部写回。

**对比 ClipBin**: 我们只保存一种类型（text/image/file_path），丢失了富文本信息。这是后续需要改进的主要架构差异。

## 3. 图片处理

- **存储**: 原始二进制数据存 SwiftData（类似我们存 SQLite BLOB）
- **缩略图**: **懒生成** — 只在 UI 需要显示时才创建，不预处理
- **尺寸**: 默认 340×40px 缩略图，预览图最大 2048×1536
- **OCR**: 用 Vision framework 异步识别图片中的文字，使其可搜索
- **启示**: 我们的 PNG 编码方式正确，但应该加懒生成缩略图

## 4. 粘贴实现（核心）

用户选择历史条目后，Maccy 的粘贴流程：

### Step 1: 写入系统剪贴板
```swift
pasteboard.clearContents()
for content in contents {
    pasteboard.setData(content.value, forType: content.type)
}
// 标记来源为 Maccy（用于去重时忽略自己写入的内容）
pasteboard.setString("", forType: .fromMaccy)
```

### Step 2: 模拟 Cmd+V 按键
```swift
let source = CGEventSource(stateID: .hidEventState)
let vCode = Sauce.shared.keyCode(for: .v)
let keyDown = CGEvent(keyboardEventSource: source, virtualKey: vCode, keyDown: true)
keyDown?.flags = [.maskCommand]
keyDown?.post(tap: .cgSessionEventTap)
// ... keyUp 事件
```

- 使用 **CGEvent** 模拟键盘事件（需要辅助功能权限）
- 考虑了键盘布局差异（QWERTY vs Dvorak）
- 设置 `CGEventFlags` 抑制本地事件

### Step 3: 标记来源
- 写入 `.fromMaccy` 标记，下次轮询时检测到此标记就跳过（避免把自己的粘贴操作又记录一次）

**启示**: 我们需要用同样的 CGEvent 方式模拟粘贴，而不是仅仅写入剪贴板。

## 5. 去重策略

- 通过内容 hash 检测重复
- 重复时更新 `lastCopiedAt` 和 `numberOfCopies`
- 跟踪 "supersedes" 关系（新的条目包含更多类型时替换旧的）
- 用 `.modified` 字段跟踪中间状态的剪贴板变化

**对比 ClipBin**: 我们的 hash 去重基本正确，但没有 supersedes 逻辑。

## 6. 窗口管理

- 使用 `NSPanel`（不是 NSWindow）— 不会抢占焦点
- `.nonactivatingPanel` 样式
- `level = .statusBar` 置于状态栏层级
- `hidesOnDeactivate = false` 防止切换窗口时自动隐藏
- 定位：默认跟随鼠标光标位置

**对比 ClipBin**: 我们用 Tauri 的 WebviewWindow + alwaysOnTop，效果类似但没有 NSPanel 的非激活特性。这可能是导致 blur 问题的根源。

## 7. 数据存储

- **SwiftData**（Apple 的现代 ORM，替代 Core Data）
- 路径: `~/Library/Application Support/Maccy/Storage.sqlite`
- HistoryItem: 元数据（日期、来源 app、使用次数、pin）
- HistoryItemContent: 内容（type + binary data），1:N 关系

## 8. 搜索

- 4 种模式: exact, fuzzy (Fuse 算法), regex, mixed
- 节流 0.2s 防止卡顿
- 搜索在主线程但有节流保护

## 9. 对 ClipBin 的改进建议

| 领域 | Maccy 做法 | ClipBin 现状 | 建议 |
|------|-----------|-------------|------|
| 内容类型 | 一条记录存多种类型 | 只存一种 | 后续改为多类型存储 |
| 粘贴 | CGEvent 模拟 Cmd+V | 未实现 | **立即实现** |
| 缩略图 | 懒生成 | 全量 PNG 编码 | 优化为按需生成 |
| 窗口 | NSPanel 非激活 | WebviewWindow | 考虑原生 NSPanel |
| 去重 | supersedes + 来源标记 | hash 去重 | 加 fromClipBin 标记 |
| 过滤 | 3 层过滤（系统 + 类型 + app） | 无 | 加密码管理器过滤 |
| 搜索 | 4 种模式 | 简单 FTS5 | 加 fuzzy search |
| 快捷键 | 数字键快选 + 字母 pin | 方向键 + Enter | 加数字键快选 |

## 10. 总结

Maccy 的核心优势是**原生 macOS 集成**（NSPanel、CGEvent、Vision OCR）和**多内容类型保存**。作为 Tauri 应用，ClipBin 在这些方面有天然劣势，但可以通过 Rust FFI 调用 macOS API 来弥补。

最紧迫需要实现的是**粘贴功能**（CGEvent 模拟 Cmd+V）和**自身写入标记**（避免循环记录）。
