// Regression checks: real Rust rendering; mocked OS handoff. Never visits a web URL.
const { chromium } = require("playwright");
const assert = require("node:assert/strict"),
  fs = require("node:fs"),
  os = require("node:os"),
  path = require("node:path");
const { execFileSync } = require("node:child_process"),
  { pathToFileURL } = require("node:url");
const root = path.resolve(__dirname, "../.."),
  fixtures = fs.mkdtempSync(path.join(os.tmpdir(), "md-external-test-"));
const source = path.join(fixtures, "links.md");
fs.writeFileSync(
  source,
  "# 外部リンク\n\n[安全そうな表示名](https://evil.example/path?q=one&x=2#part)\n\n[HTTP](http://example.com/)\n\n[日本語](https://例え.テスト/資料)\n\n[認証付き](https://user:password@example.com/)\n\n[ローカル](other.md)",
);
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
      source,
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
    const errors = [],
      network = [];
    page.on("pageerror", (e) => errors.push(e.message));
    page.on("request", (r) => {
      if (/^https?:/.test(r.url())) network.push(r.url());
    });
    const url = pathToFileURL(path.join(root, "reader/ui/index.html")).href;
    await page.goto(url);
    await page.evaluate((d) => {
      window.calls = [];
      window.counter = 0;
      window.failOpen = false;
      window.slowPrepare = false;
      window.__TAURI_INTERNALS__ = {
        invoke: async (name, args) => {
          window.calls.push({ name, args });
          if (name === "prepare_external") {
            if (window.slowPrepare)
              return new Promise((resolve) => (window.releaseReview = resolve));
            if (args.url.includes("user:")) throw "invalid";
            const target = args.url.includes("例え")
              ? "https://xn--r8jz45g.xn--zckzah/%E8%B3%87%E6%96%99"
              : args.url;
            return {
              ticket: String(++window.counter),
              url: target,
              host: target.includes("evil")
                ? "evil.example"
                : target.includes("xn--")
                  ? "xn--r8jz45g.xn--zckzah"
                  : "example.com",
              insecure: target.startsWith("http:"),
            };
          }
          if (name === "open_external" && window.failOpen)
            throw window.failOpen;
          return null;
        },
      };
      show(d);
    }, doc);
    await page.evaluate(() => new Promise(requestAnimationFrame));
    const opens = () =>
      page.evaluate(() =>
        window.calls.filter((c) => c.name === "open_external"),
      );
    const link = page.getByRole("link", {
      name: "安全そうな表示名",
      exact: true,
    });
    assert.equal(await link.getAttribute("href"), "");
    assert.equal((await opens()).length, 0);
    await link.click();
    await page.waitForFunction(() => !$("external-open").disabled);
    assert.equal(
      await page.locator("#external-host").textContent(),
      "evil.example",
    );
    assert.equal(
      await page.locator("#external-url").textContent(),
      "https://evil.example/path?q=one&x=2#part",
    );
    assert(
      await page
        .locator("#external-cancel")
        .evaluate((e) => e === document.activeElement),
    );
    assert.equal((await opens()).length, 0);
    assert.equal(page.url(), url);
    await page.keyboard.press("Enter");
    assert(!(await page.locator("#external-dialog").evaluate((e) => e.open)));
    assert.equal((await opens()).length, 0);
    assert(await link.evaluate((e) => e === document.activeElement));
    await link.click();
    await page.keyboard.press("Escape");
    assert(!(await page.locator("#external-dialog").evaluate((e) => e.open)));
    assert.equal((await opens()).length, 0);
    await page.getByRole("link", { name: "HTTP", exact: true }).click();
    await page.waitForFunction(() => !$("external-open").disabled);
    assert(await page.locator("#external-insecure").isVisible());
    await page.locator("#external-cancel").click();
    await page.getByRole("link", { name: "日本語", exact: true }).click();
    await page.waitForFunction(() => !$("external-open").disabled);
    assert.equal(
      await page.locator("#external-host").textContent(),
      "xn--r8jz45g.xn--zckzah",
    );
    await page.locator("#external-cancel").click();
    await page.getByRole("link", { name: "認証付き", exact: true }).click();
    await page.waitForFunction(() =>
      $("external-error").textContent.includes("開けません"),
    );
    assert(await page.locator("#external-open").isDisabled());
    await page.locator("#external-cancel").click();
    // A canceled preparation cannot display a late confirmation.
    await page.evaluate(() => (window.slowPrepare = true));
    await link.click();
    await page.locator("#external-cancel").click();
    await page.evaluate(() => {
      window.slowPrepare = false;
      window.releaseReview({
        ticket: "late",
        url: "https://evil.example/",
        host: "evil.example",
        insecure: false,
      });
    });
    await page.waitForTimeout(30);
    assert(!(await page.locator("#external-dialog").evaluate((e) => e.open)));
    assert.equal((await opens()).length, 0);
    assert(
      await page.evaluate(() =>
        window.calls.some(
          (c) => c.name === "cancel_external" && c.args.ticket === "late",
        ),
      ),
    );
    // Explicit confirmation hands off a ticket only, once, never a second URL.
    await link.click();
    await page.waitForFunction(() => !$("external-open").disabled);
    await page.evaluate(() => {
      $("external-open").click();
      $("external-open").click();
    });
    await page.waitForFunction(() => !$("external-dialog").open);
    const handed = await opens();
    assert.equal(handed.length, 1);
    assert.deepEqual(Object.keys(handed[0].args), ["ticket"]);
    // Launcher failure and expired approval require a fresh confirmation.
    await page.evaluate(() => (window.failOpen = "failed"));
    await link.click();
    await page.waitForFunction(() => !$("external-open").disabled);
    await page.locator("#external-open").click();
    assert.match(
      await page.locator("#external-error").textContent(),
      /標準ブラウザを開けません/,
    );
    assert(await page.locator("#external-open").isDisabled());
    await page.locator("#external-cancel").click();
    await page.evaluate(
      () => (window.failOpen = "確認の有効期限が切れました。"),
    );
    await link.click();
    await page.waitForFunction(() => !$("external-open").disabled);
    await page.locator("#external-open").click();
    assert.match(
      await page.locator("#external-error").textContent(),
      /有効期限/,
    );
    await page.locator("#external-cancel").click();
    await page.evaluate(() => (window.failOpen = false));
    // Refreshing or closing the document cancels its modal.
    await link.click();
    await page.waitForFunction(() => !$("external-open").disabled);
    await page.evaluate(() => paint(tabs.get(active).doc, true));
    assert(!(await page.locator("#external-dialog").evaluate((e) => e.open)));
    await link.click({ button: "middle" });
    await page.waitForFunction(() => !$("external-open").disabled);
    await page.locator("#external-cancel").click();
    await page.getByRole("link", { name: "ローカル", exact: true }).click();
    assert(
      await page.evaluate(() =>
        window.calls.some((c) => c.name === "open_local"),
      ),
    );
    assert(!(await page.locator("#external-dialog").evaluate((e) => e.open)));
    await page.evaluate(() => changeLanguage("en"));
    await link.click();
    await page.waitForFunction(() => !$("external-open").disabled);
    assert.equal(
      await page.locator("#external-open").textContent(),
      "Open in browser",
    );
    await page.screenshot({
      path: path.join(os.tmpdir(), "md-external-desktop.png"),
    });
    await page.setViewportSize({ width: 550, height: 700 });
    await page.evaluate(
      () => (document.documentElement.style.fontSize = "200%"),
    );
    assert.equal(
      await page.evaluate(
        () => document.documentElement.scrollWidth > innerWidth,
      ),
      false,
    );
    await page.screenshot({
      path: path.join(os.tmpdir(), "md-external-narrow.png"),
    });
    await page.evaluate(() => closeTab(active));
    assert(!(await page.locator("#external-dialog").evaluate((e) => e.open)));
    assert.deepEqual(network, []);
    assert.deepEqual(errors, []);
    console.log(
      "PASS: inert links/no network, exact URL and ASCII host, cancel/default focus/Enter/Escape, HTTP warning, invalid URLs, stale preparation, single handoff, launcher error/expiry, refresh/close cancellation, middle click, local links, JA/EN, narrow/200%",
    );
    console.log("fixtures: " + fixtures);
  } finally {
    await browser.close();
  }
})().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
