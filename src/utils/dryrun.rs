use once_cell::sync::OnceCell;

/// A global static variable to hold if we want to use dry-run.
/// This allows the boolean to be set once and accessed from anywhere
/// in the application.
pub static DRYRUN: OnceCell<bool> = OnceCell::new();

/// Returns if we want to execute a dry-run or not
///
/// Defaults to `false` if not explicitly set.
pub fn get_dry_run() -> bool {
    *DRYRUN.get().unwrap_or(&false)
}
