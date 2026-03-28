import { useRef, useState, useCallback, useEffect } from "react";
import { useClipStore } from "../stores/clipStore";
import ClipCard from "./ClipCard";

export default function ClipList() {
  const clips = useClipStore((s) => s.clips);
  const isLoading = useClipStore((s) => s.isLoading);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setSelectedIndex(0);
  }, [clips]);

  useEffect(() => {
    if (!scrollRef.current) return;
    const cards = scrollRef.current.children;
    if (cards[selectedIndex]) {
      (cards[selectedIndex] as HTMLElement).scrollIntoView?.({
        behavior: "smooth",
        block: "nearest",
        inline: "center",
      });
    }
  }, [selectedIndex]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (clips.length === 0) return;
      if (e.key === "ArrowLeft") {
        e.preventDefault();
        setSelectedIndex((i) => Math.max(0, i - 1));
      } else if (e.key === "ArrowRight") {
        e.preventDefault();
        setSelectedIndex((i) => Math.min(clips.length - 1, i + 1));
      } else if (e.key === "Enter") {
        e.preventDefault();
        const clip = clips[selectedIndex];
        if (clip) {
          useClipStore.getState().pasteClip(clip.id);
        }
      } else if (e.key === "Backspace" || e.key === "Delete") {
        // Delete selected clip
        e.preventDefault();
        const clip = clips[selectedIndex];
        if (clip) {
          useClipStore.getState().deleteClip(clip.id);
        }
      } else if (e.key === "c" && (e.metaKey || e.ctrlKey)) {
        // Cmd+C: copy selected clip to clipboard without pasting
        e.preventDefault();
        const clip = clips[selectedIndex];
        if (clip) {
          useClipStore.getState().copyClip(clip.id);
        }
      } else if (e.key >= "1" && e.key <= "9") {
        // Number keys for quick paste
        const index = parseInt(e.key) - 1;
        if (index < clips.length) {
          e.preventDefault();
          useClipStore.getState().pasteClip(clips[index].id);
        }
      }
    },
    [clips, selectedIndex],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  if (isLoading && clips.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center text-sm text-gray-500">
        Loading...
      </div>
    );
  }

  if (clips.length === 0) {
    const searchQuery = useClipStore.getState().searchQuery;
    return (
      <div
        className="flex flex-1 flex-col items-center justify-center gap-2 text-gray-500"
        data-testid="empty-state"
      >
        <svg
          className="h-8 w-8 text-gray-600"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
        >
          <path d="M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2" />
          <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
        </svg>
        <span className="text-sm">
          {searchQuery ? "No matching clips" : "No clips yet — copy something!"}
        </span>
        {!searchQuery && (
          <span className="text-xs text-gray-600">Press ⇧⌘V to open after copying</span>
        )}
      </div>
    );
  }

  return (
    <div
      ref={scrollRef}
      className="flex flex-1 items-stretch gap-2.5 overflow-x-auto px-3 pb-3 scrollbar-hide"
      data-testid="clip-list"
    >
      {clips.map((clip, index) => (
        <ClipCard
          key={clip.id}
          clip={clip}
          isSelected={index === selectedIndex}
          shortcutNumber={index < 9 ? index + 1 : undefined}
        />
      ))}
    </div>
  );
}
