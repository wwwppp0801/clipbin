import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import SearchBar from "./components/SearchBar";
import ClipList from "./components/ClipList";
import SettingsDialog from "./components/SettingsDialog";
import { useClipStore } from "./stores/clipStore";

function App() {
  const fetchClips = useClipStore((s) => s.fetchClips);
  const listenForChanges = useClipStore((s) => s.listenForChanges);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [animState, setAnimState] = useState<"entering" | "exiting" | "visible" | "hidden">(
    "hidden",
  );

  // Listen for show/hide events from Rust
  useEffect(() => {
    const unlistenShow = listen("window-will-show", () => {
      fetchClips();
      setAnimState("entering");
    });

    const unlistenHide = listen("window-will-hide", () => {
      setAnimState("exiting");
      // After exit animation completes, tell Rust to actually hide
      setTimeout(() => {
        invoke("do_hide_window").catch(() => {});
        setAnimState("hidden");
      }, 200);
    });

    return () => {
      unlistenShow.then((fn) => fn());
      unlistenHide.then((fn) => fn());
    };
  }, [fetchClips]);

  // When entrance animation ends, mark as visible
  const handleAnimationEnd = useCallback(() => {
    if (animState === "entering") {
      setAnimState("visible");
    }
  }, [animState]);

  // Listen for clipboard changes
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listenForChanges().then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [listenForChanges]);

  const animClass =
    animState === "entering"
      ? "animate-slide-up"
      : animState === "exiting"
        ? "animate-slide-down"
        : "";

  return (
    <div
      className={`flex h-screen flex-col overflow-hidden rounded-2xl border border-gray-700/50 bg-gray-900/95 text-white shadow-2xl backdrop-blur-xl ${animClass}`}
      onAnimationEnd={handleAnimationEnd}
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
