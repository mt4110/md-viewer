import { $ } from "./platform.js";
import { appearance, luminance, persist, prefs } from "./preferences.js";
import { changeLanguage } from "./localization.js";

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
$("reset").onclick = () => {
  Object.assign(prefs, presets.paper, { size: 18, width: 44 });
  appearance();
  persist();
};
