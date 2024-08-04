use serde::{Deserialize, Serialize};

/// A modify version of the `typst.toml` adding options to utpm
#[derive(Serialize, Deserialize, Clone)]
pub struct Extra {
    /// The name of where you store your packages (default: local)
    pub namespace: Option<String>,

    /// List of url's for your dependencies (will be resolved with install command)
    pub dependencies: Option<Vec<String>>,
}

impl Extra {
    pub fn default() -> Self {
        Self {
            namespace: Some("local".to_string()),
            dependencies: None,
        }
    }

    pub fn new(namespace: Option<String>, dependencies: Option<Vec<String>>) -> Self {
        Self {
            namespace,
            dependencies,
        }
    }
}

// #[derive(Debug, ValueEnum, Serialize, Deserialize, Clone, PartialEq, Eq, EnumString)]
// #[strum(serialize_all = "lowercase")]
// #[clap(rename_all = "lower")]
