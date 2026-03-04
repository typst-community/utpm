use once_cell::sync::OnceCell;

use crate::commands::Cli;

/// A global static variable holding the args with which UTPM has been called
pub static ARGS: OnceCell<Cli> = OnceCell::new();

/// Returns the arguments
pub fn get_args() -> &'static Cli {
    ARGS.get().expect("ARGS should be initialized")
}
