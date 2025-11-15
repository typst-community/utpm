// Library exports for UTPM
// This file exposes internal modules for testing and external use

// Re-export shadow_rs build info
use shadow_rs::shadow;
shadow!(build);

pub mod commands;
pub mod utils;

// Re-export commonly used items for external use
pub use utils::{regex_package, regex_import, copy_dir_all};

// Re-export for macro usage (internal)
pub use utils::output::OutputFormat;
