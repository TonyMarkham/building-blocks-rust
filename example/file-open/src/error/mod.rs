pub(crate) mod result;

// -------------------------------------------------------------------------- //

use error_location::ErrorLocation;
use std::path::PathBuf;
use tokio::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message} {location}")]
    Generic {
        message: String,
        location: ErrorLocation,
    },

    #[error("failed to open {}: {source} {location}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
        location: ErrorLocation,
    },

    #[error("file not found: {} {location}", path.display())]
    NotFound {
        path: PathBuf,
        location: ErrorLocation,
    },

    #[error("permission denied: {} {location}", path.display())]
    PermissionDenied {
        path: PathBuf,
        location: ErrorLocation,
    },
}

impl Error {
    pub fn from_io(source: io::Error, path: impl Into<PathBuf>, location: ErrorLocation) -> Self {
        let path = path.into();
        match source.kind() {
            io::ErrorKind::NotFound => Self::NotFound { path, location },
            io::ErrorKind::PermissionDenied => Self::PermissionDenied { path, location },
            _ => Self::Io {
                path,
                source,
                location,
            },
        }
    }
}
