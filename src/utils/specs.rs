use serde::{Deserialize, Serialize};
use typst_syntax::package::ToolInfo;

/// Represents the `[tool.utpm]` section in the `typst.toml` manifest.
///
/// This struct holds UTPM-specific configuration for a package.
#[derive(Serialize, Deserialize, Clone)]
pub struct Extra {
    /// The namespace where the package is stored (e.g., "local", "preview").
    pub namespace: Option<String>,

    /// A list of URLs for package dependencies, to be resolved by the `install` command.
    pub dependencies: Option<Vec<String>>,

    /// A list of file patterns to exclude when publishing the package.
    pub exclude: Option<Vec<String>>,
}

impl Default for Extra {
    /// Creates a default `Extra` instance with the namespace set to "local".
    fn default() -> Self {
        Self {
            namespace: Some("local".to_string()),
            dependencies: None,
            exclude: None,
        }
    }
}

impl Extra {
    /// Creates a new `Extra` instance with the given configuration.
    pub fn new(
        namespace: Option<String>,
        dependencies: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
    ) -> Self {
        Self {
            namespace,
            dependencies,
            exclude,
        }
    }
}

impl From<Option<ToolInfo>> for Extra {
    /// Converts an `Option<Tool>` from a `Manifest` into an `Extra` struct.
    ///
    /// This allows for easy extraction of the `[tool.utpm]` section.
    fn from(op_tool: Option<ToolInfo>) -> Self {
        let map = &toml::map::Map::new();
        if let Some(tool) = op_tool {
            let a = tool.sections.get("utpm").unwrap_or(map);
            Self {
                namespace: if let Some(b) = a.get("namespace") {
                    Some(b.to_string())
                } else {
                    None
                },
                dependencies: if let Some(b) = a.get("dependencies") {
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
            namespace: if let Some(b) = a.get("namespace") {
                Some(b.to_string())
            } else {
                None
            },
            dependencies: if let Some(b) = a.get("dependencies") {
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
