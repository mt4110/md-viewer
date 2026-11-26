import { t } from "./preferences.js";

function wrapDocumentTables(article) {
  for (const table of article.querySelectorAll("table")) {
    const wrap = document.createElement("div");
    wrap.className = "table-wrap";
    wrap.tabIndex = 0;
    wrap.setAttribute("aria-label", t("table"));
    table.replaceWith(wrap);
    wrap.append(table);
  }
}

export { wrapDocumentTables };
