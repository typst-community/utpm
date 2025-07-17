use tracing::instrument;

use crate::{
    utils::state::{Result, UtpmError},
    utpm_log,
};

use super::{unlink, BulkDeleteArgs, UnlinkArgs};

/// Deletes multiple packages from the local storage.
#[instrument]
pub fn run(cmd: &BulkDeleteArgs) -> Result<bool> {
    let mut vec: Vec<UtpmError> = Vec::new();
    for name in &cmd.names {
        match unlink::run(&UnlinkArgs {
            package: name.into(),
            yes: true,
        }) {
            Ok(_) => {
                utpm_log!(info, "- {} deleted", name);
            }
            Err(err) => {
                utpm_log!(info, "X {} not found", name);
                utpm_log!(error, "{}", err);
                vec.push(err);
            }
        };
    }
    utpm_log!(
        "{}/{} successful",
        cmd.names.len() - vec.len(),
        cmd.names.len()
    );
    Ok(true)
}