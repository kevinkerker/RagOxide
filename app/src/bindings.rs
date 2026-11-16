//! Generation of the TypeScript bindings `ui/api/bindings.ts`.
//!
//! The file contains every type that crosses the boundary (declared with `ts-rs`), the
//! command list as the interface `Commands` and the event list as the interface `Events`.
//! The test `tests/bindings.rs` fails while the checked-in file differs from the output;
//! `UPDATE_BINDINGS=1` rewrites it.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use ts_rs::{Config, TS};

use crate::api::{self, Signature};
use crate::commands;

/// Header of the generated file.
const HEADER: &str = "\
// Generated from the Rust types in kb-core and app/src (api.rs, commands.rs). Do not edit.
// Regenerate with `pnpm gen:types` in ui/. `cargo test -p ragoxide-app` fails while this
// file is outdated.
";

struct Declarations<'a> {
    cfg: &'a Config,
    out: String,
    names: BTreeSet<String>,
    dependencies: Vec<(String, String)>,
}

impl Declarations<'_> {
    fn add<T: TS + 'static>(&mut self) {
        let name = T::ident(self.cfg);
        if let Some(docs) = T::docs() {
            self.out.push_str(&docs);
        }
        let _ = writeln!(self.out, "export {}\n", T::decl(self.cfg));
        for dependency in T::dependencies(self.cfg) {
            self.dependencies
                .push((name.clone(), dependency.ts_name.clone()));
        }
        self.names.insert(name);
    }
}

macro_rules! declare {
    ($declarations:ident: $($ty:ty),* $(,)?) => {
        $( $declarations.add::<$ty>(); )*
    };
}

fn push_docs(out: &mut String, docs: &[&str], indent: &str) {
    if docs.is_empty() {
        return;
    }
    let _ = writeln!(out, "{indent}/**");
    for line in docs {
        let _ = writeln!(out, "{indent} *{}", line.trim_end());
    }
    let _ = writeln!(out, "{indent} */");
}

fn push_list(out: &mut String, interface: &str, constant: &str, signatures: &[Signature]) {
    let _ = writeln!(out, "export interface {interface} {{");
    for signature in signatures {
        push_docs(out, &signature.docs, "  ");
        if interface == "Events" {
            let _ = writeln!(out, "  {}: {};", signature.name, signature.result);
        } else {
            let _ = writeln!(
                out,
                "  {}: {{ args: {}; event: {}; result: {} }};",
                signature.name,
                signature.args.as_deref().unwrap_or("null"),
                signature.event.as_deref().unwrap_or("null"),
                signature.result,
            );
        }
    }
    let _ = writeln!(out, "}}\n");
    let _ = writeln!(out, "export const {constant} = [");
    for signature in signatures {
        let _ = writeln!(out, "  \"{}\",", signature.name);
    }
    let _ = writeln!(out, "] as const;");
}

/// Generates the content of `ui/api/bindings.ts`.
///
/// Panics if a declared type refers to a type that is not declared, so that the file
/// is always self-contained.
pub fn generate_bindings() -> String {
    use crate::api::*;
    use kb_core::*;

    let cfg = Config::new().with_large_int("number");
    let mut declarations = Declarations {
        cfg: &cfg,
        out: String::new(),
        names: BTreeSet::new(),
        dependencies: Vec::new(),
    };
    declare!(declarations:
        // Documents and anchors (kb-core)
        DocumentId, DocumentFormat, SourceFile, Document, Page, Element, ElementKind, Table,
        TableCell, Provenance, Anchor, BoundingBox, CellRange,
        // Search, answers and chats (kb-core)
        ChunkId, Chunk, SearchMode, SearchRequest, SearchHit, Citation, Answer, AnswerEvent,
        Timestamp, ChatId, AnswerOutcome, AnswerPhase, PhaseProgress, ChatTurn, Chat,
        ChatSummary,
        // Library, synchronisation and errors (kb-core)
        ErrorKind, DocumentRecord, DocumentStatus, FileStatus, IngestEvent, FileResult,
        ScopeEvent, SyncMode,
        // Settings and models (kb-core)
        AnswerLevel, HardwareTier, UiLanguage, ColorScheme, Settings, HardwareInfo, ModelRole, ModelEntry,
        ModelFile, FileSource, HuggingFaceSource, ReleaseSource, ModelFileStatus, ModelEvent,
        MissingModelFile, ModelStorage,
        // Arguments, results, errors and event payloads of the app
        OperationId, AskArgs, AskAgainArgs, OperationArgs, SyncArgs, ProvideModelsArgs, RenameChatArgs, ChatArgs,
        DocumentArgs, PathArgs, PathsArgs, FolderPurpose, PickFolderArgs, AnswerLevelArgs,
        SettingsChange, DocumentDetails, DocumentFileInfo, DocumentFile, FolderStatus,
        WatchedFolder, LibraryScope, LevelState, SetupState,
        ApiError, IngestProgress, IngestFinished, ScopeNotice, ModelProgress,
    );
    for (user, dependency) in &declarations.dependencies {
        assert!(
            declarations.names.contains(dependency),
            "{user} uses {dependency}, which is not declared in bindings.rs"
        );
    }

    let mut out = String::from(HEADER);
    out.push('\n');
    out.push_str(&declarations.out);
    push_list(
        &mut out,
        "Commands",
        "COMMAND_NAMES",
        &commands::command_signatures(&cfg),
    );
    out.push('\n');
    push_list(
        &mut out,
        "Events",
        "EVENT_NAMES",
        &api::event_signatures(&cfg),
    );
    out
}
