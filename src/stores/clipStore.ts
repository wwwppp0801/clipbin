import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface ClipItem {
  id: number;
  content_type: "text" | "html" | "image" | "file_path";
  text_content: string | null;
  image_preview: string | null;
  source_app: string | null;
  created_at: string;
  last_used_at: string;
  use_count: number;
  is_pinned: boolean;
}

interface ClipStore {
  clips: ClipItem[];
  searchQuery: string;
  isLoading: boolean;

  setSearchQuery: (query: string) => void;
  fetchClips: () => Promise<void>;
  searchClips: (query: string) => Promise<void>;
  deleteClip: (id: number) => Promise<void>;
  copyClip: (id: number) => Promise<void>;
  pasteClip: (id: number) => Promise<void>;
  toastMessage: string;
  showToast: (msg: string) => void;
  togglePin: (id: number) => Promise<void>;
  clearHistory: () => Promise<void>;
  addClip: (clip: ClipItem) => void;
  listenForChanges: () => Promise<UnlistenFn>;
}

export const useClipStore = create<ClipStore>((set, get) => ({
  clips: [],
  searchQuery: "",
  isLoading: false,
  toastMessage: "",

  showToast: (msg: string) => {
    set({ toastMessage: msg });
    setTimeout(() => set({ toastMessage: "" }), 1500);
  },

  setSearchQuery: (query: string) => {
    set({ searchQuery: query });
    if (query.trim()) {
      get().searchClips(query);
    } else {
      get().fetchClips();
    }
  },

  fetchClips: async () => {
    set({ isLoading: true });
    try {
      const clips = await invoke<ClipItem[]>("get_clips", { limit: 50, offset: 0 });
      set({ clips, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },

  searchClips: async (query: string) => {
    set({ isLoading: true });
    try {
      const clips = await invoke<ClipItem[]>("search_clips", { query, limit: 50 });
      set({ clips, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },

  deleteClip: async (id: number) => {
    try {
      await invoke("delete_clip", { id });
      set((state) => ({
        clips: state.clips.filter((c) => c.id !== id),
      }));
    } catch {
      // silently fail
    }
  },

  copyClip: async (id: number) => {
    try {
      await invoke("copy_clip", { id });
      get().showToast("Copied to clipboard");
    } catch {
      // silently fail
    }
  },

  pasteClip: async (id: number) => {
    try {
      await invoke("paste_clip", { id });
    } catch {
      // silently fail
    }
  },

  togglePin: async (id: number) => {
    try {
      const newPinned = await invoke<boolean>("toggle_pin", { id });
      set((state) => ({
        clips: state.clips.map((c) => (c.id === id ? { ...c, is_pinned: newPinned } : c)),
      }));
    } catch {
      // silently fail
    }
  },

  clearHistory: async () => {
    try {
      await invoke("clear_history");
      set((state) => ({
        clips: state.clips.filter((c) => c.is_pinned),
      }));
    } catch {
      // silently fail
    }
  },

  addClip: (clip: ClipItem) => {
    set((state) => ({
      clips: [clip, ...state.clips.filter((c) => c.id !== clip.id)],
    }));
  },

  listenForChanges: async () => {
    const unlisten = await listen<ClipItem>("clipboard-changed", (event) => {
      get().addClip(event.payload);
    });
    return unlisten;
  },
}));
