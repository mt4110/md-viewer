import { locales } from "./translations.js";
import { $ } from "./platform.js";
import { errorTranslations } from "./errors.js";

let prefs = {
  language: "ja",
  size: 18,
  width: 44,
  paper: "#faf9f6",
  ink: "#292e32",
  autoSaveFile: true,
  formatOnSave: true,
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

export { appearance, luminance, persist, prefs, status, t };
