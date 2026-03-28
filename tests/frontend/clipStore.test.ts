import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useClipStore, type ClipItem } from "../../src/stores/clipStore";

const mockInvoke = vi.mocked(invoke);

const mockClip: ClipItem = {
  id: 1,
  content_type: "text",
  text_content: "hello world",
  image_preview: null,
  created_at: "2024-01-01T00:00:00Z",
  last_used_at: "2024-01-01T00:00:00Z",
  use_count: 1,
  is_pinned: false,
};

const mockClip2: ClipItem = {
  id: 2,
  content_type: "text",
  text_content: "goodbye world",
  image_preview: null,
  created_at: "2024-01-01T00:01:00Z",
  last_used_at: "2024-01-01T00:01:00Z",
  use_count: 1,
  is_pinned: false,
};

describe("clipStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useClipStore.setState({ clips: [], searchQuery: "", isLoading: false });
  });

  it("fetchClips calls invoke and updates state", async () => {
    mockInvoke.mockResolvedValue([mockClip, mockClip2]);

    await useClipStore.getState().fetchClips();

    expect(mockInvoke).toHaveBeenCalledWith("get_clips", { limit: 50, offset: 0 });
    expect(useClipStore.getState().clips).toHaveLength(2);
    expect(useClipStore.getState().isLoading).toBe(false);
  });

  it("searchClips calls invoke with query", async () => {
    mockInvoke.mockResolvedValue([mockClip]);

    await useClipStore.getState().searchClips("hello");

    expect(mockInvoke).toHaveBeenCalledWith("search_clips", { query: "hello", limit: 50 });
    expect(useClipStore.getState().clips).toHaveLength(1);
  });

  it("deleteClip removes clip from state", async () => {
    useClipStore.setState({ clips: [mockClip, mockClip2] });
    mockInvoke.mockResolvedValue(undefined);

    await useClipStore.getState().deleteClip(1);

    expect(mockInvoke).toHaveBeenCalledWith("delete_clip", { id: 1 });
    expect(useClipStore.getState().clips).toHaveLength(1);
    expect(useClipStore.getState().clips[0].id).toBe(2);
  });

  it("addClip prepends new clip and deduplicates", () => {
    useClipStore.setState({ clips: [mockClip] });

    useClipStore.getState().addClip(mockClip2);

    const clips = useClipStore.getState().clips;
    expect(clips).toHaveLength(2);
    expect(clips[0].id).toBe(2); // new clip first
  });

  it("addClip replaces existing clip with same id", () => {
    useClipStore.setState({ clips: [mockClip, mockClip2] });
    const updatedClip = { ...mockClip, text_content: "updated" };

    useClipStore.getState().addClip(updatedClip);

    const clips = useClipStore.getState().clips;
    expect(clips).toHaveLength(2);
    expect(clips[0].text_content).toBe("updated");
  });

  it("setSearchQuery triggers fetch when empty", async () => {
    mockInvoke.mockResolvedValue([]);

    useClipStore.getState().setSearchQuery("");

    // fetchClips should be called
    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_clips", { limit: 50, offset: 0 });
    });
  });

  it("setSearchQuery triggers search when non-empty", async () => {
    mockInvoke.mockResolvedValue([]);

    useClipStore.getState().setSearchQuery("test");

    await vi.waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("search_clips", { query: "test", limit: 50 });
    });
  });
});
