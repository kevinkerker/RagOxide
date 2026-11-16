//! An in-memory file system for the [`MockIngestor`](super::MockIngestor).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use super::{fixtures, placeholder_hash};
use crate::document::key_within;
use crate::{Result, Timestamp, canonical_path, path_key};

/// Modification time of a newly added file: 2026-11-15T00:00:00Z.
const START: Timestamp = Timestamp(1_794_700_800_000);

struct FileEntry {
    path: PathBuf,
    /// Content version; each content change increments it.
    version: u32,
    size: u64,
    modified: Timestamp,
}

#[derive(Default)]
struct FilesState {
    /// Existing files by path key.
    files: BTreeMap<String, FileEntry>,
    /// Path keys of directories created explicitly (empty ones included).
    dirs: BTreeSet<String>,
    /// Path keys of directories that cannot be read, with everything below them.
    unreachable: BTreeSet<String>,
    /// Number of content hashes computed so far.
    hashes: usize,
}

/// Files the mock ingestor sees instead of the disk.
///
/// A directory exists if it was created with [`MockFiles::add_dir`] (empty directories
/// included) or a file lies below it; a path that is a file is never a directory. A
/// directory marked unreachable (and everything below it) cannot be read, like an
/// offline network drive. Each file has a
/// content version, a size and a modification time; [`MockFiles::hash_count`] tells how
/// often a content hash was computed.
#[derive(Default)]
pub struct MockFiles {
    state: Mutex<FilesState>,
}

impl MockFiles {
    /// Folder that holds the fixture files in [`MockFiles::fixtures`].
    pub const FIXTURE_FOLDER: &'static str = r"C:\Ablage";

    /// No files.
    pub fn new() -> Self {
        Self::default()
    }

    /// The seven fixture documents as files in [`MockFiles::FIXTURE_FOLDER`], matching
    /// the sources in `documents.json`.
    pub fn fixtures() -> Self {
        let files = Self::new();
        for name in fixtures::documents().into_keys() {
            files
                .add_file(&Path::new(Self::FIXTURE_FOLDER).join(name))
                .expect("fixture paths are absolute");
        }
        files
    }

    fn lock(&self) -> MutexGuard<'_, FilesState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn update(&self, path: &Path, change: impl FnOnce(&mut FileEntry)) -> Result<()> {
        let key = path_key(path)?;
        if let Some(entry) = self.lock().files.get_mut(&key) {
            change(entry);
        }
        Ok(())
    }

    /// Creates a file (or resets an existing one to its first version) with a size of
    /// 1000 bytes and a fixed modification time.
    pub fn add_file(&self, path: &Path) -> Result<()> {
        let path = canonical_path(path)?;
        let entry = FileEntry {
            path: path.clone(),
            version: 0,
            size: 1000,
            modified: START,
        };
        self.lock().files.insert(path_key(&path)?, entry);
        Ok(())
    }

    /// Changes the content of a file: its hash changes, it grows by one byte and its
    /// modification time moves forward by a minute.
    pub fn modify(&self, path: &Path) -> Result<()> {
        self.update(path, |e| {
            e.version += 1;
            e.size += 1;
            e.modified = Timestamp(e.modified.0 + 60_000);
        })
    }

    /// Creates a directory (empty unless files are added below it).
    pub fn add_dir(&self, path: &Path) -> Result<()> {
        let key = path_key(path)?;
        self.lock().dirs.insert(key);
        Ok(())
    }

    /// Changes the content and size of a file but keeps its modification time.
    pub fn modify_keeping_time(&self, path: &Path) -> Result<()> {
        self.update(path, |e| {
            e.version += 1;
            e.size += 1;
        })
    }

    /// Changes the content of a file but keeps size and modification time, like a
    /// tool that restores timestamps. The case the shortcut cannot see.
    pub fn modify_keeping_metadata(&self, path: &Path) -> Result<()> {
        self.update(path, |e| e.version += 1)
    }

    /// Moves the modification time forward by a minute without changing the content.
    pub fn touch(&self, path: &Path) -> Result<()> {
        self.update(path, |e| e.modified = Timestamp(e.modified.0 + 60_000))
    }

    /// Deletes a file, or a directory with everything below it.
    pub fn remove(&self, path: &Path) -> Result<()> {
        let key = path_key(path)?;
        let mut state = self.lock();
        state.files.retain(|k, _| !key_within(k, &key));
        state.dirs.retain(|k| !key_within(k, &key));
        Ok(())
    }

    /// Makes a directory and everything below it unreadable or readable again.
    pub fn set_reachable(&self, dir: &Path, reachable: bool) -> Result<()> {
        let key = path_key(dir)?;
        let mut state = self.lock();
        if reachable {
            state.unreachable.remove(&key);
        } else {
            state.unreachable.insert(key);
        }
        Ok(())
    }

    /// `true` if `dir` can be read (it or a directory above it is not unreachable).
    pub fn readable(&self, dir: &Path) -> bool {
        let Ok(key) = path_key(dir) else {
            return false;
        };
        !self.lock().unreachable.iter().any(|u| key_within(&key, u))
    }

    /// `true` if `path` is a directory (see the type docs).
    pub fn is_dir(&self, path: &Path) -> bool {
        let Ok(key) = path_key(path) else {
            return false;
        };
        let state = self.lock();
        !state.files.contains_key(&key)
            && (state.dirs.iter().any(|d| key_within(d, &key))
                || state.files.keys().any(|k| key_within(k, &key)))
    }

    /// `true` if `path` is a file or a directory.
    pub fn exists(&self, path: &Path) -> bool {
        let Ok(key) = path_key(path) else {
            return false;
        };
        self.lock().files.contains_key(&key) || self.is_dir(path)
    }

    /// Number of content hashes computed so far.
    pub fn hash_count(&self) -> usize {
        self.lock().hashes
    }

    /// Readable files strictly below `dir`.
    pub(crate) fn files_below(&self, dir: &Path) -> Vec<PathBuf> {
        let Ok(key) = path_key(dir) else {
            return Vec::new();
        };
        let state = self.lock();
        state
            .files
            .iter()
            .filter(|(k, _)| **k != key && key_within(k, &key))
            .filter(|(k, _)| !state.unreachable.iter().any(|u| key_within(k, u)))
            .map(|(_, e)| e.path.clone())
            .collect()
    }

    /// Size and modification time of a file.
    pub(crate) fn metadata(&self, path: &Path) -> Result<Option<(u64, Timestamp)>> {
        let key = path_key(path)?;
        Ok(self.lock().files.get(&key).map(|e| (e.size, e.modified)))
    }

    /// Content hash of a file in the format of a SHA-256: [`placeholder_hash`] of the path
    /// for the first version, derived from it for later versions. Counted.
    pub(crate) fn hash(&self, path: &Path) -> Result<Option<String>> {
        let key = path_key(path)?;
        let version = {
            let mut state = self.lock();
            state.hashes += 1;
            state.files.get(&key).map(|e| e.version)
        };
        version
            .map(|v| match v {
                0 => placeholder_hash(path),
                v => placeholder_hash(&path.join(format!("version-{v}"))),
            })
            .transpose()
    }
}
