import { useCallback, useEffect, useRef } from "react";
import { useClipStore } from "../stores/clipStore";

interface SearchBarProps {
  onOpenSettings: () => void;
}

export default function SearchBar({ onOpenSettings }: SearchBarProps) {
  const searchQuery = useClipStore((s) => s.searchQuery);
  const setSearchQuery = useClipStore((s) => s.setSearchQuery);
  const clipCount = useClipStore((s) => s.clips.length);
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value;
      if (debounceRef.current) clearTimeout(debounceRef.current);
      debounceRef.current = setTimeout(() => {
        setSearchQuery(value);
      }, 200);
    },
    [setSearchQuery],
  );

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div className="relative flex items-center gap-2 px-3 pt-1 pb-2" data-testid="search-bar">
      <input
        ref={inputRef}
        type="text"
        placeholder="Search clips..."
        aria-label="Search clipboard history"
        defaultValue={searchQuery}
        onChange={handleChange}
        data-testid="search-input"
        className="h-8 flex-1 rounded-lg border border-gray-700 bg-gray-800 px-3 pr-16 text-sm text-white placeholder-gray-500 outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
      />
      {searchQuery && (
        <span className="pointer-events-none absolute right-20 text-[10px] text-gray-500">
          {clipCount} result{clipCount !== 1 ? "s" : ""}
        </span>
      )}
      <button
        onClick={onOpenSettings}
        data-testid="settings-button"
        className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-gray-700 bg-gray-800 text-gray-400 transition-colors hover:bg-gray-700 hover:text-white"
        title="Settings"
        aria-label="Open settings"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
          <circle cx="12" cy="12" r="3" />
        </svg>
      </button>
    </div>
  );
}
