use tracing::instrument;

use crate::{
    utils::{
        registry::{get_all_packages, get_packages_name_version},
        state::Result,
    },
    utpm_log,
};

use super::GetArgs;

/// Retrieves and displays package information from the Typst Universe registry.
///
/// If specific packages are requested, displays only those packages.
/// Otherwise, displays all available packages.
#[instrument(skip(cmd))]
pub async fn run(cmd: &GetArgs) -> Result<bool> {
    utpm_log!(trace, "executing get command");
    if !cmd.packages.is_empty() {
        let packages = get_packages_name_version().await?;
        for e in &cmd.packages {
            if !packages.contains_key(e) {
                utpm_log!(warn, "Package not found", "input" => e);
                continue;
            }
            let package = packages.get(e).unwrap();
            utpm_log!(info, package);
        }
    } else {
        let packages: Vec<_> = get_all_packages().await?;
        for package in packages {
            utpm_log!(info, package);
        }
    }

    Ok(true)
}
