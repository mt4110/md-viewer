import { $, invoke } from "./platform.js";
import { wrapDocumentTables } from "./tables.js";
import { writerState } from "./writer-state.js";
import { input, renderStatus } from "./writer-form.js";

let previewQueue = Promise.resolve();
function paintPreview(html) {
  const article = $("writer-document");
  article.innerHTML = html;
  for (const link of article.querySelectorAll("a")) link.removeAttribute("href");
  wrapDocumentTables(article);
  $("writer-preview-empty").hidden = Boolean(html.trim());
}

function refreshPreview() {
  clearTimeout(writerState.previewTimer);
  const revision = ++writerState.previewRevision;
  const target = writerState.draft;
  const markdown = input().value;
  const request = previewQueue.then(async () => {
    if (!target || revision !== writerState.previewRevision) return false;
    try {
      const html = markdown ? await invoke("preview_draft", { source: target.source, markdown }) : "";
      if (revision !== writerState.previewRevision) return false;
      paintPreview(html);
      writerState.previewError = null;
      renderStatus();
      return true;
    } catch (error) {
      if (revision === writerState.previewRevision) {
        paintPreview("");
        $("writer-preview-empty").hidden = true;
        writerState.previewError = typeof error === "string" ? error : "previewFailed";
        renderStatus();
      }
      return false;
    }
  });
  previewQueue = request;
  return request;
}

function setPane(guide) {
  $("writer-guide").hidden = !guide;
  $("writer-preview").hidden = guide;
  $("writer-guide-toggle").setAttribute("aria-pressed", String(guide));
  $("writer-preview-toggle").setAttribute("aria-pressed", String(!guide));
  if (!guide) refreshPreview();
}

function schedulePreview() {
  ++writerState.previewRevision;
  clearTimeout(writerState.previewTimer);
  if (!writerState.composing && !$("writer-preview").hidden)
    writerState.previewTimer = setTimeout(refreshPreview, 160);
}

export { paintPreview, refreshPreview, schedulePreview, setPane };
