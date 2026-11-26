import { categories } from "./guide-catalog.js";
import { $ } from "./platform.js";

"use strict";

const writerGuide = (() => {

  const labels = {
    ja: {
      query: "何を書きたいですか？",
      placeholder: "表、手順、チェックリストなど",
      intents: ["表を作りたい", "手順を並べたい", "チェックリストを作りたい", "リンクを貼りたい"],
      toc: "目次",
      top: "目次へ戻る",
      help: "サンプルを押すとカーソル位置に挿入します。",
      clear: "すべてのサンプル",
      results: (count) => count ? count + "件のサンプル" : "見つかりませんでした。「表」「手順」「画像」などで検索してください。",
      references: "公式資料",
      links: ["CommonMark チュートリアル（英語）", "CommonMark 記法一覧（英語）", "CommonMark 仕様（英語）", "GitHub Docs の基本記法", "GitHub Docs の表", "GitHub Docs のコードブロック", "GitHub Flavored Markdown 仕様（英語）"],
    },
    en: {
      query: "What do you want to write?",
      placeholder: "Table, steps, checklist…",
      intents: ["Make a table", "Number the steps", "Make a checklist", "Add a link"],
      toc: "Contents",
      top: "Back to contents",
      help: "Choose a sample to insert at the cursor.",
      clear: "All samples",
      results: (count) => count ? count + (count === 1 ? " sample" : " samples") : "No matching samples. Try “table,” “steps,” or “image.”",
      references: "Official references",
      links: ["CommonMark tutorial", "CommonMark reference", "CommonMark specification", "GitHub Docs basic syntax", "GitHub Docs on tables", "GitHub Docs on code blocks", "GitHub Flavored Markdown specification"],
    },
  };
  const pages = ["tutorial", "reference", "commonmark", "github", "tables", "code", "gfm"];
  let language = "ja";
  let onInsert;
  let onReference;
  let composing = false;
  const normalize = (value) => value.normalize("NFKC").toLowerCase().trim().replace(/\s+/g, " ");
  const searchable = categories.flatMap(({ samples }) => samples.map((entry) => ({
    entry,
    names: [entry.ja, entry.en].map(normalize),
    terms: entry.terms.map(normalize),
  })));

  function match(query, { names, terms }) {
    if (names.includes(query)) return 0;
    if (terms.includes(query)) return 1;
    const text = [...names, ...terms];
    const contains = (term, word) => word.length === 1 ? term === word : term.includes(word);
    if (text.some((term) => contains(term, query))) return 2;
    const keywords = query
      .replace(/(?:を|が)?(?:作成|作り|書き|貼り|入れ|並べ|追加)(?:し)?たい$/u, "")
      .split(" ")
      .filter((word) => word && !/^(?:a|an|the|i|want|to|create|make|add|how|can|do)$/u.test(word));
    if (keywords.length && keywords.every((word) => text.some((term) => contains(term, word)))) return 3;
    return Infinity;
  }

  function button(label, action) {
    const node = document.createElement("button");
    node.type = "button";
    node.textContent = label;
    node.onclick = action;
    return node;
  }

  function listItem(node) {
    const li = document.createElement("li");
    li.append(node);
    return li;
  }

  function jump(id) {
    const heading = $(id);
    heading.scrollIntoView({ block: "start" });
    heading.focus({ preventScroll: true });
  }

  function card(entry) {
    const markdown = language === "ja" ? entry.text : entry.english;
    const item = button(entry[language], () => onInsert(markdown, entry.inline));
    const title = document.createElement("strong");
    const snippet = document.createElement("code");
    title.textContent = entry[language];
    snippet.textContent = markdown;
    item.className = "guide-sample";
    item.replaceChildren(title, snippet);
    item.setAttribute("aria-label", entry[language]);
    if (entry.note) {
      const note = document.createElement("span");
      note.textContent = entry.note[language === "ja" ? 0 : 1];
      item.append(note);
    }
    return item;
  }

  function section(category) {
    const group = document.createElement("section");
    const header = document.createElement("div");
    const heading = document.createElement("h3");
    heading.id = "guide-category-" + category.id;
    heading.tabIndex = -1;
    heading.textContent = category[language];
    group.setAttribute("aria-labelledby", heading.id);
    header.className = "guide-category-heading";
    header.append(heading, button(labels[language].top, () => jump("guide-toc-title")));
    group.append(header, ...category.samples.map(card));
    return group;
  }

  function renderSamples() {
    const query = normalize($("guide-query").value);
    $("guide-toc").hidden = Boolean(query);
    $("guide-references").hidden = Boolean(query);
    $("guide-clear").hidden = !query;
    if (!query) {
      $("guide-search-status").textContent = "";
      $("writer-samples").replaceChildren(...categories.map(section));
      return;
    }
    const hits = searchable.map((item) => ({ item, rank: match(query, item) }))
      .filter(({ rank }) => Number.isFinite(rank))
      .sort((a, b) => a.rank - b.rank);
    $("guide-search-status").textContent = labels[language].results(hits.length);
    $("writer-samples").replaceChildren(...hits.map(({ item }) => card(item.entry)));
  }

  function selectIntent(intent) {
    $("guide-query").value = intent;
    renderSamples();
    $("guide-query").focus({ preventScroll: true });
  }

  function localize(locale) {
    language = locale;
    const copy = labels[locale];
    $("guide-query-label").textContent = copy.query;
    $("guide-query").placeholder = copy.placeholder;
    $("guide-toc-title").textContent = copy.toc;
    $("guide-references-title").textContent = copy.references;
    $("writer-guide-help").textContent = copy.help;
    $("guide-clear").textContent = copy.clear;
    $("guide-intents").replaceChildren(...copy.intents.map((intent) => button(intent, () => selectIntent(intent))));
    $("guide-categories").replaceChildren(
      ...categories.map((category) => listItem(button(category[locale], () => jump("guide-category-" + category.id)))),
      listItem(button(copy.references, () => jump("guide-references-title"))),
    );
    $("guide-links").replaceChildren(...pages.map((page, index) =>
      listItem(button(copy.links[index], (event) => onReference(page, event.currentTarget))),
    ));
    renderSamples();
  }

  function init(insert, openReference) {
    onInsert = insert;
    onReference = openReference;
    $("guide-clear").onclick = () => selectIntent("");
    const input = $("guide-query");
    input.addEventListener("compositionstart", () => { composing = true; });
    input.addEventListener("compositionend", () => { composing = false; renderSamples(); });
    input.addEventListener("input", (event) => {
      if (!composing && !event.isComposing) renderSamples();
    });
  }

  return { init, localize };
})();

export { writerGuide };
