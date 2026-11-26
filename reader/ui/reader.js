import { $, invoke } from "./platform.js";
import { activate, closeTab, show, tabButtons } from "./tabs.js";
import { prefs, status } from "./preferences.js";
import { writer } from "./writer.js";
import { changeLanguage, localize } from "./localization.js";
import { readingState, sessionState, tabs } from "./reader-state.js";
import { closeWithSave, installSession } from "./session.js";
import { receiveUpdate } from "./watch.js";
import "./keyboard.js";
import "./appearance-controls.js";
import "./reader-controls.js";
import "./update-settings.js";
import "./export-dialog.js";

async function drain() {
  for (const result of await invoke("take_documents"))
    if (result.Ok) show(result.Ok);
    else status(result.Err);
}
function listen(event, callback) {
  return invoke("plugin:event|listen", {
    event,
    target: { kind: "Any" },
    handler: window.__TAURI_INTERNALS__.transformCallback(callback),
  });
}
writer.init();
localize();
if (!window.__TAURI_INTERNALS__) sessionState.sessionReady = true;
if (window.__TAURI_INTERNALS__) {
  invoke("app_version").then((version) => {
    if (typeof version !== "string" || !version) return;
    $("app-version-value").textContent = version;
    $("app-version").hidden = false;
  }).catch(() => {});
  listen("save-and-close", () => closeWithSave());
  listen("document-updated", (event) => receiveUpdate(event.payload))
    .then(() =>
      listen("documents-ready", () => {
        if (sessionState.sessionReady) drain();
      }),
    )
    .then(async () => {
      let selected = null,
        warning = null;
      try {
        const loaded = await invoke("load_session");
        installSession(loaded.session);
        tabButtons();
        selected = loaded.session.selected;
        warning = loaded.warning;
      } catch (error) {
        sessionState.sessionAvailable = false;
        warning = String(error);
      }
      sessionState.sessionReady = true;
      const startupActivation = readingState.activation;
      const incoming = await invoke("take_documents");
      if (incoming.length) {
        for (const result of incoming) {
          if (result.Ok) show(result.Ok);
          else status(result.Err);
        }
      } else if (tabs.size && readingState.activation === startupActivation)
        await activate(
          selected && tabs.has(selected) ? selected : tabs.keys().next().value,
        );
      if (warning) status(warning);
      await writer.restore();
    })
    .catch((error) => status(String(error)));
  listen("menu-action", (event) => {
    if (event.payload === "open") $("open").click();
    else if (event.payload === "new-draft") $("new-draft").click();
    else if (event.payload === "export") $("export-toggle").click();
    else if (event.payload === "close" && writer.isOpen()) writer.back();
    else if (event.payload === "close" && readingState.active) closeTab(readingState.active);
    else if (event.payload === "settings" && !writer.isOpen()) {
      $("tools").hidden = false;
      $("menu-toggle").setAttribute("aria-expanded", "true");
      $("settings").hidden = false;
      $("settings-toggle").setAttribute("aria-expanded", "true");
    } else if (event.payload === "ja" || event.payload === "en")
      changeLanguage(event.payload);
  });
  invoke("set_language", { language: prefs.language }).catch((error) =>
    status(String(error)),
  );
}
