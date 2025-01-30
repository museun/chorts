#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
pub enum Reason {
    CompilerMessage {
        message: Message,
    },
    BuildFinished {
        success: bool,
    },
    #[serde(other)]
    Ignored,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Message {
    pub code: Option<Code>,
    pub message: String,
    pub level: Level,
    pub spans: Vec<Span>,
    pub children: Vec<Self>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Code {
    pub code: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    Warning,
    Error,
    Failurenote,
    Help,
    Note,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Span {
    pub column_start: usize,
    pub line_start: usize,
    #[serde(deserialize_with = "normalize_file_name")]
    pub file_name: String,
    #[serde(deserialize_with = "maybe_empty_string")]
    pub suggested_replacement: Option<String>,
    pub text: Vec<Text>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Text {
    pub highlight_start: usize,
    pub highlight_end: usize,
    pub text: String,
}

fn normalize_file_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize as _;
    let mut out = String::deserialize(deserializer)?;
    // TODO actually do normalization
    if out.contains('\\') {
        out = out.replace('\\', "/");
    }
    Ok(out)
}

fn maybe_empty_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize as _;
    let mut out = <Option<String>>::deserialize(deserializer)?;
    if out.as_ref().filter(|c| !c.trim().is_empty()).is_some() {
        out.take();
    }
    Ok(out)
}
