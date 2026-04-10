use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum LlmFmtError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Yaml(serde_yaml::Error),
    Csv(csv::Error),
    Toml(toml::de::Error),
    Toon(toon_format::ToonError),
    Message(String),
}

pub type Result<T> = std::result::Result<T, LlmFmtError>;

impl Display for LlmFmtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Json(err) => write!(f, "{err}"),
            Self::Yaml(err) => write!(f, "{err}"),
            Self::Csv(err) => write!(f, "{err}"),
            Self::Toml(err) => write!(f, "{err}"),
            Self::Toon(err) => write!(f, "{err}"),
            Self::Message(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for LlmFmtError {}

impl From<std::io::Error> for LlmFmtError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for LlmFmtError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<serde_yaml::Error> for LlmFmtError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::Yaml(value)
    }
}

impl From<csv::Error> for LlmFmtError {
    fn from(value: csv::Error) -> Self {
        Self::Csv(value)
    }
}

impl From<toml::de::Error> for LlmFmtError {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}

impl From<toon_format::ToonError> for LlmFmtError {
    fn from(value: toon_format::ToonError) -> Self {
        Self::Toon(value)
    }
}

impl From<String> for LlmFmtError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<&str> for LlmFmtError {
    fn from(value: &str) -> Self {
        Self::Message(value.to_owned())
    }
}
