import { $ } from "./platform.js";

function toggleResults(visible) {
  $("search-results").hidden = !visible;
  $("results-toggle").setAttribute("aria-expanded", String(visible));
}
$("results-toggle").onclick = () => toggleResults($("search-results").hidden);

export { toggleResults };
