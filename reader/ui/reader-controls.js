import { $, invoke } from "./platform.js";
import { prefs, status } from "./preferences.js";
import { readingState } from "./reader-state.js";
import { captureReadingPosition, restoreReadingPosition, storedAnchor } from "./reading-position.js";

function closeMenu() {
  $("tools").hidden = true;
  $("menu-toggle").setAttribute("aria-expanded", "false");
}

function openSearch() {
  closeMenu();
  document.querySelector(".search").hidden = false;
  $("query").focus();
}
$("menu-toggle").onclick = () => {
  $("tools").hidden = !$("tools").hidden;
  $("menu-toggle").setAttribute("aria-expanded", String(!$("tools").hidden));
};
$("find-toggle").onclick = openSearch;
$("tools").append($("settings"));
$("open").onclick = () =>
  invoke("choose_file").catch(() =>
    status(
      prefs.language === "ja"
        ? "ファイル選択を開けません。"
        : "Cannot open the file picker.",
    ),
  );
$("contents").onclick = () => {
  closeMenu();
  const opening = $("outline").hidden;
  const origin = readingState.outlineOrigin;
  readingState.outlineOrigin = opening ? storedAnchor(captureReadingPosition()) : null;
  $("outline").hidden = !opening;
  if (!opening && origin) restoreReadingPosition(origin);
  $("contents").setAttribute("aria-expanded", String(!$("outline").hidden));
};
$("settings-toggle").onclick = () => {
  $("settings").hidden = !$("settings").hidden;
  $("settings-toggle").setAttribute(
    "aria-expanded",
    String(!$("settings").hidden),
  );
};

export { closeMenu, openSearch };
