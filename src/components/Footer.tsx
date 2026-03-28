import { useState } from "react";
import { useClipStore } from "../stores/clipStore";

export default function Footer() {
  const clips = useClipStore((s) => s.clips);
  const clearHistory = useClipStore((s) => s.clearHistory);
  const pinnedCount = clips.filter((c) => c.is_pinned).length;
  const [confirming, setConfirming] = useState(false);

  if (clips.length === 0) return null;

  const handleClear = () => {
    if (confirming) {
      clearHistory();
      setConfirming(false);
    } else {
      setConfirming(true);
      // Auto-reset after 3 seconds
      setTimeout(() => setConfirming(false), 3000);
    }
  };

  return (
    <div className="flex items-center justify-between border-t border-gray-700/30 px-4 py-1.5">
      <span className="text-[10px] text-gray-500">
        {clips.length} clip{clips.length !== 1 ? "s" : ""}
        {pinnedCount > 0 && ` · ${pinnedCount} pinned`}
      </span>
      <button
        onClick={handleClear}
        className={`text-[10px] transition-colors ${
          confirming ? "font-medium text-red-400" : "text-gray-500 hover:text-red-400"
        }`}
        data-testid="clear-history"
      >
        {confirming ? "Click again to confirm" : "Clear All"}
      </button>
    </div>
  );
}
