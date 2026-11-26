import { applyFixes } from "markdownlint";
import { lint } from "markdownlint/sync";

const rules = {
  default: false,
  MD009: { br_spaces: 2, code_blocks: false, strict: false },
  MD012: { maximum: 1 },
  MD047: true,
};

self.onmessage = ({ data: text }) => {
  try {
    const result = lint({
      strings: { document: text },
      config: rules,
      noInlineConfig: true,
    });
    self.postMessage({ text: applyFixes(text, result.document) });
  } catch {
    self.postMessage({ failed: true });
  }
};
