use clap::ValueEnum;
use once_cell::sync::OnceCell;

// Differents format we want to support
#[derive(Copy, Clone, PartialEq, Eq, shadow_rs::Debug, ValueEnum)]
pub enum OutputFormat {
    Json, //todo: feature default true
    Yaml, //todo: feature default false
    Toml, //todo: feature default false
    Text, //todo: feature default true
    Hjson, //todo: feature default false
}

pub static OUTPUT_FORMAT: OnceCell<OutputFormat> = OnceCell::new();

pub fn get_output_format() -> OutputFormat {
    *OUTPUT_FORMAT.get().unwrap_or(&OutputFormat::Text)
}

