use tracing::instrument;

use crate::{
    utils::{paths::package_path, state::Result},
    utpm_log,
};

/// Prints the path to the local typst packages directory.
#[instrument]
pub async fn run() -> Result<bool> {
    utpm_log!(trace, "executing package_path command");
    utpm_log!("Packages are located at: '{}'", package_path()?.display());
    Ok(true)
}
