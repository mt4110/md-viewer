import { wrapDocumentTables } from "./tables.js";
import { isWriting } from "./writer-state.js";
import { status, t } from "./preferences.js";
import { dismissExternal, reviewExternal } from "./external-review.js";
import { findState, readingState } from "./reader-state.js";
import { closeMenu } from "./reader-controls.js";
import { $, invoke } from "./platform.js";
import { goToHeading, historyPosition } from "./navigation.js";

function paint(doc, automatic = false) {
  dismissExternal(false);
  readingState.outlineOrigin = null;
  if (!automatic) closeMenu();
  $("document").innerHTML = doc.html;
  $("name").textContent = doc.name;
  if (!isWriting()) document.title = doc.name + " — md-viewer";
  $("outline").replaceChildren();
  for (const h of doc.headings) {
    const a = document.createElement("a");
    a.textContent = h.title;
    a.href = "#" + encodeURIComponent(h.id);
    a.style.paddingLeft = (h.level - 1) * 10 + "px";
    a.onclick = (e) => {
      e.preventDefault();
      $("outline").hidden = true;
      $("contents").setAttribute("aria-expanded", "false");
      $("main").focus({ preventScroll: true });
      const origin = readingState.outlineOrigin;
      readingState.outlineOrigin = null;
      goToHeading(h.id, origin || historyPosition());
    };
    $("outline").append(a);
  }
  wrapDocumentTables($("document"));
  for (const a of $("document").querySelectorAll("a"))
    if (a.getAttribute("href")?.startsWith("#")) {
      a.addEventListener("click", (e) => {
        e.preventDefault();
        try {
          goToHeading(decodeURIComponent(a.getAttribute("href").slice(1)));
        } catch {
          status("missingHeading");
        }
      });
    } else if (a.getAttribute("href") === "")
      a.addEventListener("click", (e) => {
        e.preventDefault();
        if (a.dataset.external) reviewExternal(doc.id, a.dataset.external, a);
        else if (a.dataset.local)
          invoke("open_local", {
            source: doc.id,
            target: a.dataset.local,
          }).catch((error) => status(String(error)));
        else status("unsupported");
      });
  for (const a of $("document").querySelectorAll("a[data-external]")) {
    a.addEventListener("auxclick", (e) => {
      e.preventDefault();
      if (e.button === 1) reviewExternal(doc.id, a.dataset.external, a);
    });
    a.addEventListener("dragstart", (e) => e.preventDefault());
  }
  $("status").dataset.key = "";
  $("status").textContent = "";
  if (!doc.html.trim()) status("empty");
  if (!automatic && !isWriting()) {
    $("main").focus();
    window.scrollTo(0, 0);
  }
  if (!findState.navigatingSearch) document.dispatchEvent(new Event("reader-search-refresh"));
}

export { paint };
