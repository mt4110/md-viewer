import { isWriting } from "./writer-state.js";
import { $, invoke } from "./platform.js";
import { status } from "./preferences.js";
import { findState, readingState, tabs } from "./reader-state.js";
import { captureReadingPosition, restoreReadingPosition } from "./reading-position.js";
import { paint } from "./document-view.js";
import { scheduleSave } from "./session.js";

let watchRevision = 0,
  pendingUpdate = null;
function startWatch(source) {
  const revision = ++watchRevision;
  pendingUpdate = null;
  if (window.__TAURI_INTERNALS__)
    invoke("watch_document", { source, revision }).catch((error) => {
      if (revision === watchRevision) status(String(error));
    });
}
function updateBlocked() {
  const selection = window.getSelection();
  return (
    findState.composing ||
    (!!selection &&
      !selection.isCollapsed &&
      ($("document").contains(selection.anchorNode) ||
        $("document").contains(selection.focusNode)))
  );
}
function receiveUpdate(payload) {
  if (
    payload.revision !== watchRevision ||
    payload.source !== readingState.active ||
    !tabs.has(readingState.active)
  )
    return;
  if (payload.result.Err) {
    pendingUpdate = null;
    status(payload.result.Err);
    return;
  }
  const doc = payload.result.Ok;
  if (!doc) return;
  if (doc.html === tabs.get(readingState.active).doc.html) {
    pendingUpdate = null;
    return;
  }
  pendingUpdate = payload;
  if (updateBlocked()) {
    status("updatePending");
    return;
  }
  flushUpdate();
}
function flushUpdate() {
  if (!pendingUpdate || updateBlocked()) return;
  const payload = pendingUpdate;
  pendingUpdate = null;
  if (
    payload.revision !== watchRevision ||
    payload.source !== readingState.active ||
    !tabs.has(readingState.active)
  )
    return;
  const doc = payload.result.Ok;
  const size =
    [...tabs.entries()].reduce(
      (n, [id, tab]) =>
        n + (id === readingState.active ? doc.html.length : tab.doc?.html.length || 0),
      0,
    ) * 2;
  if (size > 64 * 1024 * 1024) {
    status("limit");
    return;
  }
  const anchor = isWriting() ? null : captureReadingPosition();
  tabs.get(readingState.active).doc = doc;
  paint(doc, true);
  if (anchor) {
    const exact = restoreReadingPosition(anchor);
    tabs.get(readingState.active).position = window.scrollY;
    status(exact ? "updated" : "updateFallback");
  }
  scheduleSave();
}
window.addEventListener("focus", () => {
  if (readingState.active) startWatch(readingState.active);
});
document.addEventListener("selectionchange", () => {
  if (pendingUpdate && !updateBlocked()) flushUpdate();
});

export { flushUpdate, receiveUpdate, startWatch };
