use failure::ResultExt;
use std::path::Path;

use sync::{CopyAction, CopyModel, DirTree, FileType, Presence};

use super::errors::{RestoreError, RestoreErrorType};

/// Modified options for the restore action on ConfigFile. Check the properties to see which
/// behaviour they control
#[derive(Debug, Copy, Clone)]
pub struct RestoreOptions {
    /// Enables/Disables overwrite on the original locations during the restore. If the original
    /// location of the file backed up already exists this function will overwrite the location
    /// with the file backed up instead of exiting with an error.
    ///
    /// In short words: (overwrite == true) => function wil overwrite files on the original
    /// locations.
    pub(crate) overwrite: bool,
    /// Controls if the model should be ran or not. In case the model does not run, the
    /// intended actions will be logged into the screen
    pub(crate) run: bool,
}

impl RestoreOptions {
    /// Creates a new set of options for the restore operation.
    pub fn new(overwrite: bool, run: bool) -> Self {
        Self { overwrite, run }
    }
}

pub struct Restore;

impl Restore {
    pub fn from_point(
        restore: &Path,
        backup: &Path,
        overwrite: bool,
    ) -> Result<CopyModel, RestoreError> {
        let tree = DirTree::new(&restore, &backup).context(RestoreErrorType::Scan)?;
        Ok(tree
            .iter()
            .filter(|e| {
                e.presence() == Presence::Dst
                    || overwrite && e.presence() == Presence::Both && e.kind() != FileType::Dir
            }).map(|e| {
                if e.kind() == FileType::Dir && e.presence() == Presence::Dst {
                    CopyAction::CreateDir {
                        target: restore.join(e.path()),
                    }
                } else {
                    CopyAction::CopyFile {
                        src: backup.join(e.path()),
                        dst: restore.join(e.path()),
                    }
                }
            }).collect())
    }
}
