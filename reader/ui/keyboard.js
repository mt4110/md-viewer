import { toggleResults } from "./search-results.js";
import { $ } from "./platform.js";
import { exportDialog } from "./export-state.js";
import { findState, readingState, tabs } from "./reader-state.js";
import { writer } from "./writer.js";
import { travel } from "./navigation.js";
import { startWatch } from "./watch.js";
import { closeMenu, openSearch } from "./reader-controls.js";
import { appearance, persist, prefs } from "./preferences.js";
import { activate, closeTab } from "./tabs.js";
import { search, step } from "./search.js";
import { restoreReadingPosition } from "./reading-position.js";

window.addEventListener("keydown", (e) => {
  if ($("external-dialog").open || exportDialog.open || $("draft-close-dialog").open) return;
  if (e.isComposing || findState.composing) return;
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "n") {
    e.preventDefault();
    $("new-draft").click();
    return;
  }
  if (writer.isOpen()) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      writer.save(e.shiftKey);
    } else if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "w") {
      e.preventDefault();
      writer.back();
    }
    return;
  }
  const editable =
    e.target instanceof Element &&
    e.target.closest("input,textarea,select,[contenteditable=true]");
  const back =
    (e.altKey && !e.metaKey && !e.ctrlKey && e.key === "ArrowLeft") ||
    (e.metaKey &&
      !e.altKey &&
      !e.ctrlKey &&
      (e.key === "[" || e.code === "BracketLeft"));
  const forward =
    (e.altKey && !e.metaKey && !e.ctrlKey && e.key === "ArrowRight") ||
    (e.metaKey &&
      !e.altKey &&
      !e.ctrlKey &&
      (e.key === "]" || e.code === "BracketRight"));
  if (!editable && !e.shiftKey && (back || forward)) {
    e.preventDefault();
    travel(back ? "back" : "forward");
    return;
  }
  if ((e.metaKey || e.ctrlKey) && e.key === "o") {
    e.preventDefault();
    $("open").click();
  }
  if ((e.metaKey || e.ctrlKey) && e.key === "r") {
    e.preventDefault();
    if (readingState.active) startWatch(readingState.active);
  }
  if ((e.metaKey || e.ctrlKey) && e.key === "f") {
    e.preventDefault();
    openSearch();
  }
  if ((e.metaKey || e.ctrlKey) && ["+", "=", "-", "0"].includes(e.key)) {
    e.preventDefault();
    if (e.key === "0") {
      prefs.size = 18;
      appearance();
      persist();
    } else $(e.key === "-" ? "smaller" : "larger").click();
  }
  if ((e.metaKey || e.ctrlKey) && e.key === "w") {
    e.preventDefault();
    if (readingState.active) closeTab(readingState.active);
  }
  if (e.ctrlKey && e.key === "Tab" && tabs.size) {
    e.preventDefault();
    const ids = [...tabs.keys()];
    activate(
      ids[
        (ids.indexOf(readingState.active) + (e.shiftKey ? -1 : 1) + ids.length) % ids.length
      ],
    );
  }
  if (e.key === "Enter" && document.activeElement === $("query")) {
    e.preventDefault();
    step(e.shiftKey ? -1 : 1);
  }
  if (e.key === "Escape") {
    const origin = !$("outline").hidden && readingState.outlineOrigin;
    $("outline").hidden = true;
    readingState.outlineOrigin = null;
    document.querySelector(".search").hidden = true;
    $("query").value = "";
    search();
    closeMenu();
    $("settings").hidden = true;
    $("contents").setAttribute("aria-expanded", "false");
    $("main").focus({ preventScroll: true });
    if (origin) restoreReadingPosition(origin);
  }
});
