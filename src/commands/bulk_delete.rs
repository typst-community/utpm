use tracing::instrument;

use crate::{
    utils::state::{Result, UtpmError},
    utpm_log,
};

use super::{unlink, BulkDeleteArgs, UnlinkArgs};

/// Deletes multiple packages from the local storage.
#[instrument]
pub async fn run(cmd: &BulkDeleteArgs) -> Result<bool> {
    utpm_log!(trace, "executing bulk_delete command");
    let mut vec: Vec<UtpmError> = Vec::new();
    // Iterate over the list of package names provided.
    for name in &cmd.names {
        // Call the `unlink` command for each package.
        // `yes: true` bypasses the confirmation prompt.
        match unlink::run(&UnlinkArgs {
            package: name.into(),
            yes: true,
        }).await {
            Ok(_) => {
                utpm_log!(info, "- {} deleted", name);
            }
            Err(err) => {
                // If an error occurs, log it and add it to a vector for summary.
                utpm_log!(info, "X {} not found", name);
                utpm_log!(error, "{}", err);
                vec.push(err);
            }
        };
    }
    // Log a summary of the operation.
    utpm_log!(
        "{}/{} successful",
        cmd.names.len() - vec.len(),
        cmd.names.len()
    );
    Ok(true)
}
