import { isWriting } from "./writer-state.js";
import { findState, readingState, searchCache, tabs } from "./reader-state.js";
import { scheduleSave } from "./session.js";
import { captureReadingPosition, restoreReadingPosition, savePosition, storedAnchor } from "./reading-position.js";
import { jump, navigationButtons } from "./navigation.js";
import { $, invoke } from "./platform.js";
import { status, t } from "./preferences.js";
import { dismissExternal } from "./external-review.js";
import { startWatch } from "./watch.js";
import { paint } from "./document-view.js";

function tabButtons() {
  navigationButtons();
  $("export-toggle").disabled = !isWriting() && !tabs.get(readingState.active)?.doc;
  $("tabs").hidden = tabs.size < 2;
  $("tabs").replaceChildren();
  for (const [id, tab] of tabs) {
    const button = document.createElement("button");
    button.textContent = tab.doc?.name || tab.name;
    button.title = id;
    button.setAttribute("aria-pressed", String(id === readingState.active));
    button.onclick = () => activate(id);
    const close = document.createElement("button");
    close.textContent = "×";
    close.setAttribute("aria-label", (tab.doc?.name || tab.name) + t("close"));
    close.onclick = () => closeTab(id);
    $("tabs").append(button, close);
  }
}
function closeTab(id) {
  if (id === readingState.active) dismissExternal(false);
  if (id === readingState.active && !isWriting()) $("export-dialog").close();
  tabs.delete(id);
  searchCache.delete(id);
  if (readingState.active === id) {
    readingState.active = null;
    startWatch(null);
    const first = tabs.keys().next();
    if (!first.done) activate(first.value);
    else {
      $("document").textContent = t("choose");
      $("name").textContent = "";
      $("outline").replaceChildren();
    }
  }
  tabButtons();
  document.dispatchEvent(new Event("reader-search-refresh"));
  scheduleSave();
}
async function activate(id, keepPosition = false) {
  dismissExternal(false);
  if (id !== readingState.active && !isWriting()) $("export-dialog").close();
  if (!findState.navigatingSearch) {
    ++findState.searchRevision;
    clearTimeout(findState.searchTimer);
  }
  if (!keepPosition) savePosition();
  readingState.active = id;
  const request = ++readingState.activation;
  const tab = tabs.get(id);
  tabButtons();
  if (!tab) return;
  if (!tab.doc) {
    startWatch(null);
    $("document").textContent = t("restoring");
    $("name").textContent = tab.name;
    $("outline").replaceChildren();
    try {
      const doc = await invoke("restore_document", { source: id });
      if (tabs.get(id) !== tab) return;
      if (!fitsDocument(id, doc)) throw new Error("cacheLimit");
      tab.doc = doc;
    } catch (error) {
      if (request !== readingState.activation || readingState.active !== id) return;
      $("document").textContent = t(
        error.message === "cacheLimit" ? "cacheLimit" : "restoreMissing",
      );
      const retry = document.createElement("button");
      retry.textContent = t("retry");
      retry.onclick = () => activate(id);
      $("document").append(document.createElement("br"), retry);
      scheduleSave();
      return;
    }
    if (request !== readingState.activation || readingState.active !== id) return;
  }
  const snapshot = tab.doc;
  paint(snapshot);
  tabButtons();
  requestAnimationFrame(() => {
    if (
      readingState.active !== id ||
      request !== readingState.activation ||
      tabs.get(id)?.doc !== snapshot
    )
      return;
    if (!isWriting()) {
      if (tab.savedAnchor) {
        if (!restoreReadingPosition(tab.savedAnchor)) status("restoredNearby");
      } else window.scrollTo(0, tab.position);
    }
    startWatch(id);
    scheduleSave();
  });
}

function fitsDocument(id, doc) {
  return (
    [...tabs.entries()].reduce(
      (n, [key, tab]) => n + (key === id ? 0 : tab.doc?.html.length || 0),
      doc.html.length,
    ) *
      2 <=
    64 * 1024 * 1024
  );
}
function show(doc, replace = false) {
  const id = doc.id || doc.name;
  if (tabs.has(id)) {
    if (replace || !tabs.get(id).doc) {
      if (!fitsDocument(id, doc)) {
        status("cacheLimit");
        return;
      }
      tabs.get(id).doc = doc;
      searchCache.delete(id);
    }
    activate(id);
    jump(doc);
    return;
  }
  if (
    tabs.size >= 20 ||
    [...tabs.values()].reduce(
      (n, tab) => n + (tab.doc?.html.length || 0),
      doc.html.length,
    ) *
      2 >
      64 * 1024 * 1024
  ) {
    status("limit");
    return;
  }
  tabs.set(id, { doc, position: 0 });
  activate(id);
  jump(doc);
}

export { activate, closeTab, show, tabButtons };
