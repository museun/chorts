use std::{
    ffi::{OsStr, OsString},
    io::Read,
    path::{Path, PathBuf},
    process::Stdio,
};

#[derive(Clone, Debug)]
pub struct Flag {
    key: OsString,
    value: OsString,
}

impl Flag {
    pub fn new(key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Toolchain {
    #[default]
    Stable,
    Nightly,
}

impl Toolchain {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Nightly => "nightly",
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Tool {
    #[default]
    Clippy,
    Check,
}

impl Tool {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Clippy => "clippy",
            Self::Check => "check",
        }
    }
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub enum Target {
    #[default]
    Default,
    All,
    Test,
    Example,
    Specific(Vec<String>),
}

impl Target {
    pub fn specific<T: ToString>(targets: impl IntoIterator<Item = T>) -> Self {
        Self::Specific(targets.into_iter().map(|s| s.to_string()).collect())
    }
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub enum Features {
    #[default]
    Default,
    All,
    None,
    Specific(Vec<String>),
}

impl Features {
    pub fn specific<T: ToString>(features: impl IntoIterator<Item = T>) -> Self {
        Self::Specific(features.into_iter().map(|s| s.to_string()).collect())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Command {
    // TODO dedupe this
    args: Vec<OsString>,
    flags: Vec<Flag>,
    tool: Tool,
    path: Option<PathBuf>,
    toolchain: Toolchain,
    target: Target,
    features: Features,
}

impl Command {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    pub fn with_features(mut self, features: Features) -> Self {
        self.features = features;
        self
    }

    pub fn with_tool(mut self, tool: Tool) -> Self {
        self.tool = tool;
        self
    }

    pub fn with_toolchain(mut self, toolchain: Toolchain) -> Self {
        self.toolchain = toolchain;
        self
    }

    pub fn with_manifest_path(mut self, path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        anyhow::ensure!(
            Self::is_valid_manifest_path(path),
            "Invalid path to Cargo.toml: {}",
            path.display()
        );
        self.path = Some(path.to_path_buf());
        Ok(self)
    }

    pub fn with_arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn with_args<T: Into<OsString>>(mut self, args: impl IntoIterator<Item = T>) -> Self {
        self.args.extend(args.into_iter().map(|c| c.into()));
        self
    }

    pub fn with_flag(mut self, flag: Flag) -> Self {
        self.flags.push(flag);
        self
    }

    pub fn with_flags(mut self, flags: impl IntoIterator<Item = Flag>) -> Self {
        self.flags.extend(flags);
        self
    }
}

impl Command {
    pub fn gather(&self) -> anyhow::Result<Vec<crate::data::Reason>> {
        self.run().map(|json| {
            serde_json::Deserializer::from_reader(json)
                .into_iter()
                .flatten()
                .collect()
        })
    }

    pub fn build(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new("rustup");
        cmd.stdout(Stdio::piped());
        cmd.args([
            "run",
            self.toolchain.as_str(),
            "cargo",
            self.tool.as_str(),
            "--message-format=json",
        ]);

        if let Some(path) = self.path.as_deref() {
            cmd.arg("--manifest-path");
            cmd.arg(path);
        }

        match &self.target {
            Target::All => {
                cmd.arg("--all-targets");
            }
            Target::Test => {
                cmd.arg("--tests");
            }
            Target::Example => {
                cmd.arg("--examples");
            }
            Target::Specific(targets) => {
                for target in targets {
                    cmd.arg("--target").arg(target);
                }
            }
            Target::Default => {}
        }

        match &self.features {
            Features::All => {
                cmd.arg("--all-features");
            }
            Features::None => {
                cmd.arg("--no-default-features");
            }
            Features::Specific(features) => {
                for feature in features {
                    cmd.arg("--features").arg(feature);
                }
            }
            Features::Default => {}
        }

        cmd.arg("--");

        if !self.args.is_empty() {
            cmd.args(&self.args);
        }

        for Flag { key, value } in &self.flags {
            cmd.arg(key).arg(value);
        }

        cmd
    }

    pub fn run(&self) -> anyhow::Result<impl Read> {
        let child = self.build().spawn()?;
        let stdout = child.stdout.expect("stdout attached to child process");
        Ok(stdout)
    }
}

impl Command {
    pub fn without_meta(s: String) -> String {
        s.replace("--message-format=json ", "")
    }

    pub fn command_as_string(
        cmd: &std::process::Command,
        filter: impl Fn(String) -> String,
    ) -> String {
        let args =
            cmd.get_args()
                .map(std::ffi::OsStr::to_string_lossy)
                .fold(String::new(), |mut a, c| {
                    if !a.is_empty() {
                        a.push(' ');
                    }
                    a.push_str(&c);
                    a
                });

        let args = filter(args);
        let args = args.trim();
        let name = cmd.get_program().to_string_lossy();
        format!("{name} {args}")
    }
}

impl Command {
    fn is_valid_manifest_path(path: &Path) -> bool {
        path.is_file() && path.file_name().and_then(OsStr::to_str) == Some("Cargo.toml")
    }
}
