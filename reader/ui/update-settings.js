import { t } from "./preferences.js";
import { $, invoke } from "./platform.js";

let updateState = null;
let checkingUpdate = false;
let openingReleases = false;
const comparisonLabels = {
  newer: "updateNewer",
  current: "updateCurrent",
  older: "updateOlder",
};
const updateErrorLabels = {
  rateLimited: "updateRateLimited",
  invalidRelease: "updateInvalidRelease",
};

function renderUpdateStatus() {
  let message = updateState ? t(updateState.label) : "";
  if (updateState?.release) {
    const { version, hasInstaller } = updateState.release;
    message = message.replace("{version}", version);
    if (!hasInstaller) message += " " + t("updateNoInstaller");
  }
  $("update-status").textContent = message;
}

$("check-update").onclick = async () => {
  if (checkingUpdate || openingReleases) return;
  checkingUpdate = true;
  $("check-update").setAttribute("aria-disabled", "true");
  $("open-releases").hidden = true;
  updateState = { label: "updateChecking" };
  renderUpdateStatus();
  try {
    const result = await invoke("check_update");
    updateState = result.status === "release"
      ? { label: comparisonLabels[result.comparison], release: result }
      : { label: "updateNoRelease" };
  } catch (error) {
    updateState = { label: updateErrorLabels[error] || "updateUnavailable" };
  } finally {
    checkingUpdate = false;
    $("check-update").removeAttribute("aria-disabled");
    $("open-releases").hidden = false;
    renderUpdateStatus();
  }
};

$("open-releases").onclick = async () => {
  if (openingReleases || checkingUpdate) return;
  openingReleases = true;
  $("open-releases").setAttribute("aria-disabled", "true");
  try {
    await invoke("open_releases");
  } catch {
    updateState = { label: "updateBrowserFailed" };
    renderUpdateStatus();
  } finally {
    $("open-releases").removeAttribute("aria-disabled");
    openingReleases = false;
  }
};

export { renderUpdateStatus };
