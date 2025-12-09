use clap::ValueEnum;
use once_cell::sync::OnceCell;

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

/// A global static variable to hold the configured output format.
/// This allows the output format to be set once and accessed from anywhere
/// in the application.
pub static OUTPUT_FORMAT: OnceCell<OutputFormat> = OnceCell::new();

/// Returns the currently configured output format.
///
/// Defaults to `OutputFormat::Text` if not explicitly set.
pub fn get_output_format() -> OutputFormat {
    *OUTPUT_FORMAT.get().unwrap_or(&OutputFormat::Text)
}
