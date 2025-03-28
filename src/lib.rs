use std::{
    fs::Metadata,
    path::{Path, PathBuf},
};

mod visit;
pub use visit::{Filename, Highlight, Visit, Visitor};

pub fn locate_manifest(path: impl AsRef<Path>) -> Result<PathBuf, Error> {
    let path = path.as_ref();
    match path.components().last() {
        Some(s) if s.as_os_str() == "Cargo.toml" => {}
        Some(..) => {
            if !path.is_dir() {
                return Err(Error::NonManifestFile(path.to_path_buf()));
            }
            let temp = path.join("Cargo.toml");
            let Some(..) = std::fs::metadata(&temp).ok().filter(Metadata::is_file) else {
                return Err(Error::CannotFindCargoToml(path.to_path_buf()));
            };
            return Ok(temp);
        }
        _ => return Err(Error::MissingManifest),
    }
    Ok(path.to_path_buf())
}

pub mod data;

mod command;
pub use command::{Command, Features, Flag, Target, Tool, Toolchain};

#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum Error {
    /// A directory or manifest file must be provided
    MissingManifest,
    /// A non-manifest file was provided: {0}
    NonManifestFile(PathBuf),
    /// Tried to find a Cargo.toml, but couldn't find one in {0}
    CannotFindCargoToml(PathBuf),
    /// Invalid path to Cargo.toml: {0}
    InvalidPath(PathBuf),
    /// An I/O error occurred
    Io(#[from] std::io::Error),
}
