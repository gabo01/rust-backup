use std::fmt::Debug;
use std::fs;
use std::iter::FromIterator;
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::symlink_file as symlink;
use std::path::PathBuf;

use super::super::Model;
use {Debuggable, FnBox};

/// An alias for a list of actions
pub type Actions = Vec<CopyAction>;

/// Represents the actions to perform in order to execute a full copy model for a specific operation.
/// These operations can be, but are not limited to, backup and restore. The different variants
/// represent the different operations an action can take. It is expected that these actions are
/// determined based on a comparison tree of the two locations which should have been previously
/// built.
#[derive(Debug)]
pub enum CopyAction {
    /// Creates a directory on the target location, this is expected to be done if a tree node
    /// is present in one location but not in the other. This action creates the target dir and
    /// any path ancestor not present already on the file system.
    CreateDir {
        #[allow(missing_docs)]
        target: PathBuf,
    },
    /// Performs a full copy of the file from src to dst, a thing to notice is that in complex
    /// operations, src and dst may not exactly match the result of taking src and dst + the node
    /// path from the tree.
    CopyFile {
        #[allow(missing_docs)]
        src: PathBuf,
        #[allow(missing_docs)]
        dst: PathBuf,
    },
    /// Creates a symlink on dst that points to src. As said on the CopyFile docs, src and dst may
    /// not exactly match, but are supposed to be derived from, the src and dst of the comparison
    /// tree.
    CopyLink {
        #[allow(missing_docs)]
        src: PathBuf,
        #[allow(missing_docs)]
        dst: PathBuf,
    },
}

/// Copy model for a specific operation. Take an operation such as a backup, you can describe that
/// operation with a series of actions such as: create dir a, copy file b to c. These model is
/// esentially a list of actions to perform in order to say an operation has been done and a
/// function to call after the operation is completed.
#[derive(Debug)]
pub struct CopyModel<'a> {
    actions: Vec<CopyAction>,
    cleaner: Debuggable<FnBox + 'a>,
}

impl<'a> CopyModel<'a> {
    #[allow(missing_docs)]
    pub fn new<C: FnOnce() + 'a>(actions: Vec<CopyAction>, cleaner: C) -> Self {
        Self {
            actions,
            cleaner: closure!(cleaner),
        }
    }
}

impl<'a> Default for CopyModel<'a> {
    fn default() -> Self {
        Self {
            actions: vec![],
            cleaner: closure!(|| {}),
        }
    }
}

impl<'a> Model for CopyModel<'a> {
    type Action = CopyAction;
    type Error = ::std::io::Error;

    fn run(self) -> Result<(), Self::Error> {
        for action in &self.actions {
            apply(action)?;
        }

        self.cleaner.value.call_box();
        Ok(())
    }

    fn log<L: for<'b> Fn(&'b Self::Action)>(&self, logger: &L) {
        self.actions.iter().for_each(|e| logger(e));
    }

    fn log_run<L>(self, logger: &L) -> Result<(), Self::Error>
    where
        L: for<'b> Fn(&'b Self::Action),
    {
        for action in &self.actions {
            apply(action)?;
            logger(action);
        }

        Ok(())
    }
}

/// A set of individual models that are operated together
pub struct MultipleCopyModel<'a> {
    models: Vec<CopyModel<'a>>,
}

impl<'a> MultipleCopyModel<'a> {
    #[allow(missing_docs)]
    pub fn new(models: Vec<CopyModel<'a>>) -> Self {
        Self { models }
    }
}

impl<'a> Model for MultipleCopyModel<'a> {
    type Action = <CopyModel<'a> as Model>::Action;
    type Error = <CopyModel<'a> as Model>::Error;

    fn run(self) -> Result<(), Self::Error> {
        for model in self.models {
            model.run()?;
        }

        Ok(())
    }

    fn log<L: for<'b> Fn(&'b Self::Action)>(&self, logger: &L) {
        for model in &self.models {
            model.log(logger);
        }
    }

    fn log_run<L>(self, logger: &L) -> Result<(), Self::Error>
    where
        L: for<'b> Fn(&'b Self::Action),
    {
        for model in self.models {
            model.log_run(logger)?;
        }

        Ok(())
    }
}

fn apply(action: &CopyAction) -> ::std::io::Result<()> {
    match action {
        CopyAction::CreateDir { ref target } => {
            if !target.exists() {
                fs::create_dir_all(target)?;
            }
        }

        CopyAction::CopyFile { ref src, ref dst } => {
            if let Ok(metadata) = fs::symlink_metadata(dst) {
                if metadata.file_type().is_symlink() {
                    fs::remove_file(dst)?;
                }
            }

            fs::copy(src, dst)?;
        }

        CopyAction::CopyLink { ref src, ref dst } => {
            symlink(src, dst)?;
        }
    }

    Ok(())
}