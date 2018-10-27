use failure::{Backtrace, Context, Fail};
use logger::highlight;
use std::fmt::{self, Display};

mod helpers;
pub use self::helpers::PathRepr;

#[derive(Clone, Debug, Fail, Eq, PartialEq)]
pub enum LoadErrorType {
    File(PathRepr),
    Parse(PathRepr),
}

impl Display for LoadErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadErrorType::File(ref path) => write!(
                f,
                "Unable to read configuration from disk path {}",
                highlight(path)
            ),

            LoadErrorType::Parse(ref path) => write!(
                f,
                "Configuration format on disk path {} is not valid",
                highlight(path)
            ),
        }
    }
}

#[derive(Debug)]
pub struct LoadError {
    inner: Context<LoadErrorType>,
}

impl Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Fail for LoadError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl From<LoadErrorType> for LoadError {
    fn from(kind: LoadErrorType) -> Self {
        Self {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<LoadErrorType>> for LoadError {
    fn from(inner: Context<LoadErrorType>) -> Self {
        Self { inner }
    }
}

#[derive(Clone, Debug, Fail, Eq, PartialEq)]
pub enum SaveErrorType {
    File(PathRepr),
}

impl Display for SaveErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SaveErrorType::File(ref path) => write!(
                f,
                "Unable to save configuration into disk path {}",
                highlight(path)
            ),
        }
    }
}

#[derive(Debug)]
pub struct SaveError {
    inner: Context<SaveErrorType>,
}

impl Fail for SaveError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<SaveErrorType> for SaveError {
    fn from(kind: SaveErrorType) -> Self {
        Self {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<SaveErrorType>> for SaveError {
    fn from(inner: Context<SaveErrorType>) -> Self {
        Self { inner }
    }
}

#[derive(Copy, Clone, Debug, Fail, Eq, PartialEq)]
pub enum BackupErrorType {
    #[fail(display = "Unable to read the directory tree")]
    Scan,
    #[fail(display = "Unable to perform the backup operation")]
    Execute,
}

#[derive(Debug)]
pub struct BackupError {
    inner: Context<BackupErrorType>,
}

impl Fail for BackupError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<BackupErrorType> for BackupError {
    fn from(kind: BackupErrorType) -> Self {
        Self {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<BackupErrorType>> for BackupError {
    fn from(inner: Context<BackupErrorType>) -> Self {
        Self { inner }
    }
}

#[derive(Copy, Clone, Debug, Fail, Eq, PartialEq)]
pub enum RestoreErrorType {
    #[fail(display = "Unable to read the directory tree")]
    Scan,
    #[fail(display = "Unable to perform the backup operation")]
    Execute,
}

#[derive(Debug)]
pub struct RestoreError {
    inner: Context<RestoreErrorType>,
}

impl Fail for RestoreError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for RestoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<RestoreErrorType> for RestoreError {
    fn from(kind: RestoreErrorType) -> Self {
        Self {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<RestoreErrorType>> for RestoreError {
    fn from(inner: Context<RestoreErrorType>) -> Self {
        Self { inner }
    }
}
