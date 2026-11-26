import { $ } from "./platform.js";
import { labels } from "./writer-labels.js";
import { prefs } from "./preferences.js";
import { writerState } from "./writer-state.js";
import { errorTranslations } from "./errors.js";

const input = () => $("markdown-input");
const strings = () => labels[prefs.language];
const name = () => writerState.draft?.name || (writerState.draft?.recovered ? strings().recoveredName : strings().untitled);
const dirty = () => writerState.draft && input().value !== writerState.draft.baseline;

function describe(key) {
  return strings()[key] || (prefs.language === "en" ? errorTranslations[key] : key);
}

function renderStatus() {
  $("writer-status").textContent = describe(writerState.message || writerState.previewError) || "";
}

function syncControls() {
  $("writer-name").textContent = name();
  $("writer-dirty").hidden = !dirty();
  const busy = writerState.saving || writerState.formatting;
  $("writer-save").disabled = busy || (Boolean(writerState.draft?.source) && !dirty());
  $("writer-save-as").disabled = busy || !writerState.draft?.source;
  $("writer-format").disabled = busy || !input().value;
  $("writer-back").disabled = busy;
  input().readOnly = writerState.formatting || (writerState.saving && !writerState.backgroundSave);
}

export { dirty, input, name, renderStatus, strings, syncControls };
