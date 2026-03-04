use clap::ValueEnum;

use crate::args::get_args;

/// Defines the supported output formats for command results.
#[derive(Copy, Clone, PartialEq, Eq, shadow_rs::Debug, ValueEnum)]
pub enum OutputFormat {
    /// JSON format.
    #[cfg(feature = "output_json")]
    Json,
    /// YAML format.
    #[cfg(feature = "output_yaml")]
    Yaml,
    /// TOML format.
    Toml,
    /// Plain text format.
    Text,
    /// Hjson format.
    #[cfg(feature = "output_hjson")]
    Hjson,
}

/// Returns the currently configured output format.
///
/// Defaults to `OutputFormat::Text` if not explicitly set.
pub fn get_output_format() -> OutputFormat {
    get_args().output_format
}
