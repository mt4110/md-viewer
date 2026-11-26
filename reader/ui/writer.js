import { input, name, renderStatus, strings, syncControls } from "./writer-form.js";
import { savePosition } from "./reading-position.js";
import { $, invoke } from "./platform.js";
import { persist, prefs, status, t } from "./preferences.js";
import { writerState } from "./writer-state.js";
import { writerGuide } from "./writer-guide.js";
import { paintPreview, refreshPreview, schedulePreview, setPane } from "./writer-preview.js";
import { exportDialog, exportState } from "./export-state.js";
import { confirmClose, initPersistence, save, scheduleAutosave } from "./writer-persistence.js";
import { activate, show, tabButtons } from "./tabs.js";
import { closeMenu } from "./reader-controls.js";
import { dismissExternal, showExternalReview } from "./external-review.js";
import { readingState, tabs } from "./reader-state.js";
import { applyFormatting, formatText } from "./writer-format.js";

function localizeWriter() {
  for (const [id, label] of Object.entries(strings())) {
    if ($(id)) $(id).textContent = label;
  }
  writerGuide.localize(prefs.language);
  $("writer-autosave").checked = prefs.autoSaveFile !== false;
  $("writer-format-on-save").checked = prefs.formatOnSave !== false;
  for (const table of $("writer-document").querySelectorAll(".table-wrap"))
    table.setAttribute("aria-label", t("table"));
  syncControls();
  renderStatus();
  $("writer-preview-empty").textContent = strings().empty;
}

function insert(sample, inline) {
  if (writerState.saving || writerState.formatting || writerState.composing) return;
  const field = input();
  const { selectionStart, selectionEnd, value } = field;
  const before = value.slice(0, selectionStart);
  const after = value.slice(selectionEnd);
  const leading = before && !before.endsWith("\n\n") ? (before.endsWith("\n") ? "\n" : "\n\n") : "";
  const trailing = after.startsWith("\n\n") ? "" : (after.startsWith("\n") ? "\n" : "\n\n");
  const addition = inline ? sample : leading + sample + trailing;
  field.focus({ preventScroll: true });
  field.setSelectionRange(selectionStart, selectionEnd);
  if (!document.execCommand("insertText", false, addition)) {
    field.setRangeText(addition, selectionStart, selectionEnd, "end");
    changed();
  }
}

async function open(doc = null, recovery = null) {
  if (writerState.opening || writerState.saving || writerState.formatting || exportState.inProgress || exportDialog.open) return;
  if (writerState.draft && !await confirmClose()) return;
  writerState.opening = true;
  try {
    const original = doc ? await invoke("read_draft", { source: doc.id }) : null;
    savePosition();
    writerState.draft = { source: doc?.id || null, name: doc?.name, original, baseline: "", recovered: recovery !== null };
    input().value = original?.replace(/^\uFEFF/, "") || "";
    writerState.draft.baseline = input().value;
    if (recovery !== null) input().value = recovery;
    writerState.message = recovery === null ? null : "recovered";
    writerState.previewError = null;
    closeMenu();
    dismissExternal(false);
    document.body.classList.add("writing");
    $("reading-layout").hidden = true;
    $("writer").hidden = false;
    document.title = name() + " — md-viewer";
    paintPreview("");
    setPane(false);
    syncControls();
    renderStatus();
    tabButtons();
    window.scrollTo(0, 0);
    input().focus();
  } catch (error) {
    if (writerState.draft) {
      writerState.message = typeof error === "string" ? error : "Markdownを読み込めませんでした。";
      renderStatus();
    } else {
      status(typeof error === "string" ? error : "Markdownを読み込めませんでした。");
    }
  } finally {
    writerState.opening = false;
  }
}

async function back() {
  if (exportState.inProgress || !await confirmClose()) return;
  writerState.draft = null;
  ++writerState.previewRevision;
  clearTimeout(writerState.previewTimer);
  clearTimeout(writerState.recoveryTimer);
  clearTimeout(writerState.autosaveTimer);
  input().value = "";
  paintPreview("");
  document.body.classList.remove("writing");
  $("writer").hidden = true;
  $("reading-layout").hidden = false;
  tabButtons();
  if (readingState.active) await activate(readingState.active, true);
  else {
    document.title = "md-viewer";
    $("write-toggle").focus();
  }
}

function init() {
  initPersistence((doc) => show(doc, true));
  writerGuide.init(insert, (page, trigger) => showExternalReview(
    "prepare_guide_link",
    { page, language: prefs.language },
    trigger,
    () => Boolean(writerState.draft) && trigger.isConnected && !$("writer-guide").hidden,
  ));
  $("write-toggle").onclick = () => open(tabs.get(readingState.active)?.doc);
  $("new-draft").onclick = () => open();
  $("writer-back").onclick = back;
  $("writer-save").onclick = () => save();
  $("writer-save-as").onclick = () => save(true);
  $("writer-format").onclick = formatDraft;
  $("writer-autosave").onchange = () => {
    prefs.autoSaveFile = $("writer-autosave").checked;
    if (writerState.draft) writerState.draft.autosaveBlocked = false;
    persist();
    scheduleAutosave();
  };
  $("writer-format-on-save").onchange = () => {
    prefs.formatOnSave = $("writer-format-on-save").checked;
    persist();
  };
  $("writer-preview-toggle").onclick = () => setPane(false);
  $("writer-guide-toggle").onclick = () => setPane(true);
  $("draft-discard").onclick = () => $("draft-close-dialog").close("discard");
  $("draft-cancel").onclick = () => $("draft-close-dialog").close("cancel");
  $("draft-close-save").onclick = () => $("draft-close-dialog").close("save");
  input().addEventListener("input", changed);
  input().addEventListener("compositionstart", () => {
    writerState.composing = true;
    clearTimeout(writerState.recoveryTimer);
    clearTimeout(writerState.autosaveTimer);
  });
  input().addEventListener("compositionend", () => { writerState.composing = false; changed(); });
}

const writer = {
  init, localize: localizeWriter, save, back, confirmClose, refreshPreview, restore,
  isOpen: () => Boolean(writerState.draft),
  target: () => writerState.draft && { name: name(), source: writerState.draft.source, markdown: input().value },
};

function changed() {
  writerState.message = null;
  syncControls();
  renderStatus();
  schedulePreview();
  scheduleAutosave();
}

async function formatDraft() {
  if (!writerState.draft || writerState.saving || writerState.formatting || writerState.composing) return;
  const original = input().value;
  writerState.formatting = true;
  syncControls();
  try {
    const formatted = await formatText(original);
    input().readOnly = false;
    input().focus({ preventScroll: true });
    applyFormatting(formatted);
    changed();
    writerState.message = formatted === original ? "alreadyFormatted" : "formatted";
  } catch {
    writerState.message = "formatFailed";
  } finally {
    writerState.formatting = false;
    syncControls();
    renderStatus();
    scheduleAutosave();
  }
}

async function restore() {
  try {
    const recovery = await invoke("load_recovery_draft");
    if (typeof recovery !== "string" || !recovery || writerState.draft || writerState.opening) return;
    writerState.lastRecovery = recovery;
    await open(null, recovery);
  } catch {
    status("下書きを復元できませんでした。");
  }
}

export { writer };
