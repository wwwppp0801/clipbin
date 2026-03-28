import { useClipStore, type ClipItem } from "../stores/clipStore";

interface PreviewDialogProps {
  clip: ClipItem | null;
  onClose: () => void;
}

export default function PreviewDialog({ clip, onClose }: PreviewDialogProps) {
  const copyClip = useClipStore((s) => s.copyClip);
  const pasteClip = useClipStore((s) => s.pasteClip);

  if (!clip) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="max-h-[80vh] w-[600px] overflow-auto rounded-2xl border border-gray-700/50 bg-gray-900 p-5 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
        data-testid="preview-dialog"
      >
        <div className="mb-3 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="rounded bg-blue-500/20 px-2 py-0.5 text-xs font-medium text-blue-400">
              {clip.content_type}
            </span>
            {clip.source_app && <span className="text-xs text-gray-500">{clip.source_app}</span>}
          </div>
          <div className="flex items-center gap-1">
            <button
              onClick={() => {
                copyClip(clip.id);
              }}
              className="rounded-lg px-2.5 py-1 text-xs text-gray-400 hover:bg-gray-800 hover:text-white"
            >
              Copy
            </button>
            <button
              onClick={() => {
                onClose();
                pasteClip(clip.id);
              }}
              className="rounded-lg bg-blue-600 px-2.5 py-1 text-xs font-medium text-white hover:bg-blue-500"
            >
              Paste
            </button>
            <button
              onClick={onClose}
              className="ml-1 rounded-lg p-1 text-gray-400 hover:bg-gray-800 hover:text-white"
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        </div>

        {clip.content_type === "image" && clip.image_preview ? (
          <img
            src={clip.image_preview}
            alt="Clipboard image"
            className="max-h-[60vh] w-full rounded-lg object-contain"
          />
        ) : (
          <pre className="whitespace-pre-wrap break-words rounded-lg bg-gray-800/50 p-4 font-mono text-sm leading-relaxed text-gray-200">
            {clip.text_content}
          </pre>
        )}

        <div className="mt-3 flex items-center justify-between text-xs text-gray-500">
          <span>{clip.text_content && `${clip.text_content.length} characters`}</span>
          <span>{clip.created_at && new Date(clip.created_at).toLocaleString()}</span>
        </div>
      </div>
    </div>
  );
}
