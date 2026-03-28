import { useCallback, useEffect, useRef } from "react";
import { useClipStore } from "../stores/clipStore";

export default function SearchBar() {
  const searchQuery = useClipStore((s) => s.searchQuery);
  const setSearchQuery = useClipStore((s) => s.setSearchQuery);
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>();

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
    <div className="px-3 pt-3 pb-2" data-testid="search-bar">
      <input
        ref={inputRef}
        type="text"
        placeholder="Search clips..."
        defaultValue={searchQuery}
        onChange={handleChange}
        data-testid="search-input"
        className="w-full rounded-lg border border-gray-700 bg-gray-800 px-3 py-2 text-sm text-white placeholder-gray-500 outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
      />
    </div>
  );
}
