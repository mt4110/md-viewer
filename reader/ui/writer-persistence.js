import { writerState } from "./writer-state.js";
import { $, invoke } from "./platform.js";
import { dirty, input, renderStatus, syncControls } from "./writer-form.js";
import { prefs } from "./preferences.js";
import { exportDialog, exportState } from "./export-state.js";
import { applyFormatting, formatText } from "./writer-format.js";
import { paintPreview } from "./writer-preview.js";

let onSaved;
function initPersistence(saved) {
  onSaved = saved;
}

let recoveryQueue = Promise.resolve(true);
function checkpoint(markdown) {
  if (!window.__TAURI_INTERNALS__) return Promise.resolve(true);
  recoveryQueue = recoveryQueue.then(async () => {
    if (markdown === writerState.lastRecovery) return true;
    try {
      await invoke("save_recovery_draft", { markdown });
      writerState.lastRecovery = markdown;
      if (markdown !== null && writerState.draft && !writerState.draft.source && !writerState.message && input().value === markdown) {
        writerState.message = "draftCheckpoint";
        renderStatus();
      }
      return true;
    } catch {
      writerState.message = "recoveryFailed";
      renderStatus();
      return false;
    }
  });
  return recoveryQueue;
}

function scheduleAutosave() {
  clearTimeout(writerState.recoveryTimer);
  clearTimeout(writerState.autosaveTimer);
  if (!writerState.draft || writerState.composing || (writerState.saving && !writerState.backgroundSave) || writerState.formatting || $("draft-close-dialog").open) return;
  writerState.recoveryTimer = setTimeout(() => checkpoint(dirty() ? input().value : null), 400);
  if (!writerState.saving && dirty() && writerState.draft.source && !writerState.draft.autosaveBlocked && prefs.autoSaveFile !== false)
    writerState.autosaveTimer = setTimeout(() => {
      if (!exportState.inProgress && !exportDialog.open) save(false, true);
      else scheduleAutosave();
    }, 1600);
}

async function save(asCopy = false, automatic = false) {
  if (!writerState.draft || writerState.saving || writerState.formatting || writerState.composing) return false;
  if (writerState.draft.source && !asCopy && !dirty()) return true;
  const target = writerState.draft;
  const original = input().value;
  let markdown = original;
  writerState.saving = true;
  writerState.backgroundSave = automatic;
  writerState.message = "saving";
  syncControls();
  renderStatus();
  try {
    let formatFailed = false;
    if (prefs.formatOnSave !== false) {
      try { markdown = await formatText(markdown); }
      catch { formatFailed = true; }
    }
    const saved = await invoke("save_draft", {
      source: target.source,
      markdown,
      expected: asCopy ? null : target.original,
    });
    if (!saved) {
      writerState.message = null;
      return false;
    }
    const doc = saved.document;
    const unchanged = input().value === original && !writerState.composing;
    if (unchanged) {
      input().readOnly = false;
      applyFormatting(markdown);
      ++writerState.previewRevision;
      clearTimeout(writerState.previewTimer);
      paintPreview(doc.html);
      writerState.previewError = null;
    }
    writerState.draft = { source: doc.id, name: doc.name, original: saved.original, baseline: markdown };
    document.title = doc.name + " — md-viewer";
    writerState.message = formatFailed ? "savedUnformatted" : automatic ? "autoSaved" : "saved";
    onSaved(doc);
    clearTimeout(writerState.recoveryTimer);
    if (!writerState.composing) await checkpoint(dirty() ? input().value : null);
    return true;
  } catch (error) {
    target.autosaveBlocked = true;
    writerState.message = typeof error === "string" ? error : "saveFailed";
    return false;
  } finally {
    writerState.saving = false;
    writerState.backgroundSave = false;
    syncControls();
    renderStatus();
    scheduleAutosave();
  }
}

async function confirmClose() {
  const dialog = $("draft-close-dialog");
  if (writerState.saving || writerState.formatting || writerState.opening || writerState.composing || dialog.open) return false;
  clearTimeout(writerState.autosaveTimer);
  clearTimeout(writerState.recoveryTimer);
  if (!dirty()) return checkpoint(null);
  dialog.returnValue = "cancel";
  dialog.showModal();
  const action = await new Promise((resolve) => {
    dialog.addEventListener("close", () => resolve(dialog.returnValue), { once: true });
  });
  if (action === "save") return save();
  if (action === "discard") return checkpoint(null);
  scheduleAutosave();
  return false;
}

export { checkpoint, confirmClose, initPersistence, save, scheduleAutosave };
