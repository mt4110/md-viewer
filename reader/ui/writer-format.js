import { input } from "./writer-form.js";
import { invoke } from "./platform.js";

async function formatText(text) {
  if (!text) return text;
  const formatted = await new Promise((resolve, reject) => {
    if (new TextEncoder().encode(text).length > 2 * 1024 * 1024) {
      reject();
      return;
    }
    const worker = new Worker("vendor/markdownlint-worker.js");
    const finish = (callback, result) => {
      clearTimeout(timer);
      worker.terminate();
      callback(result);
    };
    const timer = setTimeout(() => finish(reject), 3000);
    worker.onmessage = ({ data }) => {
      if (typeof data.text === "string") finish(resolve, data.text);
      else finish(reject);
    };
    worker.onerror = (event) => {
      event.preventDefault();
      finish(reject);
    };
    worker.postMessage(text);
  });
  if (formatted === text) return text;
  const unchanged = await invoke("validate_format", { original: text, formatted });
  return unchanged ? formatted : text;
}

function applyFormatting(text) {
  const field = input();
  const old = field.value;
  if (text === old) return;
  const { selectionStart, selectionEnd, scrollTop } = field;
  let start = 0;
  while (start < old.length && start < text.length && old[start] === text[start]) start++;
  let suffix = 0;
  while (suffix < old.length - start && suffix < text.length - start &&
    old[old.length - suffix - 1] === text[text.length - suffix - 1]) suffix++;
  const oldEnd = old.length - suffix;
  const newEnd = text.length - suffix;
  const position = (index) => index <= start ? index :
    index >= oldEnd ? index + newEnd - oldEnd : Math.min(index, newEnd);
  const replacement = text.slice(start, newEnd);
  if (document.activeElement === field && !field.readOnly) {
    field.setSelectionRange(start, oldEnd);
    document.execCommand(replacement ? "insertText" : "delete", false, replacement);
  }
  if (field.value !== text) field.value = text;
  field.setSelectionRange(position(selectionStart), position(selectionEnd));
  field.scrollTop = scrollTop;
}

export { applyFormatting, formatText };
