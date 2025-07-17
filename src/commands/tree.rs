use tracing::{instrument, warn};

use crate::{commands::list::run as R, utils::state::Result};
use crate::utpm_log;

use super::ListTreeArgs;

/// DEPRECIATED
#[instrument(skip(cmd))]
pub fn run(cmd: &ListTreeArgs) -> Result<bool> {
    utpm_log!(warn, "Command is depreciated. Use list --tree instead.");
    let mut new_cmd = cmd.clone();
    new_cmd.tree = true;
    return R(&new_cmd);
}
