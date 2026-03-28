import { useEffect, useState } from "react";
import SearchBar from "./components/SearchBar";
import ClipList from "./components/ClipList";
import SettingsDialog from "./components/SettingsDialog";
import { useClipStore } from "./stores/clipStore";

function App() {
  const fetchClips = useClipStore((s) => s.fetchClips);
  const listenForChanges = useClipStore((s) => s.listenForChanges);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [animClass, setAnimClass] = useState("animate-slide-up");

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

  // Re-trigger entrance animation each time window becomes visible
  useEffect(() => {
    const handleFocus = () => {
      setAnimClass("");
      requestAnimationFrame(() => {
        setAnimClass("animate-slide-up");
      });
      // Refresh clips when window opens
      fetchClips();
    };
    window.addEventListener("focus", handleFocus);
    return () => window.removeEventListener("focus", handleFocus);
  }, [fetchClips]);

  return (
    <div
      className={`flex h-screen flex-col overflow-hidden rounded-2xl border border-gray-700/50 bg-gray-900/95 text-white shadow-2xl backdrop-blur-xl ${animClass}`}
    >
      <div className="flex justify-center pt-1.5 pb-0">
        <div className="h-1 w-10 rounded-full bg-gray-600/50" />
      </div>
      <SearchBar onOpenSettings={() => setSettingsOpen(true)} />
      <ClipList />
      <SettingsDialog isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
}

export default App;
