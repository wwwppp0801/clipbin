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
    return (
      <div
        className="flex flex-1 items-center justify-center text-sm text-gray-500"
        data-testid="empty-state"
      >
        No clips yet. Copy something!
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
