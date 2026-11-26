import { confirmClose } from "./writer-persistence.js";
import { savePosition } from "./reading-position.js";
import { readingState, sessionState, tabs } from "./reader-state.js";
import { $, invoke } from "./platform.js";
import { status } from "./preferences.js";
import { exportDialog, exportState } from "./export-state.js";

let saveTimer = null, saveQueue = Promise.resolve();

function snapshotSession() {
  savePosition();
  return {
    version: 1,
    restore_enabled: sessionState.restoreEnabled,
    selected: sessionState.restoreEnabled ? readingState.active : null,
    tabs: sessionState.restoreEnabled
      ? [...tabs].map(([path, tab]) => ({
          path,
          anchor: tab.savedAnchor || null,
        }))
      : [],
  };
}
function saveSession() {
  if (!sessionState.sessionReady || !sessionState.sessionAvailable || !window.__TAURI_INTERNALS__)
    return Promise.resolve();
  const session = snapshotSession();
  saveQueue = saveQueue
    .catch(() => {})
    .then(() => invoke("save_session", { session }))
    .then(() => {
      if ($("status").dataset.key === "sessionSaveFailed") {
        $("status").dataset.key = "";
        $("status").textContent = "";
      }
      $("quit-without-session").hidden = true;
    });
  return saveQueue;
}
function scheduleSave() {
  if (!sessionState.sessionReady) return;
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    saveSession().catch(() => status("sessionSaveFailed"));
  }, 250);
}
let closing = false;
async function closeWithSave() {
  if (closing || exportState.inProgress || exportDialog.open) return;
  closing = true;
  try {
    if (!await confirmClose()) return;
    clearTimeout(saveTimer);
    await saveSession();
    await invoke("finish_close");
  } catch {
    status("sessionSaveFailed");
    $("quit-without-session").hidden = false;
  } finally {
    closing = false;
  }
}
function installSession(session) {
  sessionState.restoreEnabled = session.restore_enabled;
  $("restore-session").checked = sessionState.restoreEnabled;
  for (const tab of session.tabs) {
    tabs.set(tab.path, {
      doc: null,
      name: tab.path.split(/[\\/]/).pop(),
      position: tab.anchor?.scroll || 0,
      savedAnchor: tab.anchor,
    });
  }
}
window.addEventListener("scroll", scheduleSave, { passive: true });

$("restore-session").onchange = () => {
  sessionState.restoreEnabled = $("restore-session").checked;
  saveSession().catch(() => status("sessionSaveFailed"));
};
$("quit-without-session").onclick = async () => {
  if (await confirmClose()) invoke("finish_close");
};

export { closeWithSave, installSession, scheduleSave };
