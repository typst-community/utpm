use serde::{Deserialize, Serialize};
use typst_project::manifest::tool::Tool;

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

impl From<Option<Tool>> for Extra {
    /// Converts an `Option<Tool>` from a `Manifest` into an `Extra` struct.
    ///
    /// This allows for easy extraction of the `[tool.utpm]` section.
    fn from(op_tool: Option<Tool>) -> Self {
        match op_tool {
            Some(tool) => tool
                .get_section("utpm")
                .unwrap_or(Some(Extra::default()))
                .unwrap_or(Extra::default()),
            None => Extra::default(),
        }
    }
}
