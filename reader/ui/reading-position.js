import { isWriting } from "./writer-state.js";
import { readingState, tabs } from "./reader-state.js";
import { $ } from "./platform.js";

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
function savePosition() {
  if (!isWriting() && readingState.active && tabs.get(readingState.active)?.doc) {
    tabs.get(readingState.active).position = window.scrollY;
    tabs.get(readingState.active).savedAnchor = storedAnchor(captureReadingPosition());
  }
}

export { captureReadingPosition, readingTop, restoreReadingPosition, savePosition, storedAnchor };
