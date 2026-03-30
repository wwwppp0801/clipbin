import {} from "react";
import { invoke } from "@tauri-apps/api/core";
import { useClipStore, type ClipItem } from "../stores/clipStore";
import { formatRelativeTime, isUrl, isJson } from "../lib/utils";

interface ClipCardProps {
  clip: ClipItem;
  isSelected: boolean;
  shortcutNumber?: number;
}

const TYPE_COLORS: Record<string, string> = {
  text: "bg-blue-500/20 text-blue-400",
  html: "bg-green-500/20 text-green-400",
  image: "bg-purple-500/20 text-purple-400",
  file_path: "bg-amber-500/20 text-amber-400",
};

export default function ClipCard({ clip, isSelected, shortcutNumber }: ClipCardProps) {
  const pasteClip = useClipStore((s) => s.pasteClip);
  const setPreviewClipId = useClipStore((s) => s.setPreviewClipId);

  const handleClick = (e: React.MouseEvent) => {
    // On macOS, ctrl+click is right-click — don't paste
    if (e.ctrlKey || e.metaKey) return;
    pasteClip(clip.id);
  };

  const handleDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setPreviewClipId(clip.id);
  };

  const handleContextMenu = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    useClipStore.setState({ contextMenuClipId: clip.id });
    // Pause blur BEFORE showing menu to prevent race condition:
    // ctrl+click fires blur before the sync Tauri command runs
    await invoke("set_blur_paused", { paused: true });
    invoke("show_clip_context_menu", {
      clipId: clip.id,
      isPinned: clip.is_pinned,
    }).catch(() => {});
  };

  const TYPE_LABELS: Record<string, string> = {
    text: "Text",
    html: "Rich Text",
    image: "Image",
    file_path: "File",
  };
  const isJsonContent = clip.content_type === "text" && isJson(clip.text_content);
  const typeLabel = isJsonContent ? "JSON" : TYPE_LABELS[clip.content_type] || "Text";
  const colorClass = isJsonContent
    ? "bg-orange-500/20 text-orange-400"
    : TYPE_COLORS[clip.content_type] || TYPE_COLORS.text;

  return (
    <div
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      onContextMenu={handleContextMenu}
      data-testid="clip-card"
      className={`group relative flex w-[220px] shrink-0 cursor-pointer flex-col overflow-hidden rounded-xl border transition-all duration-150 ${
        isSelected
          ? "border-blue-500 bg-gray-800 ring-1 ring-blue-500/50 shadow-lg shadow-blue-500/20"
          : "border-gray-700/40 bg-gray-800/50 hover:border-gray-600 hover:bg-gray-800/80"
      }`}
    >
      {/* Header: content type + shortcut number */}
      <div className="flex items-center justify-between gap-1 border-b border-gray-700/30 px-3 py-1.5">
        <div className="flex shrink-0 items-center gap-1.5">
          {shortcutNumber !== undefined && (
            <span className="flex h-4 w-4 items-center justify-center rounded bg-gray-600/50 text-[9px] font-bold text-gray-300">
              {shortcutNumber}
            </span>
          )}
          <span className={`whitespace-nowrap rounded px-1.5 py-0.5 text-[10px] font-medium ${colorClass}`}>
            {typeLabel}
          </span>
        </div>
        <div className="flex min-w-0 items-center gap-1">
          {clip.is_pinned && <span className="shrink-0 text-[10px] text-yellow-500">📌</span>}
          <div className="flex min-w-0 items-center gap-1 text-[10px] text-gray-500">
            {clip.source_app && <span className="max-w-[50px] truncate">{clip.source_app}</span>}
            {clip.source_app && <span className="shrink-0">·</span>}
            <span className="shrink-0">{formatRelativeTime(clip.created_at)}</span>
            {clip.use_count > 1 && (
              <>
                <span className="shrink-0">·</span>
                <span className="shrink-0">{clip.use_count}x</span>
              </>
            )}
          </div>
        </div>
      </div>

      {/* Content preview */}
      <div className="flex-1 overflow-hidden p-3">
        {clip.content_type === "image" && clip.image_preview ? (
          <img
            src={clip.image_preview}
            alt="Clipboard image"
            className="h-full w-full rounded-lg object-cover"
            data-testid="clip-image"
          />
        ) : clip.content_type === "file_path" ? (
          <div className="flex flex-col gap-1" data-testid="clip-text">
            {(clip.text_content || "").split("\n").map((path, i) => (
              <div key={i} className="flex items-center gap-1.5 text-xs text-gray-300">
                <svg
                  className="h-3.5 w-3.5 shrink-0 text-amber-400"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                >
                  <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
                  <polyline points="14 2 14 8 20 8" />
                </svg>
                <span className="truncate">{path.split("/").pop()}</span>
              </div>
            ))}
          </div>
        ) : isUrl(clip.text_content) ? (
          <div className="flex items-center gap-2" data-testid="clip-text">
            <svg
              className="h-4 w-4 shrink-0 text-blue-400"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71" />
              <path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71" />
            </svg>
            <span className="truncate text-xs text-blue-400">
              {clip.text_content?.replace(/^https?:\/\//, "")}
            </span>
          </div>
        ) : (
          <p
            className={`line-clamp-6 whitespace-pre-wrap break-words text-xs leading-relaxed text-gray-300 ${
              clip.content_type === "html" ? "font-sans" : "font-mono"
            }`}
            data-testid="clip-text"
          >
            {clip.text_content}
          </p>
        )}
      </div>

      {/* Delete button (hover) */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          useClipStore.getState().deleteClip(clip.id);
        }}
        data-testid="clip-delete"
        className="absolute top-1.5 right-1.5 rounded-full bg-black/60 p-1 text-gray-400 opacity-0 backdrop-blur-sm transition-opacity hover:text-red-400 group-hover:opacity-100"
        title="Delete"
        aria-label="Delete clip"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>
  );
}
