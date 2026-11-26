async function loadReader(page, url) {
  await page.goto(url);
  await page.evaluate(async () => {
    const modules = [
      "platform", "tabs", "session", "reader-state", "reading-position",
      "navigation", "document-view", "localization", "watch",
    ];
    const interfaces = await Promise.all(modules.map(name => import("./" + name + ".js")));
    const { readingState, findState, ...actions } = Object.assign({}, ...interfaces);
    Object.assign(window, actions);
    Object.defineProperties(window, {
      active: { get: () => readingState.active },
      composing: { get: () => findState.composing },
    });
  });
}

module.exports = { loadReader };
