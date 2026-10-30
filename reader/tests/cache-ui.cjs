const { chromium } = require("playwright");
const assert = require("node:assert/strict"),
  path = require("node:path");
const { pathToFileURL } = require("node:url");
(async () => {
  const browser = await chromium.launch(
    process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE
      ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE }
      : {},
  );
  try {
    const p = await browser.newPage();
    await p.goto(
      pathToFileURL(path.resolve(__dirname, "../ui/index.html")).href,
    );
    await p.evaluate(async () => {
      window.__TAURI_INTERNALS__ = {
        invoke: async (name) =>
          name === "restore_document"
            ? {
                id: "/two.md",
                name: "two",
                html: "y".repeat(17 * 1024 * 1024),
                headings: [],
              }
            : null,
      };
      tabs.set("/one.md", {
        doc: { id: "/one.md", html: "x".repeat(17 * 1024 * 1024) },
        position: 0,
      });
      tabs.set("/two.md", { doc: null, name: "two", position: 0 });
      await activate("/two.md");
    });
    assert(await p.evaluate(() => tabs.get("/two.md").doc === null));
    assert.match(await p.locator("#document").textContent(), /上限/);
    await p.evaluate(async () => {
      tabs.get("/one.md").doc = null;
      window.__TAURI_INTERNALS__.invoke = async (name) =>
        name === "restore_document"
          ? { id: "/two.md", name: "two", html: "<h1>small</h1>", headings: [] }
          : null;
      await activate("/two.md");
    });
    assert.equal(await p.locator("#document h1").textContent(), "small");
    console.log(
      "PASS: restored cache limit rejects without retaining candidate; retry after freeing cache works",
    );
  } finally {
    await browser.close();
  }
})().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
