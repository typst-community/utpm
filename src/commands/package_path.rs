use tracing::instrument;

use crate::{utils::{paths::d_packages, state::{Result}}, utpm_println};

#[instrument]
pub fn run() -> Result<bool> {
    utpm_println!("Packages are located at: '{}'", d_packages()?);
    Ok(true)
}
