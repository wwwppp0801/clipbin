import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import ClipList from "../../src/components/ClipList";
import { useClipStore, type ClipItem } from "../../src/stores/clipStore";

const mockClips: ClipItem[] = [
  {
    id: 1,
    content_type: "text",
    text_content: "first clip",
    image_preview: null,
    created_at: new Date().toISOString(),
    last_used_at: new Date().toISOString(),
    use_count: 1,
    is_pinned: false,
  },
  {
    id: 2,
    content_type: "text",
    text_content: "second clip",
    image_preview: null,
    created_at: new Date().toISOString(),
    last_used_at: new Date().toISOString(),
    use_count: 3,
    is_pinned: false,
  },
];

describe("ClipList", () => {
  beforeEach(() => {
    useClipStore.setState({ clips: [], isLoading: false, searchQuery: "" });
  });

  it("shows empty state when no clips", () => {
    render(<ClipList />);
    expect(screen.getByTestId("empty-state")).toBeInTheDocument();
    expect(screen.getByText(/No clips yet/)).toBeInTheDocument();
  });

  it("renders clip cards when clips exist", () => {
    useClipStore.setState({ clips: mockClips });
    render(<ClipList />);

    expect(screen.getByTestId("clip-list")).toBeInTheDocument();
    expect(screen.getAllByTestId("clip-card")).toHaveLength(2);
    expect(screen.getByText("first clip")).toBeInTheDocument();
    expect(screen.getByText("second clip")).toBeInTheDocument();
  });

  it("shows loading state", () => {
    useClipStore.setState({ isLoading: true, clips: [] });
    render(<ClipList />);
    expect(screen.getByText("Loading...")).toBeInTheDocument();
  });
});
