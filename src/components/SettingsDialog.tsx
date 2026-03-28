import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Settings {
  hotkey: string;
  max_clips: number;
}

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function SettingsDialog({ isOpen, onClose }: SettingsDialogProps) {
  const [hotkey, setHotkey] = useState("Shift+CmdOrCtrl+V");
  const [maxClips, setMaxClips] = useState(500);
  const [isSaving, setIsSaving] = useState(false);
  const [recordingHotkey, setRecordingHotkey] = useState(false);

  useEffect(() => {
    if (isOpen) {
      invoke<Settings>("get_settings").then((s) => {
        setHotkey(s.hotkey);
        setMaxClips(s.max_clips);
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
      await invoke("save_settings", { hotkey, maxClips });
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
        className="w-[360px] rounded-2xl border border-gray-700/50 bg-gray-900 p-5 shadow-2xl"
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
