const writerState = {
  draft: null,
  opening: false,
  saving: false,
  backgroundSave: false,
  formatting: false,
  composing: false,
  message: null,
  previewError: null,
  previewTimer: undefined,
  previewRevision: 0,
  recoveryTimer: undefined,
  autosaveTimer: undefined,
  lastRecovery: null,
};

const isWriting = () => Boolean(writerState.draft);

export { isWriting, writerState };
