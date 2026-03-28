import { describe, it, expect } from "vitest";
import { formatRelativeTime, truncateText, getContentIcon, isUrl } from "../../src/lib/utils";

describe("truncateText", () => {
  it("returns short text unchanged", () => {
    expect(truncateText("hello", 10)).toBe("hello");
  });

  it("truncates long text with ellipsis", () => {
    expect(truncateText("hello world this is long", 10)).toBe("hello worl...");
  });

  it("handles exact length", () => {
    expect(truncateText("12345", 5)).toBe("12345");
  });

  it("handles empty string", () => {
    expect(truncateText("", 10)).toBe("");
  });
});

describe("formatRelativeTime", () => {
  it("returns 'just now' for recent times", () => {
    const now = new Date().toISOString();
    expect(formatRelativeTime(now)).toBe("just now");
  });

  it("returns minutes for times within an hour", () => {
    const fiveMinAgo = new Date(Date.now() - 5 * 60 * 1000).toISOString();
    expect(formatRelativeTime(fiveMinAgo)).toBe("5m ago");
  });

  it("returns hours for times within a day", () => {
    const twoHoursAgo = new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString();
    expect(formatRelativeTime(twoHoursAgo)).toBe("2h ago");
  });

  it("returns days for times older than a day", () => {
    const threeDaysAgo = new Date(Date.now() - 3 * 24 * 60 * 60 * 1000).toISOString();
    expect(formatRelativeTime(threeDaysAgo)).toBe("3d ago");
  });
});

describe("getContentIcon", () => {
  it("returns correct icon for text", () => {
    expect(getContentIcon("text")).toBe("text");
  });

  it("returns correct icon for image", () => {
    expect(getContentIcon("image")).toBe("image");
  });

  it("returns correct icon for file_path", () => {
    expect(getContentIcon("file_path")).toBe("file");
  });
});

describe("isUrl", () => {
  it("detects http URLs", () => {
    expect(isUrl("http://example.com")).toBe(true);
  });

  it("detects https URLs", () => {
    expect(isUrl("https://example.com/path?q=1")).toBe(true);
  });

  it("rejects plain text", () => {
    expect(isUrl("hello world")).toBe(false);
  });

  it("rejects multi-line URLs", () => {
    expect(isUrl("https://a.com\nhttps://b.com")).toBe(false);
  });

  it("handles null/undefined", () => {
    expect(isUrl(null)).toBe(false);
    expect(isUrl(undefined)).toBe(false);
  });
});
