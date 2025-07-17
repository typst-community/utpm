use tracing::{instrument, warn};

use crate::utpm_log;
use crate::{commands::list::run as R, utils::state::Result};

use super::ListTreeArgs;

/// [DEPRECATED] Displays packages as a tree.
///
/// This command is deprecated and will be removed in a future version.
/// Use `list --tree` instead. It delegates to `list::run` with the `--tree` flag.
#[instrument(skip(cmd))]
pub fn run(cmd: &ListTreeArgs) -> Result<bool> {
    utpm_log!(warn, "Command is depreciated. Use list --tree instead.");
    let mut new_cmd = cmd.clone();
    new_cmd.tree = true;
    return R(&new_cmd);
}