import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Settings {
  hotkey: string;
  max_clips: number;
  ignored_apps: string[];
}

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function SettingsDialog({ isOpen, onClose }: SettingsDialogProps) {
  const [hotkey, setHotkey] = useState("Shift+CmdOrCtrl+V");
  const [maxClips, setMaxClips] = useState(500);
  const [ignoredApps, setIgnoredApps] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [recordingHotkey, setRecordingHotkey] = useState(false);

  useEffect(() => {
    if (isOpen) {
      invoke<Settings>("get_settings").then((s) => {
        setHotkey(s.hotkey);
        setMaxClips(s.max_clips);
        setIgnoredApps((s.ignored_apps || []).join(", "));
      });
    }
  }, [isOpen]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (!recordingHotkey) return;
      e.preventDefault();
      e.stopPropagation();

      const parts: string[] = [];
      if (e.metaKey) parts.push("CmdOrCtrl");
      if (e.ctrlKey && !e.metaKey) parts.push("CmdOrCtrl");
      if (e.shiftKey) parts.push("Shift");
      if (e.altKey) parts.push("Alt");

      const key = e.key;
      if (!["Meta", "Control", "Shift", "Alt"].includes(key)) {
        parts.push(key.length === 1 ? key.toUpperCase() : key);
        setHotkey(parts.join("+"));
        setRecordingHotkey(false);
      }
    },
    [recordingHotkey],
  );

  const handleSave = async () => {
    setIsSaving(true);
    try {
      const ignoredAppsList = ignoredApps
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      await invoke("save_settings", { hotkey, maxClips, ignoredApps: ignoredAppsList });
      onClose();
    } catch (err) {
      console.error("Failed to save settings:", err);
    } finally {
      setIsSaving(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div
        className="max-h-[80vh] w-[360px] overflow-y-auto rounded-2xl border border-gray-700/50 bg-gray-900 p-5 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
        data-testid="settings-dialog"
      >
        <h2 className="mb-4 text-base font-semibold text-white">Settings</h2>

        {/* Hotkey */}
        <div className="mb-4">
          <label className="mb-1.5 block text-xs font-medium text-gray-400">Toggle Hotkey</label>
          <div
            onClick={() => setRecordingHotkey(true)}
            onKeyDown={handleKeyDown}
            tabIndex={0}
            data-testid="hotkey-input"
            className={`flex h-9 cursor-pointer items-center rounded-lg border px-3 text-sm outline-none ${
              recordingHotkey
                ? "border-blue-500 bg-blue-500/10 text-blue-400"
                : "border-gray-700 bg-gray-800 text-white"
            }`}
          >
            {recordingHotkey ? "Press a key combination..." : hotkey}
          </div>
        </div>

        {/* Max clips */}
        <div className="mb-5">
          <label className="mb-1.5 block text-xs font-medium text-gray-400">
            Max Clipboard History
          </label>
          <input
            type="number"
            min={10}
            max={10000}
            value={maxClips}
            onChange={(e) => setMaxClips(Number(e.target.value))}
            data-testid="max-clips-input"
            className="h-9 w-full rounded-lg border border-gray-700 bg-gray-800 px-3 text-sm text-white outline-none focus:border-blue-500"
          />
        </div>

        {/* Ignored Apps */}
        <div className="mb-4">
          <label className="mb-1.5 block text-xs font-medium text-gray-400">
            Ignored Apps (comma-separated)
          </label>
          <input
            type="text"
            value={ignoredApps}
            onChange={(e) => setIgnoredApps(e.target.value)}
            placeholder="1Password, Keychain Access"
            className="h-9 w-full rounded-lg border border-gray-700 bg-gray-800 px-3 text-sm text-white outline-none focus:border-blue-500"
          />
          <p className="mt-1 text-[10px] text-gray-500">
            Clips from these apps won&apos;t be saved
          </p>
        </div>

        {/* Keyboard Shortcuts Reference */}
        <div className="mb-5 rounded-lg border border-gray-700/50 bg-gray-800/50 p-3">
          <h3 className="mb-2 text-xs font-medium text-gray-400">Keyboard Shortcuts</h3>
          <div className="grid grid-cols-2 gap-y-1 text-[11px]">
            <span className="text-gray-500">Toggle panel</span>
            <span className="text-right text-gray-300">⇧⌘V</span>
            <span className="text-gray-500">Paste selected</span>
            <span className="text-right text-gray-300">Enter</span>
            <span className="text-gray-500">Copy to clipboard</span>
            <span className="text-right text-gray-300">⌘C</span>
            <span className="text-gray-500">Quick paste</span>
            <span className="text-right text-gray-300">1-9</span>
            <span className="text-gray-500">Navigate</span>
            <span className="text-right text-gray-300">← →</span>
            <span className="text-gray-500">First / Last</span>
            <span className="text-right text-gray-300">Home / End</span>
            <span className="text-gray-500">Toggle pin</span>
            <span className="text-right text-gray-300">⌘P</span>
            <span className="text-gray-500">Delete clip</span>
            <span className="text-right text-gray-300">⌫</span>
            <span className="text-gray-500">Focus search</span>
            <span className="text-right text-gray-300">Tab</span>
            <span className="text-gray-500">Dismiss</span>
            <span className="text-right text-gray-300">Esc</span>
            <span className="text-gray-500">Preview full</span>
            <span className="text-right text-gray-300">Double-click</span>
          </div>
        </div>

        {/* About */}
        <div className="mb-5 text-center text-[10px] text-gray-500">ClipBin v0.2.0</div>

        {/* Buttons */}
        <div className="flex justify-end gap-2">
          <button
            onClick={onClose}
            className="rounded-lg px-4 py-1.5 text-sm text-gray-400 transition-colors hover:bg-gray-800 hover:text-white"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving}
            data-testid="settings-save"
            className="rounded-lg bg-blue-600 px-4 py-1.5 text-sm font-medium text-white transition-colors hover:bg-blue-500 disabled:opacity-50"
          >
            {isSaving ? "Saving..." : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}
