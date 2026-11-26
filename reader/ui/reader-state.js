const tabs = new Map();
const searchCache = new Map();

const readingState = {
  active: null,
  activation: 0,
  outlineOrigin: null,
};

const findState = {
  searchRevision: 0,
  searchTimer: undefined,
  navigatingSearch: false,
  composing: false,
};

const sessionState = {
  sessionReady: false,
  restoreEnabled: true,
  sessionAvailable: true,
};

export { findState, readingState, searchCache, sessionState, tabs };
