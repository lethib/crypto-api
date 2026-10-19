use std::fmt::{Debug, Display};

pub enum ConfigError {
    Missing(&'static str),
    Invalid { var: &'static str, reason: String },
    SecretTooShort { min: usize },
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(var) => write!(f, "{var} is not set"),
            Self::Invalid { var, reason } => write!(f, "{var} is invalid: {reason}"),
            Self::SecretTooShort { min } => {
                write!(f, "HMAC_SECRET must be at least {min} bytes long")
            }
        }
    }
}

// `main` prints the errors it returns with Debug: reuse the human-readable message.
impl Debug for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl std::error::Error for ConfigError {}
