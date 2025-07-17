use tracing::instrument;

use crate::{
    utils::{paths::d_packages, state::Result},
    utpm_log,
};

#[instrument]
pub fn run() -> Result<bool> {
    utpm_log!("Packages are located at: '{}'", d_packages()?);
    Ok(true)
}
