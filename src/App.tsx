import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import SearchBar from "./components/SearchBar";
import ClipList from "./components/ClipList";
import SettingsDialog from "./components/SettingsDialog";
import { useClipStore } from "./stores/clipStore";

type AnimState = "entering" | "exiting" | "visible" | "hidden";

function App() {
  const fetchClips = useClipStore((s) => s.fetchClips);
  const listenForChanges = useClipStore((s) => s.listenForChanges);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [animState, setAnimState] = useState<AnimState>("hidden");

  // Initial fetch on mount (for web/E2E testing and first load)
  useEffect(() => {
    fetchClips();
    setAnimState("visible");
  }, [fetchClips]);

  useEffect(() => {
    const unlistenShow = listen("window-will-show", () => {
      fetchClips();
      setAnimState("entering");
    });

    const unlistenHide = listen("window-will-hide", () => {
      setAnimState("exiting");
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

  const handleAnimationEnd = useCallback(() => {
    if (animState === "entering") {
      setAnimState("visible");
    }
  }, [animState]);

  // Escape key to dismiss
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && animState === "visible") {
        invoke("do_hide_window").catch(() => {});
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [animState]);

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
