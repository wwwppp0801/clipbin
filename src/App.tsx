import { useEffect } from "react";
import SearchBar from "./components/SearchBar";
import ClipList from "./components/ClipList";
import { useClipStore } from "./stores/clipStore";

function App() {
  const fetchClips = useClipStore((s) => s.fetchClips);
  const listenForChanges = useClipStore((s) => s.listenForChanges);

  useEffect(() => {
    fetchClips();
    let unlisten: (() => void) | undefined;
    listenForChanges().then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [fetchClips, listenForChanges]);

  return (
    <div className="flex h-screen flex-col bg-gray-900 text-white">
      <SearchBar />
      <ClipList />
    </div>
  );
}

export default App;
