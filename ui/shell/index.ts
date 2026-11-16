// What the shell offers the areas (see ui/README.md, "Rahmen und Zusammenspiel"):
//
//   import { t, workspace, evidence, askLock, Icon } from "$shell";
export { i18n, LANGUAGES, t, type Params } from "./i18n.svelte";
export {
  navigate,
  navigation,
  ROUTES,
  viewerTargetOf,
  type EvidenceKind,
  type Route,
  type ViewerTarget,
} from "./navigation.svelte";
export {
  evidence,
  evidenceColumn,
  places,
  portal,
  workspace,
  type EvidenceSource,
} from "./workspace.svelte";
export { answerLevel, askLock, setup, SETUP_STEPS, type LevelStatus } from "./setup.svelte";
export { libraryStatus, openStatusDetail } from "./status.svelte";
export { removedExclusions, sourceState, type SourceState } from "./library.svelte";
export { viewport, type LayoutMode } from "./viewport.svelte";
export { default as Icon, type IconName } from "./Icon.svelte";
