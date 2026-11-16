// State shared inside the chat area only (answer column and history). Placeholder until
// package K builds the area.
let version = $state(0);

export const chatState = {
  /** Increases whenever a chat was saved or deleted, so the history reloads. */
  get version(): number {
    return version;
  },
  changed(): void {
    version += 1;
  },
};
