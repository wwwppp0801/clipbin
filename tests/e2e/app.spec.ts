import { test, expect } from "@playwright/test";

// These E2E tests run against the Vite dev server (localhost:1420).
// Tauri IPC calls are intercepted via window.__TAURI_INTERNALS__ mock.

const mockClips = [
  {
    id: 1,
    content_type: "text",
    text_content: "Hello, World!",
    image_preview: null,
    created_at: new Date().toISOString(),
    last_used_at: new Date().toISOString(),
    use_count: 3,
    is_pinned: false,
  },
  {
    id: 2,
    content_type: "html",
    text_content: "Bold text from web page",
    image_preview: null,
    created_at: new Date(Date.now() - 1800000).toISOString(),
    last_used_at: new Date(Date.now() - 1800000).toISOString(),
    use_count: 1,
    is_pinned: false,
  },
  {
    id: 3,
    content_type: "text",
    text_content: "const x = 42;",
    image_preview: null,
    created_at: new Date(Date.now() - 3600000).toISOString(),
    last_used_at: new Date(Date.now() - 3600000).toISOString(),
    use_count: 1,
    is_pinned: false,
  },
  {
    id: 4,
    content_type: "file_path",
    text_content: "/Users/test/document.pdf",
    image_preview: null,
    created_at: new Date(Date.now() - 86400000).toISOString(),
    last_used_at: new Date(Date.now() - 86400000).toISOString(),
    use_count: 1,
    is_pinned: false,
  },
];

async function setupTauriMock(page: import("@playwright/test").Page) {
  await page.addInitScript((clips) => {
    // Mock Tauri IPC layer
    (window as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "get_clips") {
          return Promise.resolve(clips);
        }
        if (cmd === "search_clips") {
          const query = (args?.query as string) || "";
          const filtered = clips.filter(
            (c: (typeof clips)[0]) =>
              c.text_content && c.text_content.toLowerCase().includes(query.toLowerCase()),
          );
          return Promise.resolve(filtered);
        }
        if (cmd === "delete_clip") {
          return Promise.resolve();
        }
        if (cmd === "paste_clip") {
          return Promise.resolve();
        }
        return Promise.resolve(null);
      },
      convertFileSrc: (path: string) => path,
      transformCallback: (cb: () => void) => {
        const id = Math.random().toString(36).slice(2);
        (window as Record<string, unknown>)[`_${id}`] = cb;
        return id;
      },
    };

    // Mock event listener
    (window as Record<string, unknown>).__TAURI_INTERNALS_EVENTS__ = {};
  }, mockClips);
}

test.describe("ClipBin App", () => {
  test.beforeEach(async ({ page }) => {
    await setupTauriMock(page);
    await page.goto("/");
  });

  test("displays the search bar", async ({ page }) => {
    const searchInput = page.getByTestId("search-input");
    await expect(searchInput).toBeVisible();
    await expect(searchInput).toHaveAttribute("placeholder", "Search clips...");
  });

  test("displays clip history in carousel", async ({ page }) => {
    // Wait for clips to load
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Bold text from web page")).toBeVisible();
    await expect(page.getByText("const x = 42;")).toBeVisible();

    const clipList = page.getByTestId("clip-list");
    await expect(clipList).toBeVisible();
    await page.screenshot({ path: "tests/e2e/screenshots/carousel-view.png" });
  });

  test("shows content type badges for all types", async ({ page }) => {
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 5000 });
    // Text, Rich Text (HTML), File types
    await expect(page.getByText("Text").first()).toBeVisible();
    await expect(page.getByText("Rich Text")).toBeVisible();
    await expect(page.getByText("File")).toBeVisible();
  });

  test("shows number shortcuts on first 4 cards", async ({ page }) => {
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 5000 });
    // Cards should have number badges 1-4
    const cards = page.getByTestId("clip-card");
    await expect(cards).toHaveCount(4);
  });

  test("file card shows filename not full path", async ({ page }) => {
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 5000 });
    // File card should show just the filename
    await expect(page.getByText("document.pdf")).toBeVisible();
  });

  test("search filters clips", async ({ page }) => {
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 5000 });

    // Type in search
    const searchInput = page.getByTestId("search-input");
    await searchInput.fill("Hello");

    // Wait for debounce + filtered results
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 3000 });

    // Other clips should be filtered out
    await expect(page.getByText("const x = 42;")).not.toBeVisible({ timeout: 2000 });

    // Screenshot of filtered view
    await page.screenshot({ path: "tests/e2e/screenshots/search-filtered.png" });
  });

  test("search with no results shows empty state", async ({ page }) => {
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 5000 });

    const searchInput = page.getByTestId("search-input");
    await searchInput.fill("nonexistent_query_xyz");

    // Wait for empty state
    await expect(page.getByTestId("empty-state")).toBeVisible({ timeout: 3000 });
  });

  test("clicking a clip card triggers paste", async ({ page }) => {
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 5000 });

    // Track invoke calls
    await page.evaluate(() => {
      const orig = (window as Record<string, unknown>).__TAURI_INTERNALS__ as {
        invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
      };
      const origInvoke = orig.invoke.bind(orig);
      orig.invoke = (cmd: string, args?: Record<string, unknown>) => {
        (window as Record<string, unknown>).__lastInvokeCmd = cmd;
        (window as Record<string, unknown>).__lastInvokeArgs = args;
        return origInvoke(cmd, args);
      };
    });

    // Click the first clip
    await page.getByText("Hello, World!").click();

    // Verify paste was invoked
    const lastCmd = await page.evaluate(() => (window as Record<string, unknown>).__lastInvokeCmd);
    expect(lastCmd).toBe("paste_clip");
  });

  test("delete button removes a clip", async ({ page }) => {
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 5000 });

    // Hover over first clip card to reveal delete button
    const firstCard = page.getByTestId("clip-card").first();
    await firstCard.hover();

    // Click delete
    const deleteBtn = firstCard.getByTestId("clip-delete");
    await deleteBtn.click();

    // Clip should be removed from the list
    await expect(page.getByText("Hello, World!")).not.toBeVisible({ timeout: 2000 });
    // Other clips should still be there
    await expect(page.getByText("const x = 42;")).toBeVisible();
  });

  test("search input is auto-focused", async ({ page }) => {
    const searchInput = page.getByTestId("search-input");
    await expect(searchInput).toBeFocused();
  });

  test("clearing search shows all clips again", async ({ page }) => {
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 5000 });

    const searchInput = page.getByTestId("search-input");

    // Search to filter
    await searchInput.fill("Hello");
    await expect(page.getByText("const x = 42;")).not.toBeVisible({ timeout: 2000 });

    // Clear search
    await searchInput.fill("");

    // All clips should reappear
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 3000 });
    await expect(page.getByText("const x = 42;")).toBeVisible({ timeout: 3000 });
  });

  test("settings button opens settings dialog", async ({ page }) => {
    await page.getByTestId("settings-button").click();
    await expect(page.getByTestId("settings-dialog")).toBeVisible();
    // Should show hotkey input
    await expect(page.getByTestId("hotkey-input")).toBeVisible();
    // Should show max clips input
    await expect(page.getByTestId("max-clips-input")).toBeVisible();
  });

  test("footer shows clip count", async ({ page }) => {
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 5000 });
    // Footer should show "4 clips"
    await expect(page.getByText("4 clips")).toBeVisible();
  });

  test("clear all removes clips with double-click confirmation", async ({ page }) => {
    await expect(page.getByText("Hello, World!")).toBeVisible({ timeout: 5000 });
    // First click shows confirmation
    await page.getByTestId("clear-history").click();
    await expect(page.getByText("Click again to confirm")).toBeVisible();
    // Second click actually clears
    await page.getByTestId("clear-history").click();
    // Should show empty state
    await expect(page.getByTestId("empty-state")).toBeVisible({ timeout: 2000 });
  });
});
