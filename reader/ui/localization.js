import { appearance, persist, prefs, status, t } from "./preferences.js";
import { $, invoke } from "./platform.js";
import { tabButtons } from "./tabs.js";
import { renderUpdateStatus } from "./update-settings.js";
import { writer } from "./writer.js";
import { search } from "./search.js";
import { locales } from "./translations.js";

function localize() {
  document.documentElement.lang = prefs.language;
  document.querySelector(".search").setAttribute("aria-label", t("query"));
  for (const [id, key] of Object.entries({
    "export-toggle": "export",
    "export-title": "export",
    "export-word": "exportWord",
    "export-google": "exportGoogle",
    "export-pdf-help": "exportPdfHelp",
    "export-word-help": "exportWordHelp",
    "export-google-help": "exportGoogleHelp",
    "export-close": "exportClose",
    "external-title": "externalTitle",
    "external-explanation": "externalExplanation",
    "external-host-label": "externalHost",
    "external-cancel": "externalCancel",
    "external-open": "externalOpen",
    "external-insecure": "externalInsecure",
    back: "back",
    forward: "forward",
    "restore-label": "restoreSession",
    "quit-without-session": "quitWithoutSession",
    "menu-toggle": "menu",
    "find-toggle": "find",
    open: "open",
    contents: "contents",
    theme: "theme",
    "settings-toggle": "settings",
    "update-title": "updateTitle",
    "app-version-label": "appVersion",
    "check-update": "updateCheck",
    "open-releases": "updateReleases",
    previous: "previous",
    next: "next",
    reset: "reset",
    "foreground-label": "foreground",
    "background-label": "background",
    "preset-label": "preset",
    "width-label": "width",
  }))
    $(id).textContent = t(key);
  $("results-toggle").textContent = t("results");
  $("scope-label").textContent = t("scope");
  $("search-scope").options[0].textContent = t("thisDocument");
  $("search-scope").options[1].textContent = t("openTabs");
  $("search-results").setAttribute("aria-label", t("find"));
  document.querySelector("label[for=query]").textContent = t("query");
  document.querySelector("label[for=language]").textContent = t("language");
  $("smaller").setAttribute("aria-label", t("smaller"));
  $("larger").setAttribute("aria-label", t("larger"));
  $("settings").setAttribute("aria-label", t("settings"));
  $("outline").setAttribute("aria-label", t("contents"));
  for (const table of $("document").querySelectorAll(".table-wrap"))
    table.setAttribute("aria-label", t("table"));
  $("tabs").setAttribute("aria-label", t("tabs"));
  $("language").value = prefs.language;
  for (const option of $("preset").options)
    option.textContent = t(option.value);
  const empty = $("document").querySelector(".empty");
  if (empty) {
    empty.querySelector(".eyebrow").textContent = t("eyebrow");
    empty.querySelector("h1").textContent = t("welcome");
    empty.querySelectorAll("p")[1].textContent = t("hint");
    empty.querySelector(".hint").textContent = t("readonly");
  }
  tabButtons();
  appearance();
  renderUpdateStatus();
  writer.localize();
  search();
  if ($("status").dataset.key) status($("status").dataset.key);
}
function changeLanguage(language) {
  if (!locales[language]) return;
  prefs.language = language;
  localize();
  persist();
  if (window.__TAURI_INTERNALS__)
    invoke("set_language", { language }).catch((error) =>
      status(String(error)),
    );
}

export { changeLanguage, localize };
