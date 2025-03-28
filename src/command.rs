use std::{
    ffi::{OsStr, OsString},
    io::{BufReader, Read},
    path::{Path, PathBuf},
    process::Stdio,
};

use crate::Error;

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

    pub fn warning(value: impl Into<OsString>) -> Self {
        Self::new("-W", value)
    }

    pub fn allow(value: impl Into<OsString>) -> Self {
        Self::new("-A", value)
    }

    pub fn deny(value: impl Into<OsString>) -> Self {
        Self::new("-D", value)
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[cfg(feature = "clap")]
impl Toolchain {
    pub const STABLE: &str = "stable";
    pub const NIGHTLY: &str = "nightly";

    pub fn parse(matches: &mut clap::ArgMatches) -> Self {
        for (k, v) in [(Self::STABLE, Self::Stable), (Self::NIGHTLY, Self::Nightly)] {
            if matches.get_flag(k) {
                return v;
            }
        }

        Self::Stable
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[cfg(feature = "clap")]
impl Tool {
    pub const CLIPPY: &str = "clippy";
    pub const CHECK: &str = "check";

    pub fn parse(matches: &mut clap::ArgMatches) -> Self {
        for (k, v) in [(Self::CLIPPY, Self::Clippy), (Self::CHECK, Self::Check)] {
            if matches.get_flag(k) {
                return v;
            }
        }

        Self::Clippy
    }
}

#[cfg(feature = "clap")]
impl ::clap::ValueEnum for Tool {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Clippy, Self::Check]
    }

    fn to_possible_value(&self) -> Option<::clap::builder::PossibleValue> {
        Some(match self {
            Self::Clippy => ::clap::builder::PossibleValue::new(self.as_str()),
            Self::Check => ::clap::builder::PossibleValue::new(self.as_str()),
        })
    }
}

// FIXME this should allow multiple
#[derive(Debug, Clone, Default)]
pub enum Target {
    #[default]
    Lib,
    Bins,
    Bin(String),
    Examples,
    Example(String),
    Tests,
    Test(String),
    Benches,
    Bench(String),
    AllTargets,
}

#[cfg(feature = "clap")]
impl Target {
    pub const LIB: &str = "lib";
    pub const BINS: &str = "bins";
    pub const EXAMPLES: &str = "examples";
    pub const TESTS: &str = "tests";
    pub const BENCHES: &str = "benches";
    pub const ALL_TARGETS: &str = "all_targets";

    pub const BIN: &str = "bin";
    pub const EXAMPLE: &str = "example";
    pub const TEST: &str = "test";
    pub const BENCH: &str = "bench";

    pub fn parse(matches: &mut clap::ArgMatches) -> Self {
        for (k, v) in [
            (Self::LIB, Self::Lib),
            (Self::BINS, Self::Bins),
            (Self::EXAMPLES, Self::Examples),
            (Self::TESTS, Self::Tests),
            (Self::BENCHES, Self::Benches),
            (Self::ALL_TARGETS, Self::AllTargets),
        ] {
            if matches.get_flag(k) {
                return v;
            }
        }

        for (k, v) in [
            (Self::BIN, Self::Bin as fn(String) -> Self),
            (Self::EXAMPLE, Self::Example),
            (Self::TEST, Self::Test),
            (Self::BENCH, Self::Bench),
        ] {
            if let Some(value) = matches.remove_one(k) {
                return v(value);
            }
        }

        Self::Lib
    }
}

#[derive(Debug, Clone, Default)]
pub enum Features {
    All,
    None,
    #[default]
    Default,
    Specific(Vec<String>),
}

#[cfg(feature = "clap")]
impl Features {
    pub const ALL_FEATURES: &str = "all_features";
    pub const NO_FEATURES: &str = "no_features";
    pub const FEATURES: &str = "features";

    pub fn parse(matches: &mut clap::ArgMatches) -> Self {
        if matches.get_flag(Self::ALL_FEATURES) {
            return Self::All;
        }
        if matches.get_flag(Self::NO_FEATURES) {
            return Self::None;
        }

        if let Some(features) = matches.remove_many::<String>(Self::FEATURES) {
            let mut out = vec![];
            for feature in features {
                out.extend(feature.split_terminator(',').map(|t| t.to_owned()))
            }
            out.sort_unstable();
            out.dedup();
            return Self::Specific(out);
        }

        Self::Default
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

    pub fn with_manifest_path(mut self, path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        if !Self::is_valid_manifest_path(path) {
            return Err(Error::InvalidPath(path.to_path_buf()));
        }
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
    pub fn gather(&self) -> Result<Vec<crate::data::Reason>, Error> {
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

        // TODO this should be a Vec<Target>
        match &self.target {
            Target::Lib => {
                cmd.arg("--lib");
            }
            Target::Bins => {
                cmd.arg("--bins");
            }
            Target::Bin(bin) => {
                cmd.arg("--bin").arg(bin);
            }
            Target::Examples => {
                cmd.arg("--examples");
            }
            Target::Example(example) => {
                cmd.arg("--example").arg(example);
            }
            Target::Tests => {
                cmd.arg("--tests");
            }
            Target::Test(test) => {
                cmd.arg("--test").arg(test);
            }
            Target::Benches => {
                cmd.arg("--benches");
            }
            Target::Bench(bench) => {
                cmd.arg("--bench").arg(bench);
            }
            Target::AllTargets => {
                cmd.arg("--all-targets");
            }
        }

        match &self.features {
            Features::All => {
                cmd.arg("--all-features");
            }
            Features::None => {
                cmd.arg("--no-default-features");
            }

            Features::Specific(list) => {
                for feature in list {
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

    pub fn run(&self) -> Result<impl Read, Error> {
        Ok(BufReader::new(
            self.build()
                .spawn()?
                .stdout
                .expect("stdout attached to child process"),
        ))
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
