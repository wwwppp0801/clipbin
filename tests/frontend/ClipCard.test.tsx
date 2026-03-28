import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import ClipCard from "../../src/components/ClipCard";
import { useClipStore, type ClipItem } from "../../src/stores/clipStore";

const textClip: ClipItem = {
  id: 1,
  content_type: "text",
  text_content: "Hello, World!",
  image_preview: null,
  created_at: new Date().toISOString(),
  last_used_at: new Date().toISOString(),
  use_count: 1,
  is_pinned: false,
};

const imageClip: ClipItem = {
  id: 2,
  content_type: "image",
  text_content: null,
  image_preview: "data:image/png;base64,abc123",
  created_at: new Date().toISOString(),
  last_used_at: new Date().toISOString(),
  use_count: 2,
  is_pinned: false,
};

const fileClip: ClipItem = {
  id: 3,
  content_type: "file_path",
  text_content: "/Users/test/document.pdf",
  image_preview: null,
  created_at: new Date().toISOString(),
  last_used_at: new Date().toISOString(),
  use_count: 1,
  is_pinned: false,
};

describe("ClipCard", () => {
  beforeEach(() => {
    useClipStore.setState({ clips: [], isLoading: false, searchQuery: "" });
  });

  it("renders text clip content", () => {
    render(<ClipCard clip={textClip} isSelected={false} />);
    expect(screen.getByText("Hello, World!")).toBeInTheDocument();
    expect(screen.getByTestId("clip-text")).toBeInTheDocument();
  });

  it("renders image clip with preview", () => {
    render(<ClipCard clip={imageClip} isSelected={false} />);
    const img = screen.getByTestId("clip-image");
    expect(img).toBeInTheDocument();
    expect(img).toHaveAttribute("src", "data:image/png;base64,abc123");
  });

  it("renders file path clip", () => {
    render(<ClipCard clip={fileClip} isSelected={false} />);
    expect(screen.getByText("document.pdf")).toBeInTheDocument();
    expect(screen.getByText("File")).toBeInTheDocument();
  });

  it("shows content type badge", () => {
    render(<ClipCard clip={imageClip} isSelected={false} />);
    expect(screen.getByText("Image")).toBeInTheDocument();
  });

  it("shows Text badge for text clips", () => {
    render(<ClipCard clip={textClip} isSelected={false} />);
    expect(screen.getByText("Text")).toBeInTheDocument();
  });

  it("calls pasteClip on click", () => {
    const pasteSpy = vi.fn();
    useClipStore.setState({ pasteClip: pasteSpy } as never);

    render(<ClipCard clip={textClip} isSelected={false} />);
    fireEvent.click(screen.getByTestId("clip-card"));
    expect(pasteSpy).toHaveBeenCalledWith(1);
  });

  it("calls deleteClip on delete button click", () => {
    const deleteSpy = vi.fn();
    useClipStore.setState({ deleteClip: deleteSpy } as never);

    render(<ClipCard clip={textClip} isSelected={false} />);
    fireEvent.click(screen.getByTestId("clip-delete"));
    expect(deleteSpy).toHaveBeenCalledWith(1);
  });
});
