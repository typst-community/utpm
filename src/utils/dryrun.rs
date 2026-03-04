use crate::args::get_args;

/// Returns if we want to execute a dry-run or not
///
/// Defaults to `false` if not explicitly set.
pub fn get_dry_run() -> bool {
    get_args().dry_run
}
