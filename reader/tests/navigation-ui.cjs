// Permanent regression checks. Requires Playwright and Chromium; IPC is mocked.
const { chromium } = require("playwright");
const assert = require("node:assert/strict");
const fs = require("node:fs"),
  os = require("node:os"),
  path = require("node:path");
const { execFileSync } = require("node:child_process");
const { pathToFileURL } = require("node:url");
const root = path.resolve(__dirname, "../..");
const fixtures = fs.mkdtempSync(path.join(os.tmpdir(), "md-navigation-"));
let md = "# 文書A\n\n[節70へ](#節70) [壊れた見出し](#missing)\n\n";
for (let i = 0; i < 130; i++) {
  if (i % 10 === 0) md += `## 節${i}\n\n`;
  md += `段落${i}。日本語の読書位置を確認します。${i === 90 ? "検索対象unique" : ""}\n\n`;
}
for (const heading of ["foo", "foo-1", "foo"]) {
  md += `# ${heading}\n\n` + "見出し移動の確認用本文。\n\n".repeat(30);
}
md += "[末尾のfooへ](#foo-2)\n\n";
fs.writeFileSync(path.join(fixtures, "A.md"), md);
const doc = JSON.parse(
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
      path.join(root, "reader/Cargo.toml"),
      "--example",
      "render",
      "--",
      path.join(fixtures, "A.md"),
    ],
    { encoding: "utf8" },
  ),
);
(async () => {
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
    await page.goto(
      pathToFileURL(path.join(root, "reader/ui/index.html")).href,
    );
    await page.evaluate((d) => {
      window.__TAURI_INTERNALS__ = { invoke: async () => null };
      show(d);
    }, doc);
    const frame = () => page.evaluate(() => new Promise(requestAnimationFrame));
    await frame();
    const scroll = async (n) => {
      await page.evaluate((n) => {
        [...$("document").querySelectorAll("p")]
          .find((p) => p.textContent.startsWith(`段落${n}。`))
          .scrollIntoView();
        window.scrollBy(0, -readingTop() - 7);
      }, n);
    };
    const anchor = () =>
      page.evaluate(() => storedAnchor(captureReadingPosition()));
    const near = async (expected) => {
      const actual = await anchor();
      assert.equal(actual.block_hash, expected.block_hash);
      assert.equal(actual.occurrence, expected.occurrence);
      assert(
        Math.abs(actual.top - expected.top) < 2,
        JSON.stringify({ actual, expected }),
      );
    };
    assert(await page.locator("#back").isDisabled());
    assert(await page.locator("#forward").isDisabled());
    assert(await page.locator("#tools").isHidden());
    await scroll(10);
    const origin = await anchor();
    await page.locator("#menu-toggle").click();
    await page.locator("#contents").click();
    await page
      .locator("#outline a")
      .filter({ hasText: /^節70$/ })
      .click();
    assert.equal(
      await page.evaluate(() => captureReadingPosition().heading),
      "節70",
    );
    assert.equal(
      await page.evaluate(() => tabs.get(active).history.back.length),
      1,
    );
    await scroll(75);
    const destination = await anchor();
    await page.keyboard.press("Meta+[");
    await near(origin);
    assert(await page.locator("#back").isDisabled());
    assert(!(await page.locator("#forward").isDisabled()));
    await page.keyboard.press("Meta+]");
    await near(destination);
    await page.keyboard.press("Alt+ArrowLeft");
    await near(origin);
    await page.keyboard.press("Alt+ArrowRight");
    await near(destination);
    await page.keyboard.press("Meta+[");
    await page.evaluate(() => goToHeading("節40"));
    assert(await page.locator("#forward").isDisabled());
    const count = await page.evaluate(
      () => tabs.get(active).history.back.length,
    );
    await page.evaluate(() => goToHeading("節40"));
    assert.equal(
      await page.evaluate(() => tabs.get(active).history.back.length),
      count,
    );
    await page.evaluate(() => goToHeading("missing"));
    assert.equal(
      await page.evaluate(() => tabs.get(active).history.back.length),
      count,
    );
    // Inline fragment links use the same history and never the WebView global history.
    await page.locator("#document a").first().click();
    assert.equal(
      await page.evaluate(() => captureReadingPosition().heading),
      "節70",
    );
    assert.equal(await page.evaluate(() => location.hash), "");
    const aPosition = await anchor();
    const aHistory = await page.evaluate(() =>
      JSON.stringify(tabs.get(active).history),
    );
    const second = { ...doc, id: doc.id + "-B", name: "B.md" };
    await page.evaluate((d) => show(d), second);
    await frame();
    assert(await page.locator("#back").isDisabled());
    await scroll(20);
    const bOrigin = await anchor();
    await page.evaluate(() => goToHeading("節60"));
    const bTarget = await anchor();
    await page.evaluate((id) => activate(id), doc.id);
    await frame();
    await near(aPosition);
    assert.equal(
      await page.evaluate(() => JSON.stringify(tabs.get(active).history)),
      aHistory,
    );
    await page.evaluate((id) => activate(id), second.id);
    await frame();
    await near(bTarget);
    await page.keyboard.press("Meta+[");
    await near(bOrigin);
    // Search-result jumps are recorded in the destination tab.
    await page.keyboard.press("Control+f");
    await page.locator("#query").fill("検索対象unique");
    const beforeSearch = await anchor();
    await page.locator("#next").click();
    assert(await page.locator("mark.current").count());
    await page.locator("#main").focus();
    await page.keyboard.press("Meta+[");
    await near(beforeSearch);
    // Text input/IME shortcuts are not consumed by history navigation.
    await page.locator("#query").focus();
    const stack = await page.evaluate(() =>
      JSON.stringify(tabs.get(active).history),
    );
    await page.keyboard.press("Meta+]");
    assert.equal(
      await page.evaluate(() => JSON.stringify(tabs.get(active).history)),
      stack,
    );
    await page.locator("#query").dispatchEvent("compositionstart");
    await page.locator("#main").focus();
    await page.keyboard.press("Meta+]");
    assert.equal(
      await page.evaluate(() => JSON.stringify(tabs.get(active).history)),
      stack,
    );
    await page.locator("#query").dispatchEvent("compositionend");
    await page.keyboard.press("Escape");
    // Anchors survive inserted content, with a visible fallback if their paragraph disappears.
    await scroll(10);
    const beforeUpdate = await anchor();
    await page.evaluate(() => goToHeading("節80"));
    await page.evaluate(() => {
      const d = tabs.get(active).doc;
      d.html = "<p>挿入した文章</p>" + d.html;
      paint(d, true);
      travel("back");
    });
    await near(beforeUpdate);
    await page.evaluate(() => {
      travel("forward");
      const d = tabs.get(active).doc;
      d.html = d.html.replace(/<p>段落10。[^<]*<\/p>/, "");
      paint(d, true);
      travel("back");
    });
    assert.match(await page.locator("#status").textContent(), /近い位置/);
    // Bound history and discard it when a tab closes.
    await page.evaluate(() => {
      for (let i = 0; i < 120; i++) goToHeading(i % 2 ? "節60" : "節20");
    });
    assert.equal(
      await page.evaluate(() => tabs.get(active).history.back.length),
      100,
    );
    await page.evaluate(() => closeTab(active));
    await frame();
    assert.equal(await page.evaluate(() => tabs.size), 1);
    await page.evaluate((id) => closeTab(id), doc.id);
    assert(await page.locator("#back").isDisabled());
    assert(await page.locator("#forward").isDisabled());
    // An old deferred fragment cannot jump inside a newly selected tab.
    await page.evaluate(
      ({ a, b }) => {
        show({ ...a, anchor: "節70" });
        show(b);
      },
      { a: doc, b: second },
    );
    await frame();
    await frame();
    assert.equal(await page.locator("#name").textContent(), "B.md");
    assert.equal(
      await page.evaluate(() => tabs.get(active).history?.back.length || 0),
      0,
    );
    // Numbered titles and duplicate titles must have distinct TOC/fragment targets.
    await page.evaluate((id) => activate(id), doc.id);
    await frame();
    await page.locator("#menu-toggle").click();
    await page.locator("#contents").click();
    await page.locator("#outline a").filter({ hasText: /^foo$/ }).last().click();
    assert.equal(
      await page.evaluate(() => captureReadingPosition().heading),
      "foo-2",
    );
    await page.evaluate(() => goToHeading("foo-1"));
    assert.equal(
      await page.evaluate(() => captureReadingPosition().heading),
      "foo-1",
    );
    await page.locator('#document a[href="#foo-2"]').click();
    assert.equal(
      await page.evaluate(() => captureReadingPosition().heading),
      "foo-2",
    );
    await page.evaluate(() => changeLanguage("en"));
    assert.equal(await page.locator("#back").textContent(), "Back");
    assert.equal(await page.locator("#forward").textContent(), "Forward");
    await page.locator("#menu-toggle").click();
    await page.screenshot({
      path: path.join(os.tmpdir(), "md-navigation-desktop.png"),
    });
    await page.setViewportSize({ width: 550, height: 700 });
    assert.equal(
      await page.evaluate(
        () => document.documentElement.scrollWidth > innerWidth,
      ),
      false,
    );
    await page.screenshot({
      path: path.join(os.tmpdir(), "md-navigation-narrow.png"),
    });
    assert.deepEqual(errors, []);
    console.log(
      "PASS: TOC/fragment/search history, collision-free heading targets, exact reading offsets, forward branching, per-tab positions, shortcuts/input/IME guards, update/fallback, 100-entry cap, close/empty, stale fragment cancellation, JA/EN, narrow layout, console",
    );
    console.log("fixtures: " + fixtures);
  } finally {
    await browser.close();
  }
})().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
