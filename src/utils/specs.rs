use serde::{Deserialize, Serialize};
use typst_syntax::package::ToolInfo;

/// Represents the `[tool.utpm]` section in the `typst.toml` manifest.
///
/// This struct holds UTPM-specific configuration for a package.
#[derive(Serialize, Deserialize, Clone)]
pub struct Extra {
    /// A list of file patterns to exclude when publishing the package.
    pub exclude: Option<Vec<String>>,
}

impl Default for Extra {
    /// Creates a default `Extra` instance.
    fn default() -> Self {
        Self {
            exclude: None,
        }
    }
}

impl Extra {
    /// Creates a new `Extra` instance with the given configuration.
    pub fn new(
        exclude: Option<Vec<String>>,
    ) -> Self {
        Self {
            exclude,
        }
    }
}

impl From<Option<ToolInfo>> for Extra {
    /// Converts an `Option<Tool>` from a `Manifest` into an `Extra` struct.
    ///
    /// This allows for easy extraction of the `[tool.utpm]` section.
    fn from(op_tool: Option<ToolInfo>) -> Self {
        if let Some(op) = op_tool {
            Extra::from(op)
        } else {
            Extra::default()
        }
    }
}

impl From<ToolInfo> for Extra {
    /// Converts an `Tool` from a `Manifest` into an `Extra` struct.
    ///
    /// This allows for easy extraction of the `[tool.utpm]` section.
    fn from(op_tool: ToolInfo) -> Self {
        let map = &toml::map::Map::new();
        let tool = op_tool;
        let a = tool.sections.get("utpm").unwrap_or(map);
        Self {
            exclude: if let Some(b) = a.get("exclude") {
                Some(
                    b.as_array()
                        .unwrap()
                        .iter()
                        .map(|f| f.to_string())
                        .collect(),
                )
            } else {
                None
            },
        }
    }
}
