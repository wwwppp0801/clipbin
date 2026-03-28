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
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") {
        if (!["ArrowLeft", "ArrowRight", "Enter"].includes(e.key)) return;
      }
      if (e.key === "Home") {
        e.preventDefault();
        setSelectedIndex(0);
      } else if (e.key === "End") {
        e.preventDefault();
        setSelectedIndex(clips.length - 1);
      } else if (e.key === "ArrowLeft") {
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
      } else if (e.key === "Tab") {
        // Tab: focus search input
        e.preventDefault();
        const input = document.querySelector('[data-testid="search-input"]') as HTMLInputElement;
        input?.focus();
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
      } else if (e.key === "v" && (e.metaKey || e.ctrlKey) && e.shiftKey) {
        // Cmd+Shift+V: paste as plain text (same as Enter since we store plain text)
        e.preventDefault();
        const clip = clips[selectedIndex];
        if (clip) {
          useClipStore.getState().pasteClip(clip.id);
        }
      } else if (e.key === "p" && (e.metaKey || e.ctrlKey)) {
        // Cmd+P: toggle pin on selected clip
        e.preventDefault();
        const clip = clips[selectedIndex];
        if (clip) {
          useClipStore.getState().togglePin(clip.id);
        }
      } else if (e.key >= "1" && e.key <= "9" && !e.metaKey && !e.ctrlKey) {
        // Number keys for quick paste
        const index = parseInt(e.key) - 1;
        if (index < clips.length) {
          e.preventDefault();
          useClipStore.getState().pasteClip(clips[index].id);
        }
      } else if (e.key === "?" && e.shiftKey) {
        // Show keyboard shortcuts (open settings)
        e.preventDefault();
        const settingsBtn = document.querySelector(
          '[data-testid="settings-button"]',
        ) as HTMLElement;
        settingsBtn?.click();
      } else if (e.key.length === 1 && !e.metaKey && !e.ctrlKey && !e.altKey) {
        // Printable character — auto-focus search and type there
        const input = document.querySelector('[data-testid="search-input"]') as HTMLInputElement;
        if (input && document.activeElement !== input) {
          input.focus();
          // Don't prevent default — let the character go into the input
        }
      }
    },
    [clips, selectedIndex],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  // Convert vertical scroll to horizontal scroll in the carousel
  const handleWheel = useCallback((e: React.WheelEvent) => {
    if (scrollRef.current && Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
      scrollRef.current.scrollLeft += e.deltaY;
    }
  }, []);

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
    <div className="relative flex-1">
      {/* Right fade overlay to hint more clips */}
      <div className="pointer-events-none absolute top-0 right-0 z-10 h-full w-8 bg-gradient-to-l from-gray-900/80 to-transparent" />
      <div
        ref={scrollRef}
        className="flex h-full items-stretch gap-2.5 overflow-x-auto px-3 pb-3 scrollbar-hide"
        data-testid="clip-list"
        onWheel={handleWheel}
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
    </div>
  );
}
