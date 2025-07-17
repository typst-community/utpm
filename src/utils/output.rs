use clap::ValueEnum;
use once_cell::sync::OnceCell;

// Differents format we want to support
#[derive(Copy, Clone, PartialEq, Eq, shadow_rs::Debug, ValueEnum)]
pub enum OutputFormat {
    #[cfg(feature = "output_json")]
    Json,
    #[cfg(feature = "output_yaml")]
    Yaml,
    #[cfg(feature = "output_toml")]
    Toml,
    #[cfg(feature = "output_text")]
    Text,
    #[cfg(feature = "output_hjson")]
    Hjson,
}

pub static OUTPUT_FORMAT: OnceCell<OutputFormat> = OnceCell::new();

pub fn get_output_format() -> OutputFormat {
    *OUTPUT_FORMAT.get().unwrap_or(&OutputFormat::Text)
}
