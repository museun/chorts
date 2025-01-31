use std::{
    fs::Metadata,
    path::{Path, PathBuf},
};

mod visit;
pub use visit::{Filename, Highlight, Visit, Visitor};

pub fn locate_manifest(path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let path = path.as_ref();
    match path.components().last() {
        Some(s) if s.as_os_str() == "Cargo.toml" => {}
        Some(..) => {
            anyhow::ensure!(
                path.is_dir(),
                "a non-manifest file was provided: {}",
                path.display()
            );
            let temp = path.join("Cargo.toml");
            anyhow::ensure!(
                std::fs::metadata(&temp)
                    .ok()
                    .filter(Metadata::is_file)
                    .is_some(),
                "tried to find a Cargo.toml, but couldn't fine one in {}",
                path.display()
            );
            return Ok(temp);
        }
        _ => anyhow::bail!("a directory or manifest file must be provided"),
    }
    Ok(path.to_path_buf())
}

pub mod data;

mod command;
pub use command::{Command, Features, Flag, Target, Tool, Toolchain};
