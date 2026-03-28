import { test } from "@playwright/test";

/**
 * Screenshot capture for product documentation.
 * Run with: pnpm playwright test screenshots.spec.ts
 * Screenshots saved to docs/images/
 */

const mockClips = [
  {
    id: 1,
    content_type: "text",
    text_content: "Hello, World! This is a plain text clip that was copied from the terminal.",
    image_preview: null,
    created_at: new Date().toISOString(),
    last_used_at: new Date().toISOString(),
    use_count: 5,
    is_pinned: false,
  },
  {
    id: 2,
    content_type: "html",
    text_content:
      "Apple today announced macOS 16 with redesigned Finder and improved clipboard management across all devices.",
    image_preview: null,
    created_at: new Date(Date.now() - 600000).toISOString(),
    last_used_at: new Date(Date.now() - 600000).toISOString(),
    use_count: 2,
    is_pinned: false,
  },
  {
    id: 3,
    content_type: "text",
    text_content:
      'const handlePaste = async (id: number) => {\n  await invoke("paste_clip", { id });\n};',
    image_preview: null,
    created_at: new Date(Date.now() - 1800000).toISOString(),
    last_used_at: new Date(Date.now() - 1800000).toISOString(),
    use_count: 1,
    is_pinned: false,
  },
  {
    id: 4,
    content_type: "file_path",
    text_content: "/Users/wangpeng/Documents/presentation.key",
    image_preview: null,
    created_at: new Date(Date.now() - 3600000).toISOString(),
    last_used_at: new Date(Date.now() - 3600000).toISOString(),
    use_count: 1,
    is_pinned: false,
  },
  {
    id: 5,
    content_type: "text",
    text_content: "npm install -g @tauri-apps/cli",
    image_preview: null,
    created_at: new Date(Date.now() - 7200000).toISOString(),
    last_used_at: new Date(Date.now() - 7200000).toISOString(),
    use_count: 3,
    is_pinned: false,
  },
  {
    id: 6,
    content_type: "html",
    text_content:
      "Rust is a multi-paradigm, general-purpose programming language that emphasizes performance, type safety, and concurrency.",
    image_preview: null,
    created_at: new Date(Date.now() - 14400000).toISOString(),
    last_used_at: new Date(Date.now() - 14400000).toISOString(),
    use_count: 1,
    is_pinned: false,
  },
];

async function setupMock(page: import("@playwright/test").Page) {
  await page.addInitScript((clips) => {
    (window as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "get_clips") return Promise.resolve(clips);
        if (cmd === "search_clips") {
          const q = (args?.query as string) || "";
          return Promise.resolve(
            clips.filter(
              (c: (typeof clips)[0]) =>
                c.text_content && c.text_content.toLowerCase().includes(q.toLowerCase()),
            ),
          );
        }
        return Promise.resolve(null);
      },
      transformCallback: (cb: () => void) => {
        const id = Math.random().toString(36).slice(2);
        (window as Record<string, unknown>)[`_${id}`] = cb;
        return id;
      },
      convertFileSrc: (path: string) => path,
    };
  }, mockClips);
}

test.describe("Product Screenshots", () => {
  test.beforeEach(async ({ page }) => {
    await setupMock(page);
    await page.goto("/");
    await page.waitForTimeout(500);
  });

  test("01 - Main carousel view", async ({ page }) => {
    await page.screenshot({ path: "docs/images/01-main-view.png" });
  });

  test("02 - Search filtering", async ({ page }) => {
    await page.getByTestId("search-input").fill("Rust");
    await page.waitForTimeout(500);
    await page.screenshot({ path: "docs/images/02-search.png" });
  });

  test("03 - Empty search results", async ({ page }) => {
    await page.getByTestId("search-input").fill("nonexistent_xyz");
    await page.waitForTimeout(500);
    await page.screenshot({ path: "docs/images/03-empty-search.png" });
  });

  test("04 - Settings button visible", async ({ page }) => {
    await page.screenshot({ path: "docs/images/04-settings-button.png" });
  });
});
