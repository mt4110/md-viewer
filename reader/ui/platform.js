const $ = (id) => document.getElementById(id);
const invoke = (name, args) => window.__TAURI_INTERNALS__.invoke(name, args);

export { $, invoke };
