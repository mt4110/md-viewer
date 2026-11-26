import { toggleResults } from "./search-results.js";
import { isWriting } from "./writer-state.js";
import { readingState, tabs } from "./reader-state.js";
import { $ } from "./platform.js";
import { scheduleSave } from "./session.js";
import { captureReadingPosition, readingTop, restoreReadingPosition, savePosition, storedAnchor } from "./reading-position.js";
import { closeMenu } from "./reader-controls.js";
import { status } from "./preferences.js";

// Positions belong to their tab; history stores bounded, source-free anchors.
const HISTORY_LIMIT = 100;
function historyFor(tab) {
  return (tab.history ||= { back: [], forward: [] });
}
function navigationButtons() {
  const tab = tabs.get(readingState.active),
    history = tab?.history;
  $("back").disabled = !tab?.doc || !history?.back.length;
  $("forward").disabled = !tab?.doc || !history?.forward.length;
}
function historyPosition() {
  return storedAnchor(captureReadingPosition());
}
function samePosition(a, b) {
  return (
    a &&
    b &&
    a.block_hash === b.block_hash &&
    a.occurrence === b.occurrence &&
    Math.abs(a.top - b.top) < 2
  );
}
function recordJump(move, before = historyPosition()) {
  const tab = tabs.get(readingState.active);
  if (!tab?.doc) return;
  move();
  const after = historyPosition();
  if (before && !samePosition(before, after)) {
    const history = historyFor(tab);
    if (!samePosition(history.back.at(-1), before)) history.back.push(before);
    if (history.back.length > HISTORY_LIMIT) history.back.shift();
    history.forward = [];
  }
  savePosition();
  navigationButtons();
  scheduleSave();
}
function travel(direction) {
  const tab = tabs.get(readingState.active);
  if (!tab?.doc) return;
  const history = historyFor(tab),
    from = history[direction];
  if (!from.length) return;
  const current = historyPosition();
  const target = from.pop();
  const to = history[direction === "back" ? "forward" : "back"];
  if (current) to.push(current);
  if (to.length > HISTORY_LIMIT) to.shift();
  closeMenu();
  toggleResults(false);
  if (!restoreReadingPosition(target)) status("restoredNearby");
  $("main").focus({ preventScroll: true });
  savePosition();
  navigationButtons();
  scheduleSave();
}
$("back").onclick = () => travel("back");
$("forward").onclick = () => travel("forward");
function goToHeading(id, origin = historyPosition()) {
  const node = [...$("document").querySelectorAll("[id]")].find(
    (n) => n.id === id,
  );
  if (!node) {
    status("missingHeading");
    return;
  }
  recordJump(() => {
    node.scrollIntoView();
    window.scrollBy(0, -readingTop());
  }, origin);
}
function jump(doc) {
  if (!doc.anchor || isWriting()) return;
  const id = readingState.active,
    request = readingState.activation;
  requestAnimationFrame(() =>
    requestAnimationFrame(() => {
      if (
        readingState.active === id &&
        request === readingState.activation &&
        tabs.get(id)?.doc &&
        id === doc.id
      )
        goToHeading(doc.anchor);
    }),
  );
}

export { goToHeading, historyPosition, jump, navigationButtons, recordJump, travel };
