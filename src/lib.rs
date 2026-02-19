// Library exports for UTPM
// This file exposes internal modules for testing and external use

// Re-export shadow_rs build info
use shadow_rs::shadow;
shadow!(build);

pub mod args;
pub mod commands;
pub mod utils;

// Re-export commonly used items for external use
pub use utils::{copy_dir_all, regex_import, regex_package};

// Re-export for macro usage (internal)
pub use utils::output::OutputFormat;
