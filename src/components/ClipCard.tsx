import { useClipStore, type ClipItem } from "../stores/clipStore";
import { formatRelativeTime, truncateText } from "../lib/utils";

interface ClipCardProps {
  clip: ClipItem;
}

export default function ClipCard({ clip }: ClipCardProps) {
  const pasteClip = useClipStore((s) => s.pasteClip);
  const deleteClip = useClipStore((s) => s.deleteClip);

  const handleClick = () => {
    pasteClip(clip.id);
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    deleteClip(clip.id);
  };

  return (
    <div
      onClick={handleClick}
      data-testid="clip-card"
      className="group relative mx-3 cursor-pointer rounded-lg border border-gray-800 px-3 py-2 transition-colors hover:border-gray-600 hover:bg-gray-800/50"
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0 flex-1">
          {clip.content_type === "image" && clip.image_preview ? (
            <img
              src={clip.image_preview}
              alt="Clipboard image"
              className="max-h-20 rounded"
              data-testid="clip-image"
            />
          ) : (
            <p
              className="break-words text-sm leading-relaxed text-gray-200"
              data-testid="clip-text"
            >
              {truncateText(clip.text_content || "", 200)}
            </p>
          )}
        </div>

        <button
          onClick={handleDelete}
          data-testid="clip-delete"
          className="mt-0.5 shrink-0 rounded p-1 text-gray-600 opacity-0 transition-opacity hover:bg-gray-700 hover:text-gray-300 group-hover:opacity-100"
          title="Delete"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
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

      <div className="mt-1 flex items-center gap-2 text-xs text-gray-500">
        <span>{clip.content_type === "file_path" ? "File" : clip.content_type}</span>
        <span>{formatRelativeTime(clip.created_at)}</span>
        {clip.use_count > 1 && <span>Used {clip.use_count}x</span>}
      </div>
    </div>
  );
}
