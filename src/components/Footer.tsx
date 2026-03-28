import { useClipStore } from "../stores/clipStore";

export default function Footer() {
  const clips = useClipStore((s) => s.clips);
  const clearHistory = useClipStore((s) => s.clearHistory);
  const pinnedCount = clips.filter((c) => c.is_pinned).length;

  if (clips.length === 0) return null;

  return (
    <div className="flex items-center justify-between border-t border-gray-700/30 px-4 py-1.5">
      <span className="text-[10px] text-gray-500">
        {clips.length} clip{clips.length !== 1 ? "s" : ""}
        {pinnedCount > 0 && ` · ${pinnedCount} pinned`}
      </span>
      <button
        onClick={clearHistory}
        className="text-[10px] text-gray-500 transition-colors hover:text-red-400"
        data-testid="clear-history"
      >
        Clear All
      </button>
    </div>
  );
}
