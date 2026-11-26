import { $, invoke } from "./platform.js";
import { readingState } from "./reader-state.js";
import { closeMenu } from "./reader-controls.js";
import { t } from "./preferences.js";

// URL parsing is performed in Rust. The UI opens only a single-use review ticket.
let externalRevision = 0,
  externalTicket = null,
  externalTrigger = null;
function dismissExternal(restoreFocus = true) {
  ++externalRevision;
  const ticket = externalTicket,
    trigger = externalTrigger;
  externalTicket = null;
  externalTrigger = null;
  $("external-open").disabled = true;
  if (ticket && window.__TAURI_INTERNALS__)
    invoke("cancel_external", { ticket }).catch(() => {});
  if ($("external-dialog").open) $("external-dialog").close();
  if (restoreFocus && trigger?.isConnected)
    trigger.focus({ preventScroll: true });
}
function reviewExternal(source, url, trigger) {
  return showExternalReview("prepare_external", { source, url }, trigger, () => readingState.active === source);
}
async function showExternalReview(command, args, trigger, stillRelevant) {
  dismissExternal(false);
  closeMenu();
  const revision = externalRevision;
  externalTrigger = trigger;
  $("external-host").textContent = "";
  $("external-url").textContent = "";
  $("external-insecure").hidden = true;
  $("external-error").textContent = t("externalLoading");
  $("external-dialog").showModal();
  $("external-cancel").focus();
  try {
    const review = await invoke(command, args);
    if (
      revision !== externalRevision ||
      !stillRelevant() ||
      !$("external-dialog").open
    ) {
      await invoke("cancel_external", { ticket: review.ticket });
      return;
    }
    externalTicket = review.ticket;
    $("external-host").textContent = review.host;
    $("external-url").textContent = review.url;
    $("external-insecure").hidden = !review.insecure;
    $("external-error").textContent = "";
    $("external-open").disabled = false;
  } catch {
    if (revision === externalRevision)
      $("external-error").textContent = t("externalInvalid");
  }
}
$("external-dialog").addEventListener("cancel", (e) => {
  e.preventDefault();
  dismissExternal();
});
$("external-cancel").onclick = () => dismissExternal();
$("external-open").onclick = async () => {
  const ticket = externalTicket;
  if (!ticket || $("external-open").disabled || !$("external-dialog").open)
    return;
  const revision = externalRevision;
  externalTicket = null; // No retries or double launch after consuming a confirmation.
  $("external-open").disabled = true;
  $("external-cancel").disabled = true;
  try {
    await invoke("open_external", { ticket });
    if (revision === externalRevision) dismissExternal();
  } catch (error) {
    if (revision === externalRevision)
      $("external-error").textContent = t(
        String(error).startsWith("確認の有効期限")
          ? "externalExpired"
          : "externalFailed",
      );
  } finally {
    $("external-cancel").disabled = false;
  }
};

export { dismissExternal, reviewExternal, showExternalReview };
