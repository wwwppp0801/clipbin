import { useClipStore, type ClipItem } from "../stores/clipStore";
import { formatRelativeTime, truncateText } from "../lib/utils";

interface ClipCardProps {
  clip: ClipItem;
  isSelected: boolean;
}

export default function ClipCard({ clip, isSelected }: ClipCardProps) {
  const pasteClip = useClipStore((s) => s.pasteClip);
  const deleteClip = useClipStore((s) => s.deleteClip);

  const handleClick = () => {
    pasteClip(clip.id);
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    deleteClip(clip.id);
  };

  const typeLabel =
    clip.content_type === "file_path" ? "File" : clip.content_type === "image" ? "Image" : "Text";

  return (
    <div
      onClick={handleClick}
      data-testid="clip-card"
      className={`group relative flex h-[180px] w-[200px] shrink-0 cursor-pointer flex-col rounded-xl border p-3 transition-all ${
        isSelected
          ? "border-blue-500 bg-gray-800 shadow-lg shadow-blue-500/20"
          : "border-gray-700/50 bg-gray-800/60 hover:border-gray-600 hover:bg-gray-800"
      }`}
    >
      {/* Content preview */}
      <div className="min-h-0 flex-1 overflow-hidden">
        {clip.content_type === "image" && clip.image_preview ? (
          <img
            src={clip.image_preview}
            alt="Clipboard image"
            className="h-full w-full rounded-lg object-cover"
            data-testid="clip-image"
          />
        ) : (
          <p
            className="whitespace-pre-wrap break-words text-xs leading-relaxed text-gray-300"
            data-testid="clip-text"
          >
            {truncateText(clip.text_content || "", 160)}
          </p>
        )}
      </div>

      {/* Footer: type + time */}
      <div className="mt-2 flex items-center justify-between text-[10px] text-gray-500">
        <span className="rounded bg-gray-700/50 px-1.5 py-0.5">{typeLabel}</span>
        <span>{formatRelativeTime(clip.created_at)}</span>
      </div>

      {/* Delete button */}
      <button
        onClick={handleDelete}
        data-testid="clip-delete"
        className="absolute top-1.5 right-1.5 rounded-full bg-gray-900/80 p-1 text-gray-500 opacity-0 transition-opacity hover:text-red-400 group-hover:opacity-100"
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
    </div>
  );
}
