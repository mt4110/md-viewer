// Run with Playwright installed and its Chromium available. No network requests.
const { chromium } = require("playwright");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { pathToFileURL } = require("node:url");
const { execFileSync } = require("node:child_process");
const root = path.resolve(__dirname, "..");
const fixtures = fs.mkdtempSync(path.join(os.tmpdir(), "md-search-test-"));
function render(name, text) {
  const file = path.join(fixtures, name);
  fs.writeFileSync(file, text);
  return JSON.parse(
    execFileSync(
      "rustup",
      [
        "run",
        "1.95.0",
        "cargo",
        "run",
        "--quiet",
        "--locked",
        "--manifest-path",
        path.join(root, "Cargo.toml"),
        "--example",
        "render",
        "--",
        file,
      ],
      { encoding: "utf8" },
    ),
  );
}
(async () => {
  const a = render(
    "A.md",
    "# A\n\n日本語の **検索** です。\n\nliteral [x].* İ i 😀\n\n" +
      "長い段落。\n\n".repeat(45) +
      "末尾 検索。\n",
  );
  const b = render(
    "B.md",
    "# B\n\n別タブの 検索 です。\n\n<script>unsafe</script>",
  );
  const c = render("C.md", "# C\n\n復元された 検索。\n");
  const browser = await chromium.launch(
    process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE
      ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE }
      : {},
  );
  try {
    const page = await browser.newPage({
      viewport: { width: 1100, height: 800 },
      offline: true,
    });
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(pathToFileURL(path.join(root, "ui/index.html")).href);
    await page.evaluate(
      ({ a, b, c }) => {
        window.searchReads = [];
        window.__TAURI_INTERNALS__ = {
          invoke: async (name, args) => {
            if (name === "search_document") {
              window.searchReads.push(args.source);
              if (args.source === c.id) return c;
              throw "missing";
            }
            if (name === "restore_document") return c;
            return null;
          },
        };
        show(a);
        show(b);
        activate(a.id);
        tabs.set(c.id, { doc: null, name: c.name, position: 0 });
        tabs.set("/missing.md", { doc: null, name: "missing.md", position: 0 });
      },
      { a, b, c },
    );
    assert(await page.locator(".search").isHidden());
    await page.keyboard.press("Control+f");
    assert.equal(await page.locator("#search-scope").inputValue(), "document");
    await page.locator("#query").fill("の 検索"); // spans <strong>
    assert.equal(await page.locator("#count").textContent(), "1件");
    assert.equal(await page.locator("mark").count(), 2);
    await page.locator("#next").click();
    assert.equal(await page.locator("mark.current").count(), 2);
    await page.locator("#query").fill("[x].*");
    assert.equal(await page.locator("#count").textContent(), "1件");
    await page.locator("#query").fill("😀");
    assert.equal(await page.locator("mark").textContent(), "😀");
    await page.locator("#query").fill("検索");
    await page.locator("#search-scope").selectOption("tabs");
    await page.waitForFunction(() => $("search-results").children.length === 4);
    assert.match(
      await page.locator("#count").textContent(),
      /1文書を検索できません/,
    );
    assert.deepEqual(
      await page.locator("#search-results strong").allTextContents(),
      ["A.md", "A.md", "B.md", "C.md"],
    );
    await page.locator("#search-results button").nth(2).click();
    await page.waitForFunction(
      () =>
        $("name").textContent === "B.md" &&
        $("document").querySelector("mark.current"),
    );
    assert(await page.locator("#search-results").isHidden());
    await page.locator("#results-toggle").click();
    assert(await page.locator("#search-results").isVisible());
    await page.locator("#query").focus();
    await page.keyboard.press("Enter");
    await page.waitForFunction(
      () =>
        $("name").textContent === "C.md" &&
        $("document").querySelector("mark.current"),
    );
    await page.keyboard.press("Shift+Enter");
    await page.waitForFunction(
      () =>
        $("name").textContent === "B.md" &&
        $("document").querySelector("mark.current"),
    );
    await page.evaluate((id) => closeTab(id), b.id);
    await page.waitForFunction(() => $("search-results").children.length === 3);
    assert(
      !(
        await page.locator("#search-results strong").allTextContents()
      ).includes("B.md"),
    );
    // A changed snapshot must not jump to the old text offset.
    await page.evaluate((id) => {
      tabs.get(id).doc.html = "<p>changed content</p>";
    }, c.id);
    await page.locator("#search-results button").last().click();
    await page.waitForFunction(() => $("search-results").children.length === 2);
    assert.equal(await page.locator("mark.current").count(), 0);
    assert.match(await page.locator("#status").textContent(), /文書が変わった/);
    // Query revisions reject delayed responses; closed tabs cannot reappear.
    await page.evaluate(() => {
      tabs.set("/slow.md", { doc: null, name: "slow.md", position: 0 });
      window.__TAURI_INTERNALS__.invoke = async (name, args) => {
        if (name === "search_document" && args.source === "/slow.md")
          return new Promise((resolve) => (window.releaseSearch = resolve));
        if (name === "search_document") throw "missing";
        return null;
      };
    });
    await page.locator("#query").fill("old");
    await page.waitForFunction(
      () => typeof window.releaseSearch === "function",
    );
    await page.locator("#query").fill("new");
    await page.evaluate(() => {
      closeTab("/slow.md");
      window.releaseSearch({ html: "<p>old</p>" });
    });
    await page.waitForTimeout(250);
    assert.equal(await page.locator("#search-results button").count(), 0);
    await page.evaluate(() => {
      tabs.get(active).doc.html = "<p>" + "hit ".repeat(250) + "</p>";
      paint(tabs.get(active).doc, true);
    });
    await page.locator("#search-scope").selectOption("document");
    await page.locator("#query").dispatchEvent("compositionstart");
    assert.equal(
      await page.evaluate(() => composing),
      true,
      "compositionstart handler",
    );
    await page.evaluate(() => {
      $("query").value = "hit";
      $("query").dispatchEvent(
        new InputEvent("input", { bubbles: true, isComposing: true }),
      );
    });
    assert.equal(await page.locator("mark").count(), 0);
    await page.locator("#query").dispatchEvent("compositionend");
    assert.match(await page.locator("#count").textContent(), /先頭200件/);
    assert.equal(await page.locator("mark").count(), 200);
    await page.evaluate(() => changeLanguage("en"));
    assert.equal(
      await page.locator("#search-scope option").nth(1).textContent(),
      "Open tabs",
    );
    await page.setViewportSize({ width: 550, height: 700 });
    assert.equal(
      await page.evaluate(
        () => document.documentElement.scrollWidth > innerWidth,
      ),
      false,
    );
    await page.keyboard.press("Escape");
    assert(await page.locator(".search").isHidden());
    assert.equal(await page.locator("mark").count(), 0);
    await page.keyboard.press("Meta+f");
    assert(
      await page
        .locator("#query")
        .evaluate((e) => e === document.activeElement),
    );
    assert.deepEqual(errors, []);
    console.log(
      "PASS: Rust-rendered text, inline phrase, literal/Unicode, scopes, restored/missing tabs, cross-tab navigation, close/stale results, cap, JA/EN, shortcuts, narrow layout",
    );
    console.log(`Fixtures preserved: ${fixtures}`);
  } finally {
    await browser.close();
  }
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
