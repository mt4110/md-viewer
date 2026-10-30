"use strict";
const $ = (id) => document.getElementById(id);
const invoke = (name, args) => window.__TAURI_INTERNALS__.invoke(name, args);
const locales = {
  ja: {
    externalTitle: "外部リンクを開きますか？",
    externalExplanation:
      "表示名ではなく、接続先とURLを確認してください。開くと標準ブラウザが接続します。",
    externalHost: "接続先",
    externalCancel: "キャンセル",
    externalOpen: "ブラウザで開く",
    externalLoading: "URLを確認しています…",
    externalFailed:
      "標準ブラウザを開けませんでした。リンクを押し直してください。",
    externalInvalid:
      "このURLは開けません。HTTP／HTTPSの接続先を確認してください。",
    externalExpired: "確認の有効期限が切れました。リンクを押し直してください。",
    externalInsecure: "HTTPのリンクです。通信は暗号化されません。",
    cacheLimit:
      "保持できる文書量の上限です。ほかのタブを閉じて再試行してください。",
    back: "戻る",
    forward: "進む",
    menu: "メニュー",
    restoreSession: "前回のタブを復元",
    restoring: "文書を復元しています…",
    restoreMissing: "文書を開けません。ファイルの場所を確認してください。",
    retry: "再試行",
    restoredNearby: "元の段落が見つからないため、近い位置から再開しました。",
    sessionSaveFailed: "読書状態を保存できません。前回の情報は保持しています。",
    quitWithoutSession: "保存せず終了",
    find: "検索",
    updated: "更新しました",
    updatePending: "更新があります（選択・入力の終了後に反映）",
    updateFallback:
      "更新しました。元の段落が見つからないため近い位置を表示しています。",
    open: "ファイルを開く",
    contents: "目次",
    smaller: "文字を小さく",
    larger: "文字を大きく",
    theme: "明暗",
    settings: "表示設定",
    query: "検索語",
    results: "検索結果",
    scope: "検索範囲",
    thisDocument: "この文書",
    openTabs: "開いているタブ",
    searching: "検索中…",
    matches: "件",
    searchLimited: "（先頭200件。検索語を絞ってください）",
    searchFailed: "文書を検索できませんでした",
    searchChanged: "文書が変わったため検索結果を更新しました。",
    previous: "前",
    next: "次",
    language: "表示言語",
    foreground: "文字色",
    background: "背景色",
    preset: "配色",
    width: "読み幅",
    reset: "初期値に戻す",
    paper: "標準",
    night: "夜",
    warm: "暖色",
    close: "を閉じる",
    empty: "このファイルには本文がありません。",
    limit: "タブの上限です。ほかのタブを閉じてください。",
    unsupported: "このリンクはまだ開けません。",
    missingHeading: "指定された見出しが見つかりません。",
    choose: "ファイルを開いてください。",
    contrast:
      "文字と背景のコントラストが低めです。標準の配色もお試しください。",
    table: "表を横にスクロール",
    tabs: "開いている文書",
    eyebrow: "読むことに、集中。",
    welcome: "文書をここに。",
    hint: "Markdownファイルをドロップするか、「ファイルを開く」から選んでください。",
    readonly: "元の文書は変更しません。",
  },
  en: {
    externalTitle: "Open this external link?",
    externalExplanation:
      "Check the destination and URL, rather than the link label. Opening it lets your default browser connect.",
    externalHost: "Destination",
    externalCancel: "Cancel",
    externalOpen: "Open in browser",
    externalLoading: "Checking URL…",
    externalFailed:
      "Could not open the default browser. Click the link again to retry.",
    externalInvalid:
      "Cannot open this URL. Check the HTTP or HTTPS destination.",
    externalExpired: "Confirmation expired. Click the link again.",
    externalInsecure: "This is an HTTP link. The connection is not encrypted.",
    cacheLimit: "Document cache limit reached. Close another tab and retry.",
    back: "Back",
    forward: "Forward",
    menu: "Menu",
    restoreSession: "Restore previous tabs",
    restoring: "Restoring document…",
    restoreMissing: "Cannot open this document. Check its location.",
    retry: "Retry",
    restoredNearby: "The original paragraph was not found. Resumed nearby.",
    sessionSaveFailed:
      "Could not save reading state. Previous state was preserved.",
    quitWithoutSession: "Quit without saving",
    find: "Find",
    updated: "Updated",
    updatePending: "Update pending until selection or input ends",
    updateFallback:
      "Updated. The original paragraph was not found; showing a nearby position.",
    open: "Open file",
    contents: "Contents",
    smaller: "Smaller text",
    larger: "Larger text",
    theme: "Light / dark",
    settings: "Appearance",
    query: "Search text",
    results: "Results",
    scope: "Search in",
    thisDocument: "This document",
    openTabs: "Open tabs",
    searching: "Searching…",
    matches: " matches",
    searchLimited: " (first 200; refine your search)",
    searchFailed: " document(s) could not be searched",
    searchChanged: "Document changed. Search results refreshed.",
    previous: "Previous",
    next: "Next",
    language: "Language",
    foreground: "Text color",
    background: "Background color",
    preset: "Colors",
    width: "Reading width",
    reset: "Reset appearance",
    paper: "Paper",
    night: "Night",
    warm: "Warm",
    close: " — Close",
    empty: "This document is empty.",
    limit: "Tab limit reached. Close another tab first.",
    unsupported: "This link is not supported yet.",
    missingHeading: "The requested heading was not found.",
    choose: "Open a Markdown file.",
    contrast: "Text contrast is low. Try a preset for easier reading.",
    table: "Scroll table horizontally",
    tabs: "Open documents",
    eyebrow: "Space to read.",
    welcome: "Your document, here.",
    hint: "Drop a Markdown file here, or choose Open file.",
    readonly: "Your original file stays unchanged.",
  },
};
const errorTranslations = {
  "復元情報が壊れていたため、保全して空画面で開始しました。":
    "Saved reading state was invalid. A copy was preserved; starting with no tabs.",
  "復元情報を読み込めません。":
    "Could not read the saved session. Existing information was preserved.",
  "復元情報が大きすぎるため読み込みません。元の情報は保持しています。":
    "Saved reading state is too large. The original file was preserved.",
  "ファイルの更新監視を開始できません。":
    "Could not watch this file. Press Cmd+R or Ctrl+R to retry.",
  "更新監視でエラーが発生しました。再読込してください。":
    "File watching failed. Press Cmd+R or Ctrl+R to retry.",
  "リンク切れです。": "Broken link.",
  "リンク先が見つかりません。": "Broken link.",
  "親フォルダー外の文書です。「ファイルを開く」から選んでください。":
    "This file is outside the reference folder. Select it with Open file.",
  "元の文書を確認できません。": "The source document is no longer available.",
  "Markdownファイルを選んでください。": "Choose a Markdown file.",
  "ファイルが見つかりません。": "File not found.",
  "UTF-8の文書を選んでください。": "Choose a UTF-8 document.",
  "表示できるファイルサイズを超えています。":
    "The file exceeds the viewing size limit.",
  "ファイルを開けません。場所とアクセス権を確認してください。":
    "Cannot open this file. Check its location and permissions.",
  "このリンク形式にはまだ対応していません。":
    "This link format is not supported yet.",
};
let prefs = {
  language: "ja",
  size: 18,
  width: 44,
  paper: "#faf9f6",
  ink: "#292e32",
};
try {
  const saved = JSON.parse(
    localStorage.getItem("md-viewer-appearance") || "null",
  );
  if (
    saved &&
    ["ja", "en"].includes(saved.language) &&
    Number.isFinite(saved.size) &&
    Number.isFinite(saved.width) &&
    /^#[0-9a-f]{6}$/i.test(saved.paper) &&
    /^#[0-9a-f]{6}$/i.test(saved.ink)
  )
    prefs = {
      ...saved,
      size: Math.max(12, Math.min(36, saved.size)),
      width: Math.max(28, Math.min(60, saved.width)),
    };
} catch {}
const t = (key) => locales[prefs.language][key];
function status(key) {
  $("status").dataset.key = key;
  $("status").textContent =
    locales[prefs.language][key] ||
    (prefs.language === "en"
      ? errorTranslations[key] || "This file could not be opened."
      : key);
}
function persist() {
  try {
    localStorage.setItem("md-viewer-appearance", JSON.stringify(prefs));
  } catch {
    status(
      prefs.language === "ja"
        ? "表示設定を保存できません。"
        : "Appearance could not be saved.",
    );
  }
}
function luminance(hex) {
  const values = hex
    .match(/[a-f0-9]{2}/gi)
    .map((v) => parseInt(v, 16) / 255)
    .map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return values[0] * 0.2126 + values[1] * 0.7152 + values[2] * 0.0722;
}
function appearance() {
  const root = document.documentElement;
  root.style.setProperty("--size", prefs.size + "px");
  root.style.setProperty("--width", prefs.width + "em");
  root.style.setProperty("--paper", prefs.paper);
  root.style.setProperty("--ink", prefs.ink);
  const dark = luminance(prefs.paper) < 0.18;
  root.style.setProperty("--accent", dark ? "#9cc4b4" : "#32685b");
  root.style.setProperty("--line", dark ? "#52635c" : "#c5cec8");
  root.style.setProperty("--muted", prefs.ink);
  root.style.colorScheme = dark ? "dark" : "light";
  $("foreground").value = prefs.ink;
  $("background").value = prefs.paper;
  $("width").value = prefs.width;
  const ratio =
    (Math.max(luminance(prefs.paper), luminance(prefs.ink)) + 0.05) /
    (Math.min(luminance(prefs.paper), luminance(prefs.ink)) + 0.05);
  $("contrast").textContent = ratio < 4.5 ? t("contrast") : "";
}
function localize() {
  document.documentElement.lang = prefs.language;
  document.querySelector(".search").setAttribute("aria-label", t("query"));
  for (const [id, key] of Object.entries({
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
  search();
  if ($("status").dataset.key) status($("status").dataset.key);
}
let hits = [],
  selected = -1,
  composing = false;
const tabs = new Map();
let active = null;
let sessionReady = false,
  restoreEnabled = true,
  sessionAvailable = true,
  saveTimer = null,
  saveQueue = Promise.resolve(),
  activation = 0;
function positionHash(text) {
  let hash = 0xcbf29ce484222325n;
  for (let i = 0; i < text.length; i++) {
    hash ^= BigInt(text.charCodeAt(i));
    hash = BigInt.asUintN(64, hash * 0x100000001b3n);
  }
  return hash.toString(16).padStart(16, "0");
}
function storedAnchor(anchor) {
  if (!anchor) return null;
  return {
    block_hash: positionHash(anchor.text),
    heading_hash: anchor.heading ? positionHash(anchor.heading) : null,
    occurrence: anchor.occurrence,
    count: anchor.count,
    top: anchor.top,
    scroll: anchor.scroll,
  };
}
function snapshotSession() {
  savePosition();
  return {
    version: 1,
    restore_enabled: restoreEnabled,
    selected: restoreEnabled ? active : null,
    tabs: restoreEnabled
      ? [...tabs].map(([path, tab]) => ({
          path,
          anchor: tab.savedAnchor || null,
        }))
      : [],
  };
}
function saveSession() {
  if (!sessionReady || !sessionAvailable || !window.__TAURI_INTERNALS__)
    return Promise.resolve();
  const session = snapshotSession();
  saveQueue = saveQueue
    .catch(() => {})
    .then(() => invoke("save_session", { session }))
    .then(() => {
      if ($("status").dataset.key === "sessionSaveFailed") {
        $("status").dataset.key = "";
        $("status").textContent = "";
      }
      $("quit-without-session").hidden = true;
    });
  return saveQueue;
}
function scheduleSave() {
  if (!sessionReady) return;
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    saveSession().catch(() => status("sessionSaveFailed"));
  }, 250);
}
async function closeWithSave() {
  clearTimeout(saveTimer);
  try {
    await saveSession();
    await invoke("finish_close");
  } catch {
    status("sessionSaveFailed");
    $("quit-without-session").hidden = false;
  }
}
function installSession(session) {
  restoreEnabled = session.restore_enabled;
  $("restore-session").checked = restoreEnabled;
  for (const tab of session.tabs) {
    tabs.set(tab.path, {
      doc: null,
      name: tab.path.split(/[\\/]/).pop(),
      position: tab.anchor?.scroll || 0,
      savedAnchor: tab.anchor,
    });
  }
  tabButtons();
}
window.addEventListener("scroll", scheduleSave, { passive: true });

let watchRevision = 0,
  pendingUpdate = null;
function startWatch(source) {
  const revision = ++watchRevision;
  pendingUpdate = null;
  if (window.__TAURI_INTERNALS__)
    invoke("watch_document", { source, revision }).catch((error) => {
      if (revision === watchRevision) status(String(error));
    });
}
const readingTop = () => $("chrome")?.getBoundingClientRect().bottom || 0;
const readingBlocks = () => [
  ...$("document").querySelectorAll("h1,h2,h3,h4,h5,h6,p,pre,li,table"),
];
function captureReadingPosition() {
  const blocks = readingBlocks();
  const index = blocks.findIndex(
    (block) => block.getBoundingClientRect().bottom > readingTop(),
  );
  const node = blocks[index];
  if (!node) return null;
  const text = node.textContent;
  const matches = blocks.filter((block) => block.textContent === text);
  const heading = blocks
    .slice(0, index + 1)
    .filter((block) => /^H[1-6]$/.test(block.tagName))
    .at(-1);
  return {
    text,
    occurrence: matches.indexOf(node),
    count: matches.length,
    heading: heading?.id,
    top: node.getBoundingClientRect().top - readingTop(),
    scroll: window.scrollY,
  };
}
function restoreReadingPosition(anchor) {
  if (!anchor) return true;
  const blocks = readingBlocks();
  const matches = blocks.filter((block) =>
    anchor.block_hash
      ? positionHash(block.textContent) === anchor.block_hash
      : block.textContent === anchor.text,
  );
  let exact = matches.length === anchor.count;
  let node = exact ? matches[anchor.occurrence] : null;
  if (!node) {
    exact = false;
    node = blocks.find(
      (block) =>
        block.id &&
        (anchor.heading_hash
          ? positionHash(block.id) === anchor.heading_hash
          : block.id === anchor.heading),
    );
  }
  if (node)
    window.scrollTo(
      0,
      window.scrollY +
        node.getBoundingClientRect().top -
        (exact ? anchor.top + readingTop() : readingTop()),
    );
  else window.scrollTo(0, anchor.scroll);
  return exact;
}
function updateBlocked() {
  const selection = window.getSelection();
  return (
    composing ||
    (!!selection &&
      !selection.isCollapsed &&
      ($("document").contains(selection.anchorNode) ||
        $("document").contains(selection.focusNode)))
  );
}
function receiveUpdate(payload) {
  if (
    payload.revision !== watchRevision ||
    payload.source !== active ||
    !tabs.has(active)
  )
    return;
  if (payload.result.Err) {
    pendingUpdate = null;
    status(payload.result.Err);
    return;
  }
  const doc = payload.result.Ok;
  if (!doc) return;
  if (doc.html === tabs.get(active).doc.html) {
    pendingUpdate = null;
    return;
  }
  pendingUpdate = payload;
  if (updateBlocked()) {
    status("updatePending");
    return;
  }
  flushUpdate();
}
function flushUpdate() {
  if (!pendingUpdate || updateBlocked()) return;
  const payload = pendingUpdate;
  pendingUpdate = null;
  if (
    payload.revision !== watchRevision ||
    payload.source !== active ||
    !tabs.has(active)
  )
    return;
  const doc = payload.result.Ok;
  const size =
    [...tabs.entries()].reduce(
      (n, [id, tab]) =>
        n + (id === active ? doc.html.length : tab.doc?.html.length || 0),
      0,
    ) * 2;
  if (size > 64 * 1024 * 1024) {
    status("limit");
    return;
  }
  const anchor = captureReadingPosition();
  tabs.get(active).doc = doc;
  paint(doc, true);
  const exact = restoreReadingPosition(anchor);
  tabs.get(active).position = window.scrollY;
  status(exact ? "updated" : "updateFallback");
  scheduleSave();
}
window.addEventListener("focus", () => {
  if (active) startWatch(active);
});
document.addEventListener("selectionchange", () => {
  if (pendingUpdate && !updateBlocked()) flushUpdate();
});

function savePosition() {
  if (active && tabs.get(active)?.doc) {
    tabs.get(active).position = window.scrollY;
    tabs.get(active).savedAnchor = storedAnchor(captureReadingPosition());
  }
}
function tabButtons() {
  navigationButtons();
  $("tabs").hidden = tabs.size < 2;
  $("tabs").replaceChildren();
  for (const [id, tab] of tabs) {
    const button = document.createElement("button");
    button.textContent = tab.doc?.name || tab.name;
    button.title = id;
    button.setAttribute("aria-pressed", String(id === active));
    button.onclick = () => activate(id);
    const close = document.createElement("button");
    close.textContent = "×";
    close.setAttribute("aria-label", (tab.doc?.name || tab.name) + t("close"));
    close.onclick = () => closeTab(id);
    $("tabs").append(button, close);
  }
}
function closeTab(id) {
  if (id === active) dismissExternal(false);
  tabs.delete(id);
  searchCache.delete(id);
  if (active === id) {
    active = null;
    startWatch(null);
    const first = tabs.keys().next();
    if (!first.done) activate(first.value);
    else {
      $("document").textContent = t("choose");
      $("name").textContent = "";
      $("outline").replaceChildren();
    }
  }
  tabButtons();
  search();
  scheduleSave();
}
async function activate(id) {
  dismissExternal(false);
  if (!navigatingSearch) {
    ++searchRevision;
    clearTimeout(searchTimer);
  }
  savePosition();
  active = id;
  const request = ++activation;
  const tab = tabs.get(id);
  tabButtons();
  if (!tab) return;
  if (!tab.doc) {
    startWatch(null);
    $("document").textContent = t("restoring");
    $("name").textContent = tab.name;
    $("outline").replaceChildren();
    try {
      const doc = await invoke("restore_document", { source: id });
      if (tabs.get(id) !== tab) return;
      if (!fitsDocument(id, doc)) throw new Error("cacheLimit");
      tab.doc = doc;
    } catch (error) {
      if (request !== activation || active !== id) return;
      $("document").textContent = t(
        error.message === "cacheLimit" ? "cacheLimit" : "restoreMissing",
      );
      const retry = document.createElement("button");
      retry.textContent = t("retry");
      retry.onclick = () => activate(id);
      $("document").append(document.createElement("br"), retry);
      scheduleSave();
      return;
    }
    if (request !== activation || active !== id) return;
  }
  const snapshot = tab.doc;
  paint(snapshot);
  tabButtons();
  requestAnimationFrame(() => {
    if (
      active !== id ||
      request !== activation ||
      tabs.get(id)?.doc !== snapshot
    )
      return;
    if (tab.savedAnchor) {
      if (!restoreReadingPosition(tab.savedAnchor)) status("restoredNearby");
    } else window.scrollTo(0, tab.position);
    startWatch(id);
    scheduleSave();
  });
}

// Positions belong to their tab; history stores bounded, source-free anchors.
const HISTORY_LIMIT = 100;
let outlineOrigin = null;
function historyFor(tab) {
  return (tab.history ||= { back: [], forward: [] });
}
function navigationButtons() {
  const tab = tabs.get(active),
    history = tab?.history;
  $("back").disabled = !tab?.doc || !history?.back.length;
  $("forward").disabled = !tab?.doc || !history?.forward.length;
}
function historyPosition() {
  return storedAnchor(captureReadingPosition());
}
function samePosition(a, b) {
  return (
    a &&
    b &&
    a.block_hash === b.block_hash &&
    a.occurrence === b.occurrence &&
    Math.abs(a.top - b.top) < 2
  );
}
function recordJump(move, before = historyPosition()) {
  const tab = tabs.get(active);
  if (!tab?.doc) return;
  move();
  const after = historyPosition();
  if (before && !samePosition(before, after)) {
    const history = historyFor(tab);
    if (!samePosition(history.back.at(-1), before)) history.back.push(before);
    if (history.back.length > HISTORY_LIMIT) history.back.shift();
    history.forward = [];
  }
  savePosition();
  navigationButtons();
  scheduleSave();
}
function travel(direction) {
  const tab = tabs.get(active);
  if (!tab?.doc) return;
  const history = historyFor(tab),
    from = history[direction];
  if (!from.length) return;
  const current = historyPosition();
  const target = from.pop();
  const to = history[direction === "back" ? "forward" : "back"];
  if (current) to.push(current);
  if (to.length > HISTORY_LIMIT) to.shift();
  closeMenu();
  toggleResults(false);
  if (!restoreReadingPosition(target)) status("restoredNearby");
  $("main").focus({ preventScroll: true });
  savePosition();
  navigationButtons();
  scheduleSave();
}
$("back").onclick = () => travel("back");
$("forward").onclick = () => travel("forward");
function goToHeading(id, origin = historyPosition()) {
  const node = [...$("document").querySelectorAll("[id]")].find(
    (n) => n.id === id,
  );
  if (!node) {
    status("missingHeading");
    return;
  }
  recordJump(() => {
    node.scrollIntoView();
    window.scrollBy(0, -readingTop());
  }, origin);
}
function jump(doc) {
  if (!doc.anchor) return;
  const id = active,
    request = activation;
  requestAnimationFrame(() =>
    requestAnimationFrame(() => {
      if (
        active === id &&
        request === activation &&
        tabs.get(id)?.doc &&
        id === doc.id
      )
        goToHeading(doc.anchor);
    }),
  );
}
function fitsDocument(id, doc) {
  return (
    [...tabs.entries()].reduce(
      (n, [key, tab]) => n + (key === id ? 0 : tab.doc?.html.length || 0),
      doc.html.length,
    ) *
      2 <=
    64 * 1024 * 1024
  );
}
function show(doc) {
  const id = doc.id || doc.name;
  if (tabs.has(id)) {
    if (!tabs.get(id).doc) {
      if (!fitsDocument(id, doc)) {
        status("cacheLimit");
        return;
      }
      tabs.get(id).doc = doc;
    }
    activate(id);
    jump(doc);
    return;
  }
  if (
    tabs.size >= 20 ||
    [...tabs.values()].reduce(
      (n, tab) => n + (tab.doc?.html.length || 0),
      doc.html.length,
    ) *
      2 >
      64 * 1024 * 1024
  ) {
    status("limit");
    return;
  }
  tabs.set(id, { doc, position: 0 });
  activate(id);
  jump(doc);
}
function paint(doc, automatic = false) {
  dismissExternal(false);
  outlineOrigin = null;
  if (!automatic) closeMenu();
  $("document").innerHTML = doc.html;
  $("name").textContent = doc.name;
  document.title = doc.name + " — md-viewer";
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
      const origin = outlineOrigin;
      outlineOrigin = null;
      goToHeading(h.id, origin || historyPosition());
    };
    $("outline").append(a);
  }
  for (const table of $("document").querySelectorAll("table")) {
    const wrap = document.createElement("div");
    wrap.className = "table-wrap";
    wrap.tabIndex = 0;
    wrap.setAttribute("aria-label", t("table"));
    table.replaceWith(wrap);
    wrap.append(table);
  }
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
  if (!automatic) {
    $("main").focus();
    window.scrollTo(0, 0);
  }
  if (!navigatingSearch) search();
}
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
async function reviewExternal(source, url, trigger) {
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
    const review = await invoke("prepare_external", { source, url });
    if (
      revision !== externalRevision ||
      active !== source ||
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
// Search is limited to open tabs. Result text is never interpreted as HTML.
let searchRevision = 0,
  searchTimer,
  navigatingSearch = false;
const searchCache = new Map();
const SEARCH_LIMIT = 200,
  SEARCH_CACHE_LIMIT = 8 * 1024 * 1024;
function clearMarks() {
  for (const mark of $("document").querySelectorAll("mark"))
    mark.replaceWith(document.createTextNode(mark.textContent));
  $("document").normalize();
}
function textUnits(root) {
  const units = [],
    groups = new Map();
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  while (walker.nextNode()) {
    const node = walker.currentNode;
    if (node.parentElement.closest(".image-error, summary, .empty")) continue;
    const owner =
      node.parentElement.closest("p,pre,li,td,th,h1,h2,h3,h4,h5,h6") || root;
    if (!groups.has(owner)) {
      const unit = { text: "", nodes: [] };
      groups.set(owner, unit);
      units.push(unit);
    }
    const unit = groups.get(owner);
    unit.nodes.push({ node, start: unit.text.length });
    unit.text += node.textContent;
  }
  return units;
}
function htmlUnits(html) {
  const root = document.createElement("article");
  // HTML is generated by the Rust renderer; images are removed before DOM parsing.
  root.innerHTML = html.replace(/<img\b[^>]*>/gi, "");
  return textUnits(root).map((unit) => unit.text);
}
function markMatches(results) {
  clearMarks();
  const units = textUnits($("document"));
  for (const result of results) result.marks = [];
  for (let i = 0; i < units.length; i++) {
    const unit = units[i];
    const matches = results.filter(
      (r) => r.id === active && r.unit === i && r.text === unit.text,
    );
    if (!matches.length) continue;
    for (const { node, start } of unit.nodes) {
      const end = start + node.textContent.length;
      const frag = document.createDocumentFragment();
      let cursor = 0;
      for (const match of matches) {
        const left = Math.max(start, match.start),
          right = Math.min(end, match.end);
        if (left >= right) continue;
        frag.append(node.textContent.slice(cursor, left - start));
        const mark = document.createElement("mark");
        mark.textContent = node.textContent.slice(left - start, right - start);
        frag.append(mark);
        match.marks.push(mark);
        cursor = right - start;
      }
      if (cursor) {
        frag.append(node.textContent.slice(cursor));
        node.replaceWith(frag);
      }
    }
  }
}
function search() {
  clearTimeout(searchTimer);
  const revision = ++searchRevision;
  clearMarks();
  hits = [];
  selected = -1;
  $("search-results").replaceChildren();
  toggleResults(false);
  $("results-toggle").hidden = true;
  $("count").textContent = "";
  if (!$("query").value || document.querySelector(".search").hidden) return;
  if ($("search-scope").value === "tabs") {
    $("count").textContent = t("searching");
    searchTimer = setTimeout(() => runSearch(revision), 180);
  } else runSearch(revision);
}
async function runSearch(revision) {
  const query = $("query").value,
    across = $("search-scope").value === "tabs";
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const pattern = new RegExp(escaped, "giu");
  const results = [];
  let failed = 0,
    limited = false;
  const entries = across
    ? [...tabs.entries()]
    : tabs.has(active)
      ? [[active, tabs.get(active)]]
      : [];
  for (const [id, tab] of entries) {
    let units;
    try {
      if (tab.doc) units = htmlUnits(tab.doc.html);
      else if (searchCache.has(id)) units = searchCache.get(id);
      else {
        const doc = await invoke("search_document", { source: id });
        if (revision !== searchRevision) return;
        units = htmlUnits(doc.html);
        const bytes = units.reduce((n, text) => n + text.length * 2, 0);
        if (bytes <= SEARCH_CACHE_LIMIT) {
          let total = [...searchCache.values()].reduce(
            (n, list) => n + list.reduce((m, text) => m + text.length * 2, 0),
            0,
          );
          while (total + bytes > SEARCH_CACHE_LIMIT && searchCache.size) {
            const key = searchCache.keys().next().value;
            total -= searchCache
              .get(key)
              .reduce((n, text) => n + text.length * 2, 0);
            searchCache.delete(key);
          }
          searchCache.set(id, units);
        }
      }
    } catch {
      failed++;
      continue;
    }
    if (revision !== searchRevision) return;
    if (tabs.get(id) !== tab) continue;
    units.forEach((text, unit) => {
      pattern.lastIndex = 0;
      for (const match of text.matchAll(pattern)) {
        if (results.length === SEARCH_LIMIT) {
          limited = true;
          break;
        }
        results.push({
          id,
          name: tab.doc?.name || tab.name,
          text,
          unit,
          start: match.index,
          end: match.index + match[0].length,
          marks: [],
        });
      }
    });
    if (limited) break;
  }
  if (revision !== searchRevision) return;
  hits = results;
  markMatches(hits);
  $("count").textContent =
    `${hits.length}${t("matches")}${limited ? t("searchLimited") : ""}${failed ? ` · ${failed}${t("searchFailed")}` : ""}`;
  if (across) {
    $("results-toggle").hidden = !hits.length;
    toggleResults(hits.length > 0);
    for (let i = 0; i < hits.length; i++) {
      const hit = hits[i],
        button = document.createElement("button");
      const name = document.createElement("strong"),
        snippet = document.createElement("span");
      name.textContent = hit.name;
      button.title = hit.id;
      snippet.textContent =
        (hit.start > 35 ? "…" : "") +
        hit.text.slice(Math.max(0, hit.start - 35), hit.end + 55) +
        (hit.end + 55 < hit.text.length ? "…" : "");
      button.append(name, snippet);
      button.onclick = () => selectHit(i);
      $("search-results").append(button);
    }
  }
}
async function selectHit(index) {
  const hit = hits[index];
  if (!hit || !tabs.has(hit.id)) return;
  const revision = searchRevision,
    query = $("query").value;
  selected = index;
  if (active !== hit.id || !tabs.get(hit.id)?.doc) {
    navigatingSearch = true;
    try {
      await activate(hit.id);
    } finally {
      navigatingSearch = false;
    }
    await new Promise(requestAnimationFrame);
  }
  if (
    revision !== searchRevision ||
    selected !== index ||
    query !== $("query").value ||
    active !== hit.id ||
    !tabs.has(hit.id)
  )
    return;
  const unit = textUnits($("document"))[hit.unit];
  if (!unit || unit.text !== hit.text) {
    searchCache.delete(hit.id);
    search();
    status("searchChanged");
    return;
  }
  markMatches(hits);
  for (const result of hits)
    for (const mark of result.marks)
      mark.classList.toggle("current", result === hit);
  toggleResults(false);
  recordJump(() => hit.marks[0]?.scrollIntoView({ block: "center" }));
  $("query").focus({ preventScroll: true });
  $("count").textContent = `${index + 1} / ${hits.length}`;
}
function toggleResults(visible) {
  $("search-results").hidden = !visible;
  $("results-toggle").setAttribute("aria-expanded", String(visible));
}
$("results-toggle").onclick = () => toggleResults($("search-results").hidden);
function step(delta) {
  if (!hits.length) return;
  selectHit(
    selected < 0
      ? delta > 0
        ? 0
        : hits.length - 1
      : (selected + delta + hits.length) % hits.length,
  );
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
  const origin = outlineOrigin;
  outlineOrigin = opening ? historyPosition() : null;
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
$("smaller").onclick = () => {
  prefs.size = Math.max(12, prefs.size - 2);
  appearance();
  persist();
};
$("larger").onclick = () => {
  prefs.size = Math.min(36, prefs.size + 2);
  appearance();
  persist();
};
const presets = {
  paper: { paper: "#faf9f6", ink: "#292e32" },
  night: { paper: "#1e2423", ink: "#e4e7e1" },
  warm: { paper: "#f5ebd7", ink: "#493e30" },
};
$("theme").onclick = () => {
  Object.assign(
    prefs,
    luminance(prefs.paper) < 0.18 ? presets.paper : presets.night,
  );
  appearance();
  persist();
};
$("preset").onchange = () => {
  Object.assign(prefs, presets[$("preset").value]);
  appearance();
  persist();
};
$("language").onchange = () => changeLanguage($("language").value);
$("foreground").oninput = () => {
  prefs.ink = $("foreground").value;
  appearance();
  persist();
};
$("background").oninput = () => {
  prefs.paper = $("background").value;
  appearance();
  persist();
};
$("width").oninput = () => {
  prefs.width = Number($("width").value);
  appearance();
  persist();
};
$("restore-session").onchange = () => {
  restoreEnabled = $("restore-session").checked;
  saveSession().catch(() => status("sessionSaveFailed"));
};
$("quit-without-session").onclick = () => invoke("finish_close");
$("reset").onclick = () => {
  Object.assign(prefs, presets.paper, { size: 18, width: 44 });
  appearance();
  persist();
};
$("search-scope").onchange = search;
$("query").addEventListener("compositionstart", () => {
  composing = true;
  ++searchRevision;
  clearTimeout(searchTimer);
});
$("query").addEventListener("compositionend", () => {
  composing = false;
  search();
  flushUpdate();
});
$("query").oninput = () => {
  if (!composing) search();
};
$("next").onclick = () => step(1);
$("previous").onclick = () => step(-1);
window.addEventListener("keydown", (e) => {
  if ($("external-dialog").open) return;
  if (e.isComposing || composing) return;
  const editable =
    e.target instanceof Element &&
    e.target.closest("input,textarea,select,[contenteditable=true]");
  const back =
    (e.altKey && !e.metaKey && !e.ctrlKey && e.key === "ArrowLeft") ||
    (e.metaKey &&
      !e.altKey &&
      !e.ctrlKey &&
      (e.key === "[" || e.code === "BracketLeft"));
  const forward =
    (e.altKey && !e.metaKey && !e.ctrlKey && e.key === "ArrowRight") ||
    (e.metaKey &&
      !e.altKey &&
      !e.ctrlKey &&
      (e.key === "]" || e.code === "BracketRight"));
  if (!editable && !e.shiftKey && (back || forward)) {
    e.preventDefault();
    travel(back ? "back" : "forward");
    return;
  }
  if ((e.metaKey || e.ctrlKey) && e.key === "o") {
    e.preventDefault();
    $("open").click();
  }
  if ((e.metaKey || e.ctrlKey) && e.key === "r") {
    e.preventDefault();
    if (active) startWatch(active);
  }
  if ((e.metaKey || e.ctrlKey) && e.key === "f") {
    e.preventDefault();
    openSearch();
  }
  if ((e.metaKey || e.ctrlKey) && ["+", "=", "-", "0"].includes(e.key)) {
    e.preventDefault();
    if (e.key === "0") {
      prefs.size = 18;
      appearance();
      persist();
    } else $(e.key === "-" ? "smaller" : "larger").click();
  }
  if ((e.metaKey || e.ctrlKey) && e.key === "w") {
    e.preventDefault();
    if (active) closeTab(active);
  }
  if (e.ctrlKey && e.key === "Tab" && tabs.size) {
    e.preventDefault();
    const ids = [...tabs.keys()];
    activate(
      ids[
        (ids.indexOf(active) + (e.shiftKey ? -1 : 1) + ids.length) % ids.length
      ],
    );
  }
  if (e.key === "Enter" && document.activeElement === $("query")) {
    e.preventDefault();
    step(e.shiftKey ? -1 : 1);
  }
  if (e.key === "Escape") {
    const origin = !$("outline").hidden && outlineOrigin;
    $("outline").hidden = true;
    outlineOrigin = null;
    document.querySelector(".search").hidden = true;
    $("query").value = "";
    search();
    closeMenu();
    $("settings").hidden = true;
    $("contents").setAttribute("aria-expanded", "false");
    $("main").focus({ preventScroll: true });
    if (origin) restoreReadingPosition(origin);
  }
});
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
localize();
if (!window.__TAURI_INTERNALS__) sessionReady = true;
if (window.__TAURI_INTERNALS__) {
  listen("save-and-close", () => closeWithSave());
  listen("document-updated", (event) => receiveUpdate(event.payload))
    .then(() =>
      listen("documents-ready", () => {
        if (sessionReady) drain();
      }),
    )
    .then(async () => {
      let selected = null,
        warning = null;
      try {
        const loaded = await invoke("load_session");
        installSession(loaded.session);
        selected = loaded.session.selected;
        warning = loaded.warning;
      } catch (error) {
        sessionAvailable = false;
        warning = String(error);
      }
      sessionReady = true;
      const startupActivation = activation;
      const incoming = await invoke("take_documents");
      if (incoming.length) {
        for (const result of incoming) {
          if (result.Ok) show(result.Ok);
          else status(result.Err);
        }
      } else if (tabs.size && activation === startupActivation)
        await activate(
          selected && tabs.has(selected) ? selected : tabs.keys().next().value,
        );
      if (warning) status(warning);
    })
    .catch((error) => status(String(error)));
  listen("menu-action", (event) => {
    if (event.payload === "open") $("open").click();
    else if (event.payload === "close" && active) closeTab(active);
    else if (event.payload === "settings") {
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
