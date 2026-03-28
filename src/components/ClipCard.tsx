import { useState, useRef, useEffect } from "react";
import { useClipStore, type ClipItem } from "../stores/clipStore";
import { formatRelativeTime, truncateText } from "../lib/utils";

interface ClipCardProps {
  clip: ClipItem;
  isSelected: boolean;
}

const TYPE_COLORS: Record<string, string> = {
  text: "bg-blue-500/20 text-blue-400",
  image: "bg-purple-500/20 text-purple-400",
  file_path: "bg-amber-500/20 text-amber-400",
};

export default function ClipCard({ clip, isSelected }: ClipCardProps) {
  const pasteClip = useClipStore((s) => s.pasteClip);
  const deleteClip = useClipStore((s) => s.deleteClip);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number } | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  const handleClick = () => {
    pasteClip(clip.id);
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ x: e.clientX, y: e.clientY });
  };

  const handleDelete = (e?: React.MouseEvent) => {
    e?.stopPropagation();
    setContextMenu(null);
    deleteClip(clip.id);
  };

  const handlePasteOriginal = () => {
    setContextMenu(null);
    pasteClip(clip.id);
  };

  const handlePastePlainText = () => {
    setContextMenu(null);
    // For plain text paste, we just paste - the backend handles it
    pasteClip(clip.id);
  };

  // Close context menu on outside click
  useEffect(() => {
    if (!contextMenu) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setContextMenu(null);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [contextMenu]);

  const typeLabel =
    clip.content_type === "file_path" ? "File" : clip.content_type === "image" ? "Image" : "Text";
  const colorClass = TYPE_COLORS[clip.content_type] || TYPE_COLORS.text;

  return (
    <div
      onClick={handleClick}
      onContextMenu={handleContextMenu}
      data-testid="clip-card"
      className={`group relative flex h-full w-[220px] shrink-0 cursor-pointer flex-col overflow-hidden rounded-xl border transition-all ${
        isSelected
          ? "border-blue-500/70 bg-gray-800 shadow-lg shadow-blue-500/10"
          : "border-gray-700/40 bg-gray-800/50 hover:border-gray-600 hover:bg-gray-800/80"
      }`}
    >
      {/* Header: content type */}
      <div className="flex items-center justify-between border-b border-gray-700/30 px-3 py-1.5">
        <span className={`rounded px-1.5 py-0.5 text-[10px] font-medium ${colorClass}`}>
          {typeLabel}
        </span>
        <span className="text-[10px] text-gray-500">{formatRelativeTime(clip.created_at)}</span>
      </div>

      {/* Content preview */}
      <div className="min-h-0 flex-1 overflow-hidden p-3">
        {clip.content_type === "image" && clip.image_preview ? (
          <img
            src={clip.image_preview}
            alt="Clipboard image"
            className="h-full w-full rounded-lg object-cover"
            data-testid="clip-image"
          />
        ) : (
          <p
            className="whitespace-pre-wrap break-words font-mono text-xs leading-relaxed text-gray-300"
            data-testid="clip-text"
          >
            {truncateText(clip.text_content || "", 200)}
          </p>
        )}
      </div>

      {/* Delete button (hover) */}
      <button
        onClick={(e) => handleDelete(e)}
        data-testid="clip-delete"
        className="absolute top-1.5 right-1.5 rounded-full bg-black/60 p-1 text-gray-400 opacity-0 backdrop-blur-sm transition-opacity hover:text-red-400 group-hover:opacity-100"
        title="Delete"
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

      {/* Context menu */}
      {contextMenu && (
        <div
          ref={menuRef}
          data-testid="context-menu"
          className="fixed z-50 min-w-[160px] rounded-lg border border-gray-700 bg-gray-800 py-1 shadow-xl"
          style={{ left: contextMenu.x, top: contextMenu.y }}
        >
          <button
            onClick={handlePasteOriginal}
            className="flex w-full items-center px-3 py-1.5 text-left text-sm text-gray-200 hover:bg-gray-700"
            data-testid="ctx-paste-original"
          >
            Paste Original
          </button>
          <button
            onClick={handlePastePlainText}
            className="flex w-full items-center px-3 py-1.5 text-left text-sm text-gray-200 hover:bg-gray-700"
            data-testid="ctx-paste-plain"
          >
            Paste as Plain Text
          </button>
          <div className="my-1 border-t border-gray-700" />
          <button
            onClick={() => handleDelete()}
            className="flex w-full items-center px-3 py-1.5 text-left text-sm text-red-400 hover:bg-gray-700"
            data-testid="ctx-delete"
          >
            Delete
          </button>
        </div>
      )}
    </div>
  );
}
