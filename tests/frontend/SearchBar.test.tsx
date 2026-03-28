import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import SearchBar from "../../src/components/SearchBar";
import { useClipStore } from "../../src/stores/clipStore";

describe("SearchBar", () => {
  beforeEach(() => {
    useClipStore.setState({ searchQuery: "", clips: [], isLoading: false });
  });

  it("renders search input", () => {
    render(<SearchBar />);
    const input = screen.getByTestId("search-input");
    expect(input).toBeInTheDocument();
    expect(input).toHaveAttribute("placeholder", "Search clips...");
  });

  it("calls setSearchQuery on input change after debounce", async () => {
    const setSearchQuerySpy = vi.fn();
    useClipStore.setState({ setSearchQuery: setSearchQuerySpy } as never);

    render(<SearchBar />);
    const input = screen.getByTestId("search-input");
    fireEvent.change(input, { target: { value: "hello" } });

    // Wait for debounce (200ms)
    await vi.waitFor(
      () => {
        expect(setSearchQuerySpy).toHaveBeenCalledWith("hello");
      },
      { timeout: 500 },
    );
  });

  it("auto-focuses the input on mount", () => {
    render(<SearchBar />);
    expect(screen.getByTestId("search-input")).toHaveFocus();
  });
});
