import { exportDialog, exportState } from "./export-state.js";
import { $, invoke } from "./platform.js";
import { writer } from "./writer.js";
import { readingState, tabs } from "./reader-state.js";
import { closeMenu } from "./reader-controls.js";
import { prefs, status, t } from "./preferences.js";
import { errorTranslations } from "./errors.js";

const exportButtons = [$("export-pdf"), $("export-word"), $("export-google")];

$("export-toggle").onclick = () => {
  const draft = writer.target();
  const doc = draft || tabs.get(readingState.active)?.doc;
  if (!doc || exportDialog.open || exportState.inProgress) return;
  closeMenu();
  $("export-name").textContent = doc.name;
  $("export-name").title = doc.id || draft?.source || "";
  $("export-google").hidden = Boolean(draft);
  $("export-google-help").hidden = Boolean(draft);
  $("export-status").textContent = "";
  exportDialog.showModal();
};
$("export-close").onclick = () => exportDialog.close();
exportDialog.addEventListener("cancel", (event) => {
  if (exportState.inProgress) event.preventDefault();
});

async function runExport(command, args) {
  const draft = writer.target();
  if (!exportDialog.open || exportState.inProgress || (!draft && !tabs.get(readingState.active)?.doc)) return;
  exportState.inProgress = true;
  const controls = [...exportButtons, $("export-close")];
  const focused = document.activeElement;
  for (const button of controls) button.disabled = true;
  const message = $("export-status");
  message.textContent = command === "export_word" ? t("exportSaving") : "";
  try {
    if (draft && command === "print_document") {
      if (!await writer.refreshPreview()) {
        message.textContent = t("exportFailed");
        return;
      }
      await Promise.all(
        [...$("writer-document").querySelectorAll("img")].map((img) => img.decode().catch(() => {})),
      );
    }
    const result = await invoke(command, args);
    message.textContent = "";
    if (command === "print_document" || result?.fileName) exportDialog.close();
    if (result?.fileName) {
      status(result.hasWarnings ? "exportSavedWithWarnings" : "exportSaved");
      $("status").textContent += ` ${result.fileName}`;
    }
  } catch (error) {
    message.textContent = errorTranslations[error]
      ? (prefs.language === "en" ? errorTranslations[error] : error)
      : t("exportFailed");
  } finally {
    for (const button of controls) button.disabled = false;
    exportState.inProgress = false;
    if (exportDialog.open) focused?.focus();
  }
}
$("export-pdf").onclick = () => runExport("print_document");
$("export-word").onclick = () => {
  const draft = writer.target();
  runExport("export_word", draft ? { source: draft.source, markdown: draft.markdown } : { source: readingState.active });
};
$("export-google").onclick = () => runExport("open_google_docs");
