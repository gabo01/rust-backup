use std::fs;
use std::cell::Ref;
use std::ffi::OsString;
use logger::pathlight;
use std::path::Path;
use Result;
use failure::{Fail, ResultExt};

use super::{DirBranch, DirRoot, LinkedPoint, SyncOptions};

/// Used to handle errors in the sync process.
macro_rules! handle {
    ($warn:expr, $err:expr, $($msg:tt)*) => {
        if $warn {
            warn!($($msg)*);
            if cfg!(debug_assertions) {
                for cause in $err.causes() {
                    trace!("{}", cause);
                }
            }
        } else {
            fail!($err);
        }
    };
}

/// Used to give an object the ability to generate a 'branch' of itself. The generic type
/// T represents the type of branch that the object will generate. P represents the data needed
/// to generate the branch of the object.
///
/// This trait does two things:
///  - Generate a branch of the object through .branch()
///  - Return to the root point using the root method. This method should be called in the drop
///    implementation of T instead of calling it directly
pub(super) trait Branchable<'a, T: 'a, P> {
    fn branch(&'a self, branch: P) -> T;
    fn root(&self);
}

/// Used to give an object the ability to represent a link between two locations.
pub(super) trait Linkable<'a, T> {
    type Link: 'a;

    fn valid(&self) -> bool;
    fn to_ref(&self) -> Ref<T>;
    fn link(&'a self) -> Self::Link;
}

/// Internal recursive function used to sync two trees by using branches. See the docs of
/// DirTree::sync to understand how this function works on a general level.
pub(super) fn sync<'a, T, O>(tree: &'a T, options: O) -> Result<()>
where
    T: 'a
        + for<'b> Branchable<'a, DirBranch<'a>, &'b OsString>
        + for<'b> Linkable<'b, DirRoot, Link = LinkedPoint<'b>>,
    O: Into<SyncOptions>,
{
    let mut options = options.into();

    debug!(
        "Syncing {} with {}",
        pathlight(&tree.to_ref().dest),
        pathlight(&tree.to_ref().origin)
    );

    if !tree.valid() {
        fs::create_dir_all(&tree.to_ref().origin).context("Unable to create backup dir")?;
        options.clean = false; // no need to perform the clean check if the dir is empty
    }

    let iter = fs::read_dir(&tree.to_ref().dest).context("Unable to read dir")?;
    for entry in iter {
        match entry {
            Ok(component) => {
                let branch = tree.branch(&component.file_name());
                let class = FileSystemType::from(&branch.to_ref().dest);
                match class {
                    FileSystemType::File => {
                        if let Err(err) = branch.link().mirror(options.overwrite) {
                            handle!(
                                options.warn,
                                err,
                                "Unable to copy {}",
                                pathlight(&branch.to_ref().dest)
                            );
                        }
                    }

                    FileSystemType::Dir => {
                        if let Err(err) = sync(&branch, options) {
                            handle!(
                                options.warn,
                                err,
                                "Unable to read {}",
                                pathlight(&branch.to_ref().dest)
                            );
                        }
                    }

                    FileSystemType::Other => {
                        warn!("Unable to process {}", pathlight(&branch.to_ref().dest));
                    }
                };
            }

            Err(_) => warn!("Unable to read entry"),
        }
    }

    if options.clean {
        clean(tree);
    }

    Ok(())
}

/// Internal recursive function used to clean the backup directory of garbage files.
fn clean<'a, T>(tree: &'a T)
where
    T: 'a
        + for<'b> Branchable<'a, DirBranch<'a>, &'b OsString>
        + for<'b> Linkable<'b, DirRoot, Link = LinkedPoint<'b>>,
{
    let val = fs::read_dir(&tree.to_ref().origin);
    if let Ok(iter) = val {
        for entry in iter {
            match entry {
                Ok(component) => {
                    let branch = tree.branch(&component.file_name());

                    if !branch.to_ref().dest.exists() {
                        debug!(
                            "Unnexistant {}, removing {}",
                            pathlight(&branch.to_ref().dest),
                            pathlight(&branch.to_ref().origin)
                        );

                        if branch.to_ref().origin.is_dir() {
                            if let Err(err) = fs::remove_dir_all(&branch.to_ref().origin) {
                                error!("{}", err);
                                warn!(
                                    "Unable to remove garbage location {}",
                                    pathlight(&branch.to_ref().origin)
                                );
                            }
                        } else {
                            if let Err(err) = fs::remove_file(&branch.to_ref().origin) {
                                error!("{}", err);
                                warn!(
                                    "Unable to remove garbage location {}",
                                    pathlight(&branch.to_ref().origin)
                                );
                            }
                        }
                    }
                }

                // FIXME: improve the handle of this case
                Err(_) => warn!("Unable to read entry 2"),
            }
        }
    }
}

/// Represents the different types a path can take on the file system. It is just a convenience
/// enum for using a match instead of an if-else tree.
#[derive(Debug)]
enum FileSystemType {
    File,
    Dir,
    Other,
}

impl<P: AsRef<Path>> From<P> for FileSystemType {
    fn from(path: P) -> Self {
        let path = path.as_ref();
        if path.is_file() {
            FileSystemType::File
        } else if path.is_dir() {
            FileSystemType::Dir
        } else {
            FileSystemType::Other
        }
    }
}